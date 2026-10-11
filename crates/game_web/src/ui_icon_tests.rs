//! Actual occupied-control extraction; no DOM/GPU substitute or text icons.
use super::*;
use engine_core::RenderFrame;

const VIEW: [f32; 2] = [640.0, 360.0];
const GROUP: [f32; 2] = [20.25, 10.5];
fn current_icon(kind: &str) -> &'static PixelBody {
    &crate::flight_icons::icon(kind).unwrap().body
}
const MENU_KINDS: [&str; 8] = [
    "settings-icon",
    "resume-icon",
    "restart-icon",
    "title-icon",
    "quit-icon",
    "status-icon",
    "map-icon",
    "fullscreen-icon",
];
fn icon(kind: &str, state: f32, rect: [f32; 4]) -> UiNode {
    UiNode {
        kind: kind.into(),
        value: state,
        rect,
        ..Default::default()
    }
}
fn extract(node: UiNode, camera: [f32; 2], time: f32, reduced: bool) -> RenderFrame {
    let mut builder = FrameBuilder::new();
    draw(
        &mut builder,
        &UiScene {
            groups: vec![UiGroup {
                position: GROUP,
                nodes: vec![node],
            }],
        },
        camera,
        VIEW,
        time,
        reduced,
    );
    builder.finish()
}
fn local_point(t: engine_core::GpuTransform, p: Vec2, camera: [f32; 2]) -> Vec2 {
    t.point(p) - Vec2::from_array(camera) + Vec2::from_array(VIEW) * 0.5 - Vec2::from_array(GROUP)
}
fn corners(cell: Vec2) -> [Vec2; 4] {
    [
        cell + Vec2::splat(-0.5),
        cell + Vec2::new(0.5, -0.5),
        cell + Vec2::new(-0.5, 0.5),
        cell + Vec2::splat(0.5),
    ]
}
fn same(a: &RenderFrame, b: &RenderFrame) {
    assert_eq!(a.pixel_count, b.pixel_count);
    assert_eq!(a.batches.len(), b.batches.len());
    for (a, b) in a.batches.iter().zip(&b.batches) {
        assert_eq!(a.transforms, b.transforms);
        assert_eq!(a.pixels, b.pixels);
    }
}

#[test]
fn direct_64_cell_controls_are_cached_distinct_and_keep_save_state_color_semantics() {
    let close = current_icon("close-icon");
    assert!(std::ptr::eq(close, current_icon("close-icon")));
    let save = current_icon("save-icon");
    assert_eq!((save.width, save.height), (64, 64));
    assert_eq!(save.occupied_count(), 1503);
    let mut signatures = std::collections::HashSet::new();
    for (state, accent) in [rgb(106, 237, 193), rgb(88, 222, 239), rgb(220, 125, 130)]
        .into_iter()
        .enumerate()
    {
        let mut node = icon("save-icon", state as f32, [2.0, 3.0, 24.0, 24.0]);
        node.text = "THIS TEXT MUST NOT DRAW".into();
        let frame = extract(node, [0.0; 2], 0.0, false);
        let batch = &frame.batches[0];
        assert_eq!(frame.pixel_count, save.occupied_count());
        assert_eq!(batch.transforms.len(), 1);
        assert_eq!(batch.transforms[0].a[2..], [0.375, 0.0]);
        assert_eq!(batch.transforms[0].b, [0.0, 0.375, 0.0, 0.0]);
        for (pixel, (position, shade)) in batch.pixels.iter().zip(save.cells()) {
            assert_eq!(pixel.local, position.to_array());
            assert_eq!(
                pixel.color,
                crate::flight_icons::icon("save-icon").unwrap().color(
                    position,
                    shade,
                    crate::flight_icons::State {
                        accent,
                        busy: state == 1,
                        ..Default::default()
                    }
                )
            );
        }
        assert!(signatures.insert(batch.pixels.iter().map(|p| p.color).collect::<Vec<_>>()));
    }
    let frame = extract(
        icon("close-icon", 2.0, [0.0, 0.0, 18.0, 18.0]),
        [0.0; 2],
        0.0,
        false,
    );
    assert_eq!(frame.pixel_count, 1289);
    assert_ne!(close.occupied_count(), save.occupied_count());
    assert!(frame.batches[0].pixels.iter().all(|p| p.color >> 24 == 255));
    assert!(
        frame.batches[0]
            .pixels
            .iter()
            .map(|p| p.color)
            .collect::<std::collections::HashSet<_>>()
            .len()
            >= 4
    );
}

#[test]
fn menu_symbols_are_eight_distinct_detailed_cached_unlit_objects_without_text_placeholders() {
    let mut signatures = std::collections::HashSet::new();
    let mut total = 0;
    for (kind, count) in MENU_KINDS
        .into_iter()
        .zip([1802, 1362, 1373, 1591, 1322, 1291, 2036, 1140])
    {
        let body = current_icon(kind);
        assert!(std::ptr::eq(body, current_icon(kind)));
        assert_eq!((body.width, body.height), (64, 64));
        assert_eq!(body.occupied_count(), count);
        assert!(
            signatures.insert(
                (0..64)
                    .flat_map(|y| (0..64).map(move |x| body.is_occupied(x, y)))
                    .collect::<Vec<_>>()
            )
        );
        let mut node = icon(kind, 1234.0, [7.0, 13.0, 24.0, 24.0]);
        node.text = "IGNORED ACCESSIBLE CAPTION".into();
        let frame = extract(node, [0.0; 2], 0.0, false);
        assert_eq!(frame.pixel_count, count);
        assert_eq!(frame.batches.len(), 1);
        assert_eq!(frame.batches[0].transforms.len(), 1);
        assert_eq!(frame.batches[0].transforms[0].b[2..], [0.0, 0.0]);
        same(
            &frame,
            &extract(
                icon(kind, f32::NAN, [7.0, 13.0, 24.0, 24.0]),
                [0.0; 2],
                999.0,
                true,
            ),
        );
        total += count;
    }
    assert_eq!(
        total, 11917,
        "all eight control masks have an explicit submitted budget"
    );
    assert!(crate::kikai_art::icon("unknown-icon").is_none());
    assert_eq!(
        extract(
            icon("unknown-icon", 0.0, [0.0, 0.0, 32.0, 32.0]),
            [0.0; 2],
            0.0,
            false
        )
        .pixel_count,
        0
    );
}

#[test]
fn historical_32_cell_icon_routes_keep_their_authored_landmarks() {
    // Gear aperture, play tip, return ring/arrow, open door, broken power ring,
    // ship prow, folded map edge, and four fullscreen corners remain explicit.
    let landmarks = [
        ("settings-icon", [(15, 1), (1, 15)], [(15, 15), (0, 0)]),
        ("resume-icon", [(6, 3), (29, 16)], [(30, 16), (5, 16)]),
        ("restart-icon", [(13, 2), (16, 29)], [(16, 16), (0, 0)]),
        ("title-icon", [(16, 2), (5, 29)], [(16, 29), (0, 0)]),
        ("quit-icon", [(14, 2), (16, 29)], [(16, 20), (0, 0)]),
        ("status-icon", [(16, 2), (12, 29)], [(0, 0), (29, 29)]),
        ("map-icon", [(2, 7), (29, 3)], [(1, 7), (30, 3)]),
        ("fullscreen-icon", [(3, 3), (28, 28)], [(15, 15), (0, 0)]),
    ];
    for (kind, filled, empty) in landmarks {
        let body = crate::kikai_art::icon(kind).unwrap();
        for (x, y) in filled {
            assert!(body.is_occupied(x, y), "{kind} landmark {x},{y}");
        }
        for (x, y) in empty {
            assert!(!body.is_occupied(x, y), "{kind} open region {x},{y}");
        }
        assert!(
            body.cells()
                .map(|(_, color)| color)
                .collect::<std::collections::HashSet<_>>()
                .len()
                >= 4
        );
    }
}

#[test]
fn current_native_controls_keep_64_cell_silhouette_landmarks_and_authored_material_depth() {
    type Landmark = (
        &'static str,
        &'static [(usize, usize)],
        &'static [(usize, usize)],
    );
    let landmarks: &[Landmark] = &[
        ("status-icon", &[(32, 5), (8, 42)], &[(3, 32)]),
        ("map-icon", &[(30, 18)], &[(21, 31), (41, 31)]),
        ("settings-icon", &[(8, 31), (31, 9)], &[(31, 31)]),
        ("save-icon", &[(30, 44)], &[(25, 15), (30, 39)]),
        ("close-icon", &[(10, 10)], &[(31, 10), (10, 31)]),
        ("resume-icon", &[(14, 31), (56, 32)], &[(50, 12)]),
        ("restart-icon", &[(22, 15), (9, 38)], &[(31, 31), (30, 10)]),
        ("title-icon", &[(31, 10), (18, 45)], &[(31, 50)]),
        ("quit-icon", &[(50, 26)], &[(31, 39), (24, 10)]),
        (
            "fullscreen-icon",
            &[(10, 10), (53, 10), (10, 53), (53, 53)],
            &[(31, 31)],
        ),
    ];
    for &(kind, filled, empty) in landmarks {
        let body = current_icon(kind);
        for &(x, y) in filled {
            assert!(body.is_occupied(x, y), "{kind}: {x},{y}");
        }
        for &(x, y) in empty {
            assert!(!body.is_occupied(x, y), "{kind}: open {x},{y}");
        }
        assert!(
            body.cells()
                .map(|(_, c)| c)
                .collect::<std::collections::HashSet<_>>()
                .len()
                >= 5
        );
        // Even a compact14-logical slot retains all64 source cells with no
        // snapped minimum cell size or substitution of a lower-resolution mask.
        let f = extract(
            icon(kind, 0.0, [7.25, 13.5, 14.0, 14.0]),
            [0.0; 2],
            0.0,
            true,
        );
        assert_eq!(f.pixel_count, body.occupied_count());
        assert_eq!(f.batches[0].transforms[0].a[2], 14.0 / 64.0);
        assert_eq!(
            f.batches[0]
                .pixels
                .iter()
                .map(|p| p.local)
                .collect::<Vec<_>>(),
            body.cells().map(|(p, _)| p.to_array()).collect::<Vec<_>>()
        );
    }
}

#[test]
fn menu_icons_fit_44_physical_pixel_hitboxes_with_continuous_fractional_scaling() {
    for kind in MENU_KINDS.into_iter().chain(["close-icon", "save-icon"]) {
        for ui_scale in [1.0, 1.375, 2.4, 3.0, 4.0, 6.0] {
            let rect = [31.25, 23.5, 44.0 / ui_scale, 44.0 / ui_scale];
            let frame = extract(icon(kind, 0.0, rect), [0.0; 2], 0.0, false);
            let batch = &frame.batches[0];
            assert_eq!(frame.pixel_count, current_icon(kind).occupied_count());
            assert!((batch.transforms[0].a[2] - 44.0 / ui_scale / 64.0).abs() < 0.000001);
            let center = local_point(batch.transforms[0], Vec2::ZERO, [0.0; 2]);
            assert!(
                (center - Vec2::new(rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5))
                    .abs()
                    .max_element()
                    < 0.0001
            );
            for p in &batch.pixels {
                assert_ne!(p.color >> 24, 0);
                for c in corners(Vec2::from_array(p.local)) {
                    let point = local_point(batch.transforms[p.transform as usize], c, [0.0; 2]);
                    assert!(point.x >= rect[0] - 0.0001 && point.x <= rect[0] + rect[2] + 0.0001);
                    assert!(point.y >= rect[1] - 0.0001 && point.y <= rect[1] + rect[3] + 0.0001);
                    let physical = (point - Vec2::new(rect[0], rect[1])) * ui_scale;
                    assert!(physical.min_element() >= -0.001 && physical.max_element() <= 44.001);
                }
            }
        }
    }
}

#[test]
fn icon_hover_press_and_disabled_change_tone_without_changing_geometry_or_alpha() {
    for kind in MENU_KINDS.into_iter().chain(["close-icon", "save-icon"]) {
        let normal = extract(
            icon(kind, 0.0, [0.0, 0.0, 20.0, 20.0]),
            [0.0; 2],
            0.0,
            false,
        );
        let mut node = icon(kind, 0.0, [0.0, 0.0, 20.0, 20.0]);
        node.active = true;
        let hover = extract(node, [0.0; 2], 0.0, false);
        let mut node = icon(kind, 0.0, [0.0, 0.0, 20.0, 20.0]);
        node.active = true;
        node.selected = true;
        let pressed = extract(node, [0.0; 2], 0.0, false);
        let colors = |f: &RenderFrame| {
            f.batches[0]
                .pixels
                .iter()
                .map(|p| p.color)
                .collect::<Vec<_>>()
        };
        assert_ne!(colors(&normal), colors(&hover));
        assert_ne!(colors(&hover), colors(&pressed));
        assert_eq!(normal.batches[0].transforms, hover.batches[0].transforms);
        assert_eq!(hover.batches[0].transforms, pressed.batches[0].transforms);
        for (a, b) in hover.batches[0]
            .pixels
            .iter()
            .zip(&pressed.batches[0].pixels)
        {
            assert_eq!(a.local, b.local);
            let (a, b) = (a.color.to_le_bytes(), b.color.to_le_bytes());
            assert!(a[..3].iter().zip(&b[..3]).all(|(a, b)| a >= b));
            assert_eq!((a[3], b[3]), (255, 255));
        }
        let mut node = icon(kind, 2.0, [0.0, 0.0, 20.0, 20.0]);
        node.active = true;
        node.disabled = true;
        let disabled = extract(node, [0.0; 2], 0.0, false);
        assert!(disabled.batches[0].pixels.iter().all(|p| {
            let c = p.color.to_le_bytes();
            c[0] < c[1] && c[1] < c[2] && c[3] == 255
        }));
        assert_eq!(normal.pixel_count, disabled.pixel_count);
    }
}

#[test]
fn measured_icons_center_and_keep_every_corner_inside_their_rect_even_far_from_origin() {
    for kind in ["close-icon", "save-icon", "fullscreen-icon"] {
        for camera in [[0.0; 2], [999_000.0, -999_000.0]] {
            for size in [
                [0.25, 0.33],
                [8.0, 8.0],
                [12.25, 10.5],
                [44.0 / 2.4, 44.0 / 2.4],
                [24.0, 32.0],
            ] {
                let rect = [31.25, 23.5, size[0], size[1]];
                let frame = extract(icon(kind, 0.0, rect), camera, 0.0, false);
                assert!(frame.pixel_count > 0);
                let b = &frame.batches[0];
                assert!(
                    (b.transforms[0].a[2] - (size[0] / 64.0).min(size[1] / 64.0)).abs() < 0.000001
                );
                let center = local_point(b.transforms[0], Vec2::ZERO, camera);
                assert!(
                    (center - Vec2::new(rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5))
                        .abs()
                        .max_element()
                        <= 0.0625
                );
                for p in &b.pixels {
                    for c in corners(Vec2::from_array(p.local)) {
                        let point = local_point(b.transforms[p.transform as usize], c, camera);
                        assert!(
                            point.x >= rect[0] - 0.0625
                                && point.x <= rect[0] + rect[2] + 0.0625
                                && point.y >= rect[1] - 0.0625
                                && point.y <= rect[1] + rect[3] + 0.0625
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn clipped_icons_charge_whole_authored_bodies_and_reject_each_insufficient_budget_atomically() {
    let clip = [4.25, 1.75, 8.0, 10.0];
    for kind in MENU_KINDS.into_iter().chain(["close-icon", "save-icon"]) {
        let count = current_icon(kind).occupied_count();
        let mut node = icon(kind, 0.0, [0.0, 0.0, 18.0, 18.0]);
        node.clip = Some(clip);
        let frame = extract(node, [0.0; 2], 0.0, false);
        assert_eq!(frame.pixel_count, count);
        let mut visible = 0;
        for b in &frame.batches {
            for p in &b.pixels {
                if p.color >> 24 == 0 {
                    continue;
                }
                visible += 1;
                for c in corners(Vec2::from_array(p.local)) {
                    let point = local_point(b.transforms[p.transform as usize], c, [0.0; 2]);
                    assert!(
                        point.x >= clip[0]
                            && point.x <= clip[0] + clip[2]
                            && point.y >= clip[1]
                            && point.y <= clip[1] + clip[3]
                    );
                }
            }
        }
        assert!(visible > 0 && visible < count, "{kind}: {visible}/{count}");
        for (budget, work) in [(count - 1, count), (count, count - 1), (count, count)] {
            let mut builder = FrameBuilder::new();
            builder.push_body(
                &PixelBody::from_ascii(&["#"], &[('#', WHITE)]),
                Transform::default(),
            );
            let before = builder.frame().clone();
            let mut painter = Painter {
                frame: &mut builder,
                parent: AffineTransform::default(),
                remaining: budget,
                work_remaining: work,
                viewport: Vec2::from_array(VIEW),
                clip: None,
            };
            painter.control_icon(&icon(kind, 0.0, [0.0, 0.0, 18.0, 18.0]), 0.0, false);
            if budget < count || work < count {
                assert_eq!((painter.remaining, painter.work_remaining), (budget, work));
                same(&before, builder.frame());
            } else {
                assert_eq!((painter.remaining, painter.work_remaining), (0, 0));
                assert_eq!(builder.frame().pixel_count, count + 1);
            }
        }
    }
}

#[test]
fn save_nonfinite_state_is_static_saved_and_invalid_scene_geometry_is_closed() {
    let default = extract(
        icon("save-icon", 0.0, [0.0, 0.0, 18.0, 18.0]),
        [0.0; 2],
        0.0,
        false,
    );
    for state in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        same(
            &default,
            &extract(
                icon("save-icon", state, [0.0, 0.0, 18.0, 18.0]),
                [0.0; 2],
                1000.0,
                true,
            ),
        );
    }
    for rect in [
        [0.0, 0.0, f32::NAN, 32.0],
        [0.0, 0.0, 0.0, 32.0],
        [0.0, 0.0, -1.0, 32.0],
        [9000.0, 0.0, 32.0, 32.0],
    ] {
        assert_eq!(
            extract(icon("save-icon", 0.0, rect), [0.0; 2], 0.0, false).pixel_count,
            0
        );
    }
}

#[test]
fn icon_composes_a_reflected_sheared_parent_without_absolute_cell_expansion() {
    let parent = AffineTransform {
        position: Vec2::new(99.5, -33.0),
        basis_x: Vec2::new(-2.0, 0.25),
        basis_y: Vec2::new(0.5, 1.25),
    };
    let node = icon("close-icon", 0.0, [7.0, 13.0, 24.0, 24.0]);
    let local = Transform {
        position: Vec2::new(19.0, 25.0),
        scale: Vec2::splat(0.375),
        ..Transform::default()
    }
    .affine();
    let mut builder = FrameBuilder::new();
    let mut painter = Painter {
        frame: &mut builder,
        parent,
        remaining: MAX_UI_PIXELS,
        work_remaining: MAX_UI_PIXELS * 2,
        viewport: Vec2::from_array(VIEW),
        clip: None,
    };
    painter.control_icon(&node, 0.0, false);
    let frame = builder.finish();
    assert_eq!(frame.batches[0].transforms.len(), 1);
    let expected = parent.compose(local);
    for p in &frame.batches[0].pixels {
        for cell in corners(Vec2::from_array(p.local)) {
            assert!(
                (frame.batches[0].transforms[0].point(cell) - expected.point(cell))
                    .abs()
                    .max_element()
                    < 0.0001
            );
        }
    }
}

#[test]
fn facility_label_is_centered_below_hitbox_and_draws_only_cached_font_cells() {
    let r = [180.0, 45.0, 48.0, 48.0];
    let mut node = icon("facility-label", 0.0, r);
    node.text = "FORGE".into();
    let frame = extract(node, [0.0; 2], 0.0, false);
    assert_eq!(
        frame.pixel_count,
        "FORGE"
            .chars()
            .map(|c| RoyaleMono::body(c).occupied_count())
            .sum::<usize>()
    );
    let mut left = f32::INFINITY;
    let mut right = f32::NEG_INFINITY;
    for b in &frame.batches {
        for p in &b.pixels {
            assert_eq!(p.color, rgb(98, 143, 156));
            for c in corners(Vec2::from_array(p.local)) {
                let point = local_point(b.transforms[p.transform as usize], c, [0.0; 2]);
                assert!(point.y >= r[1] + r[3] + 2.0 && point.y <= r[1] + r[3] + 12.0);
                left = left.min(point.x);
                right = right.max(point.x);
            }
        }
    }
    assert!(((left + right) * 0.5 - (r[0] + r[2] * 0.5)).abs() < 0.001);
}

#[test]
fn restart_action_is_amber_and_preserves_bounded_semantic_label_geometry() {
    let rect = [7.25, 13.5, 80.0, 24.0];
    let mut node = icon("restart-button", 0.0, rect);
    node.text = "RESTART EXPEDITION".into();
    node.active = true;
    let frame = extract(node, [0.0; 2], 0.0, false);
    assert!(frame.pixel_count > 0);
    assert!(
        frame
            .batches
            .iter()
            .flat_map(|b| &b.pixels)
            .any(|p| p.color == rgb(255, 215, 131))
    );
    for b in &frame.batches {
        for p in &b.pixels {
            if p.color.to_le_bytes()[3] == 0 {
                continue;
            }
            assert_eq!(b.transforms[p.transform as usize].b[2..], [0.0, 0.0]);
            for c in corners(Vec2::from_array(p.local)) {
                let point = local_point(b.transforms[p.transform as usize], c, [0.0; 2]);
                assert!(
                    point.x >= rect[0]
                        && point.x <= rect[0] + rect[2]
                        && point.y >= rect[1]
                        && point.y <= rect[1] + rect[3]
                );
            }
        }
    }
}
