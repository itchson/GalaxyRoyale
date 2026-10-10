//! WASM browser adapter for Galaxy Royale's portable Rust simulation.
use crate::{pixel_ui, renderer, ui_scene};

use engine_core::{CameraZoom, FrameBuilder, Vec2};
use game_core::{Game, GameEvent, Hud, InputFrame, Profile};
use serde::Serialize;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

#[wasm_bindgen]
pub struct Galaxy {
    game: Game,
    renderer: renderer::Renderer,
    frame: FrameBuilder,
    ui_frame: FrameBuilder,
    camera_zoom: CameraZoom,
    ui_mode: u8,
    ui_scene: ui_scene::UiScene,
    ui_time: f32,
    reduced_motion: bool,
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
    let mut game = Game::new(Profile::default());
    match stage {
        "fresh" => {}
        "champion" => game.diagnostic("champion-0"),
        _ => return Err(js("Unknown checkpoint contract stage")),
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
            reduced_motion: false,
            ui_insets: [0.0; 4],
            touch_ui: false,
            ui_joystick: None,
            ui_fire: false,
            accumulator: 0.0,
            pending_destination: None,
            last_revision: revision,
            paused: false,
            shake: true,
        })
    }
    pub fn step(&mut self, milliseconds: f64, input: JsValue) -> Result<JsValue, JsValue> {
        let delta = if milliseconds.is_finite() {
            (milliseconds as f32 / 1000.0).clamp(0.0, 0.1)
        } else {
            0.0
        };
        self.ui_time += delta;
        self.camera_zoom.advance(delta);
        self.update_world_viewport();
        let mut input: InputFrame = serde_wasm_bindgen::from_value(input).map_err(js)?;
        self.ui_joystick = input.joystick;
        self.ui_fire = input.fire;
        if let Some(destination) = input.destination {
            self.pending_destination = Some(destination);
        }
        let start = now();
        if !self.paused {
            self.accumulator += delta;
            let mut steps = 0;
            while self.accumulator >= 1.0 / 60.0 && steps < 5 {
                input.destination = self.pending_destination.take();
                self.game.tick(&input, 1.0 / 60.0);
                self.accumulator -= 1.0 / 60.0;
                steps += 1;
            }
            if steps == 5 {
                self.accumulator = self.accumulator.min(1.0 / 60.0);
            }
        }
        let simulation_ms = now() - start;
        let mut hud = self.game.hud();
        let render_start = now();
        let alpha = self.accumulator * 60.0;
        self.game.render_into(alpha, &mut self.frame);
        self.ui_frame.clear();
        hud.camera = self.game.render_camera(alpha);
        let amount = if self.shake { hud.shake } else { 0.0 };
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
            pixel_ui::draw_ui(
                &mut self.ui_frame,
                &hud,
                ui_camera,
                ui_viewport,
                self.ui_mode,
            );
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
        self.renderer.draw(
            frame,
            ui_frame,
            render_camera,
            self.camera_zoom.value(),
            &self.game.light_sources(alpha),
        )?;
        let render_ms = now() - render_start;
        let profile = if self.game.profile().revision != self.last_revision {
            self.last_revision = self.game.profile().revision;
            Some(self.game.profile().clone())
        } else {
            None
        };
        to_js(&FrameResponse {
            hud,
            events: self.game.take_events(),
            profile,
            pixels: frame.pixel_count + ui_frame.pixel_count,
            simulation_ms,
            render_ms,
        })
    }
    pub fn hud(&self) -> Result<JsValue, JsValue> {
        to_js(&self.game.hud())
    }
    pub fn profile(&self) -> Result<JsValue, JsValue> {
        to_js(self.game.profile())
    }
    pub fn purchase(&mut self, id: &str) -> Result<(), JsValue> {
        self.game.purchase(id).map_err(js)
    }
    pub fn resize(&mut self, width: u32, height: u32) {
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
        self.paused = value;
        self.accumulator = 0.0;
        self.pending_destination = None;
        self.game.set_paused(value);
    }
    pub fn set_shake(&mut self, value: bool) {
        self.shake = value;
    }
    pub fn set_ui_mode(&mut self, mode: u8) {
        self.ui_mode = mode.min(3);
    }
    pub fn set_ui_scene(&mut self, scene: JsValue) -> Result<(), JsValue> {
        self.ui_scene = serde_wasm_bindgen::from_value(scene).map_err(js)?;
        Ok(())
    }
    pub fn set_reduced_motion(&mut self, value: bool) {
        self.reduced_motion = value;
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
        self.game.set_cosmetic_density(value);
    }
    pub fn interact_site(&mut self) -> Result<(), JsValue> {
        self.game.interact_site().map_err(js)
    }
    pub fn start_champion(&mut self, index: usize) -> Result<(), JsValue> {
        self.game.start_champion(index).map_err(js)
    }
    pub fn start_tournament(&mut self) -> Result<(), JsValue> {
        self.game.start_tournament().map_err(js)
    }
    pub fn craft_ship(&mut self, index: usize) -> Result<(), JsValue> {
        self.game.craft_ship(index).map_err(js)
    }
    pub fn switch_ship(&mut self, id: u8) -> Result<(), JsValue> {
        self.game.switch_ship(id).map_err(js)
    }
    pub fn acknowledge_credits(&mut self) {
        self.game.acknowledge_credits();
    }
    pub fn return_to_home(&mut self) {
        self.game.reset_to_dock();
    }
    pub fn set_touch_ui(&mut self, value: bool) {
        self.touch_ui = value;
    }
    pub fn diagnostic(&mut self, command: &str) {
        self.game.diagnostic(command);
    }
    pub fn lose_device(&self) {
        self.renderer.lose_device();
    }
    pub async fn recover(&mut self) -> Result<(), JsValue> {
        let canvas = self.renderer.canvas.clone();
        let (width, height) = (canvas.width(), canvas.height());
        self.renderer = renderer::Renderer::new(canvas).await?;
        self.renderer.resize(width, height);
        self.update_world_viewport();
        Ok(())
    }
}

impl Galaxy {
    fn update_world_viewport(&mut self) {
        let viewport = self
            .camera_zoom
            .world_extent(Vec2::from_array(self.renderer.viewport()));
        self.game.set_viewport(viewport.x, viewport.y);
    }
}
