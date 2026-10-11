//! Native occupied-quad/navigation geometry, not browser or phone evidence.
use super::*;

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

fn fixture() -> Hud {
    let mut hud = game_core::Game::new(game_core::Profile::default()).hud();
    hud.position = [1200.0, 600.0];
    hud.navigation.quest = Some(game_core::NavigationTarget {
        position: [1800.0, -600.0],
        kind: 2,
        site_id: "presentation-fixture".into(),
        name: "Native marker fixture".into(),
    });
    hud
}

#[test]
fn world_relative_markers_preserve_cardinals_distance_and_circular_clamp() {
    let rect = [6.0, 40.0, 44.0, 44.0];
    let center = Vec2::new(28.0, 62.0);
    let player = Vec2::new(700_000.0, -900_000.0);
    for (delta, expected) in [
        (Vec2::new(1200.0, 0.0), Vec2::new(8.5, 0.0)),
        (Vec2::new(0.0, -1200.0), Vec2::new(0.0, -8.5)),
        (Vec2::new(-1200.0, 0.0), Vec2::new(-8.5, 0.0)),
        (Vec2::new(0.0, 1200.0), Vec2::new(0.0, 8.5)),
    ] {
        let marker = minimap_marker(rect, player, player + delta).unwrap();
        assert_eq!(marker.position, center + expected);
        assert!(!marker.off_range);
    }
    let far = minimap_marker(rect, player, player + Vec2::new(30_000.0, -40_000.0)).unwrap();
    assert!(far.off_range);
    assert!((far.position - center).distance(Vec2::new(10.2, -13.6)) < 0.0001);
    assert!(((far.position - center).length() - 17.0).abs() < 0.0001);
    assert_eq!(
        minimap_marker(rect, player, player).unwrap().position,
        center
    );
    assert!(
        !minimap_marker(rect, player, player + Vec2::X * 2400.0)
            .unwrap()
            .off_range
    );
    assert!(minimap_marker(rect, player, Vec2::splat(f32::NAN)).is_none());
    assert!(minimap_marker(rect, Vec2::splat(f32::MAX), Vec2::splat(-f32::MAX)).is_none());
}

#[test]
fn right_above_footer_minimap_avoids_actual_header_controls_and_allows_safe_insets() {
    for (viewport, insets, touch, hit) in [
        ([533.0, 300.0], [0.0; 4], false, 22.0),
        ([533.0, 300.0], [0.0; 4], true, 22.0),
        ([320.0, 180.0], [0.0; 4], true, 44.0),
        ([320.0, 180.0], [0.0; 4], false, 44.0),
        ([360.0, 204.0], [10.0, 22.0, 14.0, 18.0], true, 44.0),
        ([180.0, 140.0], [0.0; 4], false, 22.0),
    ] {
        for zoom in [0.6_f32, 1.0, 1.6] {
            let mut i = input(viewport, touch);
            i.insets = insets;
            i.zoom = zoom;
            let scene = header_tests::scene(viewport, insets, hit);
            i.header = flight_header(&scene, insets);
            let layout = FlightStatusLayout::new(&i).unwrap();
            let Some(map) = layout.navigation else {
                // At max zoom a central player disc and both real touch pads
                // can leave no clear right-side map. Essential text survives.
                assert!(touch && layout.viewport[0] <= 320.0 && zoom > 1.0);
                assert!(layout.stats.is_some());
                continue;
            };
            let footer = layout.strip.unwrap();
            assert_eq!([map[2], map[3]], [44.0, 44.0]);
            assert_eq!(map[0] + map[2], footer[0] + footer[2]);
            assert!(footer[2] <= 180.0);
            assert!(map[1] >= 0.0 && map[1] + map[3] + 2.0 <= layout.strip.unwrap()[1]);
            assert!(map[0] + map[2] <= layout.viewport[0]);
            assert!(i.header.is_none_or(|r| !hud_overlap(map, r, 2.0)));
            if touch {
                assert!(
                    touch_hit_rects(layout.viewport)
                        .iter()
                        .all(|r| !hud_overlap(map, *r, 2.0))
                );
            }
            assert!(layout.vitals.is_none_or(|r| !hud_overlap(map, r, 2.0)));
            let full = FlightStatusLayout::full_bounds(Some(map), insets).unwrap();
            assert_eq!(full, [map[0] + insets[3], map[1] + insets[0], 44.0, 44.0]);
        }
    }
}

#[test]
fn footer_moves_with_map_right_edge_and_retains_exact_context_exports() {
    for (view, insets, touch) in [
        ([533.0, 300.0], [0.0; 4], false),
        ([320.0, 180.0], [0.0; 4], false),
        ([533.0, 300.0], [0.0; 4], true),
        ([360.0, 204.0], [10.0, 22.0, 14.0, 18.0], true),
    ] {
        let mut i = input(view, touch);
        i.insets = insets;
        let l = FlightStatusLayout::new(&i).unwrap();
        let footer = l.strip.unwrap();
        assert!(footer[2] <= 180.0);
        let edge = if touch && footer[1] == l.viewport[1] - 28.0 {
            touch_hit_rects(l.viewport)[1][0] - 4.0
        } else {
            l.viewport[0] - 6.0
        };
        assert_eq!(footer[0] + footer[2], edge);
        if !touch {
            assert_eq!(
                footer,
                [l.viewport[0] - 186.0, l.viewport[1] - 28.0, 180.0, 22.0]
            );
        }
        let context = l.context.unwrap();
        assert_eq!(
            context,
            [footer[0] + 4.0, footer[1] + 12.0, footer[2] - 8.0, 5.0]
        );
        assert_eq!(
            FlightStatusLayout::full_bounds(Some(context), insets),
            Some([
                context[0] + insets[3],
                context[1] + insets[0],
                context[2],
                context[3]
            ])
        );
        if let Some(map) = l.navigation {
            assert_eq!(map[0] + map[2], footer[0] + footer[2]);
            assert!(!hud_overlap(map, footer, 2.0));
        }
        if touch {
            assert!(
                touch_hit_rects(l.viewport)
                    .iter()
                    .all(|hit| !hud_overlap(footer, *hit, 2.0))
            );
        }
    }
}

#[test]
fn overconstrained_map_fails_closed_without_hiding_footer_or_health() {
    let mut i = input([180.0, 140.0], true);
    let layout = FlightStatusLayout::new(&i).unwrap();
    assert!(layout.navigation.is_none());
    assert!(layout.stats.is_some());
    i = input([320.0, 180.0], true);
    let map = FlightStatusLayout::new(&i).unwrap().navigation.unwrap();
    i.status_hits[0] = Some(map);
    i.status_hits[1] = Some([
        map[0],
        touch_hit_rects(i.viewport)[1][1] - 4.0 - MINIMAP_SIZE,
        MINIMAP_SIZE,
        MINIMAP_SIZE,
    ]);
    assert!(FlightStatusLayout::new(&i).unwrap().navigation.is_none());
    i.status_hits = [None; 2];
    let view = CameraView::new(Vec2::ZERO, Vec2::from_array(i.viewport), 1.0).unwrap();
    i.protected_core = Some((
        view.screen_to_world(Vec2::new(map[0] + 22.0, map[1] + 22.0)),
        2.0,
    ));
    let blocked = FlightStatusLayout::new(&i).unwrap();
    if let Some(rescue) = blocked.navigation {
        let core = Vec2::new(map[0] + 22.0, map[1] + 22.0);
        let nearest = core.clamp(
            Vec2::new(rescue[0], rescue[1]),
            Vec2::new(rescue[0] + rescue[2], rescue[1] + rescue[3]),
        );
        assert!(core.distance(nearest) > 6.0);
    }
    assert!(blocked.stats.is_some());
}

#[test]
fn compact_boss_priority_omits_optional_map_and_nonboss_layout_restores_it() {
    let mut i = input([533.0, 300.0], false);
    i.protected_core = Some((Vec2::new(20.0, 20.0), 2.0));
    assert!(FlightStatusLayout::new(&i).unwrap().navigation.is_some());
    i.attached_vitals = false;
    let compact = FlightStatusLayout::new(&i).unwrap();
    assert!(compact.navigation.is_none() && compact.stats.is_some());
    i.protected_core = None;
    assert!(FlightStatusLayout::new(&i).unwrap().navigation.is_some());
}

#[test]
fn actual_minimap_quads_are_bounded_distinct_unlit_and_zoom_independent() {
    let hud = fixture();
    let mut reference = None;
    for zoom in [0.6_f32, 1.0, 1.6] {
        let mut i = input([533.0, 300.0], false);
        i.zoom = zoom;
        let rect = FlightStatusLayout::new(&i).unwrap().navigation.unwrap();
        let origin = Vec2::new(-266.5, -150.0);
        let mut frame = FrameBuilder::new();
        draw_navigation(
            &mut Ui {
                frame: &mut frame,
                origin,
            },
            &hud,
            rect,
        );
        let mut colors = [0; 3];
        let mut signature = Vec::new();
        for batch in &frame.frame().batches {
            for pixel in &batch.pixels {
                let t = batch.transforms[pixel.transform as usize];
                assert_eq!(t.b[2..], [0.0; 2]);
                signature.push((pixel.local, pixel.color, t.a, t.b));
                if pixel.color == MINIMAP_HOME {
                    colors[0] += 1;
                }
                if pixel.color == MINIMAP_MISSION {
                    colors[1] += 1;
                }
                if pixel.color == WHITE {
                    colors[2] += 1;
                }
                for corner in [
                    Vec2::splat(-0.5),
                    Vec2::new(0.5, -0.5),
                    Vec2::new(-0.5, 0.5),
                    Vec2::splat(0.5),
                ] {
                    let p = t.point(Vec2::from_array(pixel.local) + corner) - origin;
                    assert!(
                        p.x >= rect[0]
                            && p.y >= rect[1]
                            && p.x <= rect[0] + rect[2]
                            && p.y <= rect[1] + rect[3],
                        "{p:?}"
                    );
                }
            }
        }
        assert_eq!(colors, [9, 12, 5]);
        assert!(frame.frame().pixel_count < 100);
        if let Some(previous) = &reference {
            assert_eq!(&signature, previous);
        }
        reference = Some(signature);
    }
    assert_eq!(MINIMAP_HOME.to_le_bytes(), [255, 233, 133, 255]);
    assert_eq!(MINIMAP_MISSION.to_le_bytes(), [205, 138, 255, 255]);
}

#[test]
fn undiscovered_targets_are_not_invented_and_coincident_delivery_stays_distinct() {
    let mut hud = game_core::Game::new(game_core::Profile::default()).hud();
    assert!(hud.navigation.quest.is_none());
    let rect = [6.0, 40.0, 44.0, 44.0];
    let colors = |hud: &Hud| {
        let mut frame = FrameBuilder::new();
        draw_navigation(
            &mut Ui {
                frame: &mut frame,
                origin: Vec2::ZERO,
            },
            hud,
            rect,
        );
        frame
            .frame()
            .batches
            .iter()
            .flat_map(|b| b.pixels.iter().map(|p| p.color))
            .collect::<Vec<_>>()
    };
    let fresh = colors(&hud);
    assert!(fresh.contains(&MINIMAP_HOME) && !fresh.contains(&MINIMAP_MISSION));
    hud.navigation.quest = Some(game_core::NavigationTarget {
        position: hud.navigation.home,
        kind: 0,
        site_id: "native-delivery-fixture".into(),
        name: "Return home".into(),
    });
    let delivery = colors(&hud);
    assert!(delivery.contains(&MINIMAP_HOME) && delivery.contains(&MINIMAP_MISSION));
    hud.navigation.quest = None;
    assert_eq!(colors(&hud), fresh);
}

#[test]
fn only_player_child_rotates_and_presentation_shake_does_not_change_map_bearings() {
    let mut hud = fixture();
    let rect = [6.0, 40.0, 44.0, 44.0];
    let extract = |hud: &Hud, origin: Vec2| {
        let mut frame = FrameBuilder::new();
        draw_navigation(
            &mut Ui {
                frame: &mut frame,
                origin,
            },
            hud,
            rect,
        );
        frame
            .frame()
            .batches
            .iter()
            .flat_map(|b| {
                b.pixels.iter().map(|p| {
                    let t = b.transforms[p.transform as usize];
                    (
                        p.color,
                        t.point(Vec2::from_array(p.local)) - origin,
                        t.a[2..].to_vec(),
                        t.b[..2].to_vec(),
                    )
                })
            })
            .collect::<Vec<_>>()
    };
    hud.heading = 0.0;
    let right = extract(&hud, Vec2::ZERO);
    hud.heading = std::f32::consts::FRAC_PI_2;
    let down = extract(&hud, Vec2::ZERO);
    assert_eq!(
        right.iter().filter(|p| p.0 != WHITE).collect::<Vec<_>>(),
        down.iter().filter(|p| p.0 != WHITE).collect::<Vec<_>>()
    );
    assert_ne!(
        right.iter().filter(|p| p.0 == WHITE).collect::<Vec<_>>(),
        down.iter().filter(|p| p.0 == WHITE).collect::<Vec<_>>()
    );
    let translated = extract(&hud, Vec2::new(999_000.0, -999_000.0));
    assert_eq!(down.len(), translated.len());
    for (a, b) in down.iter().zip(translated) {
        assert_eq!((a.0, &a.2, &a.3), (b.0, &b.2, &b.3));
        assert!(
            a.1.distance(b.1) <= 0.1,
            "far UI affine rounding remains bounded"
        );
    }
}
