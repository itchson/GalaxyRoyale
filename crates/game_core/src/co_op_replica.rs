//! Transactional occupied-body cache for a native guest view. This never ticks
//! gameplay or accepts a Profile. Incomplete transfers cannot replace a frame.
use crate::co_op::{PROTOCOL, ParticipantId};
use crate::co_op_publish::body_signature;
use engine_core::{FrameBuilder, PixelBody, PixelMaterial, Transform, Vec2};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashMap},
    f32::consts::FRAC_PI_2,
    rc::Rc,
};

pub const MAX_ENTITIES: usize = 512;
pub const MAX_STORAGE_CELLS: usize = 1_048_576;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pilot {
    pub id: ParticipantId,
    pub slot: u8,
    pub epoch: u32,
    pub local_seed: u32,
    #[serde(deserialize_with = "strict_identity")]
    pub identity: Option<crate::identity::KikaiIdentity>,
    pub selected_ship: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owned_loadout: Option<crate::co_op::OwnedLoadout>,
    pub position: [f32; 2],
    pub previous: [f32; 2],
    pub velocity: [f32; 2],
    pub heading: f32,
    pub previous_heading: f32,
    pub hull: f32,
    pub max_hull: f32,
    pub shield: f32,
    pub max_shield: f32,
    pub throttle: f32,
    pub last_sequence: Option<u32>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Projectile {
    pub id: String,
    pub owner: Option<ParticipantId>,
    pub owner_epoch: Option<u32>,
    pub position: [f32; 2],
    pub previous: [f32; 2],
    pub velocity: [f32; 2],
    pub damage: f32,
    pub life: f32,
    pub remaining_hits: u8,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub protocol: String,
    pub session: ParticipantId,
    pub world_seed: u32,
    pub tick: u32,
    pub revision: u32,
    pub progression: bool,
    pub participants: Vec<Pilot>,
    pub projectiles: Vec<Projectile>,
    pub dropped_notices: u32,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Counts {
    pub entities: usize,
    pub occupied_cells: usize,
    pub storage_cells: usize,
    pub entity_pages: usize,
    pub body_pages: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Header {
    pub acquisition: u32,
    pub snapshot: Snapshot,
    pub counts: Counts,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entity {
    pub id: String,
    pub kind: String,
    pub participant: Option<ParticipantId>,
    pub position: [f32; 2],
    pub previous: [f32; 2],
    pub velocity: [f32; 2],
    pub heading: f32,
    pub previous_heading: f32,
    pub scale: f32,
    pub radius: f32,
    pub hull: f32,
    pub max_hull: f32,
    pub width: usize,
    pub height: usize,
    pub occupied: usize,
    pub body_signature: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityPage {
    pub session: ParticipantId,
    pub tick: u32,
    pub revision: u32,
    pub offset: usize,
    pub next_offset: Option<usize>,
    pub entities: Vec<Entity>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyPage {
    pub session: ParticipantId,
    pub tick: u32,
    pub revision: u32,
    pub id: String,
    pub width: usize,
    pub height: usize,
    pub occupied: usize,
    pub offset: usize,
    pub next_offset: Option<usize>,
    pub cells: Vec<[u32; 2]>,
}

pub struct Scene {
    header: Header,
    entities: Vec<(Entity, Rc<PixelBody>)>,
    factory: crate::Profile,
    home: crate::Profile,
    peer: ParticipantId,
    previous_poses: Vec<([f32; 2], f32)>,
    previous_shots: Vec<[f32; 2]>,
    blend_seconds: f32,
}
/// Camera and viewport are world units, exactly as Game::set_viewport. UI
/// dimensions/zoom remain the renderer's responsibility; no presentation shake.
#[derive(Clone, Copy, Debug)]
pub struct ViewInput {
    pub camera: Vec2,
    pub viewport: Vec2,
    pub alpha: f32,
    pub reduced: bool,
    pub reduced_motion: bool,
}
impl ViewInput {
    fn valid(self) -> bool {
        vector(self.camera.to_array())
            && self.viewport.is_finite()
            && self.viewport.min_element() >= 1.0
            && self.viewport.max_element() <= 8192.0
            && finite(self.alpha, 0.0, 1.0)
    }
    fn visible(self, position: Vec2, radius: f32) -> bool {
        ((position - self.camera).abs() - self.viewport * 0.5).max_element() <= radius
    }
}
static DOCK: std::sync::LazyLock<PixelBody> = std::sync::LazyLock::new(crate::dock_body);
impl Scene {
    pub fn header(&self) -> &Header {
        &self.header
    }
    pub fn entities(&self) -> &[(Entity, Rc<PixelBody>)] {
        &self.entities
    }
    pub fn world_seed(&self) -> u32 {
        self.header.snapshot.world_seed
    }
    /// Presentation interval only: actual capture tick delta, at most 100ms.
    /// Rendering clamps alpha and never extrapolates or advances gameplay.
    pub fn blend_seconds(&self) -> f32 {
        self.blend_seconds
    }
    pub fn home_corporation(&self) -> Option<crate::identity::Corporation> {
        self.home.identity.as_ref().map(|id| id.corporation)
    }
    /// Only sources whose presence is explicit in the retained publication.
    /// Hit/exhaust/ability transient lights require future presentation metadata.
    pub fn light_sources(&self, view: ViewInput) -> Vec<crate::LightSource> {
        if !view.valid() {
            return Vec::new();
        }
        let mut sources = Vec::with_capacity(3);
        let mut nearest = [None::<(f32, &Projectile)>; 2];
        for (index, shot) in self.header.snapshot.projectiles.iter().enumerate() {
            let p = Vec2::from_array(self.previous_shots[index])
                .lerp(Vec2::from_array(shot.position), view.alpha);
            if !view.visible(p, 48.0) {
                continue;
            }
            let distance = p.distance_squared(view.camera);
            let i = if nearest[0].is_none_or(|(d, _)| distance < d) {
                0
            } else {
                1
            };
            if nearest[i].is_none_or(|(d, _)| distance < d) {
                if i == 0 {
                    nearest[1] = nearest[0];
                }
                nearest[i] = Some((distance, shot));
            }
        }
        for (_, shot) in nearest.into_iter().flatten() {
            let index = self
                .header
                .snapshot
                .projectiles
                .iter()
                .position(|s| s.id == shot.id)
                .unwrap();
            sources.push(crate::LightSource {
                position: Vec2::from_array(self.previous_shots[index])
                    .lerp(Vec2::from_array(shot.position), view.alpha)
                    .to_array(),
                radius: 42.0,
                color: if shot.owner.is_some() {
                    [1.0, 0.83, 0.42]
                } else {
                    [1.0, 0.25, 0.40]
                },
                intensity: 0.45,
            });
        }
        if view.visible(Vec2::ZERO, 160.0) {
            let [r, g, b] = self
                .home_corporation()
                .map_or([92, 181, 255], |c| c.color());
            sources.push(crate::LightSource {
                position: [0.0; 2],
                radius: 160.0,
                color: [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0],
                intensity: 0.35,
            });
        }
        sources
    }
    pub fn focus(&self, peer: ParticipantId, alpha: f32) -> Option<crate::RenderFocus> {
        if !finite(alpha, 0.0, 1.0) {
            return None;
        }
        let pilot = self.local_pilot(peer)?;
        let index = self
            .entities
            .iter()
            .position(|(e, _)| e.participant == Some(peer))?;
        let row = &self.entities[index];
        Some(crate::RenderFocus {
            player: Vec2::from_array(self.previous_poses[index].0)
                .lerp(Vec2::from_array(pilot.position), alpha),
            player_radius: row.0.radius,
            // Encounter/rig phase metadata is not in this protocol yet.
            boss: None,
        })
    }
    /// Native shared world presentation; no Game constructor, RNG, tick or save.
    /// Invalid view input leaves caller-owned extraction untouched.
    pub fn render_into_with_home(
        &self,
        frame: &mut FrameBuilder,
        view: ViewInput,
        mut draw_home: impl FnMut(&mut FrameBuilder, Transform, f32) -> bool,
    ) -> Result<()> {
        if !view.valid() {
            return Err("invalid_view");
        }
        frame.clear();
        let time = if view.reduced_motion {
            0.0
        } else {
            self.header.snapshot.tick as f32 / 60.0
        };
        crate::cosmic_background::render(
            frame,
            crate::cosmic_background::SceneInput {
                seed: self.world_seed(),
                camera: view.camera,
                viewport: view.viewport,
                time,
                reduced: view.reduced,
                reduced_motion: view.reduced_motion,
            },
        );
        crate::regions::background_with_density(
            frame,
            self.world_seed(),
            view.camera,
            view.viewport,
            time,
            view.reduced,
        );
        let lights = [
            crate::pixel_assets::color(6),
            crate::pixel_assets::color(4),
            crate::pixel_assets::color(33),
        ];
        // Canonical positions/radii come from the host seed. Mission-ready,
        // claimed and tournament progress are deliberately not inferred.
        for site in crate::regions::sites_near(self.world_seed(), view.camera) {
            let position = Vec2::from_array(site.position);
            if !view.visible(position, site.radius) {
                continue;
            }
            let signal = crate::currency_color(site.family);
            frame.push_body_colored_material(
                &DOCK,
                Transform {
                    position,
                    rotation: 0.0,
                    scale: Vec2::splat(1.05),
                },
                crate::ACTOR_MATERIAL,
                |cell, color| {
                    if color == lights[0] {
                        crate::pulse_color(signal, 1.0 + (time * 2.0 + cell.y * 0.15).sin() * 0.12)
                    } else {
                        crate::animate_station_cell(cell, color, time, lights)
                    }
                },
            );
            for index in 0..64 {
                frame.push_pixel(
                    position
                        + crate::forward(index as f32 * std::f32::consts::TAU / 64.0) * site.radius,
                    if index % 8 == 0 { 2.0 } else { 1.0 },
                    crate::with_alpha(signal, if index % 8 == 0 { 145 } else { 55 }),
                );
            }
            for index in 0..4 {
                let center =
                    position + crate::forward(index as f32 * FRAC_PI_2 + time * 0.12) * 55.0;
                for local in [Vec2::ZERO, Vec2::new(-1.0, 1.0), Vec2::new(1.0, 1.0)] {
                    frame.push_pixel(center + local, 1.0, signal);
                }
            }
        }
        for (index, champion) in crate::campaign::CHAMPIONS.iter().enumerate() {
            let position = Vec2::from_array(
                crate::campaign::arena(self.world_seed(), index)
                    .unwrap()
                    .position,
            ) - Vec2::new(0.0, 65.0);
            if !view.visible(position, 40.0) {
                continue;
            }
            frame.push_body_colored_material(
                crate::campaign::body_ref(index),
                Transform {
                    position,
                    rotation: 0.0,
                    scale: Vec2::splat(0.8),
                },
                crate::ACTOR_MATERIAL,
                |_, color| color,
            );
            for n in 0..8 {
                frame.push_pixel(
                    position
                        + crate::forward(n as f32 * std::f32::consts::TAU / 8.0 + time * 0.15)
                            * 25.0,
                    1.4,
                    crate::rgba(champion.color[0], champion.color[1], champion.color[2], 210),
                );
            }
        }
        let station = Transform {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::splat(crate::DOCK_SCALE),
        };
        if view.visible(Vec2::ZERO, 100.0) && !draw_home(frame, station, time) {
            frame.push_body_colored_material(
                &DOCK,
                station,
                crate::ACTOR_MATERIAL,
                |cell, color| {
                    crate::animate_station_cell(
                        cell,
                        crate::identity::dock_color(&self.home, color),
                        time,
                        lights.map(|c| crate::identity::dock_color(&self.home, c)),
                    )
                },
            );
        }
        if view.visible(Vec2::ZERO, crate::SAFE_RADIUS + 3.0) {
            for i in 0..96 {
                let color = if let Some(corp) = self.home_corporation() {
                    let [r, g, b] = corp.color();
                    crate::rgba(r, g, b, if i % 4 == 0 { 165 } else { 70 })
                } else {
                    let pulse = (time * 1.8 + i as f32 * 0.18).sin() * 0.5 + 0.5;
                    crate::rgba(
                        45,
                        110 + (pulse * 30.0) as u8,
                        145 + (pulse * 40.0) as u8,
                        if i % 4 == 0 { 165 } else { 70 },
                    )
                };
                frame.push_pixel(
                    crate::forward(i as f32 * std::f32::consts::TAU / 96.0) * crate::SAFE_RADIUS,
                    if i % 4 == 0 { 2.0 } else { 1.0 },
                    color,
                );
            }
        }
        crate::dock_facilities::render(
            frame,
            view.camera,
            view.viewport,
            time,
            self.home_corporation(),
        );
        crate::identity::render_crest(frame, &self.home, view.camera, view.viewport, time);
        self.append_visible(frame, view.alpha, Some(view));
        Ok(())
    }
    /// Session HUD, composed from authoritative pilot/body/build metadata.
    /// Clears every local progression/catalog/action field, without writing the
    /// provided solo HUD or any Profile. Host equipment metadata is not inferred.
    pub fn guest_hud(
        &self,
        peer: ParticipantId,
        camera: Vec2,
        mut hud: crate::Hud,
    ) -> Result<crate::Hud> {
        if !vector(camera.to_array()) {
            return Err("invalid_view");
        }
        let pilot = self.local_pilot(peer).ok_or("unknown_pilot")?;
        if peer != self.peer {
            return Err("unknown_pilot");
        }
        if pilot.slot == 0 {
            return Err("unsupported_host_hud");
        }
        let factory = &self.factory;
        // This Scene's factory belongs to its admitted guest, not arbitrary peer.
        if factory.identity != pilot.identity || factory.seed != pilot.local_seed {
            return Err("unknown_pilot");
        }
        let stats = crate::co_op_loadout::stats(factory, pilot.owned_loadout.as_ref());
        let ship = pilot.selected_ship;
        let xp = factory.campaign.ship_xp[ship as usize];
        let level = crate::campaign::level(xp);
        let rank_cap = crate::campaign::rank_cap(level);
        let position = Vec2::from_array(pilot.position);
        let site = crate::regions::sites_near(self.world_seed(), position)
            .into_iter()
            .find(|site| position.distance(Vec2::from_array(site.position)) <= site.radius);
        let home = position.length() <= crate::SAFE_RADIUS;
        let safe = home || site.is_some();
        hud.position = pilot.position;
        hud.camera = camera.to_array();
        hud.heading = pilot.heading;
        hud.hull = pilot.hull;
        hud.max_hull = pilot.max_hull;
        hud.shield = pilot.shield;
        hud.max_shield = pilot.max_shield;
        hud.danger = crate::danger(position);
        hud.safe = safe;
        hud.dock = home;
        hud.zone = if home {
            "Dock protected".into()
        } else if safe {
            "Site sanctuary".into()
        } else {
            "Co-op session / progression disabled".into()
        };
        hud.currency = [0; 3];
        hud.kills = [0; 3];
        hud.boss = None;
        hud.boss_progress.clear();
        hud.upgrades.clear();
        hud.time = self.header.snapshot.tick as f32 / 60.0;
        hud.shake = 0.0;
        hud.entities = self.entities.len() + self.header.snapshot.projectiles.len();
        hud.particles = 0;
        hud.biome = crate::regions::at(self.world_seed(), position);
        hud.site = site.map(|s| crate::SiteHud {
            id: s.id,
            name: s.name.into(),
            event: s.event.into(),
            description: "Shared sanctuary. Mission rewards are disabled in this session stage."
                .into(),
            status: "session".into(),
            can_interact: false,
        });
        hud.quest_log.clear();
        hud.discoveries = 0;
        hud.relics = 0;
        hud.campaign.champions.clear();
        hud.campaign.arena = None;
        hud.campaign.encounter = None;
        hud.campaign.ships = vec![crate::campaign::ShipHud {
            id: ship,
            name: match ship {
                0 => factory
                    .identity
                    .as_ref()
                    .map_or("Scout", |id| id.corporation.starter_name()),
                1..=8 => crate::campaign::CHAMPIONS[ship as usize - 1].ship,
                _ => "Royale Sovereign",
            }
            .into(),
            owned: true,
            selected: true,
            level,
            rank_cap,
            ..Default::default()
        }];
        hud.campaign.tournament_unlocked = false;
        hud.campaign.tournament_complete = false;
        hud.campaign.tournament_won = false;
        hud.campaign.tournament_position = [0.0; 2];
        hud.campaign.tournament_nearby = false;
        hud.campaign.standings.clear();
        hud.campaign.matches.clear();
        hud.campaign.credits_pending = false;
        hud.campaign.selected_ship = ship;
        hud.campaign.tournament_attempt = 0;
        hud.last_run = None;
        hud.ship_level = level;
        hud.ship_xp = xp;
        hud.ship_next_xp = crate::campaign::next_xp(level);
        hud.rank_cap = rank_cap;
        hud.cargo_materials = 0;
        hud.cargo_capacity = 16;
        hud.stored_materials = 0;
        hud.material_cargo = [0; 5];
        hud.material_stash = [0; 5];
        hud.material_training_cargo = [0; 3];
        hud.material_training_stash = [0; 3];
        hud.material_loss = [0; 5];
        hud.material_loss_reason.clear();
        hud.abilities.clear();
        hud.components.clear();
        hud.loadout = factory.campaign.loadouts[ship as usize].slots.map(|slot| {
            slot.and_then(|id| {
                let def = crate::abilities::get(id)?;
                Some(crate::ship_progression::EquippedAbilityHud {
                    id: def.key.into(),
                    index: id,
                    name: def.name.into(),
                    category: def.category,
                    rank: factory.campaign.abilities[id as usize].rank,
                    effective_rank: crate::ship_progression::equipped_rank(factory, id),
                })
            })
        });
        hud.ship_stats = crate::ShipStatsHud {
            speed: Vec2::from_array(pilot.velocity).length(),
            top_speed: stats.speed,
            acceleration: stats.acceleration,
            primary_damage: stats.damage,
            fire_rate: stats.fire_rate,
            hull_regen: stats.regeneration,
            shield_regen: stats.shield_regeneration,
            shield_recharge_unlocked: stats.shield_regeneration > 0.0,
            affinity: crate::shield_progression::hud(factory),
        };
        let hostiles = if safe {
            0
        } else {
            self.entities
                .iter()
                .filter(|(e, _)| {
                    matches!(e.kind.as_str(), "enemy" | "boss")
                        && e.hull > 0.0
                        && position.distance_squared(Vec2::from_array(e.position))
                            <= 1050.0 * 1050.0
                })
                .count() as u32
        };
        hud.audio = crate::AudioHud {
            area: if home {
                "haven"
            } else if safe {
                "site"
            } else if position.length() <= crate::COMBAT_RADIUS {
                "belt"
            } else {
                "reach"
            },
            biome: hud.biome.kind,
            combat: if hostiles > 0 {
                0.18 + hostiles.min(8) as f32 * 0.075
            } else {
                0.0
            },
            hostiles,
            champion: None,
            boss: !safe
                && self.entities.iter().any(|(e, _)| {
                    e.kind == "boss"
                        && e.hull > 0.0
                        && position.distance_squared(Vec2::from_array(e.position))
                            <= 1050.0 * 1050.0
                }),
            hull: pilot.hull / pilot.max_hull,
            speed: (hud.ship_stats.speed / stats.speed).clamp(0.0, 1.0),
            throttle: pilot.throttle,
            // Transient engine/particle state is not published; don't invent it.
            engines: 0,
        };
        hud.navigation = crate::NavigationHud {
            home: [0.0; 2],
            quest: None,
        };
        hud.identity = crate::identity::hud(factory);
        hud.onboarding = None;
        Ok(hud)
    }
    pub fn local_pilot(&self, peer: ParticipantId) -> Option<&Pilot> {
        self.header
            .snapshot
            .participants
            .iter()
            .find(|p| p.id == peer)
    }
    /// Occupied geometry and primary-shot presentation use the same native
    /// transforms/materials. Scenery, detailed rigs, IK and HUD are composed by
    /// the browser adapter; this append does not create a second simulation.
    pub fn append_occupied(&self, frame: &mut FrameBuilder, alpha: f32) {
        self.append_visible(frame, alpha, None);
    }
    fn append_visible(&self, frame: &mut FrameBuilder, alpha: f32, view: Option<ViewInput>) {
        let alpha = if alpha.is_finite() {
            alpha.clamp(0.0, 1.0)
        } else {
            0.0
        };
        for (index, (row, body)) in self.entities.iter().enumerate() {
            let (previous, heading) = self.previous_poses[index];
            let position = Vec2::from_array(previous).lerp(Vec2::from_array(row.position), alpha);
            // Metadata radius is gameplay data; culling also includes the actual
            // occupied body's complete canvas so a malicious small radius cannot
            // silently truncate valid imported cells.
            if view.is_some_and(|v| {
                !v.visible(
                    position,
                    row.radius.max(
                        Vec2::new(body.width as f32, body.height as f32).length() * row.scale * 0.5,
                    ),
                )
            }) {
                continue;
            }
            frame.push_body_colored_material(
                body,
                Transform {
                    position,
                    rotation: crate::interpolate_heading(heading, row.heading, alpha) + FRAC_PI_2,
                    scale: Vec2::splat(row.scale),
                },
                PixelMaterial {
                    light_response: 0.75,
                    emission: 0.0,
                },
                |_, color| color,
            );
        }
        for (index, shot) in self.header.snapshot.projectiles.iter().enumerate() {
            let position = Vec2::from_array(self.previous_shots[index])
                .lerp(Vec2::from_array(shot.position), alpha);
            let direction = Vec2::from_array(shot.velocity).normalize_or_zero();
            if view.is_some_and(|v| !v.visible(position, 8.0)) {
                continue;
            }
            let color = if shot.owner.is_none() {
                crate::rgba(255, 93, 129, 255)
            } else if shot.remaining_hits > 1 {
                crate::rgba(191, 167, 255, 255)
            } else {
                crate::rgba(255, 237, 156, 255)
            };
            frame.push_pixel(
                position,
                if shot.owner.is_none() { 3.0 } else { 2.0 },
                color,
            );
            frame.push_pixel(
                position - direction * 3.0,
                1.5,
                crate::with_alpha(color, 130),
            );
            frame.push_pixel(
                position - direction * 6.0,
                1.0,
                crate::with_alpha(color, 55),
            );
        }
    }
}
struct Pending {
    header: Header,
    rows: Vec<Entity>,
    cells: HashMap<String, Vec<[u32; 2]>>,
}
pub struct Replica {
    session: ParticipantId,
    peer: ParticipantId,
    epoch: u32,
    world_seed: u32,
    ended: bool,
    last_tick: u32,
    last_revision: u32,
    acquisition: u32,
    pending: Option<Pending>,
    scene: Option<Scene>,
}
pub type Result<T> = std::result::Result<T, &'static str>;
fn hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn finite(value: f32, minimum: f32, maximum: f32) -> bool {
    value.is_finite() && (minimum..=maximum).contains(&value)
}
fn vector(value: [f32; 2]) -> bool {
    value
        .into_iter()
        .all(|v| finite(v, -1_000_000.0, 1_000_000.0))
}
fn identity(value: &Option<crate::identity::KikaiIdentity>) -> bool {
    value.as_ref().is_none_or(|id| hex(&id.hex_id, 16))
}
fn matches(row: &Entity, old: &Entity) -> bool {
    row.id == old.id
        && row.body_signature == old.body_signature
        && row.width == old.width
        && row.height == old.height
        && row.occupied == old.occupied
}
impl Replica {
    pub fn new(
        session: ParticipantId,
        peer: ParticipantId,
        epoch: u32,
        world_seed: u32,
    ) -> Result<Self> {
        if epoch == 0 || world_seed == 0 {
            return Err("invalid_membership");
        }
        Ok(Self {
            session,
            peer,
            epoch,
            world_seed,
            ended: false,
            last_tick: 0,
            last_revision: 0,
            acquisition: 0,
            pending: None,
            scene: None,
        })
    }
    /// End invalidates both transfer and presentation; no local Game is touched.
    pub fn end(&mut self, session: ParticipantId) -> Result<()> {
        if self.ended || session != self.session {
            return Err("stale_session");
        }
        self.pending = None;
        self.scene = None;
        self.ended = true;
        Ok(())
    }
    pub fn scene(&self) -> Option<&Scene> {
        self.scene.as_ref()
    }
    pub fn stage(&mut self, header: Header) -> Result<()> {
        let snapshot = &header.snapshot;
        let count = header.counts;
        if self.ended
            || snapshot.session != self.session
            || snapshot.protocol != PROTOCOL
            || snapshot.progression
            || snapshot.world_seed != self.world_seed
            || snapshot.tick < self.last_tick
            || snapshot.revision < self.last_revision
            || header.acquisition <= self.acquisition
            || snapshot.participants.is_empty()
            || snapshot.participants.len() > 8
            || snapshot.projectiles.len() > 180
            || count.entities > MAX_ENTITIES
            || count.entities < snapshot.participants.len()
            || count.storage_cells > MAX_STORAGE_CELLS
            || count.occupied_cells > count.storage_cells
            || count.entity_pages != count.entities.div_ceil(64)
            || count.body_pages > count.entities + count.occupied_cells.div_ceil(1024)
        {
            return Err("invalid_publication");
        }
        if self.scene.as_ref().is_some_and(|old| {
            snapshot.world_seed != old.header.snapshot.world_seed
                || snapshot.tick < old.header.snapshot.tick
                || snapshot.revision < old.header.snapshot.revision
        }) {
            return Err("retired_publication");
        }
        let mut ids = BTreeSet::new();
        let mut slots = BTreeSet::new();
        for p in &snapshot.participants {
            if self
                .scene
                .as_ref()
                .and_then(|old| old.local_pilot(p.id))
                .is_some_and(|old| {
                    old.epoch == p.epoch
                        && (old.local_seed != p.local_seed
                            || old.identity != p.identity
                            || old.selected_ship != p.selected_ship
                            || old.owned_loadout != p.owned_loadout)
                })
            {
                return Err("invalid_pilot");
            }
            if !ids.insert(p.id)
                || !slots.insert(p.slot)
                || p.slot >= 8
                || p.epoch == 0
                || p.local_seed == 0
                || p.selected_ship > 9
                || !identity(&p.identity)
                || !vector(p.position)
                || !vector(p.previous)
                || !vector(p.velocity)
                || !finite(p.heading, -1_000_000.0, 1_000_000.0)
                || !finite(p.previous_heading, -1_000_000.0, 1_000_000.0)
                || !finite(p.max_hull, 0.001, 100_000.0)
                || !finite(p.hull, 0.0, p.max_hull)
                || !finite(p.max_shield, 0.0, 100_000.0)
                || !finite(p.shield, 0.0, p.max_shield)
                || !finite(p.throttle, 0.0, 1.0)
                || (p.slot != 0 && p.owned_loadout.is_none() && p.selected_ship != 0)
                || p.owned_loadout
                    .as_ref()
                    .is_some_and(|owned| owned.selected_ship != p.selected_ship)
            {
                return Err("invalid_pilot");
            }
            if p.slot != 0 || p.owned_loadout.is_some() {
                let profile = pilot_profile(p)?;
                let stats = crate::co_op_loadout::stats(&profile, p.owned_loadout.as_ref());
                if p.slot != 0 && (p.max_hull != stats.hull || p.max_shield != stats.shield) {
                    return Err("pilot_mismatch");
                }
            }
        }
        if !snapshot
            .participants
            .iter()
            .any(|p| p.id == self.peer && p.epoch == self.epoch)
            || !slots.contains(&0)
        {
            return Err("stale_membership");
        }
        let mut shots = BTreeSet::new();
        for s in &snapshot.projectiles {
            if !hex(&s.id, 16)
                || !shots.insert(&s.id)
                || !vector(s.position)
                || !vector(s.previous)
                || !vector(s.velocity)
                || !finite(s.damage, 0.0, 100_000.0)
                || !finite(s.life, 0.0, 100.0)
                || s.remaining_hits == 0
                || s.remaining_hits > 64
                || s.owner.is_some() != s.owner_epoch.is_some()
                || s.owner_epoch == Some(0)
            {
                return Err("invalid_projectile");
            }
        }
        self.acquisition = header.acquisition;
        self.last_tick = snapshot.tick;
        self.last_revision = snapshot.revision;
        self.pending = Some(Pending {
            header,
            rows: Vec::new(),
            cells: HashMap::new(),
        });
        Ok(())
    }
    fn pending(
        &mut self,
        acquisition: u32,
        session: ParticipantId,
        tick: u32,
        revision: u32,
    ) -> Result<&mut Pending> {
        let p = self.pending.as_mut().ok_or("no_publication")?;
        if self.ended
            || acquisition != p.header.acquisition
            || session != self.session
            || p.header.snapshot.tick != tick
            || p.header.snapshot.revision != revision
        {
            return Err("stale_page");
        }
        Ok(p)
    }
    pub fn entities(&mut self, acquisition: u32, page: EntityPage) -> Result<()> {
        let p = self.pending(acquisition, page.session, page.tick, page.revision)?;
        let end = page
            .offset
            .checked_add(page.entities.len())
            .ok_or("invalid_offset")?;
        if page.offset != p.rows.len()
            || page.entities.len() > 64
            || end > p.header.counts.entities
            || page.next_offset != (end < p.header.counts.entities).then_some(end)
            || (page.entities.is_empty() && end < p.header.counts.entities)
        {
            return Err("invalid_offset");
        }
        let mut previous = p.rows.last().map(|e| e.id.as_str());
        for row in &page.entities {
            if !hex(&row.id, 16)
                || previous.is_some_and(|id| id >= row.id.as_str())
                || !hex(&row.body_signature, 16)
                || row.width == 0
                || row.height == 0
                || row.width > 512
                || row.height > 512
                || row.occupied > row.width * row.height
                || !["player", "asteroid", "enemy", "boss", "segment"].contains(&row.kind.as_str())
                || (row.participant.is_some() != (row.kind == "player"))
                || !vector(row.position)
                || !vector(row.previous)
                || !vector(row.velocity)
                || !finite(row.heading, -1_000_000.0, 1_000_000.0)
                || !finite(row.previous_heading, -1_000_000.0, 1_000_000.0)
                || !finite(row.scale, 0.0, 100.0)
                || row.scale == 0.0
                || !finite(row.radius, 0.0, 10_000.0)
                || !finite(row.hull, 0.0, 100_000.0)
                || !finite(row.max_hull, 0.0, 100_000.0)
                || (row.hull > row.max_hull && !(row.kind == "segment" && row.max_hull == 0.0))
            {
                return Err("invalid_entity");
            }
            previous = Some(&row.id);
        }
        let storage = p
            .rows
            .iter()
            .chain(&page.entities)
            .map(|r| r.width * r.height)
            .sum::<usize>();
        let occupied = p
            .rows
            .iter()
            .chain(&page.entities)
            .map(|r| r.occupied)
            .sum::<usize>();
        if storage > p.header.counts.storage_cells || occupied > p.header.counts.occupied_cells {
            return Err("allocation_budget");
        }
        p.rows.extend(page.entities);
        Ok(())
    }
    pub fn body(&mut self, acquisition: u32, page: BodyPage) -> Result<()> {
        let p = self.pending(acquisition, page.session, page.tick, page.revision)?;
        let row = p
            .rows
            .iter()
            .find(|r| r.id == page.id)
            .ok_or("unknown_entity")?;
        if (page.width, page.height, page.occupied) != (row.width, row.height, row.occupied) {
            return Err("invalid_dimensions");
        }
        let existing = p.cells.get(&page.id);
        let length = existing.map_or(0, Vec::len);
        let end = page
            .offset
            .checked_add(page.cells.len())
            .ok_or("invalid_offset")?;
        if page.offset != length
            || page.cells.len() > 1024
            || end > row.occupied
            || page.next_offset != (end < row.occupied).then_some(end)
            || (page.cells.is_empty() && end < row.occupied)
        {
            return Err("invalid_offset");
        }
        let mut previous = existing.and_then(|v| v.last()).map(|v| v[0]);
        for &[index, _] in &page.cells {
            if index as usize >= row.width * row.height || previous.is_some_and(|old| index <= old)
            {
                return Err("invalid_cell_index");
            }
            previous = Some(index);
        }
        let capacity = row.occupied;
        p.cells
            .entry(page.id)
            .or_insert_with(|| Vec::with_capacity(capacity))
            .extend(page.cells);
        Ok(())
    }
    pub fn commit(&mut self, acquisition: u32, revision: u32) -> Result<()> {
        let p = self.pending.as_ref().ok_or("no_publication")?;
        if p.header.acquisition != acquisition || p.header.snapshot.revision != revision {
            return Err("stale_publication");
        }
        if p.rows.len() != p.header.counts.entities
            || p.rows.iter().map(|r| r.width * r.height).sum::<usize>()
                != p.header.counts.storage_cells
            || p.rows.iter().map(|r| r.occupied).sum::<usize>() != p.header.counts.occupied_cells
            || p.rows
                .iter()
                .map(|r| r.occupied.div_ceil(1024))
                .sum::<usize>()
                != p.header.counts.body_pages
        {
            return Err("incomplete_entities");
        }
        let mut pilots = BTreeSet::new();
        for row in &p.rows {
            if let Some(id) = row.participant {
                let pilot = p
                    .header
                    .snapshot
                    .participants
                    .iter()
                    .find(|s| s.id == id)
                    .ok_or("unknown_pilot")?;
                if !pilots.insert(id)
                    || row.position != pilot.position
                    || row.previous != pilot.previous
                    || row.velocity != pilot.velocity
                    || row.heading != pilot.heading
                    || row.previous_heading != pilot.previous_heading
                    || row.hull != pilot.hull
                    || row.max_hull != pilot.max_hull
                {
                    return Err("pilot_mismatch");
                }
            }
        }
        if pilots.len() != p.header.snapshot.participants.len() {
            return Err("missing_pilot_body");
        }
        let local = p
            .header
            .snapshot
            .participants
            .iter()
            .find(|p| p.id == self.peer)
            .unwrap();
        let factory = pilot_profile(local)?;
        if local.slot != 0 {
            let stats = crate::co_op_loadout::stats(&factory, local.owned_loadout.as_ref());
            if local.max_hull != stats.hull || local.max_shield != stats.shield {
                return Err("pilot_mismatch");
            }
            let original = crate::identity::player_body(&factory);
            let row = p
                .rows
                .iter()
                .find(|row| row.participant == Some(local.id))
                .unwrap();
            if row.width != original.width
                || row.height != original.height
                || row.scale != crate::campaign::player_scale(local.selected_ship)
            {
                return Err("pilot_mismatch");
            }
        }
        let mut entities = Vec::with_capacity(p.rows.len());
        for row in &p.rows {
            let body = if let Some(cells) = p.cells.get(&row.id) {
                if cells.len() != row.occupied {
                    return Err("incomplete_body");
                }
                let body = PixelBody::from_indexed_rgba(row.width, row.height, cells, 512 * 512)?;
                if body_signature(&body) != row.body_signature {
                    return Err("body_signature_mismatch");
                }
                Rc::new(body)
            } else if let Some((_, body)) = self
                .scene
                .as_ref()
                .and_then(|old| old.entities.iter().find(|(e, _)| matches(row, e)))
            {
                Rc::clone(body)
            } else if row.occupied == 0 {
                let body = PixelBody::from_indexed_rgba(row.width, row.height, &[], 512 * 512)?;
                if body_signature(&body) != row.body_signature {
                    return Err("body_signature_mismatch");
                }
                Rc::new(body)
            } else {
                return Err("missing_body");
            };
            entities.push((row.clone(), body));
        }
        let host = p
            .header
            .snapshot
            .participants
            .iter()
            .find(|p| p.slot == 0)
            .unwrap();
        let home = crate::Profile {
            identity: host.identity.clone(),
            ..crate::Profile::default()
        };
        let previous_poses = p
            .rows
            .iter()
            .map(|row| {
                self.scene
                    .as_ref()
                    .and_then(|old| {
                        if let Some(peer) = row.participant {
                            let before = old.local_pilot(peer)?;
                            let after = p
                                .header
                                .snapshot
                                .participants
                                .iter()
                                .find(|pilot| pilot.id == peer)?;
                            if before.epoch != after.epoch {
                                return None;
                            }
                        }
                        old.entities
                            .iter()
                            .find(|(e, _)| {
                                e.id == row.id
                                    && e.kind == row.kind
                                    && e.participant == row.participant
                            })
                            .map(|(e, _)| (e.position, e.heading))
                    })
                    .unwrap_or((row.previous, row.previous_heading))
            })
            .collect();
        let previous_shots = p
            .header
            .snapshot
            .projectiles
            .iter()
            .map(|shot| {
                self.scene
                    .as_ref()
                    .and_then(|old| {
                        old.header
                            .snapshot
                            .projectiles
                            .iter()
                            .find(|s| {
                                s.id == shot.id
                                    && s.owner == shot.owner
                                    && s.owner_epoch == shot.owner_epoch
                            })
                            .map(|s| s.position)
                    })
                    .unwrap_or(shot.previous)
            })
            .collect();
        let blend_seconds = self.scene.as_ref().map_or(1.0 / 60.0, |old| {
            (p.header
                .snapshot
                .tick
                .saturating_sub(old.header.snapshot.tick) as f32
                / 60.0)
                .clamp(1.0 / 60.0, 0.1)
        });
        let pending = self.pending.take().unwrap();
        self.scene = Some(Scene {
            header: pending.header,
            entities,
            factory,
            home,
            peer: self.peer,
            previous_poses,
            previous_shots,
            blend_seconds,
        });
        Ok(())
    }
}

fn pilot_profile(pilot: &Pilot) -> Result<crate::Profile> {
    let spec = crate::co_op::ParticipantSpec {
        id: pilot.id,
        local_seed: pilot.local_seed,
        identity: pilot.identity.clone(),
    };
    match &pilot.owned_loadout {
        Some(owned) => owned.profile(&spec),
        None => crate::co_op::guest_profile(&spec),
    }
    .map_err(|_| "invalid_pilot")
}

fn strict_identity<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<crate::identity::KikaiIdentity>, D::Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Identity {
        kind: crate::identity::KikaiKind,
        corporation: crate::identity::Corporation,
        sub_corporation: crate::identity::Corporation,
        hex_id: String,
    }
    Ok(
        Option::<Identity>::deserialize(d)?.map(|id| crate::identity::KikaiIdentity {
            kind: id.kind,
            corporation: id.corporation,
            sub_corporation: id.sub_corporation,
            hex_id: id.hex_id,
        }),
    )
}

#[cfg(test)]
#[path = "co_op_replica_tests.rs"]
mod tests;
