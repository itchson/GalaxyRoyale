//! Actual engine-quad extraction and shared-layout proofs, not browser captures.
use super::*;
use crate::ui_scene::{UiGroup, UiNode, UiScene};

fn input(viewport: [f32; 2], touch: bool) -> FlightStatusInput {
    let game = game_core::Game::new(game_core::Profile::default());
    let focus = game.render_focus(1.0);
    FlightStatusInput {
        viewport,
        insets: [0.0; 4],
        header: None,
        status_hits: [None; 2],
        touch,
        attached_vitals: true,
        player: focus.player,
        player_radius: focus.player_radius,
        protected_core: None,
        camera: game.hud().camera,
        zoom: 1.0,
    }
}

fn corners(frame: &FrameBuilder) -> impl Iterator<Item = (u32, Vec2)> + '_ {
    frame.frame().batches.iter().flat_map(|b| {
        b.pixels.iter().flat_map(|p| {
            let t = b.transforms[p.transform as usize];
            assert_eq!(t.b[2..], [0.0; 2], "native tracks stay unlit");
            [
                Vec2::splat(-0.5),
                Vec2::new(-0.5, 0.5),
                Vec2::new(0.5, -0.5),
                Vec2::splat(0.5),
            ]
            .map(|c| (p.color, t.point(Vec2::from_array(p.local) + c)))
        })
    })
}

#[test]
fn tracks_are_two_symmetric_occupied_arcs_with_continuous_fractional_fill() {
    assert_eq!(VITAL_SEGMENT.occupied_count(), 7);
    let mut f = FrameBuilder::new();
    append_curved_vitals(
        &mut f,
        AffineTransform::default(),
        [0.0, 0.0, 44.0, 12.0],
        VitalsPresentation {
            fill: [1.0; 2],
            trail: [1.0; 2],
        },
        [MINT, CYAN],
    );
    assert_eq!(f.frame().pixel_count, 308);
    let mut points = Vec::new();
    for b in &f.frame().batches {
        for p in &b.pixels {
            if p.color >> 24 == 0x90 {
                let t = b.transforms[p.transform as usize];
                points.push(t.point(Vec2::from_array(p.local)));
            }
        }
    }
    assert_eq!(points.len(), 154);
    for &p in &points {
        assert!(points.contains(&Vec2::new(44.0 - p.x, p.y)), "mirror {p:?}");
    }
    for p in points.iter().filter(|p| p.y < 5.0) {
        assert!(points.contains(&(*p + Vec2::Y * 5.0)));
    }
    assert!(points.iter().any(|p| p.x == 22.0 && p.y == 6.5));
    assert!(points.iter().any(|p| p.x == 2.0 && p.y == 0.5));
    for (_, p) in corners(&f) {
        assert!(p.x >= 0.0 && p.x <= 44.0 && p.y >= 0.0 && p.y <= 12.0);
    }
    let widths = |health| {
        let mut f = FrameBuilder::new();
        append_curved_vitals(
            &mut f,
            AffineTransform::default(),
            [0.0, 0.0, 44.0, 12.0],
            VitalsPresentation {
                fill: [health; 2],
                trail: [health; 2],
            },
            [MINT, CYAN],
        );
        f.frame()
            .batches
            .iter()
            .flat_map(|b| b.transforms.iter().map(|t| t.a[2]))
            .collect::<Vec<_>>()
    };
    assert_ne!(widths(0.599), widths(0.601), "no rounded health buckets");
    assert!(
        widths(0.001).iter().any(|x| *x > 0.0 && *x < 0.04),
        "tiny positive fill retains true quad width"
    );
}

#[test]
fn grouped_tracks_preserve_all_corners_under_fractional_nested_affine_and_budget() {
    let parent = Transform {
        position: Vec2::new(25.25, -18.5),
        rotation: 0.3,
        scale: Vec2::new(1.5, 0.6),
    }
    .affine()
    .compose(
        Transform {
            rotation: -0.2,
            scale: Vec2::new(0.8, 1.3),
            ..Transform::default()
        }
        .affine(),
    );
    let rect = [7.25, 9.5, 55.0, 9.0];
    for fill in [0.0, 0.001, 0.4, 0.599, 0.601, 1.0, f32::NAN] {
        let mut f = FrameBuilder::new();
        append_curved_vitals(
            &mut f,
            parent,
            rect,
            VitalsPresentation {
                fill: [fill; 2],
                trail: [1.0; 2],
            },
            [MINT, CYAN],
        );
        assert!(f.frame().pixel_count >= 154 && f.frame().pixel_count <= 314);
        assert!(
            f.frame().batches.len() <= 1,
            "vitals alone fit portable transform table"
        );
        for (_, p) in corners(&f) {
            let local = parent.try_inverse_point(p).unwrap();
            assert!(local.is_finite());
            assert!(
                local.x >= rect[0] - 0.001
                    && local.x <= rect[0] + rect[2] + 0.001
                    && local.y >= rect[1] - 0.001
                    && local.y <= rect[1] + rect[3] + 0.001,
                "{local:?}"
            );
        }
        println!(
            "curved vitals fill={fill} quads={} transforms={} batches={}",
            f.frame().pixel_count,
            f.frame()
                .batches
                .iter()
                .map(|b| b.transforms.len())
                .sum::<usize>(),
            f.frame().batches.len()
        );
    }
}

#[test]
fn actual_animation_preserves_damage_hold_healing_and_reduced_motion_in_curve_packets() {
    let mut hud = game_core::Game::new(game_core::Profile::default()).hud();
    hud.dock = false;
    hud.max_shield = 100.0;
    hud.shield = 100.0;
    let mut animation = VitalsAnimation::default();
    animation.advance(&hud, 0.0, false);
    hud.hull = hud.max_hull * 0.4;
    let damage = animation.advance(&hud, 0.02, false);
    let packet = |presentation| {
        let mut frame = FrameBuilder::new();
        append_curved_vitals(
            &mut frame,
            AffineTransform::default(),
            [0.0, 0.0, 44.0, 12.0],
            presentation,
            [MINT, CYAN],
        );
        frame
            .frame()
            .batches
            .iter()
            .flat_map(|b| {
                b.pixels
                    .iter()
                    .map(|p| (p.color, p.local, b.transforms[p.transform as usize]))
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(damage.fill[0], 0.4);
    assert_eq!(damage.trail[0], 1.0);
    assert!(
        packet(damage).iter().any(|p| p.0 >> 24 == 0xa0),
        "actual held tail cells"
    );
    for _ in 0..30 {
        animation.advance(&hud, 0.1, false);
    }
    let settled = animation.advance(&hud, 0.0, false);
    assert_eq!(settled.fill, settled.trail);
    assert!(packet(settled).iter().all(|p| p.0 >> 24 != 0xa0));
    hud.hull = hud.max_hull * 0.9;
    let healing = animation.advance(&hud, 0.05, false);
    assert!(healing.fill[0] > 0.4 && healing.fill[0] < 0.9);
    assert_ne!(packet(healing), packet(settled));
    let reduced = animation.advance(&hud, 0.05, true);
    assert_eq!(reduced.fill, [0.9, 1.0]);
    assert_eq!(reduced.fill, reduced.trail);
    assert_eq!(packet(reduced), packet(animation.advance(&hud, 99.0, true)));
    assert!(packet(reduced).iter().all(|p| p.0 >> 24 != 0xa0));
}

#[test]
fn below_ship_placement_uses_actual_radius_zoom_and_keeps_right_footer_unchanged() {
    for camera in [Vec2::ZERO, Vec2::new(999_000.0, -999_000.0)] {
        for zoom in [0.6, 1.0, 1.6] {
            let mut i = input([533.0, 300.0], false);
            i.insets = [7.0, 9.0, 11.0, 13.0];
            i.camera = camera.to_array();
            i.player = camera + Vec2::new(20.0, -10.0);
            i.zoom = zoom;
            let reference = FlightStatusLayout::new(&i).unwrap();
            for radius in [i.player_radius, 24.0, 33.0] {
                i.player_radius = radius;
                let layout = FlightStatusLayout::new(&i).unwrap();
                let rect = layout.vitals.unwrap();
                let center = CameraView::new(camera, Vec2::from_array(i.viewport), zoom)
                    .unwrap()
                    .world_to_screen(i.player)
                    - Vec2::new(i.insets[3], i.insets[0]);
                assert!((rect[0] + 22.0 - center.x).abs() < 0.001);
                assert!((rect[1] - center.y - radius * zoom - 5.0).abs() < 0.001);
                assert_eq!(rect[2..], VITALS_SIZE);
                assert_eq!(layout.strip, reference.strip);
                assert_eq!(layout.navigation, reference.navigation);
                let mut f = FrameBuilder::new();
                append_curved_vitals(
                    &mut f,
                    AffineTransform::default(),
                    rect,
                    VitalsPresentation {
                        fill: [1.0; 2],
                        trail: [1.0; 2],
                    },
                    [MINT, CYAN],
                );
                for (_, p) in corners(&f) {
                    assert!(
                        p.y >= center.y + radius * zoom + 5.0 - 0.001,
                        "ship clearance"
                    );
                    assert!(p.distance(center) > radius * zoom);
                }
            }
        }
    }
}

#[test]
fn blocked_below_arc_falls_back_instead_of_covering_ship_controls_or_boss_core() {
    let mut i = input([533.0, 300.0], false);
    let original = FlightStatusLayout::new(&i).unwrap();
    let rect = original.vitals.unwrap();
    let view = CameraView::new(
        Vec2::from_array(i.camera),
        Vec2::from_array(i.viewport),
        i.zoom,
    )
    .unwrap();
    i.protected_core = Some((
        view.screen_to_world(Vec2::new(rect[0] + 22.0, rect[1] + 6.0)),
        4.0,
    ));
    assert!(FlightStatusLayout::new(&i).unwrap().vitals.is_none());
    i.protected_core = None;
    i.status_hits = [Some(rect), None];
    let blocked = FlightStatusLayout::new(&i).unwrap();
    assert!(blocked.vitals.is_none() && blocked.stats.is_some());
    i.status_hits = [None; 2];
    i.player = view.screen_to_world(Vec2::new(266.5, 285.0));
    assert!(
        FlightStatusLayout::new(&i).unwrap().vitals.is_none(),
        "no above-ship edge rescue"
    );
    for viewport in [[320.0, 180.0], [180.0, 140.0]] {
        let i = input(viewport, true);
        let l = FlightStatusLayout::new(&i).unwrap();
        if let Some(r) = l.vitals {
            assert!(
                touch_hit_rects(l.viewport)
                    .iter()
                    .all(|p| !hud_overlap(r, *p, 2.0))
            );
            assert!(l.strip.is_none_or(|s| !hud_overlap(r, s, 2.0)));
        } else {
            assert!(l.stats.is_some());
        }
    }
}

#[test]
fn actual_fullscreen_icon_rect_participates_in_safe_header_union() {
    let scene = UiScene {
        groups: vec![UiGroup {
            position: [7.25, 8.5],
            nodes: vec![UiNode {
                kind: "fullscreen-icon".into(),
                rect: [280.0, 4.0, 44.0, 44.0],
                clip: Some([280.0, 6.0, 30.0, 40.0]),
                ..Default::default()
            }],
        }],
    };
    assert_eq!(
        flight_header(&scene, [2.5, 0.0, 0.0, 3.25]),
        Some([284.0, 12.0, 30.0, 40.0])
    );
}
