//! Measured-node geometry fixtures, not claimed browser/DOM captures.
use super::*;
use crate::ui_scene::{UiGroup, UiNode, UiScene};

pub(crate) fn scene(view: [f32; 2], insets: [f32; 4], hit: f32) -> UiScene {
    let [top, right, _, left] = insets;
    let width = hit * 4.0 + 13.0;
    let start = view[0] - right - 12.0 - width;
    let mut x = start;
    let nodes = ["save-icon", "status-icon", "map-icon", "settings-icon"]
        .into_iter()
        .enumerate()
        .map(|(i, kind)| {
            let rect = [x, top + 12.0, hit, hit];
            x += hit + if i == 0 { 5.0 } else { 4.0 };
            UiNode {
                kind: kind.into(),
                rect,
                clip: Some([left, top, view[0] - left - right, view[1] - top - insets[2]]),
                ..Default::default()
            }
        })
        .collect();
    UiScene {
        groups: vec![UiGroup {
            position: [0.0; 2],
            nodes,
        }],
    }
}
fn native_hud() -> Hud {
    let mut h = game_core::Game::new(game_core::Profile::default()).hud();
    h.dock = false;
    h.position = [1600.0, 0.0];
    h.currency = [999999; 3];
    h
}
fn corners_fit(
    frame: &FrameBuilder,
    origin: Vec2,
    layout: FlightLayout,
    header: [f32; 4],
    view: [f32; 2],
    touch: bool,
) {
    let telemetry: Vec<_> = [layout.biome, layout.currency, layout.radar, layout.meter]
        .into_iter()
        .flatten()
        .collect();
    for (i, r) in telemetry.iter().enumerate() {
        assert!(
            !hud_overlap(*r, header, 0.0),
            "telemetry overlaps header {r:?}/{header:?}"
        );
        assert!(telemetry[i + 1..].iter().all(|b| !hud_overlap(*r, *b, 0.0)));
        if touch {
            assert!(
                touch_hit_rects(view)
                    .iter()
                    .all(|h| !hud_overlap(*r, *h, 0.0))
            );
        }
    }
    for b in &frame.frame().batches {
        for pixel in &b.pixels {
            if pixel.color >> 24 == 0 {
                continue;
            }
            for offset in [
                Vec2::splat(-0.5),
                Vec2::new(0.5, -0.5),
                Vec2::new(-0.5, 0.5),
                Vec2::splat(0.5),
            ] {
                let p = b.transforms[pixel.transform as usize]
                    .point(Vec2::from_array(pixel.local) + offset)
                    - origin;
                assert!(
                    p.x >= -0.001
                        && p.y >= -0.001
                        && p.x <= view[0] + 0.001
                        && p.y <= view[1] + 0.001
                );
                assert!(telemetry.iter().any(|r| p.x >= r[0] - 0.001
                    && p.y >= r[1] - 0.001
                    && p.x <= r[0] + r[2] + 0.001
                    && p.y <= r[1] + r[3] + 0.001));
            }
        }
    }
}

#[test]
fn actual_scene_header_union_is_clipped_rebased_and_not_an_assumed_hit_size() {
    let insets = [7.0, 19.0, 13.0, 11.0];
    let mut s = scene([350.0, 200.0], insets, 22.375);
    let a = flight_header(&s, insets).unwrap();
    assert_eq!(
        a,
        [
            350.0 - 19.0 - 12.0 - (22.375 * 4.0 + 13.0) - 11.0,
            12.0,
            102.5,
            22.375
        ]
    );
    s.groups[0].nodes[0].clip = Some([s.groups[0].nodes[0].rect[0] + 3.25, 0.0, 500.0, 100.0]);
    let b = flight_header(&s, insets).unwrap();
    assert_eq!(b[0], a[0] + 3.25);
    assert_eq!(b[2], a[2] - 3.25);
    s.groups[0].nodes[0].rect[0] = f32::NAN;
    assert!(flight_header(&s, insets).unwrap()[0] > b[0]);
}

#[test]
fn normal_currency_moves_left_only_as_far_as_actual_toolbar_requires() {
    let h = native_hud();
    let s = scene([533.0, 300.0], [0.0; 4], 22.0);
    let header = flight_header(&s, [0.0; 4]).unwrap();
    let old = flight_layout(&h, [533.0, 300.0], false, false).unwrap();
    let new = flight_layout_with_header(&h, [533.0, 300.0], false, false, Some(header)).unwrap();
    assert_eq!(new.biome, old.biome);
    assert_eq!(new.meter, old.meter);
    assert_eq!(new.radar, old.radar);
    assert_eq!(new.currency, Some([274.0, 12.0, 142.0, 24.0]));
    assert_eq!(
        flight_layout_with_header(&h, [533.0, 300.0], false, false, None)
            .unwrap()
            .currency,
        old.currency
    );
}

#[test]
fn narrow_currency_stacks_below_real_header_and_tiny_hull_survives() {
    let h = native_hud();
    let s = scene([320.0, 180.0], [0.0; 4], 44.0);
    let layout =
        flight_layout_with_header(&h, [320.0, 180.0], false, true, flight_header(&s, [0.0; 4]))
            .unwrap();
    assert_eq!(layout.currency, Some([12.0, 60.0, 110.0, 20.0]));
    assert_eq!(layout.biome, Some([12.0, 12.0, 100.0, 20.0]));
    let s = scene([180.0, 140.0], [0.0; 4], 22.0);
    let layout =
        flight_layout_with_header(&h, [180.0, 140.0], false, true, flight_header(&s, [0.0; 4]))
            .unwrap();
    assert_eq!(layout.meter, Some([98.0, 36.0, 70.0, 16.0]));
    assert_eq!(layout.meter_style, MeterStyle::Micro);
    assert!(layout.biome.is_none() && layout.currency.is_none());
    // A physically180px-wide/44px-icon header itself cannot fit. The top strip
    // retains hull/shield above it; this is a geometry limit, not phone proof.
    let s = scene([180.0, 140.0], [0.0; 4], 44.0);
    let layout =
        flight_layout_with_header(&h, [180.0, 140.0], false, true, flight_header(&s, [0.0; 4]))
            .unwrap();
    assert_eq!(layout.meter_style, MeterStyle::Slim);
}

#[test]
fn supplied_header_geometry_and_all_actual_hud_corners_fit_viewports_with_insets() {
    let h = native_hud();
    for (view, insets, hit) in [
        ([533.0, 300.0], [0.0; 4], 22.0),
        ([320.0, 180.0], [0.0; 4], 44.0),
        ([390.0, 180.0], [7.0, 30.0, 13.0, 20.0], 44.0),
        ([779.0, 360.0], [0.0; 4], 40.62),
        ([200.0, 160.0], [7.0, 10.0, 13.0, 10.0], 22.0),
        ([180.0, 140.0], [0.0; 4], 22.0),
        ([180.0, 140.0], [0.0; 4], 44.0),
    ] {
        let s = scene(view, insets, hit);
        let header = flight_header(&s, insets).unwrap();
        let safe = [
            view[0] - insets[1] - insets[3],
            view[1] - insets[0] - insets[2],
        ];
        for touch in [false, true] {
            for minimal in [false, true] {
                let layout =
                    flight_layout_with_header(&h, safe, minimal, touch, Some(header)).unwrap();
                assert!(layout.meter.is_some());
                let camera = [123.25, -18.5];
                let origin = Vec2::from_array(camera) - Vec2::from_array(safe) * 0.5;
                let mut frame = FrameBuilder::new();
                draw_ui_with_boss_panel_and_touch_and_header(
                    &mut frame,
                    &h,
                    camera,
                    safe,
                    1,
                    None,
                    minimal,
                    touch,
                    Some(header),
                );
                corners_fit(&frame, origin, layout, header, safe, touch);
            }
        }
    }
}

#[test]
fn impossible_full_screen_control_geometry_fails_closed_instead_of_overpainting() {
    assert!(
        flight_layout_with_header(
            &native_hud(),
            [180.0, 140.0],
            false,
            true,
            Some([0.0, 0.0, 180.0, 140.0])
        )
        .is_none()
    );
    let mut frame = FrameBuilder::new();
    draw_ui_with_boss_panel_and_touch_and_header(
        &mut frame,
        &native_hud(),
        [0.0; 2],
        [180.0, 140.0],
        1,
        None,
        false,
        true,
        Some([0.0, 0.0, 180.0, 140.0]),
    );
    assert_eq!(frame.frame().pixel_count, 0);
}
