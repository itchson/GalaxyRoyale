//! WASM browser adapter for Galaxy Royale's portable Rust simulation.
use crate::{
    boss_overlay::{BossOverlay, OverlayInput},
    pixel_ui, renderer, ui_scene,
};

use engine_core::{CameraZoom, FrameBuilder, Vec2};
use game_core::{Game, GameEvent, Hud, InputFrame, Profile};
use serde::Serialize;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

#[wasm_bindgen]
pub struct Galaxy {
    game: Game,
    guest_view: Option<crate::co_op_replica_bridge::GuestView>,
    renderer: renderer::Renderer,
    frame: FrameBuilder,
    ui_frame: FrameBuilder,
    camera_zoom: CameraZoom,
    ui_mode: u8,
    ui_scene: ui_scene::UiScene,
    ui_time: f32,
    presentation_time: f64,
    boss_overlay: BossOverlay,
    last_boss_panel: Option<[f32; 4]>,
    vitals_animation: pixel_ui::VitalsAnimation,
    reduced_motion: bool,
    reduced_cosmetics: bool,
    ui_insets: [f32; 4],
    touch_ui: bool,
    ui_joystick: Option<[f32; 2]>,
    ui_fire: bool,
    accumulator: f32,
    pending_destination: Option<[f32; 2]>,
    last_revision: u64,
    paused: bool,
    shake: bool,
}
/// A prepared renderer owns no reference or borrow of the live Galaxy.
/// The shell pauses presentation while this resource is built, then consumes
/// it synchronously on the same canvas through `install_recovery`.
#[wasm_bindgen]
pub struct RendererRecovery {
    renderer: renderer::Renderer,
}
/// Prepare independently of the live game so reads, saves and modal state
/// changes do not encounter an exclusive wasm-bindgen borrow across an await.
#[wasm_bindgen]
pub async fn prepare_renderer_recovery(
    canvas: HtmlCanvasElement,
) -> Result<RendererRecovery, JsValue> {
    // A native loss takes effect before its queued DOM event. Avoid entering
    // glow's VERSION query while that existing WebGL context is genuinely lost.
    #[cfg(feature = "webgl")]
    {
        let context = canvas
            .get_context("webgl2")?
            .ok_or_else(|| js("WebGL2 context is unavailable for recovery"))?
            .dyn_into::<web_sys::WebGl2RenderingContext>()?;
        if context.is_context_lost() {
            return Err(js("WebGL2 context is lost; wait for its restored event"));
        }
    }
    Ok(RendererRecovery {
        renderer: renderer::Renderer::new(canvas).await?,
    })
}
#[derive(Serialize)]
struct FrameResponse {
    hud: Hud,
    events: Vec<GameEvent>,
    profile: Option<Profile>,
    pixels: usize,
    simulation_ms: f64,
    render_ms: f64,
}
fn js(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}
// The browser save and HUD contracts use explicit null for optional values.
// serde_wasm_bindgen::to_value defaults to undefined for Rust Option::None.
fn to_js(value: &impl Serialize) -> Result<JsValue, JsValue> {
    value
        .serialize(&serde_wasm_bindgen::Serializer::new().serialize_missing_as_null(true))
        .map_err(js)
}
/// Headless contract proof compiled only into QA bundles, never the game release.
#[cfg(feature = "qa-contract")]
#[wasm_bindgen]
pub fn checkpoint_contract(stage: &str) -> Result<JsValue, JsValue> {
    let mut profile = Profile::default();
    match stage {
        "training-pan-ready"
        | "training-renko-ready"
        | "training-hugo-ready"
        | "training-pan-upgraded"
        | "training-renko-upgraded"
        | "training-hugo-upgraded" => {
            let corporation = stage
                .split('-')
                .nth(1)
                .ok_or_else(|| js("Invalid training fixture"))?;
            profile = game_core::identity::create_profile(
                "talo",
                corporation,
                "renko",
                "0123456789abcdef",
            )
            .map_err(js)?;
            // Authored canonical save inputs, not a claim of earned combat.
            // The upgraded case executes the actual public dock transaction.
            profile.onboarding = Some(game_core::identity::OnboardingProgress {
                version: 2,
                stage: 5,
                flags: 31,
                asteroids_destroyed: 10,
                materials_collected: 10,
                legacy_complete: false,
            });
            let index = match corporation {
                "pan" => 0,
                "renko" => 1,
                _ => 2,
            };
            profile.campaign.stored_materials[0] = 10;
            profile.campaign.stored_training_materials[index] = 10;
        }
        "training-cargo-reload" | "training-legacy-complete" | "training-legacy-incomplete" => {
            profile =
                game_core::identity::create_profile("talo", "hugo", "pan", "0123456789abcdef")
                    .map_err(js)?;
            profile.currency = [7, 8, 9];
            profile.campaign.stored_materials[0] = 11;
            if stage == "training-cargo-reload" {
                profile.campaign.cargo_materials[0] = 12;
                profile.campaign.cargo_training_materials = [4, 3, 5];
                profile.campaign.stored_training_materials = [2, 3, 4];
            } else {
                profile.schema_version = 5;
                profile.onboarding = Some(game_core::identity::OnboardingProgress {
                    version: 1,
                    stage: if stage == "training-legacy-complete" {
                        6
                    } else {
                        5
                    },
                    flags: if stage == "training-legacy-complete" {
                        127
                    } else {
                        31
                    },
                    ..Default::default()
                });
            }
        }
        "kikai-pan" | "kikai-renko" | "kikai-hugo" => {
            let corporation = stage.trim_start_matches("kikai-");
            profile = game_core::identity::create_profile(
                "talo",
                corporation,
                "renko",
                "0123456789abcdef",
            )
            .map_err(js)?;
        }
        "fresh" | "champion" | "tiered-cargo" | "loss-summary" => {}
        "components-ready" | "components-crafted" | "components-upgraded" | "components-legacy" => {
            // Supplied save fixture for contract/UI review; never awarded by gameplay.
            profile =
                game_core::identity::create_profile("dora", "pan", "renko", "0123456789abcdef")
                    .map_err(js)?;
            profile.onboarding = None;
            profile.currency = [30; 3];
            profile.campaign.ship_xp[0] = 200;
            profile.campaign.stored_materials = [10; 5];
            for state in &mut profile.campaign.abilities[..3] {
                state.blueprint = true;
                state.rank = 2;
            }
            if stage == "components-legacy" {
                let mut value = serde_json::to_value(&profile).map_err(js)?;
                value["schema_version"] = serde_json::json!(6);
                value["campaign"]["components"] = serde_json::json!("ignored legacy injection");
                profile = serde_json::from_value(value).map_err(js)?;
            }
        }
        "generations" => {
            // Valid saved credit with both higher-tier queues and reserved base credit.
            let mut value = serde_json::to_value(&profile).map_err(js)?;
            value["boss_generations"] =
                serde_json::json!([[18, 9, 4, 2, 1], [6, 3, 1, 0, 0], [26, 13, 6, 2, 1]]);
            value["boss_kills"] = serde_json::json!([34, 10, 48]);
            value["kills"] = serde_json::json!([470, 158, 660]);
            value["pending"] = serde_json::json!([true, false, true]);
            value["currency"] = serde_json::json!([3, 4, 5]);
            value["revision"] = serde_json::json!(9);
            profile = serde_json::from_value(value).map_err(js)?;
        }
        "legacy-bosses" => {
            let mut value = serde_json::to_value(&profile).map_err(js)?;
            value["schema_version"] = serde_json::json!(3);
            value
                .as_object_mut()
                .ok_or_else(|| js("Invalid profile fixture"))?
                .remove("boss_generations");
            value["boss_kills"] = serde_json::json!([4, 2, 8]);
            value["kills"] = serde_json::json!([120, 60, 220]);
            value["pending"] = serde_json::json!([true, false, true]);
            value["currency"] = serde_json::json!([7, 8, 9]);
            value["revision"] = serde_json::json!(9);
            profile = serde_json::from_value(value).map_err(js)?;
        }
        "legacy-cargo" => {
            // Contract input fixture, not an earned gameplay reward. Exercise
            // the real schema-three scalar deserializer and Game migration.
            let mut value = serde_json::to_value(&profile).map_err(js)?;
            value["schema_version"] = serde_json::json!(3);
            value["currency"] = serde_json::json!([20, 30, 40]);
            value["ranks"] = serde_json::json!([2, 1, 3, 0, 0, 0, 4, 2, 1]);
            value["unlocked"] = serde_json::json!([true, true, true]);
            value["revision"] = serde_json::json!(9);
            value["campaign"]["cargo_materials"] = serde_json::json!(5);
            value["campaign"]["stored_materials"] = serde_json::json!(7);
            value["campaign"]["patches"][1] = serde_json::json!(true);
            value["campaign"]["blueprints"][1] = serde_json::json!(true);
            value["campaign"]["crafted"][1] = serde_json::json!(true);
            value["campaign"]["selected_ship"] = serde_json::json!(2);
            value["campaign"]["ship_xp"][2] = serde_json::json!(60);
            profile = serde_json::from_value(value).map_err(js)?;
        }
        "inventory" => {
            // Supplied checkpoint fixture only. Actual dock transactions below
            // are tested; these patches, XP and materials are not combat proof.
            profile.currency = [100; 3];
            profile.unlocked = [true; 3];
            profile.campaign.patches = [true; 8];
            profile.campaign.blueprints = [true; 8];
            profile.campaign.crafted = [true; 8];
            profile.campaign.royale_results = [1; 8];
            profile.campaign.victory_claimed = true;
            profile.campaign.selected_ship = 2;
            profile.campaign.ship_xp[2] = 60;
            profile.campaign.stored_materials = [50; 5];
            for state in &mut profile.campaign.abilities {
                state.blueprint = true;
            }
        }
        "cargo-reload" => {
            // A supplied in-flight checkpoint: Game must discard carried
            // materials on reconstruction without converting them to stash.
            profile.campaign.cargo_materials = [1, 2, 3, 4, 5];
            profile.campaign.stored_materials = [10, 20, 30, 40, 50];
        }
        _ => return Err(js("Unknown checkpoint contract stage")),
    }
    let mut game = Game::new(profile);
    if matches!(stage, "components-crafted" | "components-upgraded") {
        for id in ["pulse-conductor", "aegis-lattice", "flux-bearing"] {
            game.craft_component(id).map_err(js)?;
        }
        if stage == "components-upgraded" {
            for id in ["pulse-lens", "guard-coil", "flux-drive"] {
                game.upgrade_ability(id).map_err(js)?;
            }
            let before = serde_json::to_value(game.profile()).map_err(js)?;
            if game.craft_component("invalid").is_ok()
                || game.upgrade_ability("pulse-lens").is_ok()
                || serde_json::to_value(game.profile()).map_err(js)? != before
            {
                return Err(js("Failed component transaction changed progression"));
            }
        }
    }
    if stage.starts_with("training-") && stage.ends_with("-upgraded") {
        let key = match stage.split('-').nth(1) {
            Some("pan") => "flux-drive",
            Some("renko") => "pulse-lens",
            _ => "guard-coil",
        };
        game.upgrade_ability(key).map_err(js)?;
    }
    if stage == "champion" {
        game.diagnostic("champion-0");
    }
    if stage == "inventory" {
        for key in [
            "pulse-lens",
            "guard-coil",
            "flux-drive",
            "pulse-echo",
            "broadside",
        ] {
            game.craft_ability(key).map_err(js)?;
        }
        game.upgrade_ability("pulse-lens").map_err(js)?;
        game.purchase("damage").map_err(js)?;
        game.purchase("damage").map_err(js)?;
        // Craft on its actual compatible owned hull, then return: the later
        // rejection isolates family compatibility rather than missing ownership.
        game.switch_ship(3).map_err(js)?;
        game.craft_ability("guided-missile").map_err(js)?;
        game.switch_ship(2).map_err(js)?;
        for (slot, key) in ["broadside", "guard-coil", "flux-drive"].iter().enumerate() {
            game.equip_ability(slot, Some(key)).map_err(js)?;
        }
        let before = serde_json::to_value(game.profile()).map_err(js)?;
        for (slot, key) in [
            (256, None),
            (1, Some("pulse-lens")),
            (0, Some("guided-missile")),
            (0, Some("not-an-ability")),
        ] {
            if game.equip_ability(slot, key).is_ok() {
                return Err(js("Invalid equip contract was accepted"));
            }
            if serde_json::to_value(game.profile()).map_err(js)? != before {
                return Err(js("Rejected equip contract mutated the checkpoint"));
            }
        }
    }
    if matches!(stage, "tiered-cargo" | "loss-summary") {
        // Typed serializer fixtures. SaveStore roundtrips these fields without
        // claiming an actual pickup, death, reward or browser-rendered session.
        let mut checkpoint = game.profile().clone();
        if stage == "tiered-cargo" {
            checkpoint.campaign.cargo_materials = [1, 2, 3, 4, 5];
            checkpoint.campaign.stored_materials = [10, 20, 30, 40, 50];
        } else {
            checkpoint.campaign.last_run = Some(game_core::RunSummary {
                time: 3.0,
                distance_peak: 1800.0,
                enemies: 2,
                asteroids: 3,
                champions: 0,
                points_gained: [4, 5, 6],
                xp_gained: 12,
                materials_gained: 15,
                materials_lost: [1, 2, 3, 4, 5],
            });
        }
        return to_js(&checkpoint);
    }
    to_js(game.profile())
}
fn now() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}
#[wasm_bindgen]
impl Galaxy {
    pub async fn create(canvas: HtmlCanvasElement, profile: JsValue) -> Result<Galaxy, JsValue> {
        let profile = if profile.is_null() || profile.is_undefined() {
            Profile::default()
        } else {
            serde_wasm_bindgen::from_value::<Profile>(profile).map_err(js)?
        };
        let revision = profile.revision;
        let renderer = renderer::Renderer::new(canvas).await?;
        let mut game = Game::new(profile);
        let viewport = renderer.viewport();
        game.set_viewport(viewport[0], viewport[1]);
        Ok(Self {
            game,
            renderer,
            frame: FrameBuilder::new(),
            ui_frame: FrameBuilder::new(),
            camera_zoom: CameraZoom::default(),
            ui_mode: 2,
            ui_scene: ui_scene::UiScene::default(),
            ui_time: 0.0,
            presentation_time: 0.0,
            boss_overlay: BossOverlay::default(),
            last_boss_panel: None,
            vitals_animation: pixel_ui::VitalsAnimation::default(),
            reduced_motion: false,
            ui_insets: [0.0; 4],
            touch_ui: false,
            ui_joystick: None,
            ui_fire: false,
            guest_view: None,
            reduced_cosmetics: false,
            accumulator: 0.0,
            pending_destination: None,
            last_revision: revision,
            paused: false,
            shake: true,
        })
    }
    /// Local host controller only; authenticated peer mapping lives in the shell.
    pub fn co_op_batch(&mut self, request: &str) -> String {
        if self.guest_view.is_some() {
            return "{\"v\":1,\"results\":[{\"ok\":false,\"error\":\"active_session\"}]}".into();
        }
        let before = self.game.co_op_session();
        let output = crate::co_op_bridge::dispatch(&mut self.game, request);
        if before != self.game.co_op_session() {
            self.accumulator = 0.0;
            self.pending_destination = None;
            self.vitals_animation.reset();
        }
        output
    }
    /// Guest packets update a validated presentation cache, never the solo Game.
    pub fn co_op_replica(&mut self, request: &str) -> String {
        let before = self.guest_view.is_some();
        let result =
            crate::co_op_replica_bridge::dispatch(&mut self.guest_view, &self.game, request);
        if before != self.guest_view.is_some() {
            self.accumulator = 0.0;
            self.pending_destination = None;
            self.vitals_animation.reset();
            self.boss_overlay.reset();
        }
        result
    }
    pub fn step(&mut self, milliseconds: f64, input: JsValue) -> Result<JsValue, JsValue> {
        let delta = if milliseconds.is_finite() {
            (milliseconds as f32 / 1000.0).clamp(0.0, 0.1)
        } else {
            0.0
        };
        self.ui_time += delta;
        self.presentation_time += f64::from(delta);
        self.camera_zoom.advance(delta);
        self.update_world_viewport();
        let mut input: InputFrame = serde_wasm_bindgen::from_value(input).map_err(js)?;
        self.ui_joystick = input.joystick;
        self.ui_fire = input.fire;
        if let Some(destination) = input.destination {
            self.pending_destination = Some(destination);
        }
        let start = now();
        if !self.paused && self.guest_view.is_none() {
            self.accumulator += delta;
            let mut steps = 0;
            while self.accumulator >= 1.0 / 60.0 && steps < 5 {
                input.destination = self.pending_destination.take();
                if let Some(session) = self.game.co_op_session() {
                    self.game
                        .tick_co_op(session)
                        .map_err(|error| js(format!("{error:?}")))?;
                } else {
                    self.game.tick(&input, 1.0 / 60.0);
                }
                self.accumulator -= 1.0 / 60.0;
                steps += 1;
            }
            if steps == 5 {
                self.accumulator = self.accumulator.min(1.0 / 60.0);
            }
        }
        let simulation_ms = now() - start;
        if let Some(view) = &mut self.guest_view {
            view.age = (view.age + delta).min(1.0);
            if let Some(scene) = view.replica.scene() {
                let alpha = (view.age / scene.blend_seconds().max(1.0 / 240.0)).clamp(0.0, 1.0);
                if let Some(focus) = scene.focus(view.peer, alpha) {
                    view.camera = view.camera.lerp(focus.player, 1.0 - (-12.0 * delta).exp());
                }
            }
        }
        let mut hud = self.current_hud()?;
        let vitals = self
            .vitals_animation
            .advance(&hud, delta, self.reduced_motion);
        let render_start = now();
        let alpha = self.render_alpha();
        let corporation = self
            .game
            .profile()
            .identity
            .as_ref()
            .map(|id| id.corporation);
        let reduced_motion = self.reduced_motion;
        if let Some(scene) = self
            .guest_view
            .as_ref()
            .and_then(|view| view.replica.scene())
        {
            let camera = Vec2::from_array(hud.camera);
            let viewport = self
                .camera_zoom
                .world_extent(Vec2::from_array(self.renderer.viewport()));
            scene
                .render_into_with_home(
                    &mut self.frame,
                    game_core::co_op_replica::ViewInput {
                        camera,
                        viewport,
                        alpha,
                        reduced: self.reduced_cosmetics,
                        reduced_motion,
                    },
                    |frame, parent, time| {
                        crate::dock_art::append(
                            frame,
                            &crate::dock_art::DockInput {
                                parent,
                                corporation: scene.home_corporation(),
                                time,
                                reduced_motion,
                            },
                            &mut crate::dock_art::DockBudget::default(),
                        )
                        .is_some()
                    },
                )
                .map_err(js)?;
        } else {
            self.game
                .render_into_with_home(alpha, &mut self.frame, |frame, parent, time| {
                    crate::dock_art::append(
                        frame,
                        &crate::dock_art::DockInput {
                            parent,
                            corporation,
                            time,
                            reduced_motion,
                        },
                        &mut crate::dock_art::DockBudget::default(),
                    )
                    .is_some()
                });
        }
        self.ui_frame.clear();
        if self.guest_view.is_none() {
            hud.camera = self.game.render_camera(alpha);
        }
        let amount = if self.shake && !self.reduced_motion {
            hud.shake
        } else {
            0.0
        };
        let render_camera = [
            hud.camera[0] + (hud.time * 83.0).sin() * amount,
            hud.camera[1] + (hud.time * 71.0).cos() * amount,
        ];
        let [top, right, bottom, left] = self.ui_insets;
        let viewport = self.renderer.viewport();
        let ui_viewport = [viewport[0] - left - right, viewport[1] - top - bottom];
        let ui_camera = [
            render_camera[0] + (left - right) * 0.5,
            render_camera[1] + (top - bottom) * 0.5,
        ];
        if self.ui_mode == 1 || self.ui_scene.groups.is_empty() {
            let focus = self.current_focus(alpha);
            let boss_panel = if self.ui_mode == 1 {
                let panel = self.boss_overlay.update(&OverlayInput {
                    hud: &hud,
                    focus,
                    camera: render_camera,
                    viewport,
                    insets: self.ui_insets,
                    zoom: self.camera_zoom.value(),
                    now_ms: (self.presentation_time * 1000.0) as u64,
                    scene: &self.ui_scene,
                    touch: self.touch_ui,
                });
                self.last_boss_panel = panel.map(|[x, y, w, h]| [x + left, y + top, w, h]);
                panel
            } else {
                None
            };
            if self.ui_mode == 1 {
                if let Some(layout) =
                    pixel_ui::FlightStatusLayout::new(&pixel_ui::FlightStatusInput {
                        viewport,
                        insets: self.ui_insets,
                        header: pixel_ui::flight_header(&self.ui_scene, self.ui_insets),
                        status_hits: pixel_ui::flight_status_hits(&self.ui_scene, self.ui_insets),
                        touch: self.touch_ui,
                        attached_vitals: !self.boss_overlay.minimal(),
                        player: focus.player,
                        player_radius: focus.player_radius,
                        protected_core: focus.boss.map(|b| (b.core, b.core_radius)),
                        camera: hud.camera,
                        zoom: self.camera_zoom.value(),
                    })
                {
                    let semantic_context = self
                        .ui_scene
                        .groups
                        .iter()
                        .take(128)
                        .flat_map(|group| &group.nodes)
                        .take(512)
                        .any(|node| matches!(node.kind.as_str(), "status-text" | "warning-text"));
                    pixel_ui::draw_flight_status_animated(
                        &mut self.ui_frame,
                        &hud,
                        ui_camera,
                        &layout,
                        boss_panel,
                        self.ui_time,
                        self.reduced_motion,
                        semantic_context,
                        vitals,
                    );
                }
            } else {
                pixel_ui::draw_ui(
                    &mut self.ui_frame,
                    &hud,
                    ui_camera,
                    ui_viewport,
                    self.ui_mode,
                );
            }
        }
        if self.touch_ui && self.ui_mode == 1 {
            pixel_ui::draw_controls(
                &mut self.ui_frame,
                ui_camera,
                ui_viewport,
                self.ui_joystick,
                self.ui_fire,
            );
        }
        ui_scene::draw(
            &mut self.ui_frame,
            &self.ui_scene,
            render_camera,
            self.renderer.viewport(),
            self.ui_time,
            self.reduced_motion,
        );
        let frame = self.frame.frame();
        let ui_frame = self.ui_frame.frame();
        let light_sources = if let Some(scene) = self
            .guest_view
            .as_ref()
            .and_then(|view| view.replica.scene())
        {
            scene.light_sources(game_core::co_op_replica::ViewInput {
                camera: Vec2::from_array(hud.camera),
                viewport: self
                    .camera_zoom
                    .world_extent(Vec2::from_array(self.renderer.viewport())),
                alpha,
                reduced: self.reduced_cosmetics,
                reduced_motion,
            })
        } else {
            self.game.light_sources(alpha)
        };
        self.renderer.draw(
            frame,
            ui_frame,
            render_camera,
            self.camera_zoom.value(),
            &light_sources,
            hud.biome.kind,
        )?;
        let render_ms = now() - render_start;
        let profile =
            if self.guest_view.is_none() && self.game.profile().revision != self.last_revision {
                self.last_revision = self.game.profile().revision;
                Some(self.game.profile().clone())
            } else {
                None
            };
        let events = if self.guest_view.is_none() {
            self.game.take_events()
        } else {
            Vec::new()
        };
        if events.iter().any(|event| event.kind == "death") {
            self.vitals_animation.reset();
        }
        to_js(&FrameResponse {
            hud,
            events,
            profile,
            pixels: frame.pixel_count + ui_frame.pixel_count,
            simulation_ms,
            render_ms,
        })
    }
    pub fn hud(&self) -> Result<JsValue, JsValue> {
        to_js(&self.current_hud()?)
    }
    /// Bounded discovered-beacon snapshot, requested only by the open map.
    pub fn universe_map(&self) -> Result<JsValue, JsValue> {
        to_js(&self.game.universe_map())
    }
    pub fn profile(&self) -> Result<JsValue, JsValue> {
        to_js(self.game.profile())
    }
    /// Register only an untouched anonymous flight; never replace live progress.
    pub fn register_droid(
        &mut self,
        kind: &str,
        corporation: &str,
        sub_corporation: &str,
        hex_id: &str,
    ) -> Result<(), JsValue> {
        self.require_solo_actions()?;
        if !self.paused || !self.game.can_register_droid() {
            return Err(JsValue::from_str(
                "This flight already has a license or progression.",
            ));
        }
        let profile =
            game_core::identity::create_profile(kind, corporation, sub_corporation, hex_id)
                .map_err(js)?;
        self.game = Game::new(profile);
        let extent = self.world_viewport();
        self.game.set_viewport(extent[0], extent[1]);
        self.accumulator = 0.0;
        self.pending_destination = None;
        self.ui_joystick = None;
        self.ui_fire = false;
        self.boss_overlay.reset();
        self.vitals_animation.reset();
        self.last_revision = self.game.profile().revision;
        Ok(())
    }
    pub fn acknowledge_license(&mut self) -> Result<(), JsValue> {
        self.require_solo_actions()?;
        self.game.acknowledge_license().map_err(js)
    }
    pub fn purchase(&mut self, id: &str) -> Result<(), JsValue> {
        self.require_solo_actions()?;
        self.game.purchase(id).map_err(js)
    }
    /// Query the real Haven facility hit area without moving or advancing play.
    /// 0 upgrades, 1 hangar, 2 journal, 3 campaign; -1 is no available facility.
    pub fn dock_facility_at(&self, x: f32, y: f32) -> i32 {
        self.game.dock_facility_at([x, y]).map_or(-1, i32::from)
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        self.boss_overlay.reset();
        self.renderer.resize(width, height);
        self.update_world_viewport();
    }
    pub fn viewport(&self) -> Vec<f32> {
        self.renderer.viewport().to_vec()
    }
    /// Presentation target; logical UI sizing and simulation coordinates stay fixed.
    pub fn set_zoom(&mut self, value: f32) {
        self.camera_zoom.set_target(value);
        if self.reduced_motion {
            self.camera_zoom.snap();
            self.update_world_viewport();
        }
    }
    /// Current visible zoom, including its 150ms transition.
    pub fn zoom(&self) -> f32 {
        self.camera_zoom.value()
    }
    /// Current visible world dimensions. Map clicks around Hud.camera using
    /// this extent, never the pending target or the shaken presentation camera.
    pub fn world_viewport(&self) -> Vec<f32> {
        self.camera_zoom
            .world_extent(Vec2::from_array(self.renderer.viewport()))
            .to_array()
            .to_vec()
    }
    pub fn set_paused(&mut self, value: bool) {
        if self.paused != value {
            self.boss_overlay.reset();
        }
        self.paused = value;
        self.accumulator = 0.0;
        self.pending_destination = None;
        if self.guest_view.is_none() {
            self.game.set_paused(value);
        }
    }
    /// Cancel both a not-yet-stepped click and the live Game's persistent route.
    /// Existing drift and fixed-step accumulation continue under a live menu.
    pub fn cancel_course(&mut self) {
        self.pending_destination = None;
        if self.guest_view.is_none() {
            self.game.cancel_course();
        }
    }
    pub fn set_shake(&mut self, value: bool) {
        self.shake = value;
    }
    pub fn set_ui_mode(&mut self, mode: u8) {
        self.ui_mode = mode.min(3);
        if self.ui_mode != 1 {
            self.boss_overlay.reset();
        }
    }
    /// Last visible flight panel, in full logical-screen coordinates. The
    /// paused console can inspect it without advancing or changing combat.
    pub fn boss_ui_bounds(&self) -> Vec<f32> {
        self.last_boss_panel
            .map(|bounds| bounds.to_vec())
            .unwrap_or_default()
    }
    /// Full logical-screen footer bounds, shared with actual native painting.
    /// Empty means no supported safe rectangle under the supplied viewport.
    pub fn flight_status_bounds(&self) -> Vec<f32> {
        self.current_flight_status_layout()
            .and_then(|layout| {
                pixel_ui::FlightStatusLayout::full_bounds(layout.strip, self.ui_insets)
            })
            .map(|rect| rect.to_vec())
            .unwrap_or_default()
    }
    /// Seven-logical-pixel context glyph slot, distinct from accessible hits.
    pub fn flight_status_text_bounds(&self) -> Vec<f32> {
        self.current_flight_status_layout()
            .and_then(|layout| {
                pixel_ui::FlightStatusLayout::full_bounds(layout.context, self.ui_insets)
            })
            .map(|rect| rect.to_vec())
            .unwrap_or_default()
    }
    /// Full logical-screen minimap bounds, matching its native reservation.
    /// Empty means the map cannot fit safely; camera zoom does not resize it.
    pub fn flight_navigation_bounds(&self) -> Vec<f32> {
        self.current_flight_status_layout()
            .and_then(|layout| {
                pixel_ui::FlightStatusLayout::full_bounds(layout.navigation, self.ui_insets)
            })
            .map(|rect| rect.to_vec())
            .unwrap_or_default()
    }
    pub fn set_ui_scene(&mut self, scene: JsValue) -> Result<(), JsValue> {
        self.ui_scene = serde_wasm_bindgen::from_value(scene).map_err(js)?;
        Ok(())
    }
    pub fn set_reduced_motion(&mut self, value: bool) {
        self.reduced_motion = value;
        self.game.set_reduced_motion(value);
        if value {
            self.camera_zoom.snap();
            self.update_world_viewport();
        }
    }
    pub fn set_ui_insets(&mut self, top: f32, right: f32, bottom: f32, left: f32) {
        self.ui_insets = [top, right, bottom, left].map(|v| {
            if v.is_finite() {
                v.clamp(0.0, 100.0)
            } else {
                0.0
            }
        });
    }
    pub fn set_cosmetic_density(&mut self, value: bool) {
        self.reduced_cosmetics = value;
        self.game.set_cosmetic_density(value);
    }
    pub fn interact_site(&mut self) -> Result<(), JsValue> {
        self.require_solo_actions()?;
        self.game.interact_site().map_err(js)
    }
    pub fn start_champion(&mut self, index: usize) -> Result<(), JsValue> {
        self.require_solo_actions()?;
        self.game.start_champion(index).map_err(js)
    }
    pub fn start_tournament(&mut self) -> Result<(), JsValue> {
        self.require_solo_actions()?;
        self.game.start_tournament().map_err(js)
    }
    pub fn craft_ship(&mut self, index: usize) -> Result<(), JsValue> {
        self.require_solo_actions()?;
        self.game.craft_ship(index).map_err(js)
    }
    /// Craft into the shared dock inventory; this does not change any loadout.
    pub fn craft_ability(&mut self, id: &str) -> Result<(), JsValue> {
        self.require_solo_actions()?;
        self.game.craft_ability(id).map_err(js)
    }
    pub fn craft_component(&mut self, id: &str) -> Result<(), JsValue> {
        self.require_solo_actions()?;
        self.game.craft_component(id).map_err(js)
    }
    pub fn upgrade_ability(&mut self, id: &str) -> Result<(), JsValue> {
        self.require_solo_actions()?;
        self.game.upgrade_ability(id).map_err(js)
    }
    /// Slot 0 offense, 1 defense, 2 passive on the selected ship. A null or
    /// undefined ID clears that slot; all ownership/gates are enforced by Game.
    pub fn equip_ability(&mut self, slot: usize, id: Option<String>) -> Result<(), JsValue> {
        self.require_solo_actions()?;
        self.game.equip_ability(slot, id.as_deref()).map_err(js)
    }
    pub fn switch_ship(&mut self, id: u8) -> Result<(), JsValue> {
        self.require_solo_actions()?;
        self.game.switch_ship(id).map_err(js)
    }
    pub fn acknowledge_credits(&mut self) {
        if self.guest_view.is_some() || self.game.co_op_session().is_some() {
            return;
        }
        self.game.acknowledge_credits();
    }
    pub fn return_to_home(&mut self) {
        if self.guest_view.is_some() || self.game.co_op_session().is_some() {
            return;
        }
        self.boss_overlay.reset();
        self.last_boss_panel = None;
        self.game.reset_to_dock();
    }
    pub fn set_touch_ui(&mut self, value: bool) {
        self.touch_ui = value;
    }
    pub fn diagnostic(&mut self, command: &str) {
        if self.guest_view.is_some() {
            return;
        }
        self.boss_overlay.reset();
        self.last_boss_panel = None;
        self.game.diagnostic(command);
    }
    pub fn lose_device(&self) {
        self.renderer.lose_device();
    }
    /// Install a one-use candidate without suspending a borrow of this Game.
    /// Failed preparation leaves the live renderer and simulation untouched.
    pub fn install_recovery(&mut self, candidate: RendererRecovery) -> Result<(), JsValue> {
        if self.renderer.canvas != candidate.renderer.canvas {
            return Err(js("Graphics recovery belongs to a different canvas"));
        }
        let (width, height) = (
            candidate.renderer.canvas.width(),
            candidate.renderer.canvas.height(),
        );
        self.renderer = candidate.renderer;
        self.renderer.resize(width, height);
        self.boss_overlay.reset();
        self.last_boss_panel = None;
        self.update_world_viewport();
        Ok(())
    }
}

impl Galaxy {
    fn render_alpha(&self) -> f32 {
        self.guest_view
            .as_ref()
            .and_then(|view| {
                view.replica.scene().map(|scene| {
                    (view.age / scene.blend_seconds().max(1.0 / 240.0)).clamp(0.0, 1.0)
                })
            })
            .unwrap_or(self.accumulator * 60.0)
    }
    fn require_solo_actions(&self) -> Result<(), JsValue> {
        if self.guest_view.is_some() || self.game.co_op_session().is_some() {
            return Err(js("Return to your universe before changing progression."));
        }
        Ok(())
    }
    fn current_hud(&self) -> Result<Hud, JsValue> {
        let base = self.game.hud();
        if let Some(view) = &self.guest_view {
            if let Some(scene) = view.replica.scene() {
                return scene.guest_hud(view.peer, view.camera, base).map_err(js);
            }
        }
        Ok(base)
    }
    fn current_focus(&self, alpha: f32) -> game_core::RenderFocus {
        self.guest_view
            .as_ref()
            .and_then(|view| {
                view.replica
                    .scene()
                    .and_then(|scene| scene.focus(view.peer, alpha))
            })
            .unwrap_or_else(|| self.game.render_focus(alpha))
    }
    fn current_flight_status_layout(&self) -> Option<pixel_ui::FlightStatusLayout> {
        if self.ui_mode != 1 {
            return None;
        }
        let alpha = self.render_alpha();
        let focus = self.current_focus(alpha);
        pixel_ui::FlightStatusLayout::new(&pixel_ui::FlightStatusInput {
            viewport: self.renderer.viewport(),
            insets: self.ui_insets,
            header: pixel_ui::flight_header(&self.ui_scene, self.ui_insets),
            status_hits: pixel_ui::flight_status_hits(&self.ui_scene, self.ui_insets),
            touch: self.touch_ui,
            attached_vitals: !self.boss_overlay.minimal(),
            player: focus.player,
            player_radius: focus.player_radius,
            protected_core: focus.boss.map(|b| (b.core, b.core_radius)),
            camera: self.current_hud().ok()?.camera,
            zoom: self.camera_zoom.value(),
        })
    }
    fn update_world_viewport(&mut self) {
        let viewport = self
            .camera_zoom
            .world_extent(Vec2::from_array(self.renderer.viewport()));
        self.game.set_viewport(viewport.x, viewport.y);
    }
}
