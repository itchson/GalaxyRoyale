//! Actual occupied-cell bounds and projection proofs; no browser claims.
use super::*;
use crate::ui_scene::{UiGroup, UiNode, UiScene};

fn input(viewport: [f32; 2], touch: bool) -> FlightStatusInput {
    FlightStatusInput {
        viewport,
        insets: [0.0; 4],
        header: None,
        status_hits: [None; 2],
        touch,
        attached_vitals: true,
        player: Vec2::ZERO,
        player_radius: 20.0,
        protected_core: None,
        camera: [0.0; 2],
        zoom: 1.0,
    }
}
fn hud() -> Hud {
    game_core::Game::new(game_core::Profile::default()).hud()
}
fn bounds_check(frame: &FrameBuilder, origin: Vec2, layout: FlightStatusLayout) {
    let bounds: Vec<_> = [layout.strip, layout.vitals]
        .into_iter()
        .flatten()
        .collect();
    for batch in &frame.frame().batches {
        for pixel in &batch.pixels {
            if pixel.color >> 24 == 0 {
                continue;
            }
            let transform = batch.transforms[pixel.transform as usize];
            assert_eq!(transform.b[2..], [0.0, 0.0], "UI remains unlit");
            for corner in [
                Vec2::splat(-0.5),
                Vec2::new(0.5, -0.5),
                Vec2::new(-0.5, 0.5),
                Vec2::splat(0.5),
            ] {
                let p = transform.point(Vec2::from_array(pixel.local) + corner) - origin;
                assert!(
                    p.x >= -0.001
                        && p.y >= -0.001
                        && p.x <= layout.viewport[0] + 0.001
                        && p.y <= layout.viewport[1] + 0.001,
                    "{p:?}"
                );
                assert!(
                    bounds.iter().any(|r| p.x >= r[0] - 0.001
                        && p.y >= r[1] - 0.001
                        && p.x <= r[0] + r[2] + 0.001
                        && p.y <= r[1] + r[3] + 0.001),
                    "pixel outside shared reservation {p:?}"
                );
            }
        }
    }
}
type PackedCell = ([f32; 2], u32, [f32; 4], [f32; 4]);
fn packet(frame: &FrameBuilder) -> Vec<PackedCell> {
    frame
        .frame()
        .batches
        .iter()
        .flat_map(|batch| {
            batch.pixels.iter().map(|pixel| {
                let t = batch.transforms[pixel.transform as usize];
                (pixel.local, pixel.color, t.a, t.b)
            })
        })
        .collect()
}

#[test]
fn footer_and_attached_bars_share_bounds_at_supported_extents_and_insets() {
    for (viewport, insets, hit, touch) in [
        ([533.0, 300.0], [0.0; 4], 22.0, false),
        ([320.0, 180.0], [0.0; 4], 44.0, true),
        ([350.0, 200.0], [7.0, 19.0, 13.0, 11.0], 22.375, true),
        ([180.0, 140.0], [0.0; 4], 22.0, true),
    ] {
        for zoom in [0.6_f32, 1.0, 1.6] {
            let scene = header_tests::scene(viewport, insets, hit);
            let mut i = input(viewport, touch);
            i.insets = insets;
            i.zoom = zoom;
            i.header = flight_header(&scene, insets);
            let layout = FlightStatusLayout::new(&i).unwrap();
            let strip = layout.strip.expect("declared essential footer slot");
            assert!(i.header.is_none_or(|r| !hud_overlap(strip, r, 0.0)));
            if touch {
                assert!(
                    touch_hit_rects(layout.viewport)
                        .iter()
                        .all(|r| !hud_overlap(strip, *r, 0.0))
                );
            }
            if let Some(vitals) = layout.vitals {
                assert!(!hud_overlap(vitals, strip, 0.0));
                assert_eq!([vitals[2], vitals[3]], [44.0, 9.0]);
            }
            let [top, right, bottom, left] = insets;
            let render_camera = [2.0 + (left - right) * 0.5, -1.0 + (top - bottom) * 0.5];
            let mut frame = FrameBuilder::new();
            draw_flight_status(
                &mut frame,
                &hud(),
                render_camera,
                &layout,
                None,
                0.2,
                false,
                false,
            );
            assert!(frame.frame().pixel_count > 0 && frame.frame().pixel_count < 2000);
            println!(
                "HUD geometry fixture viewport={viewport:?} insets={insets:?} touch={touch} zoom={zoom} cells={} transforms={} batches={}",
                frame.frame().pixel_count,
                frame
                    .frame()
                    .batches
                    .iter()
                    .map(|b| b.transforms.len())
                    .sum::<usize>(),
                frame.frame().batches.len()
            );
            bounds_check(
                &frame,
                Vec2::from_array(render_camera) - Vec2::from_array(layout.viewport) * 0.5,
                layout,
            );
            let full = FlightStatusLayout::full_bounds(layout.strip, insets).unwrap();
            assert_eq!(full, [strip[0] + left, strip[1] + top, strip[2], strip[3]]);
        }
    }
}

#[test]
fn current_unshaken_projection_scales_offset_and_radius_but_not_bar_geometry() {
    for world_camera in [Vec2::new(123.5, -89.25), Vec2::new(999000.0, -999000.0)] {
        for zoom in [0.6_f32, 1.0, 1.6] {
            let mut i = input([533.0, 300.0], false);
            i.camera = world_camera.to_array();
            i.player = world_camera + Vec2::new(20.0, 10.0);
            i.zoom = zoom;
            i.insets = [9.0, 13.0, 7.0, 5.0];
            let layout = FlightStatusLayout::new(&i).unwrap();
            let rect = layout.vitals.unwrap();
            let view = CameraView::new(world_camera, Vec2::from_array(i.viewport), zoom).unwrap();
            let p = view.world_to_screen(i.player) - Vec2::new(5.0, 9.0);
            assert!((rect[0] + 22.0 - p.x).abs() < 0.001);
            assert!((rect[1] + 20.0 * zoom + 14.0 - p.y).abs() < 0.001);
            assert_eq!([rect[2], rect[3]], [44.0, 9.0]);
            // Footer exports have no moving-world/zoom dependency.
            let mut stationary = input(i.viewport, false);
            stationary.insets = i.insets;
            assert_eq!(
                layout.strip,
                FlightStatusLayout::new(&stationary).unwrap().strip
            );
        }
    }
}

#[test]
fn offscreen_ship_never_becomes_a_clamped_edge_marker_and_invalid_projection_fails_closed() {
    let mut i = input([320.0, 180.0], false);
    i.player = Vec2::new(1000.0, 0.0);
    let layout = FlightStatusLayout::new(&i).unwrap();
    assert!(layout.vitals.is_none());
    assert!(layout.stats.is_some());
    for zoom in [f32::NAN, 0.0, 1.6001] {
        i.zoom = zoom;
        assert!(FlightStatusLayout::new(&i).is_none());
    }
    i.zoom = 1.6;
    i.insets = [0.0, 80.0, 0.0, 80.0];
    assert!(FlightStatusLayout::new(&i).is_none());
    i.insets = [0.0; 4];
    i.camera = [f32::INFINITY, 0.0];
    assert!(FlightStatusLayout::new(&i).is_none());
}

#[test]
fn attached_tracks_avoid_a_real_core_and_compact_boss_priority_retains_footer_health() {
    let mut i = input([533.0, 300.0], false);
    let original = FlightStatusLayout::new(&i).unwrap();
    let top = original.vitals.unwrap();
    // World coordinates are projected through the same actual CameraView.
    let view = CameraView::new(Vec2::ZERO, Vec2::from_array(i.viewport), 1.0).unwrap();
    i.protected_core = Some((
        view.screen_to_world(Vec2::new(top[0] + 22.0, top[1] + 4.5)),
        8.0,
    ));
    let protected = FlightStatusLayout::new(&i).unwrap();
    assert!(
        protected.vitals.unwrap()[1] > 150.0,
        "choose below ship instead of hiding its core"
    );
    i.attached_vitals = false;
    let compact = FlightStatusLayout::new(&i).unwrap();
    assert!(compact.vitals.is_none());
    assert_eq!(compact.strip, original.strip);
    let mut f = FrameBuilder::new();
    draw_flight_status(&mut f, &hud(), [0.0; 2], &compact, None, 0.0, true, false);
    bounds_check(&f, -Vec2::from_array(compact.viewport) * 0.5, compact);
    assert!(f.frame().pixel_count > 0);
}

#[test]
fn real_contact_hit_is_separate_from_font_slot_and_forces_truthful_footer_fallback() {
    let mut i = input([320.0, 180.0], true);
    i.header = Some([119.0, 12.0, 189.0, 44.0]);
    i.zoom = 1.6;
    let mut scene = UiScene {
        groups: vec![UiGroup {
            position: [0.0; 2],
            nodes: vec![UiNode {
                kind: "status-hit".into(),
                rect: [106.0, 136.0, 122.0, 44.0],
                clip: Some([0.0, 0.0, 320.0, 180.0]),
                ..Default::default()
            }],
        }],
    };
    i.status_hits = flight_status_hits(&scene, [0.0; 4]);
    let layout = FlightStatusLayout::new(&i).unwrap();
    assert_eq!(layout.context, Some([106.0, 164.0, 122.0, 7.0]));
    assert!(
        layout.vitals.is_none(),
        "actual44px hit blocks below-ship track"
    );
    scene.groups[0].nodes[0].clip = Some([110.0, 150.0, 60.0, 20.0]);
    assert_eq!(
        flight_status_hits(&scene, [2.0, 0.0, 0.0, 3.0]),
        [Some([107.0, 148.0, 60.0, 20.0]), None]
    );
    let mut frame = FrameBuilder::new();
    crate::ui_scene::draw(&mut frame, &scene, [0.0; 2], [320.0, 180.0], 0.0, false);
    assert_eq!(
        frame.frame().pixel_count,
        0,
        "hit metadata never paints a frame"
    );
}

#[test]
fn low_hull_and_depleted_shield_are_bounded_color_cues_with_static_reduced_motion() {
    let layout = FlightStatusLayout::new(&input([533.0, 300.0], false)).unwrap();
    let mut h = hud();
    h.max_hull = 100.0;
    h.hull = 25.0;
    h.max_shield = 20.0;
    h.shield = 0.0;
    let render = |time, reduced| {
        let mut f = FrameBuilder::new();
        draw_flight_status(&mut f, &h, [0.0; 2], &layout, None, time, reduced, false);
        packet(&f)
    };
    assert_eq!(render(0.0, true), render(17.25, true));
    let a = render(0.25, false);
    let b = render(0.75, false);
    assert_ne!(a, b);
    assert_eq!(a.len(), b.len());
    for (a, b) in a.iter().zip(&b) {
        assert_eq!((a.0, a.2, a.3), (b.0, b.2, b.3));
    }
    assert_eq!(warning_color(1, f32::NAN, false), RED);
    h.max_shield = 0.0;
    h.hull = 100.0;
    let mut frame = FrameBuilder::new();
    draw_flight_status(&mut frame, &h, [0.0; 2], &layout, None, 0.5, true, false);
    assert!(
        frame
            .frame()
            .batches
            .iter()
            .flat_map(|b| &b.pixels)
            .all(|p| p.color != RED && p.color != GOLD)
    );
    // No visual resurrection/alarm after an authoritative dead-player frame.
    h.hull = 0.0;
    h.max_shield = 20.0;
    h.shield = 0.0;
    let mut frame = FrameBuilder::new();
    draw_flight_status(&mut frame, &h, [0.0; 2], &layout, None, 0.5, true, false);
    assert!(
        frame
            .frame()
            .batches
            .iter()
            .flat_map(|b| &b.pixels)
            .all(|p| p.color != RED && p.color != GOLD)
    );
}

#[test]
fn semantic_status_line_clips_whole_glyph_cells_and_preserves_ui_budget_and_unlit_material() {
    for kind in ["status-text", "warning-text"] {
        let scene = UiScene {
            groups: vec![UiGroup {
                position: [9.25, 12.5],
                nodes: vec![UiNode {
                    kind: kind.into(),
                    rect: [4.0, 8.0, 72.5, 7.0],
                    text: "LOW HULL / TOO MUCH TEXT".into(),
                    font: 3.0,
                    value: 1.0,
                    clip: Some([6.25, 8.0, 60.5, 7.0]),
                    ..Default::default()
                }],
            }],
        };
        let mut f = FrameBuilder::new();
        crate::ui_scene::draw(&mut f, &scene, [0.0; 2], [320.0, 180.0], 0.25, true);
        assert!(f.frame().pixel_count > 0 && f.frame().pixel_count < 500);
        for b in &f.frame().batches {
            for p in &b.pixels {
                let t = b.transforms[p.transform as usize];
                assert_eq!(t.b[2..], [0.0, 0.0]);
                if p.color >> 24 == 0 {
                    continue;
                }
                for c in [Vec2::splat(-0.5), Vec2::splat(0.5)] {
                    let pos = t.point(Vec2::from_array(p.local) + c) + Vec2::new(160.0, 90.0)
                        - Vec2::new(9.25, 12.5);
                    assert!(
                        pos.x >= 6.25 && pos.x <= 66.75 && pos.y >= 8.0 && pos.y <= 15.0,
                        "{pos:?}"
                    );
                }
            }
        }
        let mut massive = UiScene::default();
        for _ in 0..128 {
            massive.groups.push(UiGroup {
                position: [0.0; 2],
                nodes: vec![UiNode {
                    kind: kind.into(),
                    rect: [0.0, 0.0, 640.0, 7.0],
                    text: "W".repeat(256),
                    ..Default::default()
                }],
            });
        }
        let mut f = FrameBuilder::new();
        crate::ui_scene::draw(&mut f, &massive, [0.0; 2], [640.0, 360.0], 0.0, false);
        assert!(f.frame().pixel_count <= crate::ui_scene::MAX_UI_PIXELS);
    }
}
