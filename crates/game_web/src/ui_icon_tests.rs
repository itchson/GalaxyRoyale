//! Actual occupied-control extraction; no DOM/GPU substitute or text icons.
use super::*;
use engine_core::RenderFrame;

const VIEW: [f32; 2] = [640.0, 360.0];
const GROUP: [f32; 2] = [20.25, 10.5];
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
fn original_control_icons_are_distinct_cached_objects_with_state_colors() {
    let ptr = &*CLOSE_ICON as *const PixelBody;
    assert_eq!(ptr, &*CLOSE_ICON as *const PixelBody);
    let mut signatures = std::collections::HashSet::new();
    for (state, color) in [rgb(106, 237, 193), rgb(88, 222, 239), rgb(220, 125, 130)]
        .into_iter()
        .enumerate()
    {
        let body = &SAVE_ICONS[state];
        assert_eq!((body.width, body.height), (9, 9));
        assert!(
            signatures.insert(
                (0..9)
                    .flat_map(|y| (0..9).map(move |x| body.is_occupied(x, y)))
                    .collect::<Vec<_>>()
            )
        );
        let mut node = icon("save-icon", state as f32, [2.0, 3.0, 18.0, 18.0]);
        node.text = "THIS TEXT MUST NOT DRAW".into();
        let frame = extract(node, [0.0; 2], 0.0, false);
        assert_eq!(frame.pixel_count, body.occupied_count());
        assert_eq!(frame.batches[0].transforms.len(), 1);
        assert!(frame.batches[0].pixels.iter().any(|p| p.color == color));
        assert_eq!(frame.batches[0].transforms[0].b[2..], [0.0, 0.0]);
    }
    let frame = extract(
        icon("close-icon", 2.0, [0.0, 0.0, 18.0, 18.0]),
        [0.0; 2],
        0.0,
        false,
    );
    assert_eq!(frame.pixel_count, CLOSE_ICON.occupied_count());
    assert!(frame.batches[0].pixels.iter().all(|p| p.color == WHITE));
}

#[test]
fn measured_icons_center_and_keep_every_visible_corner_inside_their_rect() {
    for kind in ["close-icon", "save-icon"] {
        for camera in [[0.0; 2], [999_000.0, -999_000.0]] {
            for size in [
                [8.0, 8.0],
                [12.25, 10.5],
                [44.0 / 2.4, 44.0 / 2.4],
                [24.0, 32.0],
            ] {
                let rect = [31.25, 23.5, size[0], size[1]];
                let frame = extract(icon(kind, 0.0, rect), camera, 0.0, false);
                assert!(frame.pixel_count > 0);
                let b = &frame.batches[0];
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
fn control_icon_clipping_charges_all_cells_and_keeps_whole_visible_cells() {
    let clip = [4.25, 1.75, 8.0, 10.0];
    let mut node = icon("save-icon", 0.0, [0.0, 0.0, 18.0, 18.0]);
    node.clip = Some(clip);
    let frame = extract(node, [0.0; 2], 0.0, false);
    assert_eq!(frame.pixel_count, SAVE_ICONS[0].occupied_count());
    let mut visible = 0;
    for b in &frame.batches {
        for p in &b.pixels {
            if p.color.to_le_bytes()[3] == 0 {
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
    assert!(visible > 0 && visible < frame.pixel_count);
    let mut builder = FrameBuilder::new();
    let count = SAVE_ICONS[0].occupied_count();
    let mut painter = Painter {
        frame: &mut builder,
        parent: AffineTransform::default(),
        remaining: count - 1,
        work_remaining: count,
        viewport: Vec2::from_array(VIEW),
        clip: None,
    };
    painter.control_icon(&icon("save-icon", 0.0, [0.0, 0.0, 18.0, 18.0]));
    assert_eq!(painter.remaining, count - 1);
    assert_eq!(painter.work_remaining, count);
    assert_eq!(builder.frame().pixel_count, 0);
}

#[test]
fn icons_are_static_readable_when_disabled_and_nonfinite_state_defaults_saved() {
    for kind in ["close-icon", "save-icon"] {
        let a = extract(
            icon(kind, 0.0, [0.0, 0.0, 18.0, 18.0]),
            [0.0; 2],
            0.0,
            false,
        );
        let b = extract(
            icon(kind, f32::NAN, [0.0, 0.0, 18.0, 18.0]),
            [0.0; 2],
            1000.0,
            true,
        );
        same(&a, &b);
        let mut node = icon(kind, 2.0, [0.0, 0.0, 18.0, 18.0]);
        node.disabled = true;
        node.active = true;
        let disabled = extract(node, [0.0; 2], 1.0, false);
        assert!(
            disabled.batches[0]
                .pixels
                .iter()
                .all(|p| p.color == BUTTON_DISABLED_TEXT)
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
    let node = icon("close-icon", 0.0, [7.0, 13.0, 18.0, 18.0]);
    let local = Transform {
        position: Vec2::new(16.0, 22.0),
        scale: Vec2::splat(2.0),
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
    painter.control_icon(&node);
    let frame = builder.finish();
    assert_eq!(frame.batches[0].transforms.len(), 1);
    let expected = parent.compose(local);
    for p in &frame.batches[0].pixels {
        let cell = Vec2::from_array(p.local);
        assert!(
            (frame.batches[0].transforms[0].point(cell) - expected.point(cell))
                .abs()
                .max_element()
                < 0.0001
        );
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
