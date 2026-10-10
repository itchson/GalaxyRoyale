//! Presentation-only adapter between the renderer's projection and pixel HUD.
use crate::{
    boss_layout,
    pixel_ui::{flight_layout, touch_hit_rects},
    ui_scene::UiScene,
};
use boss_layout::{FrameInput, LayoutController, Outcome, Point, Rect, Screen, WorldDisc};
use engine_core::{CameraView, Vec2};
use game_core::{Hud, RenderFocus};

#[derive(Default)]
pub struct BossOverlay {
    layout: LayoutController,
    encounter: Option<u64>,
    minimal: bool,
}

pub struct OverlayInput<'a> {
    pub hud: &'a Hud,
    pub focus: RenderFocus,
    pub camera: [f32; 2],
    pub viewport: [f32; 2],
    pub insets: [f32; 4],
    pub zoom: f32,
    pub now_ms: u64,
    pub scene: &'a UiScene,
    pub touch: bool,
}

impl BossOverlay {
    pub fn reset(&mut self) {
        self.layout.reset();
        self.encounter = None;
        self.minimal = false;
    }

    pub fn minimal(&self) -> bool {
        self.minimal
    }

    /// Returns safe-area-local UI bounds. Every protected target is projected
    /// through the same f32 CameraView as the world renderer before allocation.
    pub fn update(&mut self, input: &OverlayInput<'_>) -> Option<[f32; 4]> {
        let Some(boss) = input.focus.boss.filter(|_| input.hud.boss.is_some()) else {
            self.reset();
            return None;
        };
        if self.encounter != Some(boss.encounter) {
            self.reset();
            self.encounter = Some(boss.encounter);
        }
        let viewport = Vec2::from_array(input.viewport);
        let view = CameraView::new(Vec2::from_array(input.camera), viewport, input.zoom)?;
        if input.insets.iter().any(|v| !v.is_finite() || *v < 0.0)
            || !(engine_core::MIN_CAMERA_ZOOM..=engine_core::MAX_CAMERA_ZOOM).contains(&input.zoom)
        {
            return None;
        }
        let screen = Screen {
            width: f64::from(viewport.x),
            height: f64::from(viewport.y),
            insets: input.insets.map(f64::from),
        };
        let safe_width = viewport.x - input.insets[1] - input.insets[3];
        let safe_height = viewport.y - input.insets[0] - input.insets[2];
        if safe_width < 180.0 || safe_height < 140.0 {
            return None;
        }
        self.minimal |= safe_width < 220.0;
        let point = |world| {
            let p = view.world_to_screen(world) - viewport * 0.5;
            Point::new(f64::from(p.x), f64::from(p.y))
        };
        let reserved = Reserved::from_input(input, self.minimal);
        let [top, _, _, left] = input.insets;
        if reserved.overflow {
            return None;
        }
        let frame_input = |reserved| FrameInput {
            screen,
            projection: boss_layout::Projection {
                camera: Point::new(0.0, 0.0),
                zoom: 1.0,
            },
            head: point(boss.center),
            core: Some(WorldDisc::core(
                point(boss.core),
                f64::from(boss.core_radius * input.zoom),
            )),
            player: Some(WorldDisc::player(
                point(input.focus.player),
                f64::from(input.focus.player_radius * input.zoom),
            )),
            reserved,
            now_ms: input.now_ms,
            phase_epoch: boss.phase_epoch,
        };
        let mut outcome = self
            .layout
            .update(&frame_input(&reserved.rects[..reserved.len]));
        if outcome == Outcome::NoClearAnchor && !self.minimal {
            // Prioritize the boss, hull and actual controls. Keep this choice
            // for the encounter so moving cores cannot flicker telemetry.
            self.minimal = true;
            self.layout.reset();
            let essential = Reserved::from_input(input, true);
            if !essential.overflow {
                outcome = self
                    .layout
                    .update(&frame_input(&essential.rects[..essential.len]));
            }
        }
        match outcome {
            Outcome::Placed(placement) => {
                let r = placement.screen_rect;
                Some([
                    r.x as f32 - left,
                    r.y as f32 - top,
                    r.width as f32,
                    r.height as f32,
                ])
            }
            Outcome::NoClearAnchor | Outcome::InvalidInput => None,
        }
    }
}

struct Reserved {
    rects: [Rect; boss_layout::MAX_RESERVED_RECTS],
    len: usize,
    overflow: bool,
}
impl Default for Reserved {
    fn default() -> Self {
        Self {
            rects: [Rect::new(0.0, 0.0, 1.0, 1.0); boss_layout::MAX_RESERVED_RECTS],
            len: 0,
            overflow: false,
        }
    }
}
impl Reserved {
    fn from_input(input: &OverlayInput<'_>, minimal: bool) -> Self {
        let mut reserved = Self::default();
        let [top, right, bottom, left] = input.insets;
        let [width, height] = [
            input.viewport[0] - left - right,
            input.viewport[1] - top - bottom,
        ];
        if let Some(layout) = flight_layout(input.hud, [width, height], minimal, input.touch) {
            for mut rect in [layout.biome, layout.currency, layout.radar, layout.meter]
                .into_iter()
                .flatten()
            {
                rect[0] += left;
                rect[1] += top;
                reserved.add(rect);
            }
        }
        if input.touch {
            // Actual DOM hit regions, including safe-area offsets.
            for mut rect in touch_hit_rects([width, height]) {
                rect[0] += left;
                rect[1] += top;
                reserved.add(rect);
            }
        }
        reserved.scene(input.scene);
        reserved
    }
    fn add(&mut self, rect: [f32; 4]) {
        if rect.iter().any(|v| !v.is_finite()) || rect[2] <= 0.0 || rect[3] <= 0.0 {
            return;
        }
        if self.len == self.rects.len() {
            self.overflow = true;
            return;
        }
        self.rects[self.len] = Rect::new(
            f64::from(rect[0]),
            f64::from(rect[1]),
            f64::from(rect[2]),
            f64::from(rect[3]),
        );
        self.len += 1;
    }

    fn scene(&mut self, scene: &UiScene) {
        let mut nodes = 0;
        for group in scene.groups.iter().take(128) {
            if group
                .position
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 8192.0)
            {
                continue;
            }
            for node in &group.nodes {
                if nodes == 512 {
                    return;
                }
                nodes += 1;
                if !matches!(
                    node.kind.as_str(),
                    "button"
                        | "text"
                        | "nav-tab"
                        | "primary-button"
                        | "exit-button"
                        | "offense-button"
                        | "defense-button"
                        | "passive-button"
                        | "offense-text"
                        | "defense-text"
                        | "passive-text"
                ) {
                    continue;
                }
                let mut rect = node.rect;
                if rect.iter().any(|v| !v.is_finite() || v.abs() > 8192.0)
                    || node.clip.is_some_and(|clip| {
                        clip.iter().any(|v| !v.is_finite() || v.abs() > 8192.0)
                            || clip[2] <= 0.0
                            || clip[3] <= 0.0
                    })
                {
                    continue;
                }
                if let Some(clip) = node.clip {
                    let left = rect[0].max(clip[0]);
                    let top = rect[1].max(clip[1]);
                    rect = [
                        left,
                        top,
                        (rect[0] + rect[2]).min(clip[0] + clip[2]) - left,
                        (rect[1] + rect[3]).min(clip[1] + clip[3]) - top,
                    ];
                }
                rect[0] += group.position[0];
                rect[1] += group.position[1];
                self.add(rect);
                if self.overflow {
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_scene::{UiGroup, UiNode};

    #[test]
    fn semantic_buttons_reserve_actual_clipped_group_bounds() {
        let scene = UiScene {
            groups: vec![UiGroup {
                position: [10.0, 20.0],
                nodes: vec![
                    UiNode {
                        kind: "button".into(),
                        rect: [30.0, 40.0, 80.0, 40.0],
                        clip: Some([50.0, 45.0, 100.0, 20.0]),
                        ..UiNode::default()
                    },
                    UiNode {
                        kind: "panel".into(),
                        rect: [0.0, 0.0, 600.0, 200.0],
                        ..UiNode::default()
                    },
                    UiNode {
                        kind: "button".into(),
                        rect: [0.0, 0.0, 5.0, 5.0],
                        clip: Some([50.0, 50.0, 5.0, 5.0]),
                        ..UiNode::default()
                    },
                ],
            }],
        };
        let mut reserved = Reserved::default();
        reserved.scene(&scene);
        assert_eq!(reserved.len, 1);
        assert_eq!(reserved.rects[0], Rect::new(60.0, 65.0, 60.0, 20.0));
    }

    #[test]
    fn semantic_action_and_category_kinds_reserve_the_same_clipped_rects() {
        for kind in [
            "button",
            "text",
            "nav-tab",
            "primary-button",
            "exit-button",
            "offense-button",
            "defense-button",
            "passive-button",
            "offense-text",
            "defense-text",
            "passive-text",
        ] {
            let scene = UiScene {
                groups: vec![UiGroup {
                    position: [10.0, 20.0],
                    nodes: vec![UiNode {
                        kind: kind.into(),
                        rect: [30.0, 40.0, 80.0, 40.0],
                        clip: Some([50.0, 45.0, 100.0, 20.0]),
                        ..Default::default()
                    }],
                }],
            };
            let mut reserved = Reserved::default();
            reserved.scene(&scene);
            assert_eq!(reserved.len, 1, "semantic exclusion missing: {kind}");
            assert_eq!(reserved.rects[0], Rect::new(60.0, 65.0, 60.0, 20.0));
        }
    }

    #[test]
    fn inset_reservations_cover_actual_touch_aware_pixels_and_allocated_boss_corners() {
        let mut game = game_core::Game::new(game_core::Profile::default());
        game.diagnostic("boss-bastion-5");
        let hud = game.hud();
        let camera = Vec2::from_array(game.render_camera(0.5));
        for (viewport, insets) in [
            ([240.0, 180.0], [0.0, 20.0, 0.0, 20.0]),
            ([350.0, 200.0], [7.0, 19.0, 13.0, 11.0]),
            ([400.0, 240.0], [4.0, 23.0, 7.0, 11.0]),
        ] {
            for minimal in [false, true] {
                let mut focus = game.render_focus(0.5);
                focus.player = camera;
                focus.player_radius = 20.0;
                focus.boss.as_mut().unwrap().core = camera - Vec2::Y * 70.0;
                let scene = UiScene::default();
                let input = OverlayInput {
                    hud: &hud,
                    focus,
                    camera: camera.to_array(),
                    viewport,
                    insets,
                    zoom: 1.6,
                    now_ms: 0,
                    scene: &scene,
                    touch: true,
                };
                let [top, right, bottom, left] = insets;
                let safe = [viewport[0] - left - right, viewport[1] - top - bottom];
                // Encounter reset deliberately restores auto allocation; seed
                // it explicitly so the test exercises both reservation modes.
                let mut overlay = BossOverlay {
                    minimal,
                    encounter: focus.boss.map(|b| b.encounter),
                    ..Default::default()
                };
                let panel = overlay
                    .update(&input)
                    .expect("declared clear compact geometry retains boss");
                let reserved = Reserved::from_input(&input, overlay.minimal());
                let full_panel = Rect::new(
                    (panel[0] + left).into(),
                    (panel[1] + top).into(),
                    panel[2].into(),
                    panel[3].into(),
                );
                assert!(
                    reserved.rects[..reserved.len]
                        .iter()
                        .all(|r| !full_panel.intersects(r.expanded(boss_layout::WIDGET_CLEARANCE)))
                );
                let mut frame = engine_core::FrameBuilder::new();
                let ui_camera = [
                    camera.x + (left - right) * 0.5,
                    camera.y + (top - bottom) * 0.5,
                ];
                crate::pixel_ui::draw_ui_with_boss_panel_and_touch(
                    &mut frame,
                    &hud,
                    ui_camera,
                    safe,
                    1,
                    Some(panel),
                    overlay.minimal(),
                    true,
                );
                crate::pixel_ui::draw_controls(&mut frame, ui_camera, safe, None, false);
                let layout = flight_layout(&hud, safe, overlay.minimal(), true).unwrap();
                let mut covered: Vec<_> =
                    [layout.biome, layout.currency, layout.radar, layout.meter]
                        .into_iter()
                        .flatten()
                        .map(|r| {
                            Rect::new(
                                (r[0] + left).into(),
                                (r[1] + top).into(),
                                r[2].into(),
                                r[3].into(),
                            )
                        })
                        .collect();
                covered.extend(touch_hit_rects(safe).map(|r| {
                    Rect::new(
                        (r[0] + left).into(),
                        (r[1] + top).into(),
                        r[2].into(),
                        r[3].into(),
                    )
                }));
                covered.push(full_panel);
                for batch in &frame.frame().batches {
                    for pixel in batch.pixels.iter().filter(|p| p.color >> 24 != 0) {
                        for corner in [
                            Vec2::new(-0.5, -0.5),
                            Vec2::new(0.5, -0.5),
                            Vec2::new(-0.5, 0.5),
                            Vec2::new(0.5, 0.5),
                        ] {
                            let p = batch.transforms[pixel.transform as usize]
                                .point(Vec2::from_array(pixel.local) + corner)
                                - camera
                                + Vec2::from_array(viewport) * 0.5;
                            assert!(
                                p.x >= left - 0.001
                                    && p.x <= viewport[0] - right + 0.001
                                    && p.y >= top - 0.001
                                    && p.y <= viewport[1] - bottom + 0.001
                            );
                            let p = Point::new(p.x.into(), p.y.into());
                            assert!(
                                covered
                                    .iter()
                                    .any(|r| r.expanded(0.001).distance_to(p) == 0.0),
                                "paint beyond shared exclusion: {p:?}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn genuinely_overconstrained_small_touch_view_never_places_over_essential_controls() {
        let mut game = game_core::Game::new(game_core::Profile::default());
        game.diagnostic("boss-bastion-5");
        let hud = game.hud();
        let camera = Vec2::from_array(game.render_camera(0.5));
        let mut focus = game.render_focus(0.5);
        focus.player = camera;
        focus.player_radius = 20.0;
        focus.boss.as_mut().unwrap().core = camera - Vec2::Y * 70.0;
        let scene = UiScene::default();
        let input = OverlayInput {
            hud: &hud,
            focus,
            camera: camera.to_array(),
            viewport: [180.0, 140.0],
            insets: [0.0; 4],
            zoom: 1.6,
            now_ms: 0,
            scene: &scene,
            touch: true,
        };
        let mut overlay = BossOverlay::default();
        assert!(
            overlay.update(&input).is_none(),
            "no unsafe100x40 rectangle in the actual fixed-pad geometry"
        );
        assert!(overlay.minimal());
        assert!(
            flight_layout(&hud, input.viewport, true, true)
                .unwrap()
                .meter
                .is_some()
        );
    }

    #[test]
    fn visible_toast_and_destination_text_reserve_their_rendered_bounds() {
        let scene = UiScene {
            groups: vec![UiGroup {
                position: [10.0, 20.0],
                nodes: vec![
                    UiNode {
                        kind: "text".into(),
                        text: "QUEST READY".into(),
                        rect: [40.0, 80.0, 260.0, 24.0],
                        clip: Some([0.0, 0.0, 240.0, 200.0]),
                        ..UiNode::default()
                    },
                    UiNode {
                        kind: "text".into(),
                        text: "+".into(),
                        rect: [180.0, 30.0, 20.0, 20.0],
                        ..UiNode::default()
                    },
                ],
            }],
        };
        let mut reserved = Reserved::default();
        reserved.scene(&scene);
        assert_eq!(
            &reserved.rects[..reserved.len],
            &[
                Rect::new(50.0, 100.0, 200.0, 24.0),
                Rect::new(190.0, 50.0, 20.0, 20.0)
            ]
        );
    }

    #[test]
    fn reservations_are_bounded_and_overflow_fails_closed() {
        let mut reserved = Reserved::default();
        for _ in 0..boss_layout::MAX_RESERVED_RECTS {
            reserved.add([0.0, 0.0, 1.0, 1.0]);
        }
        assert!(!reserved.overflow);
        reserved.add([0.0, 0.0, 1.0, 1.0]);
        assert!(reserved.overflow);
        assert_eq!(reserved.len, boss_layout::MAX_RESERVED_RECTS);
    }

    #[test]
    fn actual_projection_and_native_widgets_clear_core_and_player_across_zoom() {
        for viewport in [
            [533.0, 300.0],
            [720.0, 360.0],
            [400.0, 240.0],
            [320.0, 180.0],
        ] {
            for zoom in [0.6_f32, 1.0, 1.6_f32] {
                for touch in [false, true] {
                    let mut game = game_core::Game::new(game_core::Profile::default());
                    game.diagnostic("boss-bastion-5");
                    let hud = game.hud();
                    let mut focus = game.render_focus(0.5);
                    let camera = Vec2::from_array(game.render_camera(0.5)) + Vec2::new(2.0, -1.0);
                    focus.player = camera;
                    focus.player_radius = 20.0;
                    let boss = {
                        let boss = focus.boss.as_mut().unwrap();
                        boss.core = camera + Vec2::new(0.0, -70.0);
                        boss.center = boss.core;
                        *boss
                    };
                    let scene = UiScene::default();
                    let input = OverlayInput {
                        hud: &hud,
                        focus,
                        camera: camera.to_array(),
                        viewport,
                        insets: [0.0; 4],
                        zoom,
                        now_ms: 0,
                        scene: &scene,
                        touch,
                    };
                    let [x, y, w, h] = BossOverlay::default()
                        .update(&input)
                        .expect("supported landscape must retain readable boss health/phase");
                    let panel = Rect::new(x.into(), y.into(), w.into(), h.into());
                    let view = CameraView::new(camera, Vec2::from_array(viewport), zoom).unwrap();
                    for (p, r, padding, min) in [
                        (focus.player, focus.player_radius, 4.0_f32, 24.0_f32),
                        (boss.core, boss.core_radius, 8.0, 32.0),
                    ] {
                        let p = view.world_to_screen(p);
                        assert!(
                            panel.distance_to(Point::new(p.x.into(), p.y.into()))
                                > (r * zoom + padding).max(min).into()
                        );
                    }
                    assert!(
                        Rect::new(0.0, 0.0, viewport[0].into(), viewport[1].into()).contains(panel)
                    );
                }
            }
        }
    }

    #[test]
    fn narrow_safe_area_emits_the_allocated_compact_panel_and_touch_controls() {
        let mut game = game_core::Game::new(game_core::Profile::default());
        game.diagnostic("boss-bastion-5");
        let hud = game.hud();
        let camera = Vec2::from_array(game.render_camera(0.5));
        let mut focus = game.render_focus(0.5);
        focus.player = camera;
        focus.player_radius = 20.0;
        focus.boss.as_mut().unwrap().core = camera - Vec2::Y * 70.0;
        let scene = UiScene::default();
        let mut input = OverlayInput {
            hud: &hud,
            focus,
            camera: camera.to_array(),
            viewport: [240.0, 180.0],
            insets: [0.0, 20.0, 0.0, 20.0],
            zoom: 1.6,
            now_ms: 0,
            scene: &scene,
            touch: true,
        };
        let mut overlay = BossOverlay::default();
        let panel = overlay
            .update(&input)
            .expect("200px safe area retains compact row");
        assert!(overlay.minimal());
        let mut frame = engine_core::FrameBuilder::new();
        crate::pixel_ui::draw_ui_with_boss_panel_and_touch(
            &mut frame,
            &hud,
            [0.0; 2],
            [200.0, 180.0],
            1,
            Some(panel),
            overlay.minimal(),
            true,
        );
        let hud_count = frame.frame().pixel_count;
        assert!(
            hud_count > 0,
            "allocated row must actually emit native pixels"
        );
        crate::pixel_ui::draw_controls(&mut frame, [0.0; 2], [200.0, 180.0], Some([0.0; 2]), false);
        assert!(
            frame.frame().pixel_count > hud_count,
            "small safe area keeps visible touch controls"
        );
        input.insets = [0.0, 40.0, 0.0, 40.0];
        assert!(
            overlay.update(&input).is_none(),
            "sub180 safe area cannot report an invisible row"
        );
    }

    #[test]
    fn small_touch_boss_keeps_critical_widgets_and_restores_telemetry_after_encounter() {
        let mut game = game_core::Game::new(game_core::Profile::default());
        game.diagnostic("boss-bastion-5");
        let hud = game.hud();
        let mut focus = game.render_focus(0.5);
        let camera = Vec2::from_array(game.render_camera(0.5));
        focus.player = camera;
        focus.player_radius = 20.0;
        focus.boss.as_mut().unwrap().core = camera - Vec2::Y * 70.0;
        let scene = UiScene {
            groups: vec![UiGroup {
                nodes: vec![
                    UiNode {
                        kind: "button".into(),
                        rect: [278.0, 12.0, 30.0, 30.0],
                        ..UiNode::default()
                    },
                    UiNode {
                        kind: "button".into(),
                        rect: [200.0, 50.0, 100.0, 24.0],
                        ..UiNode::default()
                    },
                    UiNode {
                        kind: "button".into(),
                        rect: [200.0, 80.0, 100.0, 24.0],
                        ..UiNode::default()
                    },
                ],
                ..UiGroup::default()
            }],
        };
        let mut input = OverlayInput {
            hud: &hud,
            focus,
            camera: camera.to_array(),
            viewport: [320.0, 180.0],
            insets: [0.0; 4],
            zoom: 1.6,
            now_ms: 0,
            scene: &scene,
            touch: true,
        };
        let mut overlay = BossOverlay::default();
        let panel = overlay
            .update(&input)
            .expect("compact boss health must survive expanded quick menu");
        let retained_minimal = overlay.minimal();
        let essential = Reserved::from_input(&input, retained_minimal);
        let panel = Rect::new(
            panel[0].into(),
            panel[1].into(),
            panel[2].into(),
            panel[3].into(),
        );
        assert!(
            essential.rects[..essential.len]
                .iter()
                .all(|r| !panel.intersects(r.expanded(boss_layout::WIDGET_CLEARANCE)))
        );
        input.now_ms = 16;
        input.focus.boss.as_mut().unwrap().core += Vec2::X;
        assert!(overlay.update(&input).is_some());
        assert_eq!(
            overlay.minimal(),
            retained_minimal,
            "the shared compact geometry must not flicker optional telemetry as the core moves"
        );
        input.focus.boss = None;
        assert!(overlay.update(&input).is_none());
        assert!(
            !overlay.minimal(),
            "encounter completion restores ordinary telemetry"
        );
    }
}
