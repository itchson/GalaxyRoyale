//! Galaxy Royale's deterministic, fixed-step gameplay simulation.
pub mod abilities;
mod audio_runtime;
pub use audio_runtime::AudioHud;
mod asteroid_art;
#[cfg(test)]
mod asteroid_diversity_tests;
mod asteroid_zones;
mod biome_enemies;
mod boss_assets;
pub mod bosses;
pub mod campaign;
pub mod co_op;
mod co_op_control;
mod co_op_fx;
mod co_op_loadout;
pub mod co_op_publish;
pub mod co_op_replica;
#[cfg(test)]
mod co_op_tests;
pub mod co_op_wake_fx;
mod co_op_world_bounds;
#[cfg(test)]
mod component_tests;
pub mod components;
mod dock_facilities;
mod isotope_scar;
mod navigation;
pub use navigation::{NavigationHud, NavigationTarget};
mod material_scene;
mod pixel_assets;
mod player_presentation;
mod radiation_enemies;
#[cfg(test)]
mod raicho_draw_tests;
mod raicho_fine;
mod segmented;
mod shooting_star;
pub mod universe_map;
pub use campaign::RunSummary;
mod campaign_runtime;
mod cosmic_background;
#[cfg(test)]
mod cursor_aim_tests;
#[cfg(test)]
mod diagnostic_death_tests;
#[cfg(test)]
mod diagnostic_warp_tests;
#[cfg(test)]
mod held_thrust_tests;
mod hub_shadows;
pub mod identity;
#[cfg(test)]
mod live_overlay_tests;
#[cfg(test)]
mod particle_tests;
mod shield_progression;
pub use shield_progression::{RaichoAffinityHud, RaichoAffinitySource};
mod ship_progression;
#[cfg(test)]
mod ship_progression_tests;
#[cfg(test)]
mod starter_ability_tests;
mod training;
#[cfg(test)]
mod weapon_cadence_tests;
use bosses::BossEncounter;
use campaign_runtime::{ArenaEncounter, ChampionFighter};
pub mod regions;
use engine_core::{FrameBuilder, PixelBody, PixelMaterial, RenderFrame, Transform, Vec2};
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

const SAFE_RADIUS: f32 = 240.0;
const DOCK_RADIUS: f32 = 128.0;
const BELT_INNER: f32 = 700.0;
const COMBAT_RADIUS: f32 = 1400.0;
const CHUNK: f32 = 512.0;
const MAX_PARTICLES: usize = 1100;
const MAX_ENEMIES: usize = 48;
const MAX_BULLETS: usize = 180;
const COST: u32 = 10;
const DOCK_SCALE: f32 = 1.5;
const MAX_ARCHIVE: usize = 256;
const ACTOR_MATERIAL: PixelMaterial = PixelMaterial {
    light_response: 0.75,
    emission: 0.0,
};
/// Practical global-f32 extent until chunk-local simulation is implemented.
const WORLD_LIMIT: f32 = 1_000_000.0;

#[derive(Clone, Debug, Serialize)]
pub struct Profile {
    pub schema_version: u32,
    pub revision: u64,
    pub seed: u32,
    pub currency: [u32; 3],
    pub ranks: [u8; 9],
    pub unlocked: [bool; 3],
    pub kills: [u32; 3],
    pub boss_kills: [u32; 3],
    pub boss_generations: [[u32; 5]; 3],
    pub pity: [u8; 3],
    pub pending: [bool; 3],
    pub discoveries: Vec<String>,
    pub quests: Vec<QuestProgress>,
    pub relics: Vec<String>,
    pub campaign: campaign::CampaignProgress,
    pub identity: Option<identity::KikaiIdentity>,
    pub onboarding: Option<identity::OnboardingProgress>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            schema_version: components::SCHEMA_VERSION,
            revision: 0,
            seed: 0x4752_4f59,
            currency: [0; 3],
            ranks: [0; 9],
            unlocked: [false; 3],
            kills: [0; 3],
            boss_kills: [0; 3],
            boss_generations: [[0; 5]; 3],
            pity: [0; 3],
            pending: [false; 3],
            discoveries: Vec::new(),
            quests: Vec::new(),
            relics: Vec::new(),
            campaign: campaign::CampaignProgress::default(),
            identity: None,
            onboarding: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct QuestProgress {
    pub site_id: String,
    pub event: String,
    pub family: usize,
    pub target: u32,
    pub progress: u32,
    /// True only after the source site has issued its once-only reward.
    pub completed: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct InputFrame {
    pub joystick: Option<[f32; 2]>,
    pub destination: Option<[f32; 2]>,
    /// Transient unshaken world cursor while desktop right-fire is held.
    pub aim: Option<[f32; 2]>,
    /// Transient world cursor while desktop left thrust is held.
    pub thrust: Option<[f32; 2]>,
    pub fire: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct Hud {
    pub position: [f32; 2],
    pub camera: [f32; 2],
    pub heading: f32,
    pub hull: f32,
    pub max_hull: f32,
    pub shield: f32,
    pub max_shield: f32,
    pub zone: String,
    pub danger: u32,
    pub safe: bool,
    pub dock: bool,
    pub currency: [u32; 3],
    pub kills: [u32; 3],
    pub boss: Option<BossHud>,
    pub boss_progress: Vec<bosses::FamilyBossHud>,
    pub upgrades: Vec<UpgradeHud>,
    pub time: f32,
    pub shake: f32,
    pub entities: usize,
    pub particles: usize,
    pub biome: regions::Biome,
    pub site: Option<SiteHud>,
    pub quest_log: Vec<QuestProgress>,
    pub discoveries: usize,
    pub relics: usize,
    pub campaign: campaign::CampaignHud,
    pub last_run: Option<RunSummary>,
    pub ship_level: u32,
    pub ship_xp: u32,
    pub ship_next_xp: u32,
    pub rank_cap: u8,
    pub cargo_materials: u32,
    pub cargo_capacity: u32,
    pub stored_materials: u32,
    pub material_cargo: [u32; 5],
    pub material_stash: [u32; 5],
    pub material_training_cargo: [u32; 3],
    pub material_training_stash: [u32; 3],
    pub material_loss: [u32; 5],
    pub material_loss_reason: String,
    pub abilities: Vec<ship_progression::AbilityHud>,
    pub components: Vec<components::ComponentHud>,
    pub loadout: [Option<ship_progression::EquippedAbilityHud>; 3],
    pub ship_stats: ShipStatsHud,
    pub audio: AudioHud,
    pub navigation: NavigationHud,
    pub identity: Option<identity::IdentityHud>,
    pub onboarding: Option<identity::OnboardingHud>,
}

/// Current effective chassis, XP, hardware and equipped starter statistics.
/// Regeneration rates remain subject to the existing no-damage delays.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ShipStatsHud {
    pub speed: f32,
    pub top_speed: f32,
    pub acceleration: f32,
    pub primary_damage: f32,
    pub fire_rate: f32,
    pub hull_regen: f32,
    pub shield_regen: f32,
    pub shield_recharge_unlocked: bool,
    pub affinity: Option<RaichoAffinityHud>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SiteHud {
    pub id: String,
    pub name: String,
    pub event: String,
    pub description: String,
    pub status: String,
    pub can_interact: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct BossHud {
    pub name: String,
    pub health: f32,
    pub max_health: f32,
    pub family: usize,
    pub generation: u8,
    pub phase: String,
    pub telegraph: bool,
    pub position: [f32; 2],
    pub core_position: [f32; 2],
}

/// Presentation-only target geometry, matching the interpolated actor cells.
/// This does not replace tick-state HUD, collision or saved progression.
#[derive(Clone, Copy, Debug)]
pub struct RenderFocus {
    pub player: Vec2,
    pub player_radius: f32,
    pub boss: Option<BossFocus>,
}
#[derive(Clone, Copy, Debug)]
pub struct BossFocus {
    pub encounter: u64,
    pub center: Vec2,
    pub core: Vec2,
    pub core_radius: f32,
    pub phase_epoch: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct UpgradeHud {
    pub id: String,
    pub name: String,
    pub category: usize,
    pub rank: u8,
    pub effective_rank: u8,
    pub max_rank: u8,
    pub description: String,
    pub unlocked: bool,
    pub required_level: u32,
    pub evolution: u8,
    pub material_tier: u8,
    pub material_cost: u32,
    pub point_cost: u32,
    pub can_purchase: bool,
    pub purchase_reason: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct GameEvent {
    pub kind: String,
    pub x: f32,
    pub y: f32,
    pub intensity: f32,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct LightSource {
    pub position: [f32; 2],
    pub radius: f32,
    pub color: [f32; 3],
    pub intensity: f32,
}

struct TransientLight {
    source: LightSource,
    life: f32,
    initial_life: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Player,
    Asteroid,
    Enemy(usize),
    Boss(usize),
}

#[derive(Clone, Copy)]
struct Motion {
    position: Vec2,
    previous: Vec2,
    velocity: Vec2,
    heading: f32,
    previous_heading: f32,
}
impl Motion {
    fn new(position: Vec2, velocity: Vec2, heading: f32) -> Self {
        Self {
            position,
            previous: position,
            velocity,
            heading,
            previous_heading: heading,
        }
    }
}

#[derive(Clone, Copy)]
struct Actor {
    kind: Kind,
    health: f32,
    max_health: f32,
    radius: f32,
    scale: f32,
    damage: f32,
}
struct Brain {
    family: usize,
    boss: bool,
    age: f32,
    shot_timer: f32,
    phase: f32,
}
#[derive(Clone, Copy)]
struct Segment {
    owner: Entity,
    index: u8,
    health: f32,
    scale: f32,
    radius: f32,
    front: Vec2,
    rear: Vec2,
    attached: bool,
}
#[derive(Clone, Copy)]
struct Bullet {
    player: bool,
    damage: f32,
    life: f32,
    remaining_hits: u8,
    previous_hit: Option<Entity>,
}
struct Loot {
    category: usize,
    material_tier: u8,
    training_material: Option<u8>,
    amount: u32,
    life: f32,
}
#[derive(Clone, Copy, Debug)]
struct AsteroidBirth {
    source: AsteroidSource,
    material_tier: u8,
    variety: u8,
    orbit: Option<asteroid_zones::Orbit>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AsteroidSource {
    HavenBelt,
    SiteBelt([i32; 2]),
    Outer,
}
impl AsteroidBirth {
    fn at(seed: u32, position: Vec2) -> Self {
        if (BELT_INNER..COMBAT_RADIUS).contains(&position.length()) {
            return Self {
                source: AsteroidSource::HavenBelt,
                material_tier: 1,
                variety: 0,
                orbit: Some(asteroid_zones::Orbit::at(seed, position, Vec2::ZERO)),
            };
        }
        for site in regions::sites_near(seed, position) {
            let Some(belt) = regions::site_belt(seed, &site) else {
                continue;
            };
            if (belt.inner..belt.outer)
                .contains(&position.distance(Vec2::from_array(site.position)))
            {
                return Self {
                    source: AsteroidSource::SiteBelt(site.grid),
                    material_tier: belt.material_tier,
                    variety: 0,
                    orbit: Some(asteroid_zones::Orbit::at(
                        seed,
                        position,
                        Vec2::from_array(site.position),
                    )),
                };
            }
        }
        Self {
            source: AsteroidSource::Outer,
            material_tier: danger(position).clamp(1, 5) as u8,
            variety: 0,
            orbit: None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ParticleStyle {
    Debris,
    Fire,
    Exhaust,
    Trail,
}
const DEBRIS_DRAG: f32 = 1.6;
struct Particle {
    position: Vec2,
    origin: Vec2,
    local: Vec2,
    previous_local: Vec2,
    velocity: Vec2,
    acceleration: Vec2,
    life: f32,
    initial_life: f32,
    color: u32,
    size: f32,
    seed: u32,
    style: ParticleStyle,
}
struct StressObject;

#[derive(Clone, Copy)]
struct Stats {
    hull: f32,
    shield: f32,
    shield_regeneration: f32,
    speed: f32,
    acceleration: f32,
    damage: f32,
    fire_rate: f32,
    regeneration: f32,
}
impl Stats {
    fn from_profile(p: &Profile) -> Self {
        let scout = p.campaign.selected_ship == 0;
        let drive_level =
            campaign::level(p.campaign.ship_xp[p.campaign.selected_ship as usize]).min(12);
        let experience_drive = if p.campaign.selected_ship == 9 {
            1.0
        } else {
            1.0 + 0.025 * drive_level.saturating_sub(1) as f32
        };
        let ranks = ship_progression::hardware_ranks(p);
        let mut stats = Self {
            hull: 100.0 + 15.0 * ranks[2] as f32 + p.relics.len().min(20) as f32,
            shield: shield_progression::capacity(p),
            shield_regeneration: shield_progression::recharge_rate(p),
            speed: if scout { 100.0 } else { 180.0 }
                * experience_drive
                * (1.0 + 0.04 * ranks[5] as f32),
            acceleration: if scout { 260.0 } else { 400.0 }
                * experience_drive
                * (1.0 + 0.04 * ranks[5] as f32),
            damage: 12.0 * (1.0 + 0.12 * ranks[0] as f32),
            fire_rate: 5.0 * (1.0 + 0.10 * ranks[1] as f32),
            regeneration: 0.4 * ranks[4] as f32,
        };
        match p.campaign.selected_ship {
            2 => {
                stats.hull += 20.0;
                stats.speed *= 0.92;
            }
            5 => {
                stats.speed *= 1.1;
                stats.acceleration *= 1.15;
            }
            6 => stats.hull += 10.0,
            9 => {
                stats.hull += 65.0;
                stats.speed *= 1.15;
                stats.damage *= 1.25;
                stats.fire_rate *= 1.2;
                stats.regeneration += 1.0;
            }
            _ => {}
        }
        stats.damage *= 1.0 + 0.1 * ship_progression::equipped_rank(p, 0) as f32;
        let flux = 1.0 + 0.04 * ship_progression::equipped_rank(p, 2) as f32;
        let drive = flux * shield_progression::drive_multiplier(p);
        stats.speed *= drive;
        stats.acceleration *= drive;
        stats.damage *= shield_progression::weapon_multiplier(p);
        stats
    }
}

const UPGRADES: [(&str, &str, usize, &str); 9] = [
    ("damage", "Pulse Caliber", 0, "+12% primary damage per rank"),
    (
        "fire-rate",
        "Rapid Cycling",
        0,
        "+10% primary fire rate per rank",
    ),
    ("hull", "Reinforced Hull", 1, "+15 maximum hull per rank"),
    (
        "shield",
        "Aegis Capacitors",
        1,
        "+10 shield per rank; rank 1 unlocks recharge after 4s; rank and RAICHO level improve its rate",
    ),
    (
        "regeneration",
        "Repair Nanites",
        2,
        "+0.4 hull/s per rank after 5s without damage",
    ),
    (
        "speed",
        "Ion Thrusters",
        2,
        "+4% speed and acceleration per rank",
    ),
    (
        "pulse-echo",
        "Pulse Echo",
        0,
        "Every 8 shots releases a piercing fan; ranks improve its damage",
    ),
    (
        "repulsor",
        "Repulsor Nova",
        1,
        "Damage triggers a repulsive blast; ranks reduce its cooldown",
    ),
    (
        "ion-wake",
        "Ion Wake",
        2,
        "Fast flight leaves a damaging wake; ranks improve its damage",
    ),
];

pub struct Game {
    world: World,
    co_op: Option<co_op::Session>,
    publication: Option<co_op_publish::Publication>,
    publication_generation: u32,
    player: Entity,
    profile: Profile,
    camera: Vec2,
    previous_camera: Vec2,
    time: f32,
    expedition: f32,
    paused: bool,
    throttle: f32,
    destination: Option<Vec2>,
    fire_timer: f32,
    shot_count: u32,
    shield: f32,
    since_damage: f32,
    invulnerability: f32,
    repulsor_timer: f32,
    wake_timer: f32,
    exhaust_timer: f32,
    spawn_timer: f32,
    boss_timer: f32,
    active_boss: Option<Entity>,
    kill_credit: [u32; 3],
    was_safe: bool,
    shake: f32,
    rng: u32,
    particles: Vec<Particle>,
    cosmetic_sequence: u32,
    trail_timer: f32,
    events: Vec<GameEvent>,
    chunks: HashMap<(i32, i32), Vec<Entity>>,
    wake: Vec<(Vec2, f32)>,
    dock_body: PixelBody,
    viewport: Vec2,
    stress_mode: bool,
    nearby_sites: Vec<regions::Site>,
    site_grid: [i32; 2],
    reduced_cosmetics: bool,
    reduced_motion: bool,
    transient_lights: Vec<TransientLight>,
    arena: Option<ArenaEncounter>,
    run: Option<RunSummary>,
    boss_qa_scene: bool,
    // Transient progression isolation; unlike boss_qa_scene, this never selects AI.
    diagnostic_scene: bool,
    material_scene: bool,
    material_loss: [u32; 5],
    material_loss_reason: String,
}

impl Game {
    pub fn new(mut profile: Profile) -> Self {
        let legacy = profile.schema_version < 4;
        let migrated = profile.schema_version != components::SCHEMA_VERSION;
        if profile.schema_version < components::SCHEMA_VERSION {
            profile.campaign.components = [0; components::COUNT];
        }
        if legacy {
            for rank in &mut profile.ranks {
                *rank = (*rank).min(5);
            }
            for i in 0..3 {
                if !profile.unlocked[i] {
                    profile.ranks[i + 6] = 0;
                } else {
                    profile.ranks[i + 6] = profile.ranks[i + 6].max(1);
                }
            }
        }
        if profile.seed == 0 {
            profile.seed = 0x4752_4f59;
        }
        let identity_changed = identity::sanitize(&mut profile);
        let archive_changed = sanitize_archive(&mut profile);
        let bosses_changed = bosses::sanitize_progress(&mut profile);
        let mut campaign_changed = profile.campaign.sanitize();
        if profile.campaign.victory_claimed
            && campaign::table(profile.seed, &profile.campaign).0[0].entrant != 8
        {
            profile.campaign.victory_claimed = false;
            profile.campaign.credits_seen = false;
            if profile.campaign.selected_ship == 9 {
                profile.campaign.selected_ship = 0;
            }
            campaign_changed = true;
        }
        let (material_loss, inventory_changed) = ship_progression::initialize(&mut profile, legacy);
        let before_training = profile.onboarding.clone();
        training::sanitize_progress(&mut profile);
        let training_changed = before_training != profile.onboarding;
        profile.schema_version = components::SCHEMA_VERSION;
        if archive_changed
            || campaign_changed
            || bosses_changed
            || inventory_changed
            || migrated
            || identity_changed
            || training_changed
        {
            profile.revision = profile.revision.saturating_add(1);
        }
        let stats = Stats::from_profile(&profile);
        let mut world = World::new();
        let ship = identity::player_body(&profile);
        let player_scale = campaign::player_scale(profile.campaign.selected_ship);
        let player = world.spawn((
            Motion::new(Vec2::ZERO, Vec2::ZERO, -FRAC_PI_2),
            Actor {
                kind: Kind::Player,
                health: stats.hull,
                max_health: stats.hull,
                radius: pixel_assets::radius(&ship, player_scale),
                scale: player_scale,
                damage: 0.0,
            },
            ship,
        ));
        let rng = profile.seed;
        let kill_credit = std::array::from_fn(|i| {
            profile.kills[i]
                .saturating_sub(profile.boss_generations[i][0].saturating_mul(24))
                .saturating_sub(if profile.pending[i] { 24 } else { 0 })
        });
        let nearby_sites = regions::sites_near(profile.seed, Vec2::ZERO);
        let mut game = Self {
            world,
            co_op: None,
            publication: None,
            publication_generation: 0,
            player,
            profile,
            camera: Vec2::ZERO,
            previous_camera: Vec2::ZERO,
            time: 0.0,
            expedition: 0.0,
            paused: false,
            throttle: 0.0,
            destination: None,
            fire_timer: 0.0,
            shot_count: 0,
            shield: stats.shield,
            since_damage: 10.0,
            invulnerability: 0.0,
            repulsor_timer: 0.0,
            wake_timer: 0.0,
            exhaust_timer: 0.0,
            spawn_timer: 0.0,
            boss_timer: 2.0,
            active_boss: None,
            kill_credit,
            was_safe: true,
            shake: 0.0,
            rng,
            particles: Vec::with_capacity(MAX_PARTICLES),
            cosmetic_sequence: 0,
            trail_timer: 0.0,
            events: Vec::new(),
            chunks: HashMap::new(),
            wake: Vec::new(),
            dock_body: dock_body(),
            viewport: Vec2::new(800.0, 360.0),
            stress_mode: false,
            nearby_sites,
            site_grid: [0, 0],
            reduced_cosmetics: false,
            reduced_motion: false,
            transient_lights: Vec::with_capacity(32),
            arena: None,
            run: None,
            boss_qa_scene: false,
            diagnostic_scene: false,
            material_scene: false,
            material_loss,
            material_loss_reason: if ship_progression::total(material_loss) > 0 {
                "reload".into()
            } else {
                String::new()
            },
        };
        // Expeditions reload at Haven. Retained courier cargo is delivered here,
        // including after death; the reward still requires returning to its source.
        game.update_exploration();
        game
    }

    pub fn profile(&self) -> &Profile {
        self.co_op
            .as_ref()
            .map_or(&self.profile, |session| session.solo.profile())
    }
    /// Initial pointer presses may open a Haven service without issuing a course.
    /// IDs: 0 upgrades, 1 hangar, 2 journal, 3 campaign.
    pub fn dock_facility_at(&self, world: [f32; 2]) -> Option<u8> {
        let player = self.world.get::<&Motion>(self.player).ok()?.position;
        dock_facilities::at(player, Vec2::from_array(world), SAFE_RADIUS)
    }
    pub fn take_events(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.events)
    }
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }
    /// A live UI overlay cancels click navigation without freezing the universe
    /// or removing the ship's existing momentum and normal thrust decay.
    pub fn cancel_course(&mut self) {
        self.cancel_co_op_course();
        self.destination = None;
    }

    /// Reduces visual extraction only; simulation, actor masks and loot are unchanged.
    pub fn set_cosmetic_density(&mut self, reduced: bool) {
        self.reduced_cosmetics = reduced;
    }
    pub fn set_reduced_motion(&mut self, reduced: bool) {
        self.reduced_motion = reduced;
    }
    fn cosmetic_time(&self) -> f32 {
        if self.reduced_motion { 0.0 } else { self.time }
    }

    /// The renderer uses the same world-space interpolation as actor extraction.
    /// Up to four recent effects, two nearest projectiles, engine and beacon.
    pub fn light_sources(&self, alpha: f32) -> Vec<LightSource> {
        if self.material_scene {
            return self.material_scene_lights(alpha);
        }
        let alpha = if alpha.is_finite() {
            alpha.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let camera = Vec2::from_array(self.render_camera(alpha));
        let mut sources = Vec::with_capacity(8);
        for light in self
            .transient_lights
            .iter()
            .rev()
            .filter(|light| {
                self.visible(Vec2::from_array(light.source.position), light.source.radius)
            })
            .take(4)
        {
            let mut source = light.source.clone();
            source.intensity *= (light.life / light.initial_life).clamp(0.0, 1.0);
            sources.push(source);
        }
        let mut nearest: [Option<(f32, LightSource)>; 2] = [None, None];
        for (_, (motion, bullet)) in self.world.query::<(Entity, (&Motion, &Bullet))>().iter() {
            let position = motion.previous.lerp(motion.position, alpha);
            if !self.visible(position, 62.0) {
                continue;
            }
            let distance = position.distance_squared(camera);
            if nearest[1].as_ref().is_some_and(|entry| distance >= entry.0) {
                continue;
            }
            let source = LightSource {
                position: position.to_array(),
                radius: 56.0,
                color: if !bullet.player {
                    [1.0, 0.16, 0.32]
                } else if bullet.remaining_hits > 1 {
                    [0.67, 0.45, 1.0]
                } else {
                    [1.0, 0.66, 0.2]
                },
                intensity: if bullet.player { 0.9 } else { 0.8 },
            };
            if nearest[0].as_ref().is_none_or(|entry| distance < entry.0) {
                nearest[1] = nearest[0].take();
                nearest[0] = Some((distance, source));
            } else {
                nearest[1] = Some((distance, source));
            }
        }
        sources.extend(nearest.into_iter().flatten().map(|(_, source)| source));
        if let Ok(motion) = self.world.get::<&Motion>(self.player)
            && let Ok(actor) = self.world.get::<&Actor>(self.player)
        {
            let heading = interpolate_heading(motion.previous_heading, motion.heading, alpha);
            let transform = actor_transform(
                Motion {
                    position: motion.previous.lerp(motion.position, alpha),
                    heading,
                    ..*motion
                },
                *actor,
            );
            let ports = self.exhaust_ports();
            let mut sum = Vec2::ZERO;
            let mut count = 0;
            for port in ports.into_iter().flatten() {
                sum += transform.point(port);
                count += 1;
            }
            if count > 0 {
                sources.push(LightSource {
                    position: (sum / count as f32).to_array(),
                    radius: 64.0,
                    color: identity::exhaust_light(&self.profile),
                    intensity: 0.15 + self.throttle.clamp(0.0, 1.0) * 0.75,
                });
            }
        }
        let beacon = std::iter::once(Vec2::ZERO)
            .chain(
                self.nearby_sites
                    .iter()
                    .map(|site| Vec2::from_array(site.position)),
            )
            .filter(|position| self.visible(*position, 140.0))
            .min_by(|a, b| {
                a.distance_squared(camera)
                    .total_cmp(&b.distance_squared(camera))
            });
        if let Some(position) = beacon {
            sources.push(hub_shadows::beacon_light(position, self.cosmetic_time()));
        }
        sources.truncate(8);
        sources
    }

    fn transient_light(
        &mut self,
        position: Vec2,
        radius: f32,
        color: [f32; 3],
        intensity: f32,
        life: f32,
    ) {
        if !position.is_finite()
            || !radius.is_finite()
            || !intensity.is_finite()
            || !life.is_finite()
            || !color.iter().all(|value| value.is_finite())
            || life <= 0.0
        {
            return;
        }
        if self.transient_lights.len() == 32 {
            self.transient_lights.remove(0);
        }
        self.transient_lights.push(TransientLight {
            source: LightSource {
                position: position.to_array(),
                radius: radius.clamp(24.0, 220.0),
                color: color.map(|value| value.clamp(0.0, 1.0)),
                intensity: intensity.clamp(0.0, 1.8),
            },
            life,
            initial_life: life,
        });
    }

    /// Logical render dimensions, independent of device pixel ratio.
    pub fn set_viewport(&mut self, width: f32, height: f32) {
        if width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0 {
            self.viewport = Vec2::new(width, height).clamp(Vec2::splat(1.0), Vec2::splat(8192.0));
        }
    }

    fn visible(&self, position: Vec2, radius: f32) -> bool {
        let delta = (position - self.camera).abs();
        delta.x <= self.viewport.x * 0.5 + radius && delta.y <= self.viewport.y * 0.5 + radius
    }

    pub fn render_camera(&self, alpha: f32) -> [f32; 2] {
        self.previous_camera
            .lerp(self.camera, alpha.clamp(0.0, 1.0))
            .to_array()
    }

    /// Uses the same position/shortest-heading interpolation as render_into.
    /// Pixel HUD placement therefore clears the visible reactor even between
    /// simulation ticks, during camera easing, zoom and presentation shake.
    pub fn render_focus(&self, alpha: f32) -> RenderFocus {
        let alpha = if alpha.is_finite() {
            alpha.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let interpolated = |entity| {
            let motion = *self.world.get::<&Motion>(entity).ok()?;
            let actor = *self.world.get::<&Actor>(entity).ok()?;
            let mut visible = motion;
            visible.position = motion.previous.lerp(motion.position, alpha);
            visible.heading = interpolate_heading(motion.previous_heading, motion.heading, alpha);
            Some((visible, actor))
        };
        let (player, actor) = interpolated(self.player).expect("player remains allocated");
        let entity = self.arena.map(|arena| arena.opponent).or(self.active_boss);
        let boss = entity.and_then(|entity| {
            let (motion, actor) = interpolated(entity)?;
            let mut focus = BossFocus {
                encounter: entity.to_bits().get(),
                center: motion.position,
                core: motion.position,
                core_radius: 0.0,
                phase_epoch: 0,
            };
            if let Ok(boss) = self.world.get::<&BossEncounter>(entity) {
                let family = self.world.get::<&Brain>(entity).ok()?.family;
                let art = boss_assets::art(family, boss.generation);
                focus.core = actor_transform(motion, actor).point(art.core_position);
                focus.core_radius = art.core_radius * actor.scale;
                focus.phase_epoch = (boss.cycle as u64) * 3
                    + match boss.phase {
                        bosses::Phase::Tell => 0,
                        bosses::Phase::Attack => 1,
                        bosses::Phase::Recover => 2,
                    };
            }
            Some(focus)
        });
        RenderFocus {
            player: player.position,
            player_radius: actor.radius,
            boss,
        }
    }

    fn progression_enabled(&self) -> bool {
        self.co_op.is_none()
            && !self.diagnostic_scene
            && !self.boss_qa_scene
            && !self.stress_mode
            && !self.arena.is_some_and(|a| a.diagnostic)
    }

    fn changed(&mut self) {
        self.profile.revision = self.profile.revision.saturating_add(1);
    }
    fn random(&mut self) -> f32 {
        self.rng = self.rng.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.rng >> 8) as f32 / 16777216.0
    }
    fn event(&mut self, kind: &str, position: Vec2, intensity: f32) {
        match kind {
            "hit" => self.transient_light(
                position,
                48.0,
                [1.0, 0.55, 0.19],
                0.55 + intensity * 0.5,
                0.18,
            ),
            "shield" => self.transient_light(position, 95.0, [0.22, 0.56, 1.0], 1.1, 0.32),
            "ability-nova" => {
                let (light, life) = co_op_fx::lights(position)[1].clone();
                self.transient_light(position, light.radius, light.color, light.intensity, life);
            }
            "explosion" | "death" => self.transient_light(
                position,
                70.0 + intensity * 110.0,
                [1.0, 0.38, 0.12],
                0.8 + intensity * 0.8,
                0.35 + intensity * 0.2,
            ),
            "upgrade" => self.transient_light(position, 85.0, [0.76, 0.61, 1.0], 0.9, 0.35),
            _ => {}
        }
        if self.events.len() < 128 {
            self.events.push(GameEvent {
                kind: kind.into(),
                x: position.x,
                y: position.y,
                intensity,
            });
        }
    }
    fn position(&self) -> Vec2 {
        self.world
            .get::<&Motion>(self.player)
            .map(|m| m.position)
            .unwrap_or(Vec2::ZERO)
    }
    fn safe(&self) -> bool {
        self.safe_zone_at(self.position()).is_some()
    }

    fn safe_zone_at(&self, position: Vec2) -> Option<(Vec2, f32)> {
        if position.length_squared() <= SAFE_RADIUS * SAFE_RADIUS {
            return Some((Vec2::ZERO, SAFE_RADIUS));
        }
        if let Some(site) = self.nearby_sites.iter().find(|site| {
            !self.arena_disables(site)
                && Vec2::from_array(site.position).distance_squared(position)
                    <= site.radius * site.radius
        }) {
            return Some((Vec2::from_array(site.position), site.radius));
        }
        if self.outside_site_cache(position) {
            return regions::protected(self.profile.seed, position)
                .filter(|site| !self.arena_disables(site))
                .map(|site| (Vec2::from_array(site.position), site.radius));
        }
        None
    }

    fn outside_site_cache(&self, position: Vec2) -> bool {
        let grid = region_grid(position);
        (grid[0] - self.site_grid[0]).abs() > 1 || (grid[1] - self.site_grid[1]).abs() > 1
    }

    fn current_site(&self) -> Option<&regions::Site> {
        let position = self.position();
        self.nearby_sites.iter().find(|site| {
            Vec2::from_array(site.position).distance_squared(position) <= site.radius * site.radius
        })
    }

    fn refresh_sites(&mut self) {
        let grid = region_grid(self.position());
        if grid != self.site_grid {
            self.site_grid = grid;
            self.nearby_sites = regions::sites_near(self.profile.seed, self.position());
        }
    }

    fn update_exploration(&mut self) {
        // Canonical sanctuary geometry still follows live fixture movement;
        // only persistent exploration/mission facts are suppressed.
        self.refresh_sites();
        if !self.progression_enabled() {
            return;
        }
        let mut changed = false;
        if let Some(id) = self.current_site().map(|site| site.id.clone())
            && self.profile.discoveries.len() < MAX_ARCHIVE
            && !self.profile.discoveries.contains(&id)
        {
            self.profile.discoveries.push(id);
            changed = true;
            self.event("discovery", self.position(), 0.65);
        }
        // Champion contacts use the same real 320-unit approach as their HUD.
        // Store the canonical site ID, so visiting does not create extra save keys.
        for champion in &campaign::CHAMPIONS {
            if let Some(site) = self
                .nearby_sites
                .iter()
                .find(|site| site.grid == champion.grid)
                && Vec2::from_array(site.position).distance_squared(self.position())
                    <= 320.0 * 320.0
                && self.profile.discoveries.len() < MAX_ARCHIVE
                && !self.profile.discoveries.contains(&site.id)
            {
                self.profile.discoveries.push(site.id.clone());
                changed = true;
                self.event("discovery", self.position(), 0.65);
            }
        }
        if self.position().length_squared() <= DOCK_RADIUS * DOCK_RADIUS {
            for quest in &mut self.profile.quests {
                if quest.event == "courier" && !quest.completed && quest.progress == 0 {
                    quest.progress = 1;
                    changed = true;
                }
            }
        }
        if changed {
            self.changed();
        }
    }

    /// Accept an offered mission or claim its once-only reward at the source.
    pub fn interact_site(&mut self) -> Result<(), String> {
        if !self.progression_enabled() {
            return Err("Site rewards are disabled in isolated QA scenes.".into());
        }
        self.update_exploration();
        let site = self
            .current_site()
            .cloned()
            .ok_or("Enter a site sanctuary to contact its beacon.")?;
        if self.profile.relics.contains(&site.id) {
            return Err("This site's reward has already been claimed.".into());
        }
        let existing = self
            .profile
            .quests
            .iter()
            .position(|quest| quest.site_id == site.id);
        if let Some(index) = existing {
            let quest = &self.profile.quests[index];
            if quest.completed || self.profile.relics.contains(&site.id) {
                return Err("This site's reward has already been claimed.".into());
            }
            if quest.progress < quest.target {
                return Err(if quest.event == "courier" {
                    "Deliver the retained cargo to Haven dock, then return here.".into()
                } else {
                    format!(
                        "Defeat {} more {} ships, then return here.",
                        quest.target - quest.progress,
                        family_name(quest.family)
                    )
                });
            }
            if self.profile.relics.len() >= MAX_ARCHIVE {
                return Err("The relic archive is full.".into());
            }
            self.profile.quests[index].completed = true;
            self.profile.currency[site.family] =
                self.profile.currency[site.family].saturating_add(3);
            self.profile.relics.push(site.id);
            let hull = Stats::from_profile(&self.profile).hull;
            if let Ok(mut actor) = self.world.get::<&mut Actor>(self.player) {
                actor.max_health = hull;
            }
            self.changed();
            self.event("upgrade", self.position(), 0.8);
            self.burst(self.position(), 24, currency_color(site.family), 35.0);
        } else {
            if self.profile.quests.len() >= MAX_ARCHIVE
                || self.profile.relics.len() >= MAX_ARCHIVE
                || !self.profile.discoveries.contains(&site.id)
            {
                return Err("The expedition archive is full.".into());
            }
            self.profile.quests.push(QuestProgress {
                site_id: site.id,
                event: site.event.into(),
                family: site.family,
                target: if site.event == "courier" {
                    1
                } else {
                    site.target_kills
                },
                progress: 0,
                completed: false,
            });
            self.changed();
            self.event("discovery", self.position(), 0.45);
        }
        Ok(())
    }

    pub fn tick(&mut self, input: &InputFrame, dt: f32) {
        if self.co_op.is_some() || self.paused || !dt.is_finite() || dt <= 0.0 {
            return;
        }
        // A forced QA death must resolve before regeneration, dock repair or
        // an automatic ship-signature heal can restore an already dead hull.
        if self.respawn_if_dead() {
            return;
        }
        let dt = dt.min(1.0 / 20.0);
        self.time += dt;
        if self.material_scene {
            self.tick_material_scene(input, dt);
            return;
        }
        for light in &mut self.transient_lights {
            light.life -= dt;
        }
        self.transient_lights.retain(|light| light.life > 0.0);
        self.shake = (self.shake - dt * 6.0).max(0.0);
        self.since_damage += dt;
        self.invulnerability = (self.invulnerability - dt).max(0.0);
        // Ready weapons do not accumulate idle/sanctuary burst debt.
        if self.fire_timer > 0.0 {
            self.fire_timer -= dt;
            // Normalize sub-microsecond f32 noise at an exact tick boundary.
            if self.fire_timer.abs() <= 0.000_001 {
                self.fire_timer = 0.0;
            }
        }
        self.repulsor_timer -= dt;
        self.boss_timer -= dt;
        for (_, motion) in self.world.query_mut::<(Entity, &mut Motion)>() {
            motion.previous = motion.position;
            motion.previous_heading = motion.heading;
        }
        self.previous_camera = self.camera;
        self.control(input, dt);
        let position = self.position();
        self.update_exploration();
        let safe = self.safe();
        let haven = position.length_squared() <= SAFE_RADIUS * SAFE_RADIUS;
        if self.progression_enabled() {
            self.track_run(dt);
        }
        if safe {
            if haven && !self.was_safe {
                self.enter_dock();
            }
            self.expedition = 0.0;
        } else {
            self.expedition += dt;
        }
        self.was_safe = haven;
        let stats = Stats::from_profile(&self.profile);
        if self.since_damage >= 5.0
            && let Ok(mut actor) = self.world.get::<&mut Actor>(self.player)
        {
            actor.health = (actor.health + stats.regeneration * dt).min(stats.hull);
        }
        if self.since_damage >= 4.0 {
            self.shield = (self.shield + stats.shield_regeneration * dt).min(stats.shield);
        }
        if input.fire && !safe && self.fire_timer <= 0.0 {
            self.fire(stats);
        } else if !input.fire || safe {
            self.fire_timer = self.fire_timer.max(0.0);
        }
        self.update_chunks(position);
        self.update_campaign(dt);
        self.update_spawns(position, dt);
        let previous_boss_phase = self.active_boss.and_then(|e| {
            self.world
                .get::<&BossEncounter>(e)
                .ok()
                .map(|b| (e, b.phase))
        });
        self.update_bosses(position, dt);
        self.boss_audio_transition(previous_boss_phase);
        self.update_biome_enemies(position, dt);
        self.update_motes(position, dt);
        self.update_enemies(position, dt);
        self.update_homing(dt);
        self.update_asteroid_orbits(dt);
        self.integrate(dt);
        self.enforce_asteroid_shields(true);
        self.enforce_actor_sanctuaries();
        self.confine_player();
        self.confine_arena();
        self.update_segments(dt);
        self.update_exploration();
        self.resolve_collisions(dt);
        self.enforce_asteroid_shields(false);
        self.confine_player();
        self.update_pickups(position, dt);
        if self.position().length_squared() > SAFE_RADIUS * SAFE_RADIUS {
            self.onboarding_fact(identity::DEPARTED);
        }
        if self.position().length_squared() <= SAFE_RADIUS * SAFE_RADIUS
            && self
                .world
                .get::<&Actor>(self.player)
                .is_ok_and(|a| a.health > 0.0)
        {
            self.bank_cargo();
        }
        self.update_wake(position, dt);
        self.emit_projectile_trails(dt);
        self.update_particles(dt);
        if self.stress_mode {
            self.maintain_stress();
        }
        self.camera += (self.position() - self.camera) * (1.0 - (-5.0 * dt).exp());
        self.respawn_if_dead();
        self.confine_world_geometry();
    }

    /// Preserve the same event, checkpoint and repair path for both an already
    /// dead tick-entry hull and ordinary lethal collisions later in the tick.
    fn respawn_if_dead(&mut self) -> bool {
        let dead = self
            .world
            .get::<&Actor>(self.player)
            .map(|a| a.health <= 0.0)
            .unwrap_or(true);
        if dead {
            let p = self.position();
            self.discard_cargo("death");
            self.event("death", p, 1.0);
            self.burst(p, 70, rgba(255, 130, 70, 255), 90.0);
            self.reset_to_dock();
            self.shake = 3.0;
        }
        dead
    }

    fn control(&mut self, input: &InputFrame, dt: f32) {
        let stats = Stats::from_profile(&self.profile);
        let motion = co_op_control::control_motion(
            *self.world.get::<&Motion>(self.player).unwrap(),
            &mut self.throttle,
            &mut self.destination,
            input,
            stats,
            dt,
        );
        *self.world.get::<&mut Motion>(self.player).unwrap() = motion;
        self.exhaust_timer -= dt;
        if self.throttle > 0.05 && self.exhaust_timer <= 0.0 {
            self.exhaust_timer = 0.025;
            let direction = forward(motion.heading);
            let actor = *self.world.get::<&Actor>(self.player).unwrap();
            let transform = actor_transform(motion, actor);
            for port in self.exhaust_ports().into_iter().flatten() {
                self.styled_particle(
                    transform.point(port),
                    -direction * (30.0 + 75.0 * self.throttle) + motion.velocity * 0.3,
                    0.28 + 0.14 * self.throttle,
                    identity::exhaust_color(&self.profile, 230),
                    1.3 + self.throttle * 0.6,
                    ParticleStyle::Exhaust,
                );
            }
        }
    }

    fn confine_player(&mut self) {
        let mut motion = self.world.get::<&mut Motion>(self.player).unwrap();
        let bounded = motion
            .position
            .clamp(Vec2::splat(-WORLD_LIMIT), Vec2::splat(WORLD_LIMIT));
        if bounded.x != motion.position.x {
            motion.velocity.x = 0.0;
        }
        if bounded.y != motion.position.y {
            motion.velocity.y = 0.0;
        }
        motion.position = bounded;
        motion.previous = motion
            .previous
            .clamp(Vec2::splat(-WORLD_LIMIT), Vec2::splat(WORLD_LIMIT));
    }

    fn fire(&mut self, stats: Stats) {
        let motion = *self.world.get::<&Motion>(self.player).unwrap();
        let dir = forward(motion.heading);
        let primary_fired = self.world.query::<&Bullet>().iter().count()
            < if self.stress_mode { 300 } else { MAX_BULLETS };
        self.spawn_bullet(
            motion.position + dir * 19.0,
            dir * 430.0 + motion.velocity * 0.25,
            stats.damage,
            true,
            1,
        );
        // Carry active-fire overshoot into the next period instead of rounding
        // every purchased cadence down to a fixed whole number of ticks.
        self.fire_timer += 1.0 / stats.fire_rate;
        if !primary_fired {
            return; // Capacity refusal neither counts a shot nor banks a bonus.
        }
        self.shot_count += 1;
        if ship_progression::equipped_rank(&self.profile, 0) > 0
            && self.shot_count.is_multiple_of(5)
        {
            // Pulse Lens is the basic offense slot, separate from boss Echo.
            // The shared projectile cap may refuse this bolt; it never replays.
            self.spawn_bullet(
                motion.position + dir * 21.0,
                dir * 500.0 + motion.velocity * 0.25,
                stats.damage * 1.5,
                true,
                1,
            );
        }
        self.ship_signature(motion, stats);
        self.event("shot", motion.position, 0.6);
        self.styled_particle(
            motion.position + dir * 21.0,
            dir * 10.0,
            0.08,
            rgba(255, 249, 191, 255),
            3.5,
            ParticleStyle::Fire,
        );
        let echo_evolution = bosses::evolution(&self.profile, 0);
        if ship_progression::family_rank(&self.profile, 0) > 0
            && self
                .shot_count
                .is_multiple_of(if echo_evolution >= 5 { 6 } else { 8 })
        {
            self.transient_light(
                motion.position + dir * 19.0,
                95.0,
                [0.67, 0.38, 1.0],
                1.1,
                0.25,
            );
            let damage = stats.damage
                * (0.5 + 0.1 * campaign::signature_rank(&self.profile, 6).saturating_sub(1) as f32);
            let angles: &[f32] = if echo_evolution >= 3 {
                &[-0.32, -0.16, 0.0, 0.16, 0.32, PI - 0.18, PI + 0.18]
            } else if echo_evolution >= 2 {
                &[-0.32, -0.16, 0.0, 0.16, 0.32]
            } else {
                &[-0.16, 0.0, 0.16]
            };
            let before = self.world.query::<&Bullet>().iter().count();
            for angle in angles {
                let d = forward(motion.heading + angle);
                self.spawn_bullet(
                    motion.position + dir * 19.0,
                    d * 460.0,
                    damage,
                    true,
                    if echo_evolution >= 4 { 3 } else { 2 },
                );
            }
            if self.world.query::<&Bullet>().iter().count() > before {
                self.event("ability-echo", motion.position, 0.65);
            }
            self.burst(
                motion.position + dir * 19.0,
                8,
                rgba(187, 159, 255, 255),
                25.0,
            );
        }
    }

    fn spawn_bullet(
        &mut self,
        position: Vec2,
        velocity: Vec2,
        damage: f32,
        player: bool,
        hits: u8,
    ) {
        self.spawn_bullet_entity(position, velocity, damage, player, hits);
    }

    fn spawn_bullet_entity(
        &mut self,
        position: Vec2,
        velocity: Vec2,
        damage: f32,
        player: bool,
        hits: u8,
    ) -> Option<Entity> {
        if self.world.query::<(Entity, &Bullet)>().iter().count()
            >= if self.stress_mode { 300 } else { MAX_BULLETS }
        {
            return None;
        }
        Some(self.world.spawn((
            Motion::new(position, velocity, velocity.y.atan2(velocity.x)),
            Bullet {
                player,
                damage,
                life: if player { 1.5 } else { 4.0 },
                remaining_hits: hits,
                previous_hit: None,
            },
        )))
    }

    fn update_chunks(&mut self, position: Vec2) {
        let center = (
            (position.x / CHUNK).floor() as i32,
            (position.y / CHUNK).floor() as i32,
        );
        let participants = self.co_op_positions();
        let stale: Vec<_> = self
            .chunks
            .keys()
            .copied()
            .filter(|(x, y)| {
                if self.co_op.is_some() {
                    !participants.iter().flatten().any(|p| {
                        let px = (p.x / CHUNK).floor() as i32;
                        let py = (p.y / CHUNK).floor() as i32;
                        (x - px).abs() <= 2 && (y - py).abs() <= 2
                    })
                } else {
                    (x - center.0).abs() > 2 || (y - center.1).abs() > 2
                }
            })
            .collect();
        for key in stale {
            if let Some(entities) = self.chunks.remove(&key) {
                for entity in entities {
                    let _ = self.world.despawn(entity);
                }
            }
        }
        for dy in -1..=1 {
            for dx in -1..=1 {
                let key = (center.0 + dx, center.1 + dy);
                if self.chunks.contains_key(&key) {
                    continue;
                }
                let mut spawned = Vec::new();
                for index in 0..12 {
                    let hash = hash3(self.profile.seed, key.0, key.1, index);
                    let p = Vec2::new(
                        key.0 as f32 * CHUNK + unit(hash) * CHUNK,
                        key.1 as f32 * CHUNK + unit(mix(hash)) * CHUNK,
                    );
                    let radius = p.length();
                    if radius < SAFE_RADIUS + 100.0
                        || self
                            .arena
                            .as_ref()
                            .is_some_and(|arena| p.distance(arena.center) < arena.radius + 70.0)
                        || regions::protected(self.profile.seed, p).is_some()
                    {
                        continue;
                    }
                    let mut birth = AsteroidBirth::at(self.profile.seed, p);
                    birth.variety = pixel_assets::asteroid_variety(hash);
                    if birth.source == AsteroidSource::Outer && index > 2 {
                        continue;
                    }
                    let count = self
                        .world
                        .query::<(Entity, &Actor)>()
                        .iter()
                        .filter(|(_, a)| a.kind == Kind::Asteroid)
                        .count();
                    if count >= 112 {
                        break;
                    }
                    let belt = birth.source != AsteroidSource::Outer;
                    let size = asteroid_art::size(hash, birth.material_tier, belt);
                    let scale = size * asteroid_art::CELL_SCALE;
                    let body = pixel_assets::asteroid_instance(hash, birth.material_tier);
                    if let Some(orbit) = &mut birth.orbit {
                        orbit.speed = asteroid_art::speed(hash, birth.material_tier, true);
                    }
                    let body_radius = pixel_assets::radius(&body, scale);
                    if regions::sites_near(self.profile.seed, p)
                        .iter()
                        .any(|site| {
                            p.distance(Vec2::from_array(site.position))
                                < site.radius + body_radius + 2.0
                        })
                    {
                        continue;
                    }
                    let heading = unit(mix(hash ^ 71)) * TAU;
                    let velocity = if let Some(orbit) = birth.orbit {
                        orbit.tangent(p) * orbit.speed
                    } else {
                        forward(heading) * asteroid_art::speed(hash, birth.material_tier, false)
                    };
                    let hp = 20.0 + size * 30.0;
                    let entity = self.world.spawn((
                        Motion::new(p, velocity, heading),
                        Actor {
                            kind: Kind::Asteroid,
                            health: hp,
                            max_health: hp,
                            radius: body_radius,
                            scale,
                            damage: 8.0,
                        },
                        body,
                        birth,
                    ));
                    spawned.push(entity);
                }
                self.chunks.insert(key, spawned);
            }
        }
    }

    fn update_spawns(&mut self, position: Vec2, dt: f32) {
        if self.arena.is_some()
            || self.diagnostic_boss_active()
            || position.length() < COMBAT_RADIUS
            || self.safe_zone_at(position).is_some()
        {
            return;
        }
        let tier = danger(position);
        self.spawn_timer -= dt;
        let count = self.world.query::<(Entity, &Brain)>().iter().count();
        if self.spawn_timer <= 0.0 && count < MAX_ENEMIES {
            let wave = (self.expedition % 60.0) / 60.0;
            let pressure = if wave < 0.5 {
                0.8
            } else if wave < 0.84 {
                1.45
            } else {
                0.55
            };
            self.spawn_timer = (1.5
                / (pressure
                    * (1.0 + 0.14 * tier.saturating_sub(1) as f32)
                    * (1.0 + (self.expedition / 1800.0).min(0.5))))
            .max(0.3);
            let biome = regions::at(self.profile.seed, position);
            let weights = biome.spawn_weights;
            let family = weighted_family(weights, self.random());
            let distance = self.viewport.length() * 0.5 + 60.0 + self.random() * 100.0;
            if let Some(spawn) = self.spawn_location(position, distance) {
                let entity = self.spawn_enemy(family, false, spawn, tier);
                if self.co_op.is_none() {
                    self.maybe_riveter(entity, regions::at(self.profile.seed, spawn));
                    self.maybe_mote(entity, regions::at(self.profile.seed, spawn));
                }
            }
        }
        if self.co_op.is_none()
            && self.active_boss.is_none()
            && self.boss_timer <= 0.0
            && let Some((family, generation)) = self.next_boss()
            && let Some(spawn) = self.spawn_location(position, self.viewport.length() * 0.5 + 45.0)
        {
            let entity = self.spawn_enemy(family, true, spawn, tier);
            if generation > 1 {
                self.configure_boss(entity, family, generation, false, tier);
            }
            self.active_boss = Some(entity);
            self.boss_timer = 10.0;
            self.event("boss", position, 1.0);
            self.shake = self.shake.max(2.0);
        }
    }

    fn spawn_location(&mut self, player: Vec2, distance: f32) -> Option<Vec2> {
        for _ in 0..8 {
            let candidate = player + forward(self.random() * TAU) * distance;
            if candidate.length() > SAFE_RADIUS + 48.0
                && regions::sites_near(self.profile.seed, candidate)
                    .iter()
                    .all(|site| {
                        Vec2::from_array(site.position).distance(candidate) > site.radius + 48.0
                    })
            {
                return Some(candidate);
            }
        }
        None
    }

    fn spawn_enemy(&mut self, family: usize, boss: bool, position: Vec2, tier: u32) -> Entity {
        let chain_direction = initial_chain_direction(position, self.position());
        let heading = if family == 2 {
            (-chain_direction).y.atan2((-chain_direction).x)
        } else {
            0.0
        };
        let health_base = [30.0, 48.0, 60.0][family];
        let health = health_base
            * (1.0 + tier.saturating_sub(1) as f32 * 0.2)
            * if boss { 9.0 } else { 1.0 };
        let scale = if boss {
            [1.0, 1.6, 1.5][family]
        } else {
            [0.75, 0.7, 0.75][family]
        };
        let body = enemy_body(family, boss);
        let phase = self.random() * TAU;
        let entity = self.world.spawn((
            Motion::new(position, Vec2::ZERO, heading),
            Actor {
                kind: if boss {
                    Kind::Boss(family)
                } else {
                    Kind::Enemy(family)
                },
                health,
                max_health: health,
                radius: pixel_assets::radius(&body, scale),
                scale,
                damage: if boss { 20.0 } else { 9.0 }
                    * (1.0 + tier.saturating_sub(1) as f32 * 0.12),
            },
            body,
            Brain {
                family,
                boss,
                age: 0.0,
                shot_timer: 1.0,
                phase,
            },
        ));
        if family == 2 {
            let count = if boss { 11 } else { 4 };
            self.spawn_chain(
                entity,
                coil_segment_body(boss),
                segment_scale(boss),
                count,
                if boss { 42.0 } else { 15.0 },
            );
        }
        if boss {
            self.configure_boss(entity, family, 1, false, tier);
        }
        entity
    }

    fn update_enemies(&mut self, player: Vec2, dt: f32) {
        let participants = self
            .co_op_positions()
            .map(|p| p.map(|p| (p, self.safe_zone_at(p))));
        let sanctuary = self.safe_zone_at(player);
        let safe = sanctuary.is_some();
        let refuge_center = sanctuary.map(|zone| zone.0).unwrap_or(Vec2::ZERO);
        let pursuit_range = self.viewport.length().max(1050.0);
        let mut shots = Vec::new();
        let mut remove = Vec::new();
        for (entity, (motion, actor, brain, champion, boss, riveter, mote)) in
            self.world.query_mut::<(
                Entity,
                (
                    &mut Motion,
                    &Actor,
                    &mut Brain,
                    Option<&ChampionFighter>,
                    Option<&BossEncounter>,
                    Option<&biome_enemies::Riveter>,
                    Option<&radiation_enemies::Mote>,
                ),
            )>()
        {
            if champion.is_some() || boss.is_some() || riveter.is_some() || mote.is_some() {
                continue;
            }
            let target = participants
                .iter()
                .flatten()
                .filter(|(_, refuge)| refuge.is_none())
                .min_by(|(a, _), (b, _)| {
                    a.distance_squared(motion.position)
                        .total_cmp(&b.distance_squared(motion.position))
                })
                .or_else(|| {
                    participants.iter().flatten().min_by(|(a, _), (b, _)| {
                        a.distance_squared(motion.position)
                            .total_cmp(&b.distance_squared(motion.position))
                    })
                });
            let (player, safe, refuge_center) =
                target.map_or((player, safe, refuge_center), |(p, refuge)| {
                    (
                        *p,
                        refuge.is_some(),
                        refuge.map_or(Vec2::ZERO, |zone| zone.0),
                    )
                });
            brain.age += dt;
            brain.shot_timer -= dt;
            let delta = player - motion.position;
            let distance = delta.length().max(0.1);
            let towards = delta / distance;
            let side = Vec2::new(-towards.y, towards.x);
            let tier = danger(player).max(1);
            let speed_factor = (1.0 + 0.035 * tier.saturating_sub(1) as f32).min(1.3);
            let enraged = actor.health < actor.max_health * 0.45;
            if let Some(outward) =
                push_out_of_sanctuaries(&mut motion.position, actor.radius, &self.nearby_sites)
            {
                motion.velocity = outward * motion.velocity.length().max(50.0);
            }
            if safe {
                let outward = (motion.position - refuge_center).normalize_or_zero();
                motion.velocity += (outward * 95.0 - motion.velocity) * (dt * 3.0);
                if distance > 550.0 {
                    remove.push(entity);
                }
                continue;
            }
            if distance > pursuit_range {
                remove.push(entity);
                continue;
            }
            let desired = match brain.family {
                0 => {
                    let weave =
                        (brain.age * if brain.boss { 2.2 } else { 4.2 } + brain.phase).sin();
                    if brain.boss {
                        towards * (if distance > 160.0 { 80.0 } else { -35.0 })
                            + side * weave * 92.0
                    } else {
                        towards * 85.0 + side * weave * 42.0
                    }
                }
                1 => {
                    let approach = ((distance - if brain.boss { 190.0 } else { 160.0 }) * 1.5)
                        .clamp(-72.0, 72.0);
                    towards * approach
                        + side * (brain.phase.sin().signum() * if brain.boss { 50.0 } else { 48.0 })
                }
                _ => {
                    towards * if brain.boss { 100.0 } else { 93.0 }
                        + side
                            * (brain.age * 2.2 + brain.phase).sin()
                            * if brain.boss { 85.0 } else { 38.0 }
                }
            } * speed_factor;
            motion.velocity += (desired - motion.velocity) * (1.0 - (-3.0 * dt).exp());
            motion.heading = turn_toward(motion.heading, towards.y.atan2(towards.x), 3.0 * PI * dt);
            if brain.age > 0.7 && brain.shot_timer <= 0.0 && distance < 520.0 {
                if brain.boss {
                    match brain.family {
                        0 => {
                            let count = if enraged { 12 } else { 8 };
                            for n in 0..count {
                                let dir = forward(brain.age * 0.5 + n as f32 * TAU / count as f32);
                                shots.push((motion.position, dir * 95.0, actor.damage * 0.65));
                            }
                            brain.shot_timer = if enraged { 1.5 } else { 2.2 };
                        }
                        1 => {
                            let heading = towards.y.atan2(towards.x);
                            for n in -2..=2 {
                                let dir = forward(heading + n as f32 * 0.18);
                                shots.push((
                                    motion.position + dir * actor.radius,
                                    dir * 145.0,
                                    actor.damage * 0.7,
                                ));
                            }
                            brain.shot_timer = if enraged { 0.85 } else { 1.3 };
                        }
                        _ => {
                            let heading = towards.y.atan2(towards.x);
                            for offset in [-0.38, 0.0, 0.38] {
                                shots.push((
                                    motion.position,
                                    forward(heading + offset) * 110.0,
                                    actor.damage * 0.6,
                                ));
                            }
                            brain.shot_timer = if enraged { 1.0 } else { 1.8 };
                        }
                    }
                } else if brain.family == 1 {
                    shots.push((
                        motion.position + towards * actor.radius,
                        towards * 125.0,
                        actor.damage * 0.8,
                    ));
                    brain.shot_timer = 1.8;
                } else {
                    brain.shot_timer = 2.0;
                }
            }
        }
        for (p, v, d) in shots {
            self.spawn_bullet(p, v, d, false, 1);
        }
        for entity in remove {
            self.remove_enemy(entity, false);
        }
    }

    fn enforce_actor_sanctuaries(&mut self) {
        let seed = self.profile.seed;
        let grid = self.site_grid;
        for (_, (motion, actor, champion)) in self
            .world
            .query_mut::<(Entity, (&mut Motion, &Actor, Option<&ChampionFighter>))>()
        {
            if champion.is_some() {
                continue;
            }
            if !matches!(actor.kind, Kind::Enemy(_) | Kind::Boss(_)) {
                continue;
            }
            let actor_grid = region_grid(motion.position);
            let outside_cache =
                (actor_grid[0] - grid[0]).abs() > 1 || (actor_grid[1] - grid[1]).abs() > 1;
            let distant_sites = outside_cache.then(|| regions::sites_near(seed, motion.position));
            if let Some(outward) = push_out_of_sanctuaries(
                &mut motion.position,
                actor.radius,
                distant_sites.as_deref().unwrap_or(&self.nearby_sites),
            ) {
                let inward_speed = motion.velocity.dot(outward).min(0.0);
                motion.velocity -= outward * inward_speed;
            }
        }
    }

    fn integrate(&mut self, dt: f32) {
        let mut dead = Vec::new();
        for (_, (motion, segment)) in self
            .world
            .query_mut::<(Entity, (&mut Motion, Option<&Segment>))>()
        {
            if segment.is_none() {
                motion.position += motion.velocity * dt;
            }
        }
        for (entity, bullet) in self.world.query_mut::<(Entity, &mut Bullet)>() {
            bullet.life -= dt;
            if bullet.life <= 0.0 {
                dead.push(entity);
            }
        }
        for entity in dead {
            let _ = self.world.despawn(entity);
        }
    }

    fn resolve_collisions(&mut self, dt: f32) {
        let player = self.position();
        let player_radius = self.world.get::<&Actor>(self.player).unwrap().radius;
        let actors: Vec<_> = self
            .world
            .query::<(Entity, (&Motion, &Actor))>()
            .iter()
            .map(|(e, (m, a))| (e, *m, *a))
            .collect();
        let segments: Vec<_> = self
            .world
            .query::<(Entity, (&Motion, &Segment))>()
            .iter()
            .map(|(e, (m, s))| (e, *m, s.owner, s.scale, s.radius))
            .collect();
        let bullets: Vec<_> = self
            .world
            .query::<(Entity, (&Motion, &Bullet))>()
            .iter()
            .map(|(e, (m, b))| (e, *m, *b))
            .collect();
        for (entity, motion, mut bullet) in bullets {
            if !self.world.contains(entity) {
                continue;
            }
            // A nova may reflect a later entry in this snapshot. Its new sweep
            // starts next tick; do not process the old incoming sweep as hostile.
            if self
                .world
                .get::<&Bullet>(entity)
                .ok()
                .is_some_and(|current| current.player != bullet.player)
            {
                continue;
            }
            if segment_circle(motion.previous, motion.position, Vec2::ZERO, SAFE_RADIUS).is_some()
                || self.nearby_sites.iter().any(|site| {
                    !self.arena_disables(site)
                        && site.intersects_segment(motion.previous, motion.position)
                })
                || ((self.outside_site_cache(motion.previous)
                    || self.outside_site_cache(motion.position))
                    && regions::segment_protected(
                        self.profile.seed,
                        motion.previous,
                        motion.position,
                    ))
            {
                let _ = self.world.despawn(entity);
                continue;
            }
            let mut nearest: Option<(f32, Entity, Vec2, bool, Vec2)> = None;
            for (target, m, a) in &actors {
                if bullet.previous_hit == Some(*target) {
                    continue;
                }
                if bullet.player && a.kind == Kind::Player {
                    continue;
                }
                if !bullet.player && a.kind != Kind::Player {
                    continue;
                }
                if segment_circle(motion.previous, motion.position, m.position, a.radius + 2.0)
                    .is_some()
                {
                    let transform = actor_transform(*m, *a);
                    let hit = self.world.get::<&PixelBody>(*target).ok().and_then(|body| {
                        body.hit_segment(
                            transform.inverse_point(motion.previous),
                            transform.inverse_point(motion.position),
                        )
                    });
                    if let Some(local) = hit {
                        let contact = transform.point(local);
                        let t = segment_parameter(motion.previous, motion.position, contact);
                        if nearest.as_ref().map(|n| t < n.0).unwrap_or(true) {
                            nearest = Some((t, *target, local, false, contact));
                        }
                    }
                }
            }
            if bullet.player {
                for (target, m, _owner, scale, radius) in &segments {
                    if bullet.previous_hit == Some(*target) {
                        continue;
                    }
                    if segment_circle(motion.previous, motion.position, m.position, radius + 2.0)
                        .is_some()
                    {
                        let transform = Transform {
                            position: m.position,
                            rotation: m.heading + FRAC_PI_2,
                            scale: Vec2::splat(*scale),
                        };
                        let hit = self.world.get::<&PixelBody>(*target).ok().and_then(|body| {
                            body.hit_segment(
                                transform.inverse_point(motion.previous),
                                transform.inverse_point(motion.position),
                            )
                        });
                        if let Some(local) = hit {
                            let contact = transform.point(local);
                            let t = segment_parameter(motion.previous, motion.position, contact);
                            if nearest.as_ref().map(|n| t < n.0).unwrap_or(true) {
                                nearest = Some((t, *target, local, true, contact));
                            }
                        }
                    }
                }
            }
            if let Some((_, target, local, is_segment, hit_position)) = nearest {
                if let Some(session) = &mut self.co_op {
                    session.hit_owner =
                        self.world.get::<&co_op::ShotOwner>(entity).ok().map(|o| *o);
                }
                if self.co_op.is_some() && self.world.get::<&co_op::Pilot>(target).is_ok() {
                    self.hurt_co_op_pilot(target, bullet.damage, Some(local));
                } else if target == self.player {
                    self.hurt_player(bullet.damage, Some(local));
                } else if is_segment {
                    self.damage_segment(target, bullet.damage, local);
                } else {
                    self.damage_actor(
                        target,
                        bullet.damage,
                        local,
                        motion.velocity.normalize_or_zero() * bullet.damage * 1.2,
                    );
                }
                if let Some(session) = &mut self.co_op {
                    session.hit_owner = None;
                }
                bullet.remaining_hits = bullet.remaining_hits.saturating_sub(1);
                bullet.previous_hit = Some(target);
                self.burst(
                    hit_position,
                    4,
                    if bullet.player {
                        rgba(255, 191, 95, 255)
                    } else {
                        rgba(255, 90, 120, 255)
                    },
                    20.0,
                );
                if bullet.remaining_hits == 0 {
                    let _ = self.world.despawn(entity);
                } else if let Ok(mut b) = self.world.get::<&mut Bullet>(entity) {
                    *b = bullet;
                }
            }
        }
        if self.co_op.is_some() {
            self.co_op_contacts(&actors, &segments);
        } else {
            for (entity, m, a) in &actors {
                if *entity == self.player || !self.world.contains(*entity) {
                    continue;
                }
                let delta = player - m.position;
                let min_distance = a.radius + player_radius;
                if delta.length_squared() < min_distance * min_distance && !self.safe() {
                    let direction = if delta.length_squared() > 0.01 {
                        delta.normalize()
                    } else {
                        Vec2::X
                    };
                    if let Ok(mut p) = self.world.get::<&mut Motion>(self.player) {
                        p.position = m.position + direction * min_distance;
                        p.velocity += direction * 45.0;
                    }
                    if let Ok(mut other) = self.world.get::<&mut Motion>(*entity) {
                        other.velocity -= direction * 25.0;
                    }
                    self.hurt_player(a.damage, None);
                }
            }
            for (entity, m, owner, _scale, segment_radius) in segments {
                if !self.world.contains(entity) {
                    continue;
                }
                let boss = self
                    .world
                    .get::<&Brain>(owner)
                    .map(|b| b.boss)
                    .unwrap_or(false);
                let radius = segment_radius + player_radius;
                if (player - m.position).length_squared() < radius * radius {
                    self.hurt_player(if boss { 13.0 } else { 7.0 }, None);
                }
            }
        }
        // Light object-level asteroid impulses, rather than a physics body per cell.
        let rocks: Vec<_> = actors
            .iter()
            .filter(|(_, _, a)| a.kind == Kind::Asteroid)
            .collect();
        for i in 0..rocks.len() {
            for j in i + 1..rocks.len() {
                let (a, ma, aa) = rocks[i];
                let (b, mb, ab) = rocks[j];
                let delta = mb.position - ma.position;
                let distance = delta.length();
                let overlap = aa.radius + ab.radius - distance;
                if overlap > 0.0 && distance > 0.01 {
                    let direction = delta / distance;
                    let mass_a = (aa.radius * aa.radius).max(1.0);
                    let mass_b = (ab.radius * ab.radius).max(1.0);
                    let inverse_a = 1.0 / mass_a;
                    let inverse_b = 1.0 / mass_b;
                    let mass_sum = inverse_a + inverse_b;
                    let approaching = (mb.velocity - ma.velocity).dot(direction);
                    let impulse = if approaching < 0.0 {
                        -(1.0 + 0.35) * approaching / mass_sum
                    } else {
                        0.0
                    };
                    let correction =
                        direction * (overlap - 0.15).max(0.0) * (1.0 - (-18.0 * dt).exp())
                            / mass_sum;
                    if let Ok(mut m) = self.world.get::<&mut Motion>(*a) {
                        m.velocity -= direction * impulse * inverse_a;
                        m.position -= correction * inverse_a;
                    }
                    if let Ok(mut m) = self.world.get::<&mut Motion>(*b) {
                        m.velocity += direction * impulse * inverse_b;
                        m.position += correction * inverse_b;
                    }
                }
            }
        }
    }

    fn damage_actor(&mut self, entity: Entity, damage: f32, local: Vec2, impulse: Vec2) {
        let damage = damage * self.boss_damage_multiplier(entity, local);
        let Some((motion, actor)) = self
            .world
            .get::<&Motion>(entity)
            .ok()
            .zip(self.world.get::<&Actor>(entity).ok())
            .map(|(m, a)| (*m, *a))
        else {
            return;
        };
        let transform = actor_transform(motion, actor);
        let removed = self
            .world
            .get::<&mut PixelBody>(entity)
            .map(|mut body| {
                body.damage(
                    local,
                    (if matches!(actor.kind, Kind::Boss(_)) {
                        2.4
                    } else {
                        3.0
                    }) / actor.scale,
                )
            })
            .unwrap_or_default();
        for (cell, color) in removed.into_iter().take(24) {
            let p = transform.point(cell);
            let v = impulse * 0.3 + forward(self.random() * TAU) * (8.0 + self.random() * 35.0);
            self.particle(p, v, 0.45, color, actor.scale.min(2.0));
        }
        if let Ok(mut m) = self.world.get::<&mut Motion>(entity) {
            m.velocity = (m.velocity + impulse).clamp_length_max(160.0);
        }
        let empty = self
            .world
            .get::<&PixelBody>(entity)
            .map(|b| b.occupied_count() == 0)
            .unwrap_or(true);
        let dead = if let Ok(mut a) = self.world.get::<&mut Actor>(entity) {
            a.health -= damage;
            a.health <= 0.0 || empty
        } else {
            false
        };
        self.event("hit", transform.point(local), 0.25);
        if dead {
            self.kill_actor(entity, actor.kind, motion.position);
        } else if matches!(actor.kind, Kind::Boss(_)) {
            self.shake = self.shake.max(0.2);
        }
    }

    fn damage_segment(&mut self, entity: Entity, damage: f32, local: Vec2) {
        let position = self
            .world
            .get::<&Motion>(entity)
            .map(|m| m.position)
            .unwrap_or(Vec2::ZERO);
        let owner = self.world.get::<&Segment>(entity).map(|s| s.owner).ok();
        let scale = self
            .world
            .get::<&Segment>(entity)
            .map(|s| s.scale)
            .unwrap_or(1.0);
        let contact = self
            .world
            .get::<&Motion>(entity)
            .map(|motion| {
                Transform {
                    position: motion.position,
                    rotation: motion.heading + FRAC_PI_2,
                    scale: Vec2::splat(scale),
                }
                .point(local)
            })
            .unwrap_or(position);
        self.event("hit", contact, 0.25);
        if let Ok(mut body) = self.world.get::<&mut PixelBody>(entity) {
            body.damage(local, 2.0 / scale);
        }
        let empty = self
            .world
            .get::<&PixelBody>(entity)
            .map(|b| b.occupied_count() == 0)
            .unwrap_or(true);
        let dead = if let Ok(mut segment) = self.world.get::<&mut Segment>(entity) {
            segment.health -= damage;
            segment.health <= 0.0 || empty
        } else {
            false
        };
        if dead {
            let _ = self.world.despawn(entity);
            self.burst(position, 16, rgba(129, 255, 185, 255), 45.0);
        }
        // Breaking armor also weakens the head; the family kill is credited once.
        if let Some(owner) = owner {
            let head_damage =
                damage * 0.6 * self.boss_damage_multiplier(owner, Vec2::splat(1000.0));
            let kind = self.world.get::<&Actor>(owner).map(|a| a.kind).ok();
            let dead = if let Ok(mut actor) = self.world.get::<&mut Actor>(owner) {
                actor.health -= head_damage;
                actor.health <= 0.0
            } else {
                false
            };
            if dead && let Some(kind) = kind {
                let p = self
                    .world
                    .get::<&Motion>(owner)
                    .map(|m| m.position)
                    .unwrap_or(position);
                self.kill_actor(owner, kind, p);
            }
        }
    }

    fn hurt_player(&mut self, damage: f32, local: Option<Vec2>) {
        if self.safe() || self.invulnerability > 0.0 {
            return;
        }
        let position = self.position();
        self.invulnerability = 0.45;
        self.since_damage = 0.0;
        let damage = damage * shield_progression::incoming_damage_multiplier(&self.profile);
        let absorbed = self.shield.min(damage);
        self.shield -= absorbed;
        let hull_damage = damage - absorbed;
        if absorbed > 0.0 {
            self.event("shield", position, 0.8);
            self.burst(position, 12, rgba(102, 197, 255, 230), 55.0);
        }
        if hull_damage > 0.0 {
            if let Ok(mut a) = self.world.get::<&mut Actor>(self.player) {
                a.health -= hull_damage;
            }
            let hit = local.unwrap_or(Vec2::new(0.0, 3.0));
            let removed = self
                .world
                .get::<&mut PixelBody>(self.player)
                .map(|mut b| {
                    b.damage(
                        hit,
                        2.5 / campaign::player_scale(self.profile.campaign.selected_ship),
                    )
                })
                .unwrap_or_default();
            let motion = *self.world.get::<&Motion>(self.player).unwrap();
            let actor = *self.world.get::<&Actor>(self.player).unwrap();
            let transform = actor_transform(motion, actor);
            for (cell, color) in removed {
                let velocity = forward(self.random() * TAU) * 35.0;
                self.particle(transform.point(cell), velocity, 0.6, color, 1.3);
            }
            if self
                .world
                .get::<&PixelBody>(self.player)
                .map(|b| b.occupied_count() == 0)
                .unwrap_or(true)
                && let Ok(mut a) = self.world.get::<&mut Actor>(self.player)
            {
                a.health = 0.0;
            }
        }
        self.shake = self.shake.max(2.0 + damage * 0.04);
        self.event("hit", position, 0.9);
        let nova_rank = ship_progression::family_rank(&self.profile, 1);
        let impact_rank = ship_progression::equipped_rank(&self.profile, 11);
        if (nova_rank > 0 || impact_rank > 0) && self.repulsor_timer <= 0.0 {
            let evolution = bosses::evolution(&self.profile, 1);
            self.repulsor_timer = if impact_rank > 0 {
                7.5 * (1.0 - 0.06 * impact_rank.saturating_sub(1) as f32)
            } else {
                18.0 - 2.0 * campaign::signature_rank(&self.profile, 7).saturating_sub(1) as f32
            };
            if evolution >= 4 {
                self.repulsor_timer *= 0.85;
            }
            self.emit_repulsor_nova(position, evolution, None);
        }
    }

    fn kill_actor(&mut self, entity: Entity, kind: Kind, position: Vec2) {
        if !self.world.contains(entity) {
            return;
        }
        self.record_co_op_destruction(entity, kind);
        let ordinary = self.world.get::<&BossEncounter>(entity).ok().map(|b| *b);
        if self.boss_qa_scene || ordinary.is_some_and(|b| b.diagnostic) {
            self.burst(position, 80, rgba(255, 180, 128, 255), 100.0);
            self.event("explosion", position, 1.0);
            self.shake = self.shake.max(5.0);
            self.remove_enemy(entity, true);
            return;
        }
        if self
            .arena
            .as_ref()
            .is_some_and(|arena| arena.opponent == entity)
        {
            self.remove_enemy(entity, true);
            self.burst(position, 80, rgba(255, 213, 121, 255), 100.0);
            self.event("explosion", position, 1.0);
            self.finish_arena(true, true);
            return;
        }
        if !self.progression_enabled() {
            self.burst(position, 80, rgba(255, 180, 128, 255), 100.0);
            self.event("explosion", position, 1.0);
            self.shake = self.shake.max(5.0);
            self.remove_enemy(entity, true);
            return;
        }
        self.record_kill(kind);
        if let Kind::Enemy(family) | Kind::Boss(family) = kind {
            for quest in &mut self.profile.quests {
                if quest.event == "bounty" && quest.family == family && !quest.completed {
                    quest.progress = quest.progress.saturating_add(1).min(quest.target);
                }
            }
        }
        match kind {
            Kind::Asteroid => {
                let birth = self
                    .world
                    .get::<&AsteroidBirth>(entity)
                    .map(|b| *b)
                    .unwrap_or(AsteroidBirth {
                        source: AsteroidSource::Outer,
                        material_tier: danger(position).clamp(1, 5) as u8,
                        variety: 0,
                        orbit: None,
                    });
                if birth.source != AsteroidSource::HavenBelt {
                    self.award_xp(2);
                }
                self.training_asteroid_defeated();
                if self.random() < 0.25 {
                    let named = (birth.material_tier == 1)
                        .then(|| training::material_for_variety(birth.variety))
                        .flatten();
                    if named.is_some() {
                        self.spawn_tiered_loot(position, 3, birth.material_tier, named, 1);
                    } else {
                        self.spawn_material_loot(position, birth.material_tier, 1);
                    }
                }
                if self.random() < 0.25 {
                    let category = (self.random() * 3.0) as usize;
                    self.spawn_loot(position, category, 1);
                }
            }
            Kind::Enemy(family) => {
                self.award_xp(5);
                self.profile.kills[family] = self.profile.kills[family].saturating_add(1);
                self.kill_credit[family] = self.kill_credit[family].saturating_add(1);
                if self.kill_credit[family] >= 24 && !self.profile.pending[family] {
                    self.kill_credit[family] -= 24;
                    self.profile.pending[family] = true;
                }
                if self.random() < 0.2 {
                    let category = (self.random() * 3.0) as usize;
                    self.spawn_loot(position, category, 1);
                }
                self.changed();
            }
            Kind::Boss(family) => {
                let generation = ordinary.map_or(1, |b| b.generation);
                self.award_xp(25 + 10 * generation.saturating_sub(1) as u32);
                self.profile.boss_generations[family][generation as usize - 1] =
                    self.profile.boss_generations[family][generation as usize - 1]
                        .saturating_add(1);
                self.profile.boss_kills[family] = self.profile.boss_kills[family].saturating_add(1);
                if generation == 1 {
                    self.profile.pending[family] = false;
                }
                self.profile.pity[family] = self.profile.pity[family].saturating_add(1);
                if !self.profile.campaign.abilities[family + 3].blueprint
                    && (self.profile.pity[family] >= 4 || self.random() < 0.25)
                {
                    self.profile.campaign.abilities[family + 3].blueprint = true;
                    ship_progression::mirrors(&mut self.profile);
                    self.event("upgrade", position, 1.0);
                }
                for category in 0..3 {
                    self.spawn_loot(
                        position + forward(category as f32 * TAU / 3.0) * 14.0,
                        category,
                        1,
                    );
                }
                if self.kill_credit[family] >= 24 && !self.profile.pending[family] {
                    self.kill_credit[family] -= 24;
                    self.profile.pending[family] = true;
                }
                if self.active_boss == Some(entity) {
                    self.active_boss = None;
                }
                self.boss_timer = 8.0;
                self.changed();
            }
            Kind::Player => return,
        }
        let boss = matches!(kind, Kind::Boss(_));
        self.burst(
            position,
            if boss { 85 } else { 24 },
            match kind {
                Kind::Asteroid => rgba(204, 165, 123, 255),
                Kind::Enemy(2) | Kind::Boss(2) => rgba(117, 255, 190, 255),
                _ => rgba(255, 149, 102, 255),
            },
            if boss { 110.0 } else { 60.0 },
        );
        self.event("explosion", position, if boss { 1.0 } else { 0.35 });
        self.shake = self.shake.max(if boss { 5.0 } else { 0.55 });
        self.remove_enemy(entity, true);
    }

    fn remove_enemy(&mut self, entity: Entity, killed: bool) {
        let diagnostic = self
            .world
            .get::<&BossEncounter>(entity)
            .ok()
            .is_some_and(|b| b.diagnostic);
        if self.active_boss == Some(entity) {
            self.active_boss = None;
            self.boss_timer = 6.0;
            if !killed && !diagnostic {
                self.changed();
            }
        }
        let children: Vec<_> = self
            .world
            .query::<(Entity, &Segment)>()
            .iter()
            .filter(|(_, s)| s.owner == entity)
            .map(|(e, _)| e)
            .collect();
        for child in children {
            let _ = self.world.despawn(child);
        }
        let _ = self.world.despawn(entity);
    }

    fn spawn_loot(&mut self, position: Vec2, category: usize, amount: u32) {
        self.spawn_tiered_loot(position, category, 1, None, amount);
    }
    fn spawn_material_loot(&mut self, position: Vec2, tier: u8, amount: u32) {
        self.spawn_tiered_loot(position, 3, tier.clamp(1, 5), None, amount);
    }
    fn spawn_tiered_loot(
        &mut self,
        position: Vec2,
        category: usize,
        material_tier: u8,
        training_material: Option<u8>,
        amount: u32,
    ) {
        let training_material =
            training_material.filter(|i| category == 3 && material_tier == 1 && *i < 3);
        let count = self.world.query::<(Entity, &Loot)>().iter().count();
        if count >= 96 {
            // Preserve earned drops under load by merging into the nearest matching pickup.
            let nearest = self
                .world
                .query::<(Entity, (&Motion, &Loot))>()
                .iter()
                .filter(|(_, (_, l))| {
                    l.category == category
                        && l.material_tier == material_tier
                        && l.training_material == training_material
                })
                .min_by(|(_, (a, _)), (_, (b, _))| {
                    (a.position - position)
                        .length_squared()
                        .total_cmp(&(b.position - position).length_squared())
                })
                .map(|(e, _)| e);
            if let Some(e) = nearest {
                if let Ok(mut loot) = self.world.get::<&mut Loot>(e) {
                    loot.amount = loot.amount.saturating_add(amount);
                    loot.life = 120.0;
                }
                return;
            }
        }
        self.world.spawn((
            Motion::new(position, Vec2::ZERO, 0.0),
            Loot {
                category,
                material_tier,
                training_material,
                amount,
                life: 120.0,
            },
        ));
    }

    fn update_pickups(&mut self, player: Vec2, dt: f32) {
        let mut collect = Vec::new();
        let mut remove = Vec::new();
        for (entity, (motion, loot)) in self.world.query_mut::<(Entity, (&mut Motion, &mut Loot))>()
        {
            loot.life -= dt;
            let delta = player - motion.position;
            let distance = delta.length();
            if distance < 12.0 {
                collect.push((
                    entity,
                    loot.category,
                    loot.material_tier,
                    loot.training_material,
                    loot.amount,
                    motion.position,
                ));
            } else if distance < 82.0 {
                motion.velocity = delta.normalize_or_zero() * (110.0 + (82.0 - distance) * 2.0);
            } else {
                motion.velocity *= (-5.0 * dt).exp();
            }
            if loot.life <= 0.0 {
                remove.push(entity);
            }
        }
        for (entity, category, material_tier, training_material, amount, position) in collect {
            if !self.progression_enabled() {
                let _ = self.world.despawn(entity);
                continue;
            }
            if category == 3 {
                let available = 16u32.saturating_sub(ship_progression::total(
                    self.profile.campaign.cargo_materials,
                ));
                let gained = amount.min(available);
                if gained == 0 {
                    continue;
                }
                if let Ok(mut loot) = self.world.get::<&mut Loot>(entity) {
                    loot.amount -= gained;
                }
                self.profile.campaign.cargo_materials[material_tier as usize - 1] += gained;
                if material_tier == 1
                    && let Some(index) = training_material.filter(|i| *i < 3)
                {
                    self.profile.campaign.cargo_training_materials[index as usize] += gained;
                    self.training_collected(index, gained);
                }
                if self.position().length() > SAFE_RADIUS {
                    let run = self.run.get_or_insert_with(RunSummary::default);
                    run.materials_gained = run.materials_gained.saturating_add(gained);
                }
                if gained == amount {
                    let _ = self.world.despawn(entity);
                }
                self.changed();
                self.event("pickup", position, 0.7);
                self.burst(
                    position,
                    8,
                    ship_progression::material_color(material_tier),
                    28.0,
                );
                continue;
            }
            if self.world.despawn(entity).is_ok() {
                self.credit_points(category, amount);
                if amount > 0 {
                    self.discover_basic(category);
                }
                self.event("pickup", position, category as f32 / 2.0);
                self.burst(position, 8, currency_color(category), 28.0);
            }
        }
        for entity in remove {
            let _ = self.world.despawn(entity);
        }
    }

    fn update_wake(&mut self, position: Vec2, dt: f32) {
        let evolution = bosses::evolution(&self.profile, 2);
        let duration = if evolution >= 4 {
            3.4
        } else if evolution >= 2 {
            2.6
        } else {
            2.0
        };
        let reach = if evolution >= 3 { 28.0 } else { 20.0 };
        for (_, life) in &mut self.wake {
            *life -= dt;
        }
        self.wake.retain(|(_, life)| *life > 0.0);
        let wake_rank = ship_progression::family_rank(&self.profile, 2);
        let wind_rank = ship_progression::equipped_rank(&self.profile, 10);
        if (wake_rank == 0 && wind_rank == 0) || self.safe() {
            return;
        }
        self.wake_timer -= dt;
        let motion = *self.world.get::<&Motion>(self.player).unwrap();
        if motion.velocity.length() > Stats::from_profile(&self.profile).speed * 0.5
            && self.wake_timer <= 0.0
        {
            self.wake_timer = if evolution >= 4 { 0.15 } else { 0.2 };
            self.transient_light(
                position - forward(motion.heading) * 14.0,
                48.0,
                [0.23, 1.0, 0.62],
                0.7,
                0.3,
            );
            if self.wake.len()
                >= if evolution >= 4 {
                    24
                } else if evolution >= 2 {
                    16
                } else {
                    12
                }
            {
                self.wake.remove(0);
            }
            self.wake
                .push((position - forward(motion.heading) * 14.0, duration));
            self.event("ability-wake", position, 0.45);
        }
        let damage = 6.0 * (1.0 + 0.25 * wake_rank.max(wind_rank).saturating_sub(1) as f32) * dt;
        let targets: Vec<_> = self
            .world
            .query::<(Entity, (&Motion, &Actor))>()
            .iter()
            .filter(|(_, (m, a))| {
                matches!(a.kind, Kind::Enemy(_) | Kind::Boss(_))
                    && self.safe_zone_at(m.position).is_none()
                    && (self
                        .wake
                        .iter()
                        .any(|(p, _)| (m.position - *p).length() < reach + a.radius)
                        || (evolution >= 5
                            && self.wake.windows(2).any(|pair| {
                                pair[0].0.distance_squared(pair[1].0) <= 160.0 * 160.0
                                    && segment_circle(pair[0].0, pair[1].0, Vec2::ZERO, SAFE_RADIUS)
                                        .is_none()
                                    && self.nearby_sites.iter().all(|site| {
                                        self.arena_disables(site)
                                            || !site.intersects_segment(pair[0].0, pair[1].0)
                                    })
                                    && segment_circle(
                                        pair[0].0,
                                        pair[1].0,
                                        m.position,
                                        reach + a.radius,
                                    )
                                    .is_some()
                            })))
            })
            .map(|(e, (m, a))| (e, a.kind, m.position))
            .collect();
        for (entity, kind, p) in targets {
            let dead = if let Ok(mut a) = self.world.get::<&mut Actor>(entity) {
                a.health -= damage * self.boss_damage_multiplier(entity, Vec2::splat(1000.0));
                a.health <= 0.0
            } else {
                false
            };
            if dead {
                self.kill_actor(entity, kind, p);
            }
        }
    }

    fn particle(&mut self, position: Vec2, velocity: Vec2, life: f32, color: u32, size: f32) {
        self.styled_particle(position, velocity, life, color, size, ParticleStyle::Debris);
    }
    fn styled_particle(
        &mut self,
        position: Vec2,
        velocity: Vec2,
        life: f32,
        color: u32,
        size: f32,
        style: ParticleStyle,
    ) {
        if !position.is_finite()
            || !velocity.is_finite()
            || !life.is_finite()
            || !size.is_finite()
            || life <= 0.0
            || size <= 0.0
        {
            return;
        }
        let life = life.min(3.0);
        if self.particles.len()
            < if self.stress_mode {
                2000
            } else {
                MAX_PARTICLES
            }
        {
            self.cosmetic_sequence = self.cosmetic_sequence.wrapping_add(1);
            let seed = mix(self.profile.seed ^ self.cosmetic_sequence.wrapping_mul(0x9e37_79b9));
            self.particles.push(Particle {
                position,
                origin: position,
                local: Vec2::ZERO,
                previous_local: Vec2::ZERO,
                velocity: velocity.clamp_length_max(1500.0),
                acceleration: if style == ParticleStyle::Fire {
                    Vec2::new(0.0, -12.0)
                } else {
                    Vec2::ZERO
                },
                life,
                initial_life: life,
                color,
                size,
                seed,
                style,
            });
        }
    }
    /// Two actual surviving blue rear emitter cells. Destroyed ports neither
    /// exhaust nor emit a phantom light; lookup never mutates the damage mask.
    fn exhaust_ports(&self) -> [Option<Vec2>; 2] {
        let Ok(body) = self.world.get::<&PixelBody>(self.player) else {
            return [None; 2];
        };
        std::array::from_fn(|index| {
            let target = Vec2::new(
                if index == 0 { -1.0 } else { 1.0 } * body.width as f32 * 0.20,
                body.height as f32 * 0.31,
            );
            body.cells()
                .filter(|(p, c)| {
                    let [r, g, b, _] = c.to_le_bytes();
                    p.distance_squared(target) <= 36.0
                        && p.y > body.height as f32 * 0.1
                        && if self.profile.campaign.selected_ship == 0
                            && self.profile.identity.is_some()
                        {
                            *c == identity::EMITTER
                        } else {
                            b as f32 > r as f32 * 1.05 && g as f32 > r as f32 * 1.05
                        }
                })
                .min_by(|a, b| {
                    a.0.distance_squared(target)
                        .total_cmp(&b.0.distance_squared(target))
                })
                .map(|(p, _)| p)
        })
    }
    fn emit_projectile_trails(&mut self, dt: f32) {
        self.trail_timer -= dt;
        if self.trail_timer > 0.0 {
            return;
        }
        self.trail_timer = 0.05;
        let mut nearest: [Option<(f32, Vec2, Vec2, u32)>; 8] = [None; 8];
        for (motion, bullet) in self.world.query::<(&Motion, &Bullet)>().iter() {
            if bullet.life <= 0.0 || !self.visible(motion.position, 8.0) {
                continue;
            }
            let distance = motion.position.distance_squared(self.camera);
            let entry = (
                distance,
                motion.position,
                motion.velocity,
                if !bullet.player {
                    rgba(248, 87, 106, 160)
                } else if bullet.remaining_hits > 1 {
                    rgba(169, 134, 239, 170)
                } else {
                    rgba(239, 186, 106, 160)
                },
            );
            if let Some(index) = nearest
                .iter()
                .position(|slot| slot.is_none_or(|p| distance < p.0))
            {
                for j in (index + 1..nearest.len()).rev() {
                    nearest[j] = nearest[j - 1];
                }
                nearest[index] = Some(entry);
            }
        }
        for (_, position, velocity, color) in nearest.into_iter().flatten() {
            let behind = -velocity.normalize_or_zero();
            self.styled_particle(
                position + behind * 3.0,
                behind * 18.0,
                0.18,
                color,
                0.7,
                ParticleStyle::Trail,
            );
        }
    }
    fn burst(&mut self, position: Vec2, count: usize, color: u32, speed: f32) {
        for _ in 0..count {
            let angle = self.random() * TAU;
            let velocity = forward(angle) * speed * (0.25 + self.random() * 0.75);
            let life = 0.2 + self.random() * 0.6;
            let size = 1.0 + self.random() * 1.3;
            let [r, g, _, _] = color.to_le_bytes();
            let style = if r as f32 > g as f32 * 1.2 && speed >= 70.0 {
                ParticleStyle::Fire
            } else {
                ParticleStyle::Debris
            };
            self.styled_particle(position, velocity, life, color, size, style);
        }
    }
    fn update_particles(&mut self, dt: f32) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        let dt = dt.min(0.05);
        for p in &mut self.particles {
            p.previous_local = p.local;
            let steps = (dt * 60.0).ceil().max(1.0) as u32;
            let step = dt / steps as f32;
            for _ in 0..steps {
                let age = p.initial_life - p.life;
                let phase = unit(p.seed) * TAU + age * 7.0;
                let curl = if self.reduced_motion || p.style == ParticleStyle::Debris {
                    Vec2::ZERO
                } else {
                    Vec2::new(phase.sin(), (phase * 0.71).cos())
                        * if p.style == ParticleStyle::Trail {
                            9.0
                        } else {
                            24.0
                        }
                };
                let drag: f32 = if p.style == ParticleStyle::Debris {
                    DEBRIS_DRAG
                } else {
                    3.8
                };
                let decay = (-drag * step).exp();
                p.velocity += (p.acceleration + curl) * step;
                p.local += p.velocity * ((1.0 - decay) / drag);
                p.velocity *= decay;
                p.life -= step;
            }
            p.position = p.origin + p.local;
        }
        self.particles.retain(|p| {
            p.life > 0.00001
                && p.position.is_finite()
                && p.local.is_finite()
                && p.velocity.is_finite()
        });
    }

    fn render_particles(&self, frame: &mut FrameBuilder, alpha: f32) -> (usize, usize) {
        let camera = Vec2::from_array(self.render_camera(alpha));
        let mut submitted = 0;
        let mut transforms = 0;
        for (index, p) in self.particles.iter().enumerate() {
            if (self.reduced_cosmetics && index % 2 == 1) || p.initial_life <= 0.0 {
                continue;
            }
            let age = (1.0 - p.life / p.initial_life).clamp(0.0, 1.0);
            // Continuous authored world size, independent of zoom or raster
            // coverage. Debris never grows; soft fire/gas expands from its exact
            // birth size without quantization, a minimum, or threshold pops.
            let size = p.size
                * if p.style == ParticleStyle::Debris {
                    1.0
                } else {
                    1.0 + age * 0.9
                };
            if !size.is_finite() || size <= 0.0 {
                continue;
            }
            let local = p.previous_local.lerp(p.local, alpha);
            let relative = (p.origin - camera) + local;
            if (relative.abs() - self.viewport * 0.5).max_element() > size + 4.0 {
                continue;
            }
            let mut color = p.color.to_le_bytes();
            if p.style == ParticleStyle::Fire {
                let hot = 1.0 - age;
                color[0] = (color[0] as f32 * age + 255.0 * hot) as u8;
                color[1] = (color[1] as f32 * age + 229.0 * hot) as u8;
                color[2] = (color[2] as f32 * age + 170.0 * hot) as u8;
            }
            color[3] = (color[3] as f32
                * (1.0 - age).powi(if p.style == ParticleStyle::Debris {
                    1
                } else {
                    2
                })) as u8;
            // Keep the birth anchor separate from local travel: CameraView
            // subtracts the far-world origin before GPU local-cell arithmetic.
            // One transform per particle preserves arbitrary fractional sizes.
            let transform = Transform {
                position: p.origin,
                scale: Vec2::splat(size),
                ..Transform::default()
            }
            .affine();
            let cells = [(local / size, u32::from_le_bytes(color))];
            let count = if p.style == ParticleStyle::Debris {
                frame.push_cells_affine_material(transform, PixelMaterial::default(), cells)
            } else {
                frame.push_soft_cells_affine_material(
                    transform,
                    PixelMaterial {
                        light_response: 0.0,
                        emission: match p.style {
                            ParticleStyle::Fire => 0.95,
                            ParticleStyle::Exhaust => 0.65,
                            _ => 0.4,
                        },
                    },
                    cells,
                )
            };
            submitted += count;
            transforms += usize::from(count > 0);
        }
        (submitted, transforms)
    }

    fn maintain_stress(&mut self) {
        let position = self.position();
        let missing =
            300_usize.saturating_sub(self.world.query::<(Entity, &Bullet)>().iter().count());
        for _ in 0..missing {
            let angle = self.random() * TAU;
            let offset = forward(angle) * (55.0 + self.random() * 100.0);
            let player = self.random() < 0.5;
            self.spawn_bullet(
                position + offset,
                forward(angle + 0.4) * 130.0,
                4.0,
                player,
                1,
            );
        }
        let missing = 2000_usize.saturating_sub(self.particles.len());
        self.burst(position, missing, rgba(107, 220, 255, 255), 110.0);
    }

    fn enter_dock(&mut self) {
        self.bank_cargo();
        self.finish_run();
        let stats = Stats::from_profile(&self.profile);
        if let Ok(mut actor) = self.world.get::<&mut Actor>(self.player) {
            actor.health = stats.hull;
            actor.max_health = stats.hull;
        }
        let body = identity::player_body(&self.profile);
        let scale = campaign::player_scale(self.profile.campaign.selected_ship);
        if let Ok(mut actor) = self.world.get::<&mut Actor>(self.player) {
            actor.scale = scale;
            actor.radius = pixel_assets::radius(&body, scale);
        }
        let _ = self.world.insert_one(self.player, body);
        self.shield = stats.shield;
        self.event("dock", self.position(), 0.7);
        let enemies: Vec<_> = self
            .world
            .query::<(Entity, &Brain)>()
            .iter()
            .map(|(e, _)| e)
            .collect();
        for entity in enemies {
            self.remove_enemy(entity, false);
        }
        let bullets: Vec<_> = self
            .world
            .query::<(Entity, &Bullet)>()
            .iter()
            .map(|(e, _)| e)
            .collect();
        for entity in bullets {
            let _ = self.world.despawn(entity);
        }
    }

    pub fn reset_to_dock(&mut self) {
        if self.restore_co_op() {
            return;
        }
        self.discard_cargo("abandonment");
        self.boss_qa_scene = false;
        self.material_scene = false;
        self.finish_arena(false, false);
        self.transient_lights.clear();
        self.stress_mode = false;
        let stress: Vec<_> = self
            .world
            .query::<(Entity, &StressObject)>()
            .iter()
            .map(|(e, _)| e)
            .collect();
        for entity in stress {
            let _ = self.world.despawn(entity);
        }
        let enemies: Vec<_> = self
            .world
            .query::<(Entity, &Brain)>()
            .iter()
            .map(|(e, _)| e)
            .collect();
        for entity in enemies {
            self.remove_enemy(entity, false);
        }
        let combat: Vec<_> = self
            .world
            .query::<(Entity, &Bullet)>()
            .iter()
            .map(|(e, _)| e)
            .chain(self.world.query::<(Entity, &Loot)>().iter().map(|(e, _)| e))
            .collect();
        for entity in combat {
            let _ = self.world.despawn(entity);
        }
        if let Ok(mut motion) = self.world.get::<&mut Motion>(self.player) {
            *motion = Motion::new(Vec2::ZERO, Vec2::ZERO, -FRAC_PI_2);
        }
        self.destination = None;
        self.throttle = 0.0;
        self.camera = Vec2::ZERO;
        self.previous_camera = Vec2::ZERO;
        self.expedition = 0.0;
        self.was_safe = true;
        self.invulnerability = 2.0;
        self.wake.clear();
        self.update_exploration();
        self.enter_dock();
        // Keep QA isolation through teleport/cleanup. The next real flight is
        // normal; merely crossing Haven during a fixture does not end isolation.
        self.diagnostic_scene = false;
    }

    pub fn purchase(&mut self, id: &str) -> Result<(), String> {
        let Some(index) = UPGRADES.iter().position(|u| u.0 == id) else {
            return Err("Unknown upgrade.".into());
        };
        self.hardware_check(index)?;
        let ship = self.profile.campaign.selected_ship as usize;
        let next = self.profile.campaign.loadouts[ship].hardware[index] + 1;
        let category = UPGRADES[index].2;
        self.profile.currency[category] -= COST;
        let mut materials = [0; 5];
        materials[next as usize - 1] = 1;
        self.consume_stored_materials(materials);
        self.profile.campaign.loadouts[ship].hardware[index] = next;
        self.progression_changed();
        let stats = Stats::from_profile(&self.profile);
        if let Ok(mut a) = self.world.get::<&mut Actor>(self.player) {
            a.max_health = stats.hull;
            a.health = stats.hull;
        }
        self.shield = stats.shield;
        self.burst(self.position(), 24, currency_color(category), 45.0);
        Ok(())
    }

    pub fn hud(&self) -> Hud {
        let motion = *self.world.get::<&Motion>(self.player).unwrap();
        let actor = *self.world.get::<&Actor>(self.player).unwrap();
        let distance = motion.position.length();
        let stats = Stats::from_profile(&self.profile);
        let tier = danger(motion.position);
        let site = self.current_site().map(|site| self.site_hud(site));
        let zone = if self.arena.is_some() {
            "Champion arena / combat enabled".into()
        } else if site.is_some() {
            "Site sanctuary".into()
        } else if distance <= SAFE_RADIUS {
            "Dock protected".into()
        } else if distance < BELT_INNER {
            "Quiet drift".into()
        } else if distance < COMBAT_RADIUS {
            "Asteroid belt".into()
        } else {
            format!("Outer reach · Tier {tier}")
        };
        let boss = if let Some(arena) = self.arena {
            self.world
                .get::<&Actor>(arena.opponent)
                .ok()
                .map(|a| BossHud {
                    name: campaign::CHAMPIONS[arena.index].name.into(),
                    health: a.health.max(0.0),
                    max_health: a.max_health,
                    family: campaign::CHAMPIONS[arena.index].family,
                    generation: 0,
                    phase: "Arena duel".into(),
                    telegraph: false,
                    position: self
                        .world
                        .get::<&Motion>(arena.opponent)
                        .unwrap()
                        .position
                        .to_array(),
                    core_position: self
                        .world
                        .get::<&Motion>(arena.opponent)
                        .unwrap()
                        .position
                        .to_array(),
                })
        } else {
            self.active_boss.and_then(|entity| {
                self.world.get::<&Actor>(entity).ok().and_then(|a| {
                    let boss = *self.world.get::<&BossEncounter>(entity).ok()?;
                    let family = self.world.get::<&Brain>(entity).ok()?.family;
                    let motion = *self.world.get::<&Motion>(entity).ok()?;
                    Some(BossHud {
                        name: bosses::name(family, boss.generation).into(),
                        health: a.health.max(0.0),
                        max_health: a.max_health,
                        family,
                        generation: boss.generation,
                        phase: boss.label(family).into(),
                        telegraph: boss.phase == bosses::Phase::Tell,
                        position: motion.position.to_array(),
                        core_position: actor_transform(motion, *a)
                            .point(boss_assets::art(family, boss.generation).core_position)
                            .to_array(),
                    })
                })
            })
        };
        Hud {
            position: motion.position.to_array(),
            camera: self.camera.to_array(),
            heading: motion.heading,
            hull: actor.health.max(0.0),
            max_hull: stats.hull,
            shield: self.shield,
            max_shield: stats.shield,
            zone,
            danger: tier,
            safe: self.safe(),
            dock: distance <= SAFE_RADIUS,
            currency: self.profile.currency,
            kills: self.profile.kills,
            boss,
            boss_progress: (0..3)
                .map(|family| bosses::FamilyBossHud {
                    family,
                    defeats: self.profile.boss_generations[family],
                    queued: bosses::queues(&self.profile, family),
                })
                .collect(),
            upgrades: UPGRADES
                .iter()
                .take(6)
                .enumerate()
                .map(|(i, (id, name, category, description))| UpgradeHud {
                    id: (*id).into(),
                    name: (*name).into(),
                    category: *category,
                    rank: if self.profile.campaign.selected_ship == 9 {
                        5
                    } else {
                        self.profile.ranks[i]
                    },
                    effective_rank: ship_progression::hardware_ranks(&self.profile)[i],
                    max_rank: 5,
                    description: if i >= 6 {
                        format!(
                            "{description}. Evolution {}: {}",
                            bosses::evolution(&self.profile, i - 6),
                            bosses::evolution_description(&self.profile, i - 6)
                        )
                    } else {
                        (*description).into()
                    },
                    unlocked: self.profile.campaign.selected_ship == 9
                        || i < 6
                        || self.profile.unlocked[i - 6],
                    required_level: campaign::required_level(
                        self.profile.ranks[i].saturating_add(1),
                    ),
                    evolution: if i < 6 {
                        0
                    } else {
                        bosses::evolution(&self.profile, i - 6)
                    },
                    material_tier: if self.profile.campaign.selected_ship == 9 {
                        5
                    } else {
                        self.profile.ranks[i].saturating_add(1).min(5)
                    },
                    material_cost: u32::from(
                        self.profile.campaign.selected_ship != 9 && self.profile.ranks[i] < 5,
                    ),
                    point_cost: 10,
                    can_purchase: self.hardware_check(i).is_ok(),
                    purchase_reason: self.hardware_check(i).err().unwrap_or_default(),
                })
                .collect(),
            time: self.time,
            shake: self.shake,
            entities: self.world.len() as usize,
            particles: self.particles.len(),
            biome: regions::at(self.profile.seed, motion.position),
            site,
            quest_log: self.profile.quests.clone(),
            discoveries: self.profile.discoveries.len(),
            relics: self.profile.relics.len(),
            campaign: self.campaign_hud(),
            last_run: self.profile.campaign.last_run.clone(),
            ship_level: self.ship_level(),
            ship_xp: self.profile.campaign.ship_xp[self.profile.campaign.selected_ship as usize],
            ship_next_xp: campaign::next_xp(self.ship_level()),
            rank_cap: campaign::rank_cap(self.ship_level()),
            cargo_materials: ship_progression::total(self.profile.campaign.cargo_materials),
            cargo_capacity: 16,
            stored_materials: ship_progression::total(self.profile.campaign.stored_materials),
            material_cargo: self.profile.campaign.cargo_materials,
            material_stash: self.profile.campaign.stored_materials,
            material_training_cargo: self.profile.campaign.cargo_training_materials,
            material_training_stash: self.profile.campaign.stored_training_materials,
            material_loss: self.material_loss,
            material_loss_reason: self.material_loss_reason.clone(),
            abilities: self.ability_hud(),
            components: self.component_hud(),
            loadout: self.loadout_hud(),
            ship_stats: ShipStatsHud {
                speed: motion.velocity.length(),
                top_speed: stats.speed,
                acceleration: stats.acceleration,
                primary_damage: stats.damage,
                fire_rate: stats.fire_rate,
                hull_regen: stats.regeneration,
                shield_regen: stats.shield_regeneration,
                shield_recharge_unlocked: stats.shield_regeneration > 0.0,
                affinity: shield_progression::hud(&self.profile),
            },
            audio: self.audio_hud(),
            navigation: self.navigation_hud(),
            identity: identity::hud(&self.profile),
            onboarding: identity::onboarding_hud(self),
        }
    }

    fn site_hud(&self, site: &regions::Site) -> SiteHud {
        let quest = self
            .profile
            .quests
            .iter()
            .find(|quest| quest.site_id == site.id);
        let archive_full = self.profile.relics.len() >= MAX_ARCHIVE
            || (quest.is_none()
                && (self.profile.quests.len() >= MAX_ARCHIVE
                    || !self.profile.discoveries.contains(&site.id)));
        let status = if self.profile.relics.contains(&site.id) || quest.is_some_and(|q| q.completed)
        {
            "completed"
        } else if archive_full {
            "archive-full"
        } else if let Some(quest) = quest {
            if quest.progress >= quest.target {
                "ready"
            } else {
                "active"
            }
        } else {
            "available"
        };
        let objective = if site.event == "courier" {
            "Carry a relay packet to Haven dock, then return to this beacon. Cargo survives death and reload.".into()
        } else {
            format!(
                "After accepting, defeat {} {} ships. Bosses count as one. Return here to claim.",
                site.target_kills,
                family_name(site.family)
            )
        };
        let progress = quest
            .map(|q| format!(" Progress: {}/{}.", q.progress, q.target))
            .unwrap_or_default();
        SiteHud {
            id: site.id.clone(),
            name: site.name.into(),
            event: site.event.into(),
            description: format!(
                "{objective}{progress} Reward: 3 {} fragments and a relic (+1 maximum hull, up to +20).",
                ["offensive", "defensive", "passive"][site.family]
            ),
            status: status.into(),
            can_interact: status == "available" || status == "ready",
        }
    }

    pub fn render(&self, alpha: f32) -> RenderFrame {
        let mut frame = FrameBuilder::new();
        self.render_into(alpha, &mut frame);
        frame.finish()
    }

    /// Extraction into caller-owned storage avoids reallocating frame batches.
    pub fn render_into(&self, alpha: f32, frame: &mut FrameBuilder) {
        self.render_into_with_home(alpha, frame, |_, _, _| false);
    }

    /// A presentation adapter may replace only the visible home artwork.
    /// Rejection preserves the portable body; all facility/gameplay draw order
    /// and simulation state remain identical to ordinary extraction.
    pub fn render_into_with_home(
        &self,
        alpha: f32,
        frame: &mut FrameBuilder,
        mut draw_home: impl FnMut(&mut FrameBuilder, Transform, f32) -> bool,
    ) {
        let alpha = alpha.clamp(0.0, 1.0);
        frame.clear();
        self.extract_hub_shadow(alpha, frame);
        let lights = [
            pixel_assets::color(6),
            pixel_assets::color(4),
            pixel_assets::color(33),
        ];
        let maximum_player_hull = Stats::from_profile(&self.profile).hull;
        // Spawn/dock grace exceeds the hit window; don't replay the previous
        // ship's damage tint on the freshly repaired/replaced body.
        let recent_player_damage = if self.invulnerability > 0.45 {
            0.45
        } else {
            self.since_damage
        };
        self.render_background(frame, Vec2::from_array(self.render_camera(alpha)));
        regions::background_with_density(
            frame,
            self.profile.seed,
            Vec2::from_array(self.render_camera(alpha)),
            self.viewport,
            self.cosmetic_time(),
            self.reduced_cosmetics,
        );
        self.render_sites(frame, lights);
        self.render_campaign(frame);
        // The generated station includes its four arms and its open docking bay.
        let station = Transform {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::splat(DOCK_SCALE),
        };
        if self.visible(Vec2::ZERO, 100.0) && !draw_home(frame, station, self.cosmetic_time()) {
            frame.clear_planar_shadow();
            frame.push_body_colored_material(
                &self.dock_body,
                station,
                ACTOR_MATERIAL,
                |cell, color| {
                    animate_station_cell(
                        cell,
                        identity::dock_color(&self.profile, color),
                        self.cosmetic_time(),
                        lights.map(|c| identity::dock_color(&self.profile, c)),
                    )
                },
            );
        }
        for i in 0..96 {
            if !self.visible(Vec2::ZERO, SAFE_RADIUS + 3.0) {
                break;
            }
            let angle = i as f32 * TAU / 96.0;
            let pulse = (self.cosmetic_time() * 1.8 + i as f32 * 0.18).sin() * 0.5 + 0.5;
            let color = if let Some(corp) = identity::main_corporation(&self.profile) {
                let [r, g, b] = corp.color();
                rgba(r, g, b, if i % 4 == 0 { 165 } else { 70 })
            } else {
                rgba(
                    45,
                    110 + (pulse * 30.0) as u8,
                    145 + (pulse * 40.0) as u8,
                    if i % 4 == 0 { 165 } else { 70 },
                )
            };
            frame.push_pixel(
                forward(angle) * SAFE_RADIUS,
                if i % 4 == 0 { 2.0 } else { 1.0 },
                color,
            );
        }
        dock_facilities::render(
            frame,
            Vec2::from_array(self.render_camera(alpha)),
            self.viewport,
            self.cosmetic_time(),
            identity::main_corporation(&self.profile),
        );
        identity::render_crest(
            frame,
            &self.profile,
            Vec2::from_array(self.render_camera(alpha)),
            self.viewport,
            self.cosmetic_time(),
        );
        co_op_wake_fx::append(
            frame,
            self.wake.iter().copied(),
            self.cosmetic_time(),
            |p, r| self.visible(p, r),
        );
        if let Some(session) = &self.co_op {
            let points = self.co_op_wake_points();
            let phase = if self.reduced_motion {
                0.0
            } else {
                session.tick as f32 / 60.0
            };
            co_op_wake_fx::append(
                frame,
                points.iter().map(|p| {
                    (
                        Vec2::new(p.position[0] as f32, p.position[1] as f32),
                        p.life as f32,
                    )
                }),
                phase,
                |p, r| self.visible(p, r),
            );
        }
        for (entity, (motion, actor, body, boss, riveter, mote)) in self
            .world
            .query::<(
                Entity,
                (
                    &Motion,
                    &Actor,
                    &PixelBody,
                    Option<&BossEncounter>,
                    Option<&biome_enemies::Riveter>,
                    Option<&radiation_enemies::Mote>,
                ),
            )>()
            .iter()
        {
            let mut m = *motion;
            m.position = motion.previous.lerp(motion.position, alpha);
            m.heading = interpolate_heading(motion.previous_heading, motion.heading, alpha);
            let cue_extent = if mote.is_some() {
                radiation_enemies::WARNING_REACH + 2.0
            } else {
                10.0
            };
            if !self.visible(m.position, actor.radius + cue_extent) {
                continue;
            }
            if let Some(boss) = boss
                && let Kind::Boss(family) = actor.kind
            {
                self.draw_boss(frame, body, m, *actor, *boss, family);
                continue;
            }
            if let Some(mote) = mote {
                self.draw_mote(frame, body, m, *actor, *mote);
                continue;
            }
            if let Some(riveter) = riveter {
                self.draw_riveter(frame, body, m, *actor, *riveter);
                continue;
            }
            let pilot = self.world.get::<&co_op::Pilot>(entity).ok();
            let player_hull = pilot.as_ref().map_or(maximum_player_hull, |p| p.stats.hull);
            let damage_time = pilot.as_ref().map_or(recent_player_damage, |p| {
                if p.invulnerability > 0.45 {
                    0.45
                } else {
                    p.since_damage
                }
            });
            let corporation = pilot.as_ref().map_or_else(
                || self.profile.identity.as_ref().map(|id| id.corporation),
                |p| p.spec.identity.as_ref().map(|id| id.corporation),
            );
            let selected_ship = pilot
                .as_ref()
                .map_or(self.profile.campaign.selected_ship, |p| p.selected_ship);
            let shield = pilot.as_ref().map_or(self.shield, |p| p.shield);
            let player_tint = |color| {
                player_presentation::tint(
                    color,
                    actor.health,
                    player_hull,
                    damage_time,
                    self.cosmetic_time(),
                    self.reduced_motion,
                )
            };
            let coarse_color = |cell, color| {
                let color = animate_actor_cell(cell, color, self.cosmetic_time(), lights);
                if actor.kind == Kind::Player {
                    player_tint(color)
                } else {
                    color
                }
            };
            let elevated = entity == self.player
                && frame
                    .frame()
                    .shadows
                    .as_ref()
                    .is_some_and(|s| s.caster.is_some());
            frame.with_object_meta(
                engine_core::ObjectMeta {
                    height: if elevated {
                        hub_shadows::RAICHO_HEIGHT
                    } else {
                        0.0
                    },
                    flags: 0,
                },
                |frame| {
                    let fine_drawn = actor.kind == Kind::Player
                        && selected_ship == 0
                        && corporation.is_some_and(|corporation| {
                            raicho_fine::append(
                                frame,
                                body,
                                corporation,
                                raicho_fine::DrawInput {
                                    parent: actor_transform(m, *actor).affine(),
                                    material: ACTOR_MATERIAL,
                                    time: self.cosmetic_time(),
                                    reduced_motion: self.reduced_motion,
                                },
                                &mut raicho_fine::Budget::default(),
                                coarse_color,
                                |_, color| player_tint(color),
                            )
                            .is_ok()
                        });
                    if !fine_drawn {
                        frame.push_body_colored_material(
                            body,
                            actor_transform(m, *actor),
                            ACTOR_MATERIAL,
                            coarse_color,
                        );
                    }
                },
            );
            if actor.kind == Kind::Player {
                let glow = (self.cosmetic_time() * 24.0).sin() * 0.5 + 0.5;
                if entity == self.player && self.throttle > 0.03 {
                    let transform = actor_transform(m, *actor);
                    for port in self.exhaust_ports().into_iter().flatten() {
                        frame.push_soft_cells_affine_material(
                            transform.affine().compose(
                                Transform {
                                    position: port,
                                    scale: Vec2::splat(1.8),
                                    ..Transform::default()
                                }
                                .affine(),
                            ),
                            PixelMaterial {
                                light_response: 0.0,
                                emission: 0.9,
                            },
                            (0..3).map(|i| {
                                (
                                    Vec2::new(0.0, i as f32 * 0.8),
                                    identity::jet_glow_color(
                                        &self.profile,
                                        ((0.65 + glow * 0.35) * 230.0 * self.throttle
                                            / (1.0 + i as f32 * 0.5))
                                            as u8,
                                    ),
                                )
                            }),
                        );
                    }
                }
                if shield > 0.0 {
                    for i in 0..24 {
                        let angle = i as f32 * TAU / 24.0 + self.cosmetic_time() * 0.3;
                        frame.push_pixel(
                            m.position + forward(angle) * (actor.radius + 3.0),
                            1.0,
                            rgba(92, 181, 255, if damage_time < 0.3 { 210 } else { 65 }),
                        );
                    }
                }
            }
        }
        let segment_poses = self.segment_render_poses(alpha);
        for (entity, (motion, segment, body)) in self
            .world
            .query::<(Entity, (&Motion, &Segment, &PixelBody))>()
            .iter()
        {
            let (position, heading) = segment_poses.get(&entity).copied().unwrap_or((
                motion.previous.lerp(motion.position, alpha),
                interpolate_heading(motion.previous_heading, motion.heading, alpha),
            ));
            if !self.visible(position, segment.radius) {
                continue;
            }
            let scale = segment.scale;
            let transform = Transform {
                position,
                rotation: heading + FRAC_PI_2,
                scale: Vec2::splat(scale),
            };
            if let Ok(boss) = self.world.get::<&BossEncounter>(segment.owner) {
                boss_assets::segment_art(boss.generation).rig.draw(
                    frame,
                    body,
                    transform,
                    boss.rig_state(self.cosmetic_time(), 2),
                );
                continue;
            }
            frame.push_body_colored_material(body, transform, ACTOR_MATERIAL, |cell, color| {
                animate_actor_cell(cell, color, self.cosmetic_time(), lights)
            });
        }
        for (_, (motion, bullet)) in self.world.query::<(Entity, (&Motion, &Bullet))>().iter() {
            let p = motion.previous.lerp(motion.position, alpha);
            if !self.visible(p, 8.0) {
                continue;
            }
            let d = motion.velocity.normalize_or_zero();
            let color = if bullet.player {
                if bullet.remaining_hits > 1 {
                    rgba(191, 167, 255, 255)
                } else {
                    rgba(255, 237, 156, 255)
                }
            } else {
                rgba(255, 93, 129, 255)
            };
            frame.push_pixel(p, if bullet.player { 2.0 } else { 3.0 }, color);
            frame.push_pixel(p - d * 3.0, 1.5, with_alpha(color, 130));
            frame.push_pixel(p - d * 6.0, 1.0, with_alpha(color, 55));
        }
        for (_, (motion, loot)) in self.world.query::<(Entity, (&Motion, &Loot))>().iter() {
            let p = motion.previous.lerp(motion.position, alpha)
                + Vec2::new(
                    0.0,
                    (self.cosmetic_time() * 3.0 + motion.position.x).sin() * 2.0,
                );
            if !self.visible(p, 8.0) {
                continue;
            }
            if loot.category == 3 {
                if let Some(index) = loot.training_material {
                    training::draw_material(frame, p, index);
                } else {
                    ship_progression::draw_material(frame, p, loot.material_tier);
                }
                continue;
            }
            let color = currency_color(loot.category);
            frame.push_pixel(p, 3.0, color);
            for d in [Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y] {
                frame.push_pixel(p + d * 3.0, 1.5, with_alpha(color, 180));
            }
            frame.push_pixel(p + Vec2::new(-0.6, -0.6), 1.0, rgba(255, 255, 245, 255));
        }
        self.render_particles(frame, alpha);
        if let Some(target) = self.destination
            && self.visible(target, 8.0)
        {
            for i in 0..4 {
                let direction = forward(i as f32 * FRAC_PI_2 + self.cosmetic_time() * 0.8);
                frame.push_pixel(target + direction * 6.0, 1.2, rgba(160, 230, 255, 130));
            }
        }
    }

    fn render_sites(&self, frame: &mut FrameBuilder, lights: [u32; 3]) {
        for site in &self.nearby_sites {
            let position = Vec2::from_array(site.position);
            if !self.visible(position, site.radius) {
                continue;
            }
            let quest = self
                .profile
                .quests
                .iter()
                .find(|quest| quest.site_id == site.id);
            let claimed = self.profile.relics.contains(&site.id)
                || quest.is_some_and(|quest| quest.completed);
            let ready =
                quest.is_some_and(|quest| !quest.completed && quest.progress >= quest.target);
            let signal = if claimed {
                rgba(172, 155, 97, 255)
            } else if ready {
                rgba(235, 255, 224, 255)
            } else {
                currency_color(site.family)
            };
            frame.push_body_colored_material(
                &self.dock_body,
                Transform {
                    position,
                    rotation: 0.0,
                    scale: Vec2::splat(1.05),
                },
                ACTOR_MATERIAL,
                |cell, color| {
                    if color == lights[0] {
                        pulse_color(
                            signal,
                            1.0 + (self.cosmetic_time() * 2.0 + cell.y * 0.15).sin() * 0.12,
                        )
                    } else {
                        animate_station_cell(cell, color, self.cosmetic_time(), lights)
                    }
                },
            );
            for index in 0..64 {
                let angle = index as f32 * TAU / 64.0;
                frame.push_pixel(
                    position + forward(angle) * site.radius,
                    if index % 8 == 0 { 2.0 } else { 1.0 },
                    with_alpha(signal, if index % 8 == 0 { 145 } else { 55 }),
                );
            }
            // Four local transponder signals orbit the generated beacon as child
            // objects; each keeps its own pixel transform relative to the site.
            let parent = Transform {
                position,
                rotation: self.cosmetic_time() * 0.12,
                scale: Vec2::ONE,
            };
            for index in 0..4 {
                let child = Transform {
                    position: forward(index as f32 * FRAC_PI_2) * 55.0,
                    rotation: -self.cosmetic_time() * 0.12,
                    scale: Vec2::ONE,
                };
                let transform = parent.compose(child);
                for local in [Vec2::ZERO, Vec2::new(-1.0, 1.0), Vec2::new(1.0, 1.0)] {
                    frame.push_pixel(transform.point(local), 1.0, signal);
                }
            }
        }
    }

    fn render_background(&self, frame: &mut FrameBuilder, camera: Vec2) {
        cosmic_background::render(
            frame,
            cosmic_background::SceneInput {
                seed: self.profile.seed,
                camera,
                viewport: self.viewport,
                time: self.cosmetic_time(),
                reduced: self.reduced_cosmetics,
                reduced_motion: self.reduced_motion,
            },
        );
    }

    /// Explicit deterministic QA scenes. Their ticks never earn progression;
    /// home/reset ends this transient isolation. Unknown commands are no-ops.
    pub fn diagnostic(&mut self, command: &str) -> bool {
        if self.co_op.is_some() {
            return false;
        }
        if matches!(command, "dock" | "home") {
            self.reset_to_dock();
            return true;
        }
        let previous = self.diagnostic_scene;
        // Cover reset's immediate exploration/banking before a scene teleports.
        // Invalid commands restore only this transient bit and mutate no run.
        self.diagnostic_scene = true;
        let accepted = self.diagnostic_inner(command);
        self.diagnostic_scene = if accepted { true } else { previous };
        if accepted && command != "material" {
            self.material_scene = false;
        }
        accepted
    }

    fn diagnostic_inner(&mut self, command: &str) -> bool {
        if command == "material" {
            return self.begin_material_scene();
        }
        if let Some(destination) = command.strip_prefix("warp ") {
            let mut coordinates = destination.split_whitespace();
            let x = coordinates
                .next()
                .and_then(|value| value.parse::<f32>().ok());
            let y = coordinates
                .next()
                .and_then(|value| value.parse::<f32>().ok());
            if let (Some(x), Some(y), None) = (x, y, coordinates.next()) {
                let position = Vec2::new(x, y);
                if position.is_finite() && position.abs().max_element() <= WORLD_LIMIT {
                    self.reset_to_dock();
                    self.boss_qa_scene = true;
                    self.diagnostic_scene = true;
                    self.move_player(position);
                    self.update_chunks(position);
                    return true;
                }
            }
            return false;
        }
        if self.mote_diagnostic(command) {
            self.diagnostic_scene = true;
            return true;
        }
        if self.biome_enemy_diagnostic(command) {
            self.diagnostic_scene = true;
            return true;
        }
        if self.boss_diagnostic(command) {
            self.diagnostic_scene = true;
            return true;
        }
        if self.campaign_diagnostic(command) {
            self.diagnostic_scene = true;
            return true;
        }
        match command {
            "site" => {
                self.reset_to_dock();
                self.diagnostic_scene = true;
                let site = regions::sites_near(self.profile.seed, Vec2::ZERO)
                    .into_iter()
                    .min_by(|a, b| {
                        Vec2::from_array(a.position)
                            .length_squared()
                            .total_cmp(&Vec2::from_array(b.position).length_squared())
                    });
                if let Some(site) = site {
                    self.move_player(Vec2::from_array(site.position));
                    self.update_exploration();
                }
            }
            "belt" => {
                self.reset_to_dock();
                self.diagnostic_scene = true;
                self.move_player(Vec2::new(950.0, 0.0));
                self.update_chunks(self.position());
            }
            "die" => {
                self.diagnostic_scene = true;
                if let Ok(mut a) = self.world.get::<&mut Actor>(self.player) {
                    a.health = 0.0;
                }
            }
            "damage" => {
                self.reset_to_dock();
                self.diagnostic_scene = true;
                self.move_player(Vec2::new(900.0, 0.0));
                self.invulnerability = 0.0;
                self.hurt_player(32.0, Some(Vec2::new(-3.0, 2.0)));
            }
            "combat" => {
                self.reset_to_dock();
                self.diagnostic_scene = true;
                self.move_player(Vec2::new(1600.0, 0.0));
                for family in 0..3 {
                    for n in 0..3 {
                        let angle = family as f32 * TAU / 3.0 + n as f32 * 0.25;
                        self.spawn_enemy(
                            family,
                            false,
                            self.position() + forward(angle) * (150.0 + n as f32 * 30.0),
                            1,
                        );
                    }
                }
            }
            "stress" => {
                self.reset_to_dock();
                self.diagnostic_scene = true;
                self.move_player(Vec2::new(2400.0, 0.0));
                self.stress_mode = true;
                self.invulnerability = 120.0;
                for i in 0..160 {
                    let angle = i as f32 * TAU / 160.0;
                    let offset = Vec2::new(
                        angle.cos() * self.viewport.x * 0.3,
                        angle.sin() * self.viewport.y * 0.3,
                    );
                    let enemy = self.spawn_enemy(i % 3, false, self.position() + offset, 2);
                    if let Ok(mut a) = self.world.get::<&mut Actor>(enemy) {
                        a.health = 100_000.0;
                        a.max_health = 100_000.0;
                    }
                }
                self.maintain_stress();
                let missing = 80_000_usize.saturating_sub(self.render(1.0).pixel_count);
                let cells = asteroid_body(self.profile.seed).occupied_count();
                for i in 0..missing.div_ceil(cells) {
                    let angle = i as f32 * 2.399_963;
                    let offset = forward(angle) * (45.0 + (i % 5) as f32 * 20.0);
                    let body = asteroid_body(self.profile.seed.wrapping_add(i as u32));
                    self.world.spawn((
                        Motion::new(self.position() + offset, Vec2::ZERO, angle),
                        Actor {
                            kind: Kind::Asteroid,
                            health: 100_000.0,
                            max_health: 100_000.0,
                            radius: pixel_assets::radius(&body, asteroid_art::CELL_SCALE),
                            scale: asteroid_art::CELL_SCALE,
                            damage: 8.0,
                        },
                        body,
                        StressObject,
                    ));
                }
            }
            _ => return false,
        }
        true
    }
    fn move_player(&mut self, position: Vec2) {
        if let Ok(mut m) = self.world.get::<&mut Motion>(self.player) {
            m.position = position;
            m.previous = position;
            m.velocity = Vec2::ZERO;
        }
        self.camera = position;
        self.previous_camera = position;
        self.was_safe = false;
        self.refresh_sites();
    }
}

fn region_grid(position: Vec2) -> [i32; 2] {
    let grid = (position / regions::REGION_SIZE).floor();
    [grid.x as i32, grid.y as i32]
}

/// Newly streamed chains trail away from the player, rather than crossing the
/// camera center before the head has entered its telegraphed attack cycle.
fn initial_chain_direction(head: Vec2, player: Vec2) -> Vec2 {
    let direction = (head - player).normalize_or_zero();
    if direction.length_squared() > 0.0 {
        direction
    } else {
        Vec2::Y
    }
}

fn family_name(family: usize) -> &'static str {
    ["Dart", "Bastion", "Coil"][family]
}

fn weighted_family(weights: [u8; 3], roll: f32) -> usize {
    let total: u32 = weights.iter().map(|&weight| weight as u32).sum();
    let mut ticket = roll.clamp(0.0, 0.999_999) * total as f32;
    for (family, weight) in weights.into_iter().enumerate() {
        if ticket < weight as f32 {
            return family;
        }
        ticket -= weight as f32;
    }
    2
}

fn sanitize_archive(profile: &mut Profile) -> bool {
    fn ids(seed: u32, values: &mut Vec<String>) -> bool {
        let length = values.len();
        let mut unique = HashSet::new();
        values.retain(|id| {
            unique.len() < MAX_ARCHIVE
                && regions::site_by_id(seed, id).is_some()
                && unique.insert(id.clone())
        });
        length != values.len()
    }
    let mut changed = ids(profile.seed, &mut profile.discoveries);
    changed |= ids(profile.seed, &mut profile.relics);
    let mut unique = HashSet::new();
    let length = profile.quests.len();
    profile.quests.retain_mut(|quest| {
        let Some(site) = regions::site_by_id(profile.seed, &quest.site_id) else {
            return false;
        };
        let target = if site.event == "courier" {
            1
        } else {
            site.target_kills
        };
        if unique.len() >= MAX_ARCHIVE
            || quest.event != site.event
            || quest.family != site.family
            || quest.target != target
            || !unique.insert(quest.site_id.clone())
        {
            return false;
        }
        let completed = quest.completed || profile.relics.contains(&quest.site_id);
        let progress = if completed {
            target
        } else {
            quest.progress.min(target)
        };
        changed |= completed != quest.completed || progress != quest.progress;
        quest.completed = completed;
        quest.progress = progress;
        true
    });
    changed || length != profile.quests.len()
}

fn push_out_of_sanctuaries(
    position: &mut Vec2,
    radius: f32,
    sites: &[regions::Site],
) -> Option<Vec2> {
    let mut outward = None;
    for (center, protected_radius) in std::iter::once((Vec2::ZERO, SAFE_RADIUS)).chain(
        sites
            .iter()
            .map(|site| (Vec2::from_array(site.position), site.radius)),
    ) {
        let delta = *position - center;
        let distance = delta.length();
        let boundary = protected_radius + radius + 6.0;
        if distance < boundary {
            let direction = if distance > 0.001 {
                delta / distance
            } else {
                Vec2::X
            };
            *position = center + direction * boundary;
            outward = Some(direction);
        }
    }
    outward
}

fn danger(position: Vec2) -> u32 {
    if position.length() < COMBAT_RADIUS {
        0
    } else {
        1 + ((position.length() - COMBAT_RADIUS) / 1600.0).floor() as u32
    }
}
fn forward(angle: f32) -> Vec2 {
    Vec2::new(angle.cos(), angle.sin())
}
fn turn_toward(current: f32, target: f32, max_delta: f32) -> f32 {
    let delta = (target - current + PI).rem_euclid(TAU) - PI;
    current + delta.clamp(-max_delta, max_delta)
}
fn interpolate_heading(previous: f32, current: f32, alpha: f32) -> f32 {
    previous + ((current - previous + PI).rem_euclid(TAU) - PI) * alpha
}
fn animate_station_cell(cell: Vec2, color: u32, time: f32, lights: [u32; 3]) -> u32 {
    let wave = (time * 2.8 - cell.y * 0.18 - cell.x * 0.1).sin();
    if color == lights[0] || color == lights[1] {
        pulse_color(color, 1.0 + wave * 0.12)
    } else {
        color
    }
}
fn animate_actor_cell(cell: Vec2, color: u32, time: f32, lights: [u32; 3]) -> u32 {
    if lights.contains(&color) {
        let wave = (time * 5.0 + cell.y * 0.2 + cell.x * 0.1).sin();
        pulse_color(color, 1.0 + wave * 0.1)
    } else {
        color
    }
}
fn pulse_color(color: u32, brightness: f32) -> u32 {
    let [red, green, blue, alpha] = color.to_le_bytes();
    rgba(
        (red as f32 * brightness).min(255.0) as u8,
        (green as f32 * brightness).min(255.0) as u8,
        (blue as f32 * brightness).min(255.0) as u8,
        alpha,
    )
}

fn actor_transform(motion: Motion, actor: Actor) -> Transform {
    Transform {
        position: motion.position,
        rotation: motion.heading + FRAC_PI_2,
        scale: Vec2::splat(actor.scale),
    }
}
fn rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    u32::from_le_bytes([r, g, b, a])
}
fn with_alpha(color: u32, alpha: u8) -> u32 {
    (color & 0x00ff_ffff) | ((alpha as u32) << 24)
}
fn currency_color(category: usize) -> u32 {
    [
        rgba(255, 170, 97, 255),
        rgba(115, 184, 255, 255),
        rgba(131, 242, 184, 255),
        rgba(255, 222, 148, 255),
    ][category]
}
fn mix(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846ca68b);
    x ^ (x >> 16)
}
fn hash3(seed: u32, x: i32, y: i32, index: u32) -> u32 {
    mix(seed
        ^ (x as u32).wrapping_mul(0x9e3779b9)
        ^ (y as u32).wrapping_mul(0x85ebca6b)
        ^ index.wrapping_mul(0xc2b2ae35))
}
fn unit(x: u32) -> f32 {
    (x >> 8) as f32 / 16777216.0
}
fn segment_parameter(a: Vec2, b: Vec2, point: Vec2) -> f32 {
    let delta = b - a;
    if delta.length_squared() > 0.00001 {
        ((point - a).dot(delta) / delta.length_squared()).clamp(0.0, 1.0)
    } else {
        0.0
    }
}
fn segment_circle(a: Vec2, b: Vec2, center: Vec2, radius: f32) -> Option<f32> {
    let d = b - a;
    let length = d.length_squared();
    let t = if length > 0.00001 {
        ((center - a).dot(d) / length).clamp(0.0, 1.0)
    } else {
        0.0
    };
    if (a + d * t - center).length_squared() <= radius * radius {
        Some(t)
    } else {
        None
    }
}

fn asteroid_body(seed: u32) -> PixelBody {
    pixel_assets::asteroid(seed)
}
fn enemy_body(family: usize, boss: bool) -> PixelBody {
    pixel_assets::body(match (family, boss) {
        (0, true) => "dart_boss",
        (0, false) => "dart",
        (1, _) => "bastion",
        _ => "coil_head",
    })
}
fn coil_segment_body(_boss: bool) -> PixelBody {
    pixel_assets::body("coil_segment")
}
fn segment_scale(boss: bool) -> f32 {
    if boss { 1.35 } else { 0.85 }
}
fn dock_body() -> PixelBody {
    pixel_assets::body("dock")
}
#[cfg(test)]
mod tests {
    use super::*;
    fn advance(game: &mut Game, input: &InputFrame, seconds: f32) {
        for _ in 0..(seconds * 60.0) as usize {
            game.tick(input, 1.0 / 60.0);
        }
    }
    #[test]
    fn joystick_rotates_scales_and_releases() {
        let mut g = Game::new(Profile::default());
        advance(
            &mut g,
            &InputFrame {
                joystick: Some([1.0, 0.0]),
                ..Default::default()
            },
            1.0,
        );
        let m = *g.world.get::<&Motion>(g.player).unwrap();
        assert!(m.velocity.x > Stats::from_profile(&g.profile).speed * 0.9);
        assert!(m.heading.abs() < 0.03);
        let mut half = Game::new(Profile::default());
        advance(
            &mut half,
            &InputFrame {
                joystick: Some([0.5, 0.0]),
                ..Default::default()
            },
            1.0,
        );
        let half_velocity = half.world.get::<&Motion>(half.player).unwrap().velocity.x;
        // Half travel gives ~46% thrust after the controller's center dead zone.
        assert!((0.44..0.48).contains(&(half_velocity / m.velocity.x)));
        advance(&mut g, &InputFrame::default(), 1.0);
        let m = *g.world.get::<&Motion>(g.player).unwrap();
        assert!(m.velocity.length() < 1.0);
        assert_eq!(g.throttle, 0.0);
    }
    #[test]
    fn navigation_arrives_without_orbiting() {
        let mut g = Game::new(Profile::default());
        g.tick(
            &InputFrame {
                destination: Some([170.0, 90.0]),
                ..Default::default()
            },
            1.0 / 60.0,
        );
        advance(&mut g, &InputFrame::default(), 5.0);
        assert!((g.position() - Vec2::new(170.0, 90.0)).length() < 5.0);
        assert!(g.world.get::<&Motion>(g.player).unwrap().velocity.length() < 1.0);
    }
    #[test]
    fn twenty_four_family_kills_queue_one_boss() {
        let mut g = Game::new(Profile::default());
        for _ in 0..24 {
            let e = g.spawn_enemy(0, false, Vec2::new(1600.0, 0.0), 1);
            g.kill_actor(e, Kind::Enemy(0), Vec2::ZERO);
        }
        assert_eq!(g.profile.kills, [24, 0, 0]);
        assert!(g.profile.pending[0]);
        assert_eq!(g.kill_credit[0], 0);
    }
    #[test]
    fn safe_zone_blocks_damage_and_weapons() {
        let mut g = Game::new(Profile::default());
        g.hurt_player(1000.0, None);
        advance(
            &mut g,
            &InputFrame {
                fire: true,
                ..Default::default()
            },
            1.0,
        );
        assert_eq!(g.hud().hull, 100.0);
        assert_eq!(g.world.query::<(Entity, &Bullet)>().iter().count(), 0);
    }
    #[test]
    fn purchases_use_correct_currency_and_require_dock() {
        let p = Profile {
            currency: [20, 10, 10],
            ..Default::default()
        };
        let mut g = Game::new(p);
        g.profile.campaign.stored_materials[0] = 1;
        assert!(g.purchase("damage").is_ok());
        assert_eq!(g.profile.currency, [10, 10, 10]);
        assert_eq!(g.profile.ranks[0], 1);
        g.move_player(Vec2::new(500.0, 0.0));
        assert!(g.purchase("speed").is_err());
        assert_eq!(g.profile.currency[2], 10);
    }
    #[test]
    fn death_retains_progress_and_retries_boss() {
        let p = Profile {
            currency: [7, 8, 9],
            kills: [24, 0, 0],
            pending: [true, false, false],
            ..Default::default()
        };
        let mut g = Game::new(p);
        g.diagnostic("boss-dart");
        g.diagnostic("die");
        g.tick(&InputFrame::default(), 1.0 / 60.0);
        assert_eq!(g.profile.currency, [7, 8, 9]);
        assert!(g.profile.pending[0]);
        assert_eq!(g.position(), Vec2::ZERO);
        assert_eq!(g.hud().hull, 100.0);
        assert!(g.active_boss.is_none());
    }
    #[test]
    fn fourth_boss_guarantees_signature() {
        let mut g = Game::new(Profile::default());
        for _ in 0..4 {
            let e = g.spawn_enemy(2, true, Vec2::new(1800.0, 0.0), 1);
            g.kill_actor(e, Kind::Boss(2), Vec2::new(1800.0, 0.0));
        }
        assert!(g.profile.unlocked[2]);
        assert_eq!(g.profile.ranks[8], 0); // blueprint must still be crafted/equipped
        assert_eq!(g.profile.boss_kills[2], 4);
    }
    #[test]
    fn pixel_hits_remove_cells_and_empty_objects_die() {
        let mut g = Game::new(Profile::default());
        let e = g.spawn_enemy(0, false, Vec2::new(1700.0, 0.0), 1);
        let before = g.world.get::<&PixelBody>(e).unwrap().occupied_count();
        g.damage_actor(e, 1.0, Vec2::ZERO, Vec2::ZERO);
        assert!(g.world.get::<&PixelBody>(e).unwrap().occupied_count() < before);
        g.damage_actor(e, 1000.0, Vec2::ZERO, Vec2::ZERO);
        assert!(!g.world.contains(e));
        assert_eq!(g.profile.kills[0], 1);
    }
    #[test]
    fn streaming_is_deterministic_and_bounded() {
        let mut a = Game::new(Profile::default());
        let mut b = Game::new(Profile::default());
        a.update_chunks(Vec2::new(1000.0, 0.0));
        b.update_chunks(Vec2::new(1000.0, 0.0));
        let pa: Vec<_> = a
            .world
            .query::<(Entity, (&Motion, &Actor))>()
            .iter()
            .filter(|(_, (_, actor))| actor.kind == Kind::Asteroid)
            .map(|(_, (m, _))| m.position)
            .collect();
        let pb: Vec<_> = b
            .world
            .query::<(Entity, (&Motion, &Actor))>()
            .iter()
            .filter(|(_, (_, actor))| actor.kind == Kind::Asteroid)
            .map(|(_, (m, _))| m.position)
            .collect();
        assert_eq!(pa, pb);
        for i in 0..30 {
            a.update_chunks(Vec2::new(i as f32 * CHUNK, 0.0));
        }
        assert!(a.chunks.len() <= 25);
        assert!(
            a.world
                .query::<(Entity, &Actor)>()
                .iter()
                .filter(|(_, actor)| actor.kind == Kind::Asteroid)
                .count()
                <= 112
        );
    }
    #[test]
    fn pause_freezes_combat_clock() {
        let mut g = Game::new(Profile::default());
        g.set_paused(true);
        g.tick(
            &InputFrame {
                fire: true,
                joystick: Some([1.0, 0.0]),
                destination: None,
                aim: None,
                thrust: None,
            },
            1.0 / 60.0,
        );
        assert_eq!(g.time, 0.0);
        assert_eq!(g.position(), Vec2::ZERO);
    }

    #[test]
    fn reload_keeps_surplus_kill_credit_behind_queued_boss() {
        let profile = Profile {
            kills: [101, 0, 0],
            boss_kills: [1, 0, 0],
            pending: [true, false, false],
            ..Default::default()
        };
        let g = Game::new(profile);
        assert_eq!(g.kill_credit[0], 53);
    }

    #[test]
    fn swept_projectile_hits_through_an_entire_body_in_one_tick() {
        let mut g = Game::new(Profile::default());
        let center = Vec2::new(1700.0, 0.0);
        let enemy = g.spawn_enemy(0, false, center, 1);
        g.spawn_bullet(center - Vec2::X * 40.0, Vec2::X * 1200.0, 12.0, true, 1);
        g.integrate(0.1);
        g.resolve_collisions(0.1);
        assert!(g.world.get::<&Actor>(enemy).unwrap().health < 30.0);
        assert_eq!(g.world.query::<(Entity, &Bullet)>().iter().count(), 0);
    }

    #[test]
    fn collected_pickup_credits_exactly_once() {
        let mut g = Game::new(Profile::default());
        g.spawn_loot(Vec2::ZERO, 2, 3);
        g.update_pickups(Vec2::ZERO, 1.0 / 60.0);
        g.update_pickups(Vec2::ZERO, 1.0 / 60.0);
        assert_eq!(g.profile.currency, [0, 0, 3]);
        assert_eq!(g.profile.revision, 1);
    }

    #[test]
    fn diagnostics_do_not_grant_inventory_or_basic_upgrades() {
        let mut g = Game::new(Profile::default());
        for command in [
            "combat",
            "boss-dart",
            "boss-bastion",
            "boss-coil",
            "stress",
            "damage",
            "die",
        ] {
            g.diagnostic(command);
            assert_eq!(g.profile.currency, [0; 3]);
            assert_eq!(g.profile.ranks, [0; 9]);
        }
    }

    #[test]
    fn asteroid_contacts_exchange_momentum_and_separate() {
        let mut g = Game::new(Profile::default());
        let actor = Actor {
            kind: Kind::Asteroid,
            health: 50.0,
            max_health: 50.0,
            radius: 15.0,
            scale: 1.0,
            damage: 8.0,
        };
        let a = g.world.spawn((
            Motion::new(Vec2::new(1200.0, 0.0), Vec2::new(10.0, 0.0), -FRAC_PI_2),
            actor,
            asteroid_body(1),
        ));
        let b = g.world.spawn((
            Motion::new(Vec2::new(1220.0, 0.0), Vec2::new(-10.0, 0.0), -FRAC_PI_2),
            actor,
            asteroid_body(2),
        ));
        g.resolve_collisions(1.0 / 60.0);
        let ma = *g.world.get::<&Motion>(a).unwrap();
        let mb = *g.world.get::<&Motion>(b).unwrap();
        assert!(ma.velocity.x < 0.0 && mb.velocity.x > 0.0);
        assert!((mb.position - ma.position).length() > 20.0);
    }

    #[test]
    fn projectile_orders_real_cells_instead_of_broadphase_centers() {
        let mut g = Game::new(Profile::default());
        let row = format!("{}#", " ".repeat(199));
        let color = rgba(170, 170, 170, 255);
        let far = g.world.spawn((
            Motion::new(Vec2::new(480.0, 0.0), Vec2::ZERO, -FRAC_PI_2),
            Actor {
                kind: Kind::Asteroid,
                health: 100.0,
                max_health: 100.0,
                radius: 110.0,
                scale: 1.0,
                damage: 0.0,
            },
            PixelBody::from_ascii(&[&row], &[('#', color)]),
        ));
        let near = g.world.spawn((
            Motion::new(Vec2::new(550.0, 0.0), Vec2::ZERO, -FRAC_PI_2),
            Actor {
                kind: Kind::Asteroid,
                health: 100.0,
                max_health: 100.0,
                radius: 2.0,
                scale: 1.0,
                damage: 0.0,
            },
            PixelBody::from_ascii(&["#"], &[('#', color)]),
        ));
        g.spawn_bullet(Vec2::new(400.0, 0.0), Vec2::new(2500.0, 0.0), 12.0, true, 1);
        g.integrate(0.1);
        g.resolve_collisions(0.1);
        assert!(
            !g.world.contains(near),
            "the first occupied cell must be struck"
        );
        assert_eq!(g.world.get::<&Actor>(far).unwrap().health, 100.0);
    }

    #[test]
    fn wider_viewport_extracts_newly_visible_objects() {
        let mut g = Game::new(Profile::default());
        let color = rgba(7, 9, 11, 255);
        g.world.spawn((
            Motion::new(Vec2::new(500.0, 0.0), Vec2::ZERO, -FRAC_PI_2),
            Actor {
                kind: Kind::Asteroid,
                health: 50.0,
                max_health: 50.0,
                radius: 5.0,
                scale: 1.0,
                damage: 0.0,
            },
            PixelBody::from_ascii(&["#"], &[('#', color)]),
        ));
        let contains = |frame: RenderFrame| {
            frame
                .batches
                .iter()
                .any(|b| b.pixels.iter().any(|p| p.color == color))
        };
        assert!(!contains(g.render(1.0)));
        g.set_viewport(1200.0, 360.0);
        assert!(contains(g.render(1.0)));
    }

    #[test]
    fn stress_scene_exercises_requested_real_workload_and_cleans_up() {
        let mut g = Game::new(Profile::default());
        g.diagnostic("stress");
        assert_eq!(g.world.query::<(Entity, &Brain)>().iter().count(), 160);
        assert_eq!(g.world.query::<(Entity, &Bullet)>().iter().count(), 300);
        assert_eq!(g.particles.len(), 2000);
        let pixels = g.render(1.0).pixel_count;
        assert!(pixels >= 80_000, "stress scene extracted {pixels} pixels");
        g.reset_to_dock();
        assert!(!g.stress_mode);
        assert_eq!(g.world.query::<(Entity, &StressObject)>().iter().count(), 0);
    }

    fn mission_site(seed: u32, event: &str) -> regions::Site {
        for y in -3..=3 {
            for x in -3..=3 {
                if let Some(site) = regions::site_by_id(seed, &format!("{x}:{y}"))
                    && site.event == event
                {
                    return site;
                }
            }
        }
        panic!("seeded network should contain {event} sites");
    }

    #[test]
    fn courier_survives_reload_and_death_then_claims_once_at_source() {
        let mut g = Game::new(Profile::default());
        let site = mission_site(g.profile.seed, "courier");
        g.move_player(Vec2::from_array(site.position));
        g.update_exploration();
        assert_eq!(
            g.profile.discoveries.as_slice(),
            std::slice::from_ref(&site.id)
        );
        assert!(g.interact_site().is_ok());
        assert_eq!(g.profile.quests[0].progress, 0);
        assert!(g.interact_site().is_err());
        let saved = serde_json::to_string(g.profile()).unwrap();
        let mut reloaded = Game::new(serde_json::from_str(&saved).unwrap());
        assert_eq!(reloaded.profile.quests[0].progress, 1);
        assert_eq!(reloaded.profile.currency, [0; 3]);
        assert!(!reloaded.profile.quests[0].completed);
        reloaded.move_player(Vec2::from_array(site.position));
        assert_eq!(reloaded.hud().site.unwrap().status, "ready");
        assert!(reloaded.interact_site().is_ok());
        assert_eq!(reloaded.profile.currency[site.family], 3);
        assert_eq!(
            reloaded.profile.relics.as_slice(),
            std::slice::from_ref(&site.id)
        );
        assert!(reloaded.profile.quests[0].completed);
        assert_eq!(reloaded.hud().max_hull, 101.0);
        let rewards = reloaded.profile.currency;
        assert!(reloaded.interact_site().is_err());
        assert_eq!(reloaded.profile.currency, rewards);
        // The retained courier packet survives an actual ordinary lethal hit.
        // Public QA death is intentionally unable to complete delivery progress.
        g.move_player(Vec2::from_array(site.position) + Vec2::X * (site.radius + 100.0));
        g.invulnerability = 0.0;
        g.hurt_player(1000.0, None);
        g.tick(&InputFrame::default(), 1.0 / 60.0);
        assert_eq!(g.position(), Vec2::ZERO);
        assert_eq!(g.profile.quests[0].progress, 1);
        assert_eq!(g.profile.currency, [0; 3]);
    }

    #[test]
    fn bounty_counts_only_accepted_family_and_boss_as_one_across_reload() {
        let mut g = Game::new(Profile::default());
        let site = mission_site(g.profile.seed, "bounty");
        let source = Vec2::from_array(site.position);
        let enemy = g.spawn_enemy(site.family, false, source, 1);
        g.kill_actor(enemy, Kind::Enemy(site.family), source);
        g.move_player(source);
        assert!(g.interact_site().is_ok());
        assert_eq!(g.profile.quests[0].progress, 0);
        let other_family = (site.family + 1) % 3;
        let enemy = g.spawn_enemy(other_family, false, source, 1);
        g.kill_actor(enemy, Kind::Enemy(other_family), source);
        assert_eq!(g.profile.quests[0].progress, 0);
        for boss in [false, true] {
            let enemy = g.spawn_enemy(site.family, boss, source, 1);
            let kind = if boss {
                Kind::Boss(site.family)
            } else {
                Kind::Enemy(site.family)
            };
            g.kill_actor(enemy, kind, source);
        }
        assert_eq!(g.profile.quests[0].progress, 2);
        let saved = serde_json::to_string(g.profile()).unwrap();
        let mut g = Game::new(serde_json::from_str(&saved).unwrap());
        assert_eq!(g.profile.quests[0].progress, 2);
        for _ in 2..site.target_kills {
            let enemy = g.spawn_enemy(site.family, false, source, 1);
            g.kill_actor(enemy, Kind::Enemy(site.family), source);
        }
        assert_eq!(g.profile.quests[0].progress, site.target_kills);
        assert_eq!(
            g.profile.currency, [0; 3],
            "objective completion must not claim remotely"
        );
        assert!(g.interact_site().is_err());
        g.move_player(source);
        assert!(g.interact_site().is_ok());
        assert_eq!(g.profile.currency[site.family], 3);
        assert!(g.interact_site().is_err());
    }

    #[test]
    fn site_sanctuary_blocks_combat_without_granting_haven_services() {
        let mut g = Game::new(Profile::default());
        // Authored normal-flight placement; public scene setup no longer earns
        // the discovery this ordinary sanctuary regression expects.
        let site = regions::sites_near(g.profile.seed, Vec2::ZERO).remove(0);
        g.move_player(Vec2::from_array(site.position));
        g.update_exploration();
        let center = Vec2::from_array(site.position);
        assert!(g.hud().safe && !g.hud().dock);
        g.world.get::<&mut Actor>(g.player).unwrap().health = 60.0;
        g.invulnerability = 0.0;
        for _ in 0..30 {
            g.tick(
                &InputFrame {
                    fire: true,
                    ..Default::default()
                },
                1.0 / 60.0,
            );
        }
        assert_eq!(g.world.query::<(Entity, &Brain)>().iter().count(), 0);
        assert_eq!(g.world.query::<(Entity, &Bullet)>().iter().count(), 0);
        assert_eq!(g.hud().hull, 60.0, "remote sanctuaries must not repair");
        g.hurt_player(50.0, None);
        assert_eq!(g.hud().hull, 60.0);
        assert!(g.purchase("damage").is_err());
        let enemy = g.spawn_enemy(0, false, center, 1);
        g.update_enemies(center, 1.0 / 60.0);
        let actor = *g.world.get::<&Actor>(enemy).unwrap();
        let position = g.world.get::<&Motion>(enemy).unwrap().position;
        assert!(position.distance(center) > site.radius + actor.radius);
        for player_bullet in [false, true] {
            g.spawn_bullet(
                center + Vec2::X * (site.radius + 50.0),
                Vec2::X,
                50.0,
                player_bullet,
                1,
            );
            let bullet = g
                .world
                .query::<(Entity, &Bullet)>()
                .iter()
                .next()
                .unwrap()
                .0;
            {
                let mut motion = g.world.get::<&mut Motion>(bullet).unwrap();
                motion.previous = center - Vec2::X * (site.radius + 50.0);
                motion.position = center + Vec2::X * (site.radius + 50.0);
            }
            g.resolve_collisions(1.0 / 60.0);
            assert!(
                !g.world.contains(bullet),
                "sweeps through a sanctuary must dissipate"
            );
        }
        assert_eq!(g.profile.discoveries.len(), 1);
        assert_eq!(g.profile.currency, [0; 3]);
    }

    #[test]
    fn sanctuary_guards_hold_after_long_ticks_and_beyond_local_cache() {
        let mut g = Game::new(Profile::default());
        let site = mission_site(g.profile.seed, "courier");
        let center = Vec2::from_array(site.position);
        g.move_player(center + Vec2::X * (site.radius + 500.0));
        let enemy = g.spawn_enemy(0, false, center + Vec2::X * (site.radius + 20.0), 1);
        g.world.get::<&mut Motion>(enemy).unwrap().velocity = -Vec2::X * 400.0;
        g.tick(&InputFrame::default(), 0.05);
        let radius = g.world.get::<&Actor>(enemy).unwrap().radius;
        let motion = *g.world.get::<&Motion>(enemy).unwrap();
        assert!(motion.position.distance(center) > site.radius + radius);
        assert!(motion.velocity.dot(motion.position - center) >= -0.001);
        g.reset_to_dock();
        assert!(g.outside_site_cache(center));
        let enemy = g.spawn_enemy(2, true, center, 1);
        g.enforce_actor_sanctuaries();
        let radius = g.world.get::<&Actor>(enemy).unwrap().radius;
        assert!(
            g.world
                .get::<&Motion>(enemy)
                .unwrap()
                .position
                .distance(center)
                > site.radius + radius
        );
        g.update_segments(0.05);
        for (_, (motion, segment)) in g.world.query::<(Entity, (&Motion, &Segment))>().iter() {
            if segment.owner == enemy {
                assert!(
                    motion.position.distance(center) > site.radius + 8.23 * segment_scale(true)
                );
            }
        }
    }

    #[test]
    fn legacy_progress_migrates_and_archives_sanitize_with_bounded_relic_bonus() {
        let old: Profile = serde_json::from_str(
            r#"{"schema_version":1,"revision":9,"currency":[7,8,9],"ranks":[1,2,3,4,5,1,0,0,0]}"#,
        )
        .unwrap();
        let g = Game::new(old);
        assert_eq!(g.profile.schema_version, components::SCHEMA_VERSION);
        assert_eq!(g.profile.revision, 10);
        assert_eq!(g.profile.currency, [7, 8, 9]);
        assert_eq!(g.profile.ranks[2], 3);
        let mut profile = Profile::default();
        let sites: Vec<_> = (-12..=12)
            .flat_map(|y| (-12..=12).map(move |x| (x, y)))
            .filter_map(|(x, y)| regions::site_by_id(profile.seed, &format!("{x}:{y}")))
            .take(300)
            .collect();
        profile.discoveries = vec!["invalid".into(), "999999:0".into(), sites[0].id.clone()];
        profile
            .discoveries
            .extend(sites.iter().map(|site| site.id.clone()));
        profile.relics = profile.discoveries.clone();
        profile.quests = sites
            .iter()
            .map(|site| QuestProgress {
                site_id: site.id.clone(),
                event: site.event.into(),
                family: site.family,
                target: if site.event == "courier" {
                    1
                } else {
                    site.target_kills
                },
                progress: u32::MAX,
                completed: false,
            })
            .collect();
        profile.quests.insert(
            0,
            QuestProgress {
                site_id: sites[0].id.clone(),
                event: "bounty".into(),
                family: 999,
                target: 0,
                progress: 0,
                completed: false,
            },
        );
        let g = Game::new(profile);
        assert_eq!(g.profile.discoveries.len(), MAX_ARCHIVE);
        assert_eq!(g.profile.quests.len(), MAX_ARCHIVE);
        assert_eq!(g.profile.relics.len(), MAX_ARCHIVE);
        assert_eq!(
            g.profile.relics.iter().collect::<HashSet<_>>().len(),
            MAX_ARCHIVE
        );
        assert!(
            g.profile
                .quests
                .iter()
                .all(|q| q.family < 3 && q.progress == q.target && q.completed)
        );
        assert_eq!(g.hud().max_hull, 120.0);
    }

    #[test]
    fn reduced_density_keeps_all_ship_cells_and_simulation_state() {
        let mut g = Game::new(Profile::default());
        g.diagnostic("stress");
        g.world.get::<&mut Motion>(g.player).unwrap().heading = 0.0;
        g.world
            .get::<&mut Motion>(g.player)
            .unwrap()
            .previous_heading = 0.0;
        let motion = *g.world.get::<&Motion>(g.player).unwrap();
        let actor = *g.world.get::<&Actor>(g.player).unwrap();
        let mut transform = engine_core::GpuTransform::from(actor_transform(motion, actor));
        transform.b[2] = ACTOR_MATERIAL.light_response;
        let count_ship = |frame: &RenderFrame| {
            frame
                .batches
                .iter()
                .map(|batch| {
                    batch
                        .pixels
                        .iter()
                        .filter(|pixel| batch.transforms[pixel.transform_slot()] == transform)
                        .count()
                })
                .sum::<usize>()
        };
        let profile = serde_json::to_string(g.profile()).unwrap();
        let full = g.render(1.0);
        g.set_cosmetic_density(true);
        let reduced = g.render(1.0);
        assert!(reduced.pixel_count < full.pixel_count);
        assert_eq!(count_ship(&full), 931);
        assert_eq!(count_ship(&reduced), 931);
        assert_eq!(serde_json::to_string(g.profile()).unwrap(), profile);
        assert_eq!(g.particles.len(), 2000);
        assert_eq!(g.world.query::<(Entity, &Brain)>().iter().count(), 160);
    }

    #[test]
    fn lighting_is_bounded_deterministic_and_matches_projectile_interpolation() {
        let mut g = Game::new(Profile::default());
        for x in [20.0, 50.0, 150.0] {
            g.spawn_bullet(Vec2::new(x, 0.0), Vec2::X * 100.0, 1.0, true, 1);
        }
        let bullet = g
            .world
            .query::<(Entity, (&Motion, &Bullet))>()
            .iter()
            .find(|(_, (motion, _))| motion.position.x == 20.0)
            .unwrap()
            .0;
        g.world.get::<&mut Motion>(bullet).unwrap().previous.x = 0.0;
        for index in 0..40 {
            g.transient_light(
                Vec2::new(index as f32, 10.0),
                999.0,
                [-1.0, 2.0, 0.6],
                99.0,
                0.18,
            );
        }
        assert_eq!(g.transient_lights.len(), 32);
        let lights = g.light_sources(0.5);
        assert_eq!(lights.len(), 8);
        assert_eq!(lights, g.light_sources(0.5));
        assert!(
            lights
                .iter()
                .any(|light| light.position == [10.0, 0.0] && light.radius == 56.0)
        );
        assert!(
            !lights
                .iter()
                .any(|light| light.position == [150.0, 0.0] && light.radius == 56.0)
        );
        assert_eq!(lights[0].position, [39.0, 10.0]);
        assert!(lights.iter().all(|light| {
            light.position.iter().all(|v| v.is_finite())
                && (24.0..=220.0).contains(&light.radius)
                && (0.0..=1.8).contains(&light.intensity)
                && light
                    .color
                    .iter()
                    .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
        }));
        g.transient_light(Vec2::splat(f32::NAN), 50.0, [1.0; 3], 1.0, 1.0);
        assert_eq!(g.transient_lights.len(), 32);
        for _ in 0..12 {
            g.tick(&InputFrame::default(), 0.05);
        }
        assert!(g.transient_lights.is_empty());
        assert!(
            g.light_sources(f32::NAN)
                .iter()
                .all(|light| light.position.iter().all(|v| v.is_finite()))
        );
    }

    #[test]
    fn signature_abilities_emit_their_own_real_world_light_colors() {
        let mut g = Game::new(Profile {
            schema_version: 3,
            unlocked: [true; 3],
            ranks: [0, 0, 0, 0, 0, 0, 1, 1, 1],
            ..Default::default()
        });
        g.move_player(Vec2::new(1800.0, 0.0));
        g.shot_count = 7;
        g.fire(Stats::from_profile(g.profile()));
        assert!(
            g.light_sources(1.0)
                .iter()
                .any(|light| light.radius == 95.0 && light.color == [0.67, 0.38, 1.0])
        );
        g.invulnerability = 0.0;
        g.hurt_player(10.0, None);
        assert!(
            g.light_sources(1.0)
                .iter()
                .any(|light| light.radius == 165.0 && light.color == [0.16, 0.65, 1.0])
        );
        g.world.get::<&mut Motion>(g.player).unwrap().velocity = Vec2::X * 180.0;
        g.update_wake(g.position(), 1.0 / 60.0);
        assert!(
            g.light_sources(1.0)
                .iter()
                .any(|light| light.color == [0.23, 1.0, 0.62])
        );
    }

    #[test]
    fn regional_spawn_mix_uses_biome_weights_and_avoids_sanctuaries() {
        let mut g = Game::new(Profile::default());
        let mut iron = None;
        for y in -5..=5 {
            for x in -5..=5 {
                if let Some(site) = regions::site_by_id(g.profile.seed, &format!("{x}:{y}")) {
                    let position =
                        Vec2::from_array(site.position) + Vec2::Y * (site.radius + 200.0);
                    if regions::at(g.profile.seed, position).kind == 3 {
                        iron = Some(position);
                        break;
                    }
                }
            }
            if iron.is_some() {
                break;
            }
        }
        let position = iron.expect("seeded network must contain the Iron Belt");
        g.move_player(position);
        assert!(!g.safe());
        let mut families = [0; 3];
        for _ in 0..200 {
            g.spawn_timer = 0.0;
            g.update_spawns(position, 1.0 / 60.0);
            let (entity, family, spawned) = g
                .world
                .query::<(Entity, (&Brain, &Motion))>()
                .iter()
                .map(|(entity, (brain, motion))| (entity, brain.family, motion.position))
                .next()
                .expect("unprotected region should spawn an enemy");
            assert!(regions::protected(g.profile.seed, spawned).is_none());
            families[family] += 1;
            g.remove_enemy(entity, false);
        }
        assert!(families[1] > families[0] * 2, "Iron Belt mix: {families:?}");
        assert!(families[1] > families[2] * 2, "Iron Belt mix: {families:?}");
        assert_eq!(g.profile.currency, [0; 3]);
        assert_eq!(g.profile.kills, [0; 3]);
    }

    #[test]
    fn heading_interpolation_takes_shortest_path_across_pi() {
        let heading = interpolate_heading(170.0_f32.to_radians(), -170.0_f32.to_radians(), 0.5);
        assert!(forward(heading).distance(Vec2::NEG_X) < 0.001);
    }

    #[test]
    fn render_focus_tracks_all_authored_reactors_between_ticks_without_mutation() {
        for family in ["dart", "bastion", "coil"] {
            for generation in 1..=5 {
                let mut game = Game::new(Profile::default());
                game.diagnostic(&format!("boss-{family}-{generation}"));
                let entity = game.active_boss.unwrap();
                let actor = *game.world.get::<&Actor>(entity).unwrap();
                let art_family = game.world.get::<&Brain>(entity).unwrap().family;
                let art = boss_assets::art(art_family, generation);
                let center = game.world.get::<&Motion>(entity).unwrap().position;
                {
                    let mut motion = game.world.get::<&mut Motion>(entity).unwrap();
                    motion.previous = center - Vec2::new(12.0, 8.0);
                    motion.previous_heading = 170.0_f32.to_radians();
                    motion.heading = -170.0_f32.to_radians();
                }
                let before = serde_json::to_value(game.profile()).unwrap();
                let tick_core = Vec2::from_array(game.hud().boss.unwrap().core_position);
                let mut pose = *game.world.get::<&Motion>(entity).unwrap();
                pose.position = center - Vec2::new(6.0, 4.0);
                pose.heading = PI;
                let expected = actor_transform(pose, actor).point(art.core_position);
                let focused = game.render_focus(0.5).boss.unwrap();
                assert!(focused.center.distance(pose.position) < 0.001);
                assert!(focused.core.distance(expected) < 0.001);
                assert!(focused.core.distance(tick_core) > 1.0);
                assert!((focused.core_radius - art.core_radius * actor.scale).abs() < 0.001);
                assert!(focused.core_radius > 0.0);
                assert_eq!(before, serde_json::to_value(game.profile()).unwrap());
                assert!(
                    game.render_focus(1.0)
                        .boss
                        .unwrap()
                        .core
                        .distance(tick_core)
                        < 0.001
                );
            }
        }
    }

    #[test]
    fn render_focus_handles_arena_cleanup_and_invalid_alpha() {
        let mut game = Game::new(Profile::default());
        assert!(game.render_focus(f32::NAN).boss.is_none());
        game.diagnostic("champion-0");
        let first = game.render_focus(f32::NAN);
        let last = game.render_focus(100.0);
        let champion = last.boss.unwrap();
        assert_eq!(champion.core, champion.center);
        assert!(champion.core.is_finite() && first.player.is_finite());
        assert!(last.player_radius > 0.0);
        assert_eq!(champion.encounter, first.boss.unwrap().encounter);
        game.reset_to_dock();
        assert!(game.render_focus(0.5).boss.is_none());
        assert!(game.render_focus(0.5).player.length() < 1.0);
    }

    #[test]
    fn generated_actor_scales_match_world_footprints_and_collision_bounds() {
        let mut g = Game::new(Profile::default());
        let ship = g.world.get::<&PixelBody>(g.player).unwrap();
        let player = *g.world.get::<&Actor>(g.player).unwrap();
        assert!((ship.width as f32 * player.scale - 33.6).abs() < 0.001);
        assert!(player.scale >= 0.7);
        assert!((19.0..21.0).contains(&player.radius));
        drop(ship);
        for family in 0..3 {
            let base = g.spawn_enemy(family, false, Vec2::new(1800.0, 0.0), 1);
            let boss = g.spawn_enemy(family, true, Vec2::new(2000.0, 0.0), 1);
            let base_radius = g.world.get::<&Actor>(base).unwrap().radius;
            let boss_radius = g.world.get::<&Actor>(boss).unwrap().radius;
            assert!(boss_radius > base_radius * 1.8, "family {family}");
        }
        for (_, (actor, body)) in g.world.query::<(Entity, (&Actor, &PixelBody))>().iter() {
            for (cell, _) in body.cells() {
                for corner in [
                    Vec2::new(-0.5, -0.5),
                    Vec2::new(0.5, -0.5),
                    Vec2::new(-0.5, 0.5),
                    Vec2::new(0.5, 0.5),
                ] {
                    assert!((cell + corner).length() * actor.scale <= actor.radius + 0.001);
                }
            }
        }
    }

    #[test]
    fn dock_repair_restores_generated_ship_cells_after_real_damage() {
        let mut g = Game::new(Profile::default());
        g.move_player(Vec2::new(1800.0, 0.0));
        g.shield = 0.0; // Declared depleted shield; this probe verifies body-cell repair.
        g.hurt_player(20.0, Some(Vec2::ZERO));
        assert!(
            g.world
                .get::<&PixelBody>(g.player)
                .unwrap()
                .occupied_count()
                < 931
        );
        g.reset_to_dock();
        assert_eq!(
            g.world
                .get::<&PixelBody>(g.player)
                .unwrap()
                .occupied_count(),
            931
        );
    }

    #[test]
    fn external_destinations_reject_nonfinite_and_out_of_world_values() {
        let mut g = Game::new(Profile::default());
        let valid = Vec2::new(500.0, -900.0);
        g.control(
            &InputFrame {
                destination: Some(valid.to_array()),
                ..Default::default()
            },
            1.0 / 60.0,
        );
        assert_eq!(g.destination, Some(valid));
        for point in [
            [f32::NAN, 1.0],
            [f32::INFINITY, 0.0],
            [WORLD_LIMIT + 1.0, 0.0],
            [0.0, -f32::MAX],
        ] {
            g.control(
                &InputFrame {
                    destination: Some(point),
                    ..Default::default()
                },
                1.0 / 60.0,
            );
            assert_eq!(g.destination, Some(valid));
            assert!(
                g.world
                    .get::<&Motion>(g.player)
                    .unwrap()
                    .velocity
                    .is_finite()
            );
        }
        g.control(
            &InputFrame {
                destination: Some([WORLD_LIMIT, -WORLD_LIMIT]),
                ..Default::default()
            },
            1.0 / 60.0,
        );
        assert_eq!(g.destination, Some(Vec2::new(WORLD_LIMIT, -WORLD_LIMIT)));
    }

    #[test]
    fn invalid_joystick_releases_thrust_without_poisoning_motion() {
        for joystick in [[f32::NAN, 0.0], [0.0, f32::INFINITY], [f32::MAX, f32::MAX]] {
            let mut g = Game::new(Profile::default());
            g.throttle = 1.0;
            g.control(
                &InputFrame {
                    joystick: Some(joystick),
                    ..Default::default()
                },
                1.0 / 60.0,
            );
            assert!(g.throttle < 1.0);
            let motion = g.world.get::<&Motion>(g.player).unwrap();
            assert!(motion.velocity.is_finite() && motion.heading.is_finite());
        }
    }

    #[test]
    fn continuous_thrust_and_impulses_keep_player_inside_practical_extent() {
        let mut g = Game::new(Profile::default());
        {
            let mut motion = g.world.get::<&mut Motion>(g.player).unwrap();
            *motion = Motion::new(
                Vec2::new(WORLD_LIMIT - 0.0625, -WORLD_LIMIT + 0.0625),
                Vec2::new(180.0, -180.0),
                -PI / 4.0,
            );
        }
        g.integrate(1.0 / 60.0);
        g.confine_player();
        let motion = *g.world.get::<&Motion>(g.player).unwrap();
        assert_eq!(motion.position, Vec2::new(WORLD_LIMIT, -WORLD_LIMIT));
        assert_eq!(motion.velocity, Vec2::ZERO);
        // Collisions may subsequently push the ship; the same boundary pass is
        // performed after collision resolution in the fixed-step tick.
        g.world.get::<&mut Motion>(g.player).unwrap().position += Vec2::new(100.0, -100.0);
        g.confine_player();
        assert_eq!(g.position(), Vec2::new(WORLD_LIMIT, -WORLD_LIMIT));
        g.update_chunks(g.position());
        assert!(g.chunks.len() <= 25);
        assert!(danger(g.position()) > 0);
    }
}
