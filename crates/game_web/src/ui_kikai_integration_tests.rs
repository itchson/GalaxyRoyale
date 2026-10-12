//! Actual semantic scene routes into cached Kikai portrait and licensed ship cells.
use super::*;
use crate::kikai_art::{DoraExpression, DoraInput};
use engine_core::RenderFrame;
use game_core::identity::{Corporation, starter_body};

const VIEW: [f32; 2] = [640.0, 360.0];
const GROUP: [f32; 2] = [20.25, 10.5];
fn portrait_cells() -> usize {
    portrait_cells_for(0)
}
fn portrait_cells_for(kind: usize) -> usize {
    static COUNT: std::sync::OnceLock<[usize; 3]> = std::sync::OnceLock::new();
    COUNT.get_or_init(|| {
        // Independent raw authored-mask oracle; not a draw/count mirror.
        let atlas: serde_json::Value =
            serde_json::from_str(include_str!("../../../assets/ui/kikai/detail/atlas.json"))
                .unwrap();
        ["dora", "doro", "talo"].map(|kind| {
            ["body", "eyes_neutral", "mouth_rest"]
                .into_iter()
                .map(|name| {
                    atlas["profiles"][kind]["sprites"][name]["indices"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter(|index| index.as_u64().unwrap() != 0)
                        .count()
                })
                .sum()
        })
    })[kind]
}

fn licensed_portrait(kind: u8, main: u8, sub: u8) -> UiNode {
    UiNode {
        portrait_kind: Some(kind),
        portrait_main: Some(main),
        portrait_sub: Some(sub),
        ..node("portrait", 9.0, [7.0, 9.0, 48.0, 64.0])
    }
}

fn node(kind: &str, value: f32, rect: [f32; 4]) -> UiNode {
    UiNode {
        kind: kind.into(),
        value,
        rect,
        ..Default::default()
    }
}
fn extract(nodes: Vec<UiNode>, camera: [f32; 2], time: f32, reduced: bool) -> RenderFrame {
    let mut frame = FrameBuilder::new();
    draw(
        &mut frame,
        &UiScene {
            groups: vec![UiGroup {
                position: GROUP,
                nodes,
            }],
        },
        camera,
        VIEW,
        time,
        reduced,
    );
    frame.finish()
}
fn same(a: &RenderFrame, b: &RenderFrame) {
    assert_eq!(a.pixel_count, b.pixel_count);
    assert_eq!(a.batches.len(), b.batches.len());
    for (a, b) in a.batches.iter().zip(&b.batches) {
        assert_eq!(a.transforms, b.transforms);
        assert_eq!(a.pixels, b.pixels);
    }
}
fn colors(frame: &RenderFrame) -> Vec<u32> {
    frame
        .batches
        .iter()
        .flat_map(|b| b.pixels.iter().map(|p| p.color))
        .collect()
}
fn corners(cell: Vec2) -> [Vec2; 4] {
    [
        cell + Vec2::splat(-0.5),
        cell + Vec2::new(0.5, -0.5),
        cell + Vec2::new(-0.5, 0.5),
        cell + Vec2::splat(0.5),
    ]
}
fn local_point(t: engine_core::GpuTransform, p: Vec2, camera: [f32; 2]) -> Vec2 {
    t.point(p) - Vec2::from_array(camera) + Vec2::from_array(VIEW) * 0.5 - Vec2::from_array(GROUP)
}
fn assert_rect(frame: &RenderFrame, rect: [f32; 4], camera: [f32; 2], epsilon: f32) -> usize {
    let mut visible = 0;
    for b in &frame.batches {
        for p in &b.pixels {
            assert!((p.transform_slot()) < b.transforms.len());
            let t = b.transforms[p.transform_slot()];
            assert_eq!(t.b[2..], [0.0, 0.0]);
            if p.color >> 24 == 0 {
                continue;
            }
            visible += 1;
            for c in corners(Vec2::from_array(p.local)) {
                let point = local_point(t, c, camera);
                assert!(
                    point.x >= rect[0] - epsilon
                        && point.x <= rect[0] + rect[2] + epsilon
                        && point.y >= rect[1] - epsilon
                        && point.y <= rect[1] + rect[3] + epsilon,
                    "{point:?} beyond {rect:?}"
                );
            }
        }
    }
    visible
}
fn dora_oracle(n: &UiNode, time: f32, reduced: bool) -> RenderFrame {
    let mut frame = FrameBuilder::new();
    let (mut pixels, mut work) = (MAX_UI_PIXELS, MAX_UI_PIXELS * 2);
    crate::kikai_art::append_dora(
        &mut frame,
        &DoraInput {
            rect: n.rect,
            parent: AffineTransform {
                position: Vec2::from_array(GROUP) - Vec2::from_array(VIEW) * 0.5,
                ..AffineTransform::default()
            },
            clip: n.clip,
            time,
            reduced_motion: reduced,
            expression: [
                DoraExpression::Neutral,
                DoraExpression::Focused,
                DoraExpression::Concerned,
                DoraExpression::Pleased,
            ][n.portrait_expression.min(3) as usize],
            speaking: n.portrait_speaking,
            disabled: n.disabled,
        },
        &mut pixels,
        &mut work,
    );
    frame.finish()
}

#[test]
fn profile_id9_routes_all_27_licensed_tokens_to_actual_type_cells_and_independent_roles() {
    let mut seen = std::collections::HashSet::new();
    for kind in 0..3 {
        for main in 0..3 {
            for sub in 0..3 {
                let n = licensed_portrait(kind, main, sub);
                let actual = extract(vec![n], [0.0; 2], 0.0, true);
                assert!(actual.pixel_count > 0);
                assert_eq!(actual.batches[0].transforms.len(), 3);
                assert_eq!(
                    assert_rect(&actual, [7.0, 9.0, 48.0, 64.0], [0.0; 2], 0.0001),
                    actual.pixel_count
                );
                let signature = actual
                    .batches
                    .iter()
                    .flat_map(|b| {
                        b.pixels
                            .iter()
                            .map(|p| (p.local[0].to_bits(), p.local[1].to_bits(), p.color))
                    })
                    .collect::<Vec<_>>();
                assert!(seen.insert(signature));
            }
        }
    }
    assert_eq!(seen.len(), 27);
    same(
        &extract(vec![licensed_portrait(0, 0, 1)], [0.0; 2], 0.0, true),
        &extract(
            vec![node("portrait", 8.0, [7.0, 9.0, 48.0, 64.0])],
            [0.0; 2],
            0.0,
            true,
        ),
    );
}

#[test]
fn profile_semantic_missing_invalid_or_malformed_tokens_draw_nothing_and_guide_ignores_tokens() {
    for field in ["portrait_kind", "portrait_main", "portrait_sub"] {
        for bad in [serde_json::Value::Null, 3.into(), 255.into()] {
            let mut payload = serde_json::json!({"kind":"portrait","value":9,"rect":[7,9,48,64],"portrait_kind":0,"portrait_main":0,"portrait_sub":1});
            payload[field] = bad;
            let n: UiNode = serde_json::from_value(payload).unwrap();
            assert_eq!(extract(vec![n], [0.0; 2], 0.0, true).pixel_count, 0);
        }
        for bad in [
            serde_json::json!(-1),
            serde_json::json!(256),
            serde_json::json!(0.5),
            serde_json::json!("dora"),
            serde_json::json!(true),
            serde_json::json!([]),
            serde_json::json!({"kind":0}),
        ] {
            let mut payload = serde_json::json!({"kind":"portrait","value":9,"rect":[7,9,48,64],"portrait_kind":0,"portrait_main":0,"portrait_sub":1});
            payload[field] = bad;
            let n: UiNode = serde_json::from_value(payload).unwrap();
            let actual = extract(vec![n], [0.0; 2], 0.0, true);
            assert_eq!(actual.pixel_count, 0);
        }
        let mut missing = serde_json::json!({"kind":"portrait","value":9,"rect":[7,9,48,64],"portrait_kind":0,"portrait_main":0,"portrait_sub":1});
        missing.as_object_mut().unwrap().remove(field);
        let n: UiNode = serde_json::from_value(missing).unwrap();
        assert_eq!(extract(vec![n], [0.0; 2], 0.0, true).pixel_count, 0);
    }
    assert_eq!(
        extract(
            vec![node("portrait", 9.0, [7.0, 9.0, 48.0, 64.0])],
            [0.0; 2],
            0.0,
            true
        )
        .pixel_count,
        0
    );
    let mut guide = licensed_portrait(255, 255, 255);
    guide.value = 8.0;
    same(
        &extract(vec![guide], [0.0; 2], 0.0, true),
        &extract(
            vec![node("portrait", 8.0, [7.0, 9.0, 48.0, 64.0])],
            [0.0; 2],
            0.0,
            true,
        ),
    );
    let mut mixed: UiScene = serde_json::from_value(serde_json::json!({"groups":[{"position":GROUP,"nodes":[
        {"kind":"portrait","value":9,"rect":[7,9,48,64],"portrait_kind":"doro","portrait_main":0,"portrait_sub":1},
        {"kind":"text","text":"VALID","rect":[7,9,100,20]}
    ]}]})).unwrap();
    let mut frame = FrameBuilder::new();
    draw(&mut frame, &mixed, [0.0; 2], VIEW, 0.0, true);
    mixed.groups[0].nodes.remove(0);
    let mut expected = FrameBuilder::new();
    draw(&mut expected, &mixed, [0.0; 2], VIEW, 0.0, true);
    assert!(frame.frame().pixel_count > 0);
    same(&frame.finish(), &expected.finish());
}

#[test]
fn profile_scene_fractional_clipping_motion_and_quota_preserve_type_geometry() {
    for kind in 0..3 {
        let mut n = licensed_portrait(kind, 2, 0);
        n.rect = [7.0, 9.0, 21.25, 28.5];
        n.clip = Some([10.125, 12.375, 12.25, 17.75]);
        n.portrait_speaking = true;
        let actual = extract(vec![n], [0.0; 2], 0.15, true);
        let visible = assert_rect(&actual, [10.125, 12.375, 12.25, 17.75], [0.0; 2], 0.0001);
        assert!(visible > 0 && visible < actual.pixel_count);
        let mut later = licensed_portrait(kind, 2, 0);
        later.rect = [7.0, 9.0, 21.25, 28.5];
        later.clip = Some([10.125, 12.375, 12.25, 17.75]);
        later.portrait_speaking = true;
        same(&actual, &extract(vec![later], [0.0; 2], 4.4, true));
        let count = actual.pixel_count;
        for (pixels, work) in [(count - 1, count), (count, count - 1), (count, count)] {
            let mut frame = FrameBuilder::new();
            let mut painter = Painter {
                frame: &mut frame,
                parent: AffineTransform::default(),
                remaining: pixels,
                work_remaining: work,
                viewport: Vec2::from_array(VIEW),
                clip: None,
            };
            painter.node(&licensed_portrait(kind, 2, 0), 0.0, true);
            if pixels < count || work < count {
                assert_eq!((painter.remaining, painter.work_remaining), (pixels, work));
                assert_eq!(frame.frame().pixel_count, 0);
            } else {
                assert_eq!((painter.remaining, painter.work_remaining), (0, 0));
                assert_eq!(frame.frame().pixel_count, count);
            }
        }
    }
    let actual = extract(
        (0..512)
            .map(|i| licensed_portrait((i % 3) as u8, 0, 1))
            .collect(),
        [0.0; 2],
        0.0,
        true,
    );
    assert!(actual.pixel_count > 0 && actual.pixel_count <= MAX_UI_PIXELS);
    assert!(actual.batches.iter().all(|b| {
        b.pixels
            .iter()
            .all(|p| (p.transform_slot()) < b.transforms.len())
    }));
}

#[test]
fn portrait_id8_defaults_deserialize_and_route_to_actual_three_layers() {
    let mut decoded: UiScene = serde_json::from_str(r#"{"groups":[{"position":[20.25,10.5],"nodes":[{"kind":"portrait","value":8,"rect":[7,9,48,64]}]}]}"#).unwrap();
    let n = decoded.groups[0].nodes.remove(0);
    assert_eq!(n.portrait_expression, 0);
    assert!(!n.portrait_speaking);
    let expected = dora_oracle(&n, 0.0, true);
    let actual = extract(vec![n], [0.0; 2], 0.0, true);
    same(&actual, &expected);
    assert_eq!(actual.pixel_count, portrait_cells());
    assert_eq!(actual.batches[0].transforms.len(), 3);
    for t in &actual.batches[0].transforms {
        assert_eq!((t.a[2], t.b[1]), (0.25, 0.25));
    }
    assert_eq!(
        assert_rect(&actual, [7.0, 9.0, 48.0, 64.0], [0.0; 2], 0.0001),
        portrait_cells()
    );
    for value in 0..8 {
        let old = extract(
            vec![node("portrait", value as f32, [7.0, 9.0, 48.0, 64.0])],
            [0.0; 2],
            0.0,
            true,
        );
        assert!(old.pixel_count > 0 && old.pixel_count < portrait_cells());
        assert_eq!(old.batches[0].transforms.len(), 1);
    }
}

#[test]
fn portrait_expression_fields_select_four_real_poses_without_changing_occupancy() {
    let mut poses = std::collections::HashSet::new();
    for expression in 0..4 {
        let mut n = node("portrait", 8.0, [7.0, 9.0, 48.0, 64.0]);
        n.portrait_expression = expression;
        let expected = dora_oracle(&n, 0.0, true);
        let actual = extract(vec![n], [0.0; 2], 0.0, true);
        same(&actual, &expected);
        assert_eq!(actual.pixel_count, portrait_cells());
        assert!(poses.insert(colors(&actual)));
    }
    let mut saturated = node("portrait", 8.0, [7.0, 9.0, 48.0, 64.0]);
    saturated.portrait_expression = 255;
    let actual = extract(vec![saturated], [0.0; 2], 0.0, true);
    let mut pleased = node("portrait", 8.0, [7.0, 9.0, 48.0, 64.0]);
    pleased.portrait_expression = 3;
    same(&actual, &extract(vec![pleased], [0.0; 2], 0.0, true));
}

#[test]
fn speaking_blink_and_reduced_motion_use_explicit_fields_not_hover() {
    let mut mouths = std::collections::HashSet::new();
    for time in [0.0, 0.15, 0.30] {
        let mut n = node("portrait", 8.0, [7.0, 9.0, 48.0, 64.0]);
        n.portrait_speaking = true;
        let expected = dora_oracle(&n, time, false);
        let actual = extract(vec![n], [0.0; 2], time, false);
        same(&actual, &expected);
        // The actual third transform is the selected mouth object; body shimmer
        // must not make a static mouth look like independent speech evidence.
        mouths.insert(
            actual.batches[0]
                .pixels
                .iter()
                .filter(|p| p.transform == 2)
                .map(|p| p.color)
                .collect::<Vec<_>>(),
        );
    }
    assert_eq!(mouths.len(), 3);
    let silent = |active, time, reduced, disabled| {
        let mut n = node("portrait", 8.0, [7.0, 9.0, 48.0, 64.0]);
        n.active = active;
        n.portrait_speaking = true;
        n.disabled = disabled;
        extract(vec![n], [0.0; 2], time, reduced)
    };
    same(
        &silent(false, 0.15, true, false),
        &silent(true, 4.4, true, false),
    );
    same(
        &silent(false, 0.15, false, true),
        &silent(true, 4.4, false, true),
    );
    let blinking = extract(
        vec![node("portrait", 8.0, [7.0, 9.0, 48.0, 64.0])],
        [0.0; 2],
        4.4,
        false,
    );
    let neutral = extract(
        vec![node("portrait", 8.0, [7.0, 9.0, 48.0, 64.0])],
        [0.0; 2],
        0.0,
        false,
    );
    assert_ne!(
        blinking.batches[0]
            .pixels
            .iter()
            .filter(|p| p.transform == 1)
            .map(|p| p.color)
            .collect::<Vec<_>>(),
        neutral.batches[0]
            .pixels
            .iter()
            .filter(|p| p.transform == 1)
            .map(|p| p.color)
            .collect::<Vec<_>>()
    );
}

#[test]
fn portrait_fractional_fit_has_no_floor_and_partial_scroll_keeps_whole_cells() {
    for size in [[48.0, 64.0], [21.25, 28.5], [4.8, 6.4], [96.0, 128.0]] {
        let rect = [31.25, 23.5, size[0], size[1]];
        let full = extract(vec![node("portrait", 8.0, rect)], [0.0; 2], 0.0, true);
        assert_eq!(full.pixel_count, portrait_cells());
        let scale = (size[0] / 192.0).min(size[1] / 256.0);
        for t in &full.batches[0].transforms {
            assert!((t.a[2] - scale).abs() < 0.000001);
        }
        let visible = assert_rect(&full, rect, [0.0; 2], 0.0001);
        if size[0] >= 20.0 {
            assert_eq!(visible, portrait_cells(), "size {size:?}");
        } else {
            // Strict all-corner f32 containment may suppress a boundary row at
            // this sub-five-logical-pixel stress size; no geometry is expanded
            // to compensate. All authored cells remain submitted and charged.
            assert!(visible * 100 >= portrait_cells() * 98);
        }
    }
    let clip = [23.125, 28.75, 18.5, 21.125];
    let mut n = node("portrait", 8.0, [7.0, 9.0, 48.0, 64.0]);
    n.clip = Some(clip);
    let expected = dora_oracle(&n, 0.0, true);
    let actual = extract(vec![n], [0.0; 2], 0.0, true);
    same(&actual, &expected);
    let visible = assert_rect(&actual, clip, [0.0; 2], 0.0001);
    assert!(visible > 0 && visible < portrait_cells());
    assert_eq!(
        actual.pixel_count,
        portrait_cells(),
        "alpha-zero clipped cells still consume budgets"
    );
}

#[test]
fn dora_and_licensed_ship_reject_insufficient_submitted_or_work_budget_atomically() {
    for (kind, value, count) in [
        ("portrait", 8.0, portrait_cells()),
        ("licensed-ship", 0.0, 662),
        ("licensed-ship", 1.0, 688),
        ("licensed-ship", 2.0, 1040),
    ] {
        for (pixels, work) in [(count - 1, count), (count, count - 1), (count, count)] {
            let mut f = FrameBuilder::new();
            f.push_body(
                &PixelBody::from_ascii(&["#"], &[('#', WHITE)]),
                Transform::default(),
            );
            let before = f.frame().clone();
            let mut p = Painter {
                frame: &mut f,
                parent: AffineTransform::default(),
                remaining: pixels,
                work_remaining: work,
                viewport: Vec2::from_array(VIEW),
                clip: None,
            };
            p.node(&node(kind, value, [0.0, 0.0, 48.0, 64.0]), 0.0, true);
            if pixels < count || work < count {
                assert_eq!((p.remaining, p.work_remaining), (pixels, work));
                same(&before, f.frame());
            } else {
                assert_eq!((p.remaining, p.work_remaining), (0, 0));
                assert_eq!(f.frame().pixel_count, count + 1);
                assert_eq!(
                    f.frame().batches[0].transforms.len(),
                    if kind == "portrait" { 4 } else { 2 }
                );
            }
        }
    }
}

#[test]
fn licensed_previews_share_three_cached_authoritative_corporate_bodies_and_fractional_fit() {
    let mut signatures = std::collections::HashSet::new();
    for (value, corporation, count) in [
        (0.0, Corporation::Pan, 662),
        (1.0, Corporation::Renko, 688),
        (2.0, Corporation::Hugo, 1040),
    ] {
        let body = starter_body(corporation);
        assert!(std::ptr::eq(body, starter_body(corporation)));
        assert_eq!((body.width, body.height), (48, 48));
        assert_eq!(body.occupied_count(), count);
        for size in [[32.0, 32.0], [19.25, 23.75], [0.6, 0.9]] {
            let rect = [31.25, 23.5, size[0], size[1]];
            let actual = extract(
                vec![node("licensed-ship", value, rect)],
                [0.0; 2],
                0.0,
                true,
            );
            assert_eq!(actual.pixel_count, count);
            assert_eq!(actual.batches[0].transforms.len(), 1);
            let t = actual.batches[0].transforms[0];
            let scale = (size[0] / 48.0).min(size[1] / 48.0);
            assert!((t.a[2] - scale).abs() < 0.000001);
            assert!((t.b[1] - scale).abs() < 0.000001);
            assert_eq!(assert_rect(&actual, rect, [0.0; 2], 0.0001), count);
            for (pixel, (cell, color)) in actual.batches[0].pixels.iter().zip(body.cells()) {
                assert_eq!(pixel.local, cell.to_array());
                assert_eq!(
                    pixel.color, color,
                    "preview preserves actual ship palette and occupied mask"
                );
            }
        }
        assert!(
            signatures.insert(
                body.cells()
                    .map(|(cell, color)| (cell.x.to_bits(), cell.y.to_bits(), color))
                    .collect::<Vec<_>>()
            )
        );
    }
}

#[test]
fn licensed_ship_partial_scroll_clip_keeps_all_four_cell_corners_inside() {
    // Deliberately cut between cell centers at fractional fit. A center-only
    // clip leaks half a fitted cell beyond this actual semantic scroll edge.
    let clip = [15.125, 19.375, 12.25, 11.75];
    for value in 0..3 {
        let mut n = node("licensed-ship", value as f32, [7.0, 9.0, 24.0, 24.0]);
        n.clip = Some(clip);
        let actual = extract(vec![n], [0.0; 2], 0.0, true);
        let visible = assert_rect(&actual, clip, [0.0; 2], 0.0001);
        assert!(visible > 0 && visible < actual.pixel_count);
        assert_eq!(actual.pixel_count, [662, 688, 1040][value]);
    }
}

#[test]
fn portrait_and_ship_child_affines_survive_reflection_shear_and_nonuniform_parent() {
    let parent = AffineTransform {
        position: Vec2::new(99.5, -33.0),
        basis_x: Vec2::new(-2.0, 0.25),
        basis_y: Vec2::new(0.5, 1.25),
    };
    for (kind, value, body_size, count, parts) in [
        ("portrait", 8.0, [192.0, 256.0], portrait_cells(), 3),
        ("licensed-ship", 1.0, [48.0, 48.0], 688, 1),
    ] {
        let rect = [7.25, 13.5, 21.25, 28.5];
        let local = Transform {
            position: Vec2::new(rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5),
            scale: Vec2::splat((rect[2] / body_size[0]).min(rect[3] / body_size[1])),
            ..Transform::default()
        };
        let mut f = FrameBuilder::new();
        let mut p = Painter {
            frame: &mut f,
            parent,
            remaining: MAX_UI_PIXELS,
            work_remaining: MAX_UI_PIXELS * 2,
            viewport: Vec2::from_array(VIEW),
            clip: None,
        };
        p.node(&node(kind, value, rect), 0.0, true);
        let actual = f.finish();
        assert_eq!(actual.pixel_count, count);
        assert_eq!(actual.batches[0].transforms.len(), parts);
        for b in &actual.batches {
            for pixel in &b.pixels {
                let gpu = b.transforms[pixel.transform_slot()];
                assert_eq!(gpu.b[2..], [0.0, 0.0]);
                for corner in corners(Vec2::from_array(pixel.local)) {
                    assert!(
                        (gpu.point(corner) - parent.point(local.point(corner)))
                            .abs()
                            .max_element()
                            < 0.0001
                    );
                }
            }
        }
    }
}

#[test]
fn far_camera_group_translation_stays_local_with_declared_f32_tolerance() {
    for (kind, value, count) in [
        ("portrait", 8.0, portrait_cells()),
        ("licensed-ship", 2.0, 1040),
    ] {
        for camera in [[0.0; 2], [999_000.0, -999_000.0]] {
            let rect = [31.25, 23.5, 48.0, 64.0];
            let actual = extract(vec![node(kind, value, rect)], camera, 0.0, true);
            assert_eq!(actual.pixel_count, count);
            assert_eq!(assert_rect(&actual, rect, camera, 0.0625), count);
        }
    }
}

#[test]
fn scene_cap_and_transform_batch_boundary_keep_whole_portrait_objects_and_valid_indices() {
    let mut builder = FrameBuilder::new();
    let cell = PixelBody::from_ascii(&["#"], &[('#', WHITE)]);
    for i in 0..127 {
        builder.push_body(
            &cell,
            Transform {
                position: Vec2::new(i as f32, 0.0),
                ..Transform::default()
            },
        );
    }
    let mut nodes = Vec::new();
    for value in 0..3 {
        nodes.push(node("licensed-ship", value as f32, [7.0, 9.0, 32.0, 32.0]));
    }
    for kind in [
        "status-icon",
        "map-icon",
        "settings-icon",
        "save-icon",
        "close-icon",
        "resume-icon",
        "restart-icon",
        "title-icon",
        "quit-icon",
        "fullscreen-icon",
    ] {
        nodes.push(node(kind, 0.0, [7.0, 9.0, 32.0, 32.0]));
    }
    // Controls are real objects admitted first in this cap stress fixture.
    // Remaining portraits must be admitted whole or rejected without partial
    // eyes/mouths, even across the128-transform table boundary.
    nodes.extend((0..7).map(|_| node("portrait", 8.0, [7.0, 9.0, 48.0, 64.0])));
    draw(
        &mut builder,
        &UiScene {
            groups: vec![UiGroup {
                position: GROUP,
                nodes,
            }],
        },
        [0.0; 2],
        VIEW,
        0.0,
        true,
    );
    let frame = builder.finish();
    let controls = 662 + 688 + 1040 + 14_709;
    let admitted = (MAX_UI_PIXELS - controls) / portrait_cells();
    assert!((1..=2).contains(&admitted));
    assert!(admitted * portrait_cells() + controls <= MAX_UI_PIXELS);
    assert_eq!(
        frame.pixel_count - 127,
        admitted * portrait_cells() + controls
    );
    assert!(frame.pixel_count - 127 <= MAX_UI_PIXELS);
    assert_eq!(frame.batches.len(), 2);
    assert_eq!(
        frame
            .batches
            .iter()
            .map(|b| b.transforms.len())
            .sum::<usize>(),
        127 + admitted * 3 + 3 + 10
    );
    for b in &frame.batches {
        assert!(
            b.pixels
                .iter()
                .all(|p| (p.transform_slot()) < b.transforms.len())
        );
    }
}

#[test]
fn high_detail_profiles_and_real_dense_menu_controls_fit_the_shared_scene_cap() {
    fn controls(all_symbols: bool) -> Vec<UiNode> {
        let mut controls = vec![node("panel", 0.0, [8.0, 8.0, 604.0, 326.0])];
        for (i, text) in [
            "LICENSED FLIGHT SYSTEMS",
            "CORPORATION STARTER / EQUIPPED R1",
            "PHYSICAL HAVEN BANKING REQUIRED",
            "EVERY RAICHO EARNS ITS OWN XP",
            "MATERIALS SHARE ACTUAL CARGO SPACE",
        ]
        .into_iter()
        .enumerate()
        {
            controls.push(UiNode {
                text: text.into(),
                font: 1.0,
                ..node("text", 0.0, [182.0, 28.0 + i as f32 * 23.0, 410.0, 14.0])
            });
        }
        controls.push(node("licensed-ship", 0.0, [182.0, 164.0, 64.0, 64.0]));
        let all = [
            "status-icon",
            "map-icon",
            "settings-icon",
            "save-icon",
            "close-icon",
            "resume-icon",
            "restart-icon",
            "title-icon",
            "quit-icon",
            "fullscreen-icon",
        ];
        // The current status modal exposes these five actions. The all-ten
        // stress variant and deliberately doubled interface remain separate.
        let status = [
            "close-icon",
            "resume-icon",
            "restart-icon",
            "settings-icon",
            "title-icon",
        ];
        let symbols = if all_symbols { &all[..] } else { &status[..] };
        for (i, &kind) in symbols.iter().enumerate() {
            controls.push(node(kind, 0.0, [24.0 + i as f32 * 56.0, 254.0, 32.0, 32.0]));
        }
        for (i, (kind, text)) in [
            ("primary-button", "FORGE"),
            ("button", "HANGAR"),
            ("offense-button", "MISSION"),
            ("defense-button", "SAVE"),
            ("passive-button", "SETTINGS"),
            ("exit-button", "RETURN"),
        ]
        .into_iter()
        .enumerate()
        {
            controls.push(UiNode {
                text: text.into(),
                font: 1.0,
                ..node(kind, 0.0, [24.0 + i as f32 * 96.0, 298.0, 88.0, 22.0])
            });
        }
        controls
    }
    let interface = extract(controls(false), [0.0; 2], 0.0, true);
    assert!(
        interface.pixel_count > 5000,
        "fixture must contain real dense native controls"
    );
    for kind in 0..3 {
        let mut nodes = controls(false);
        let mut portrait = licensed_portrait(kind as u8, 0, 1);
        portrait.rect = [24.0, 52.0, 120.0, 160.0];
        nodes.push(portrait);
        let expected = portrait_cells_for(kind);
        assert!(expected > 20_000 && expected <= crate::kikai_art::MAX_PORTRAIT_QUADS);
        let actual = extract(nodes, [0.0; 2], 0.0, true);
        assert_eq!(actual.pixel_count, interface.pixel_count + expected);
        assert!(actual.pixel_count <= MAX_UI_PIXELS);
        // Last three objects are the entire high-detail body, eyes and mouth.
        let last = actual.batches.last().unwrap();
        let first_portrait = last.transforms.len() - 3;
        let visible = last
            .pixels
            .iter()
            .filter(|p| p.transform_slot() >= first_portrait && p.color >> 24 != 0)
            .count();
        assert_eq!(
            visible, expected,
            "no high-detail cells hidden by budget/fit"
        );
        assert!(actual.batches.iter().all(|b| {
            b.pixels
                .iter()
                .all(|p| (p.transform_slot()) < b.transforms.len())
        }));
        println!(
            "{kind}: portrait{expected} + menu{} = {} quads",
            interface.pixel_count, actual.pixel_count
        );
    }
    let all_ten = extract(controls(true), [0.0; 2], 0.0, true);
    // The competition header adds 389 occupied cells to this single-panel
    // fixture. All ten controls and the portrait must still be admitted whole.
    assert_eq!(all_ten.pixel_count, 21_615);
    let mut all_ten_with_dora = controls(true);
    let mut dora = licensed_portrait(0, 0, 1);
    dora.rect = [24.0, 52.0, 120.0, 160.0];
    all_ten_with_dora.push(dora);
    assert_eq!(
        extract(all_ten_with_dora, [0.0; 2], 0.0, true).pixel_count,
        all_ten.pixel_count + portrait_cells_for(0),
        "efficient native chrome now admits the full all-ten stress fixture"
    );
    let overload_controls = || {
        let mut nodes = controls(true);
        nodes.extend(controls(true));
        nodes
    };
    let overloaded = extract(overload_controls(), [0.0; 2], 0.0, true);
    assert_eq!(overloaded.pixel_count, 2 * all_ten.pixel_count);
    assert!(overloaded.pixel_count + portrait_cells_for(0) > MAX_UI_PIXELS);
    for kind in 0..3 {
        let mut nodes = overload_controls();
        let mut portrait = licensed_portrait(kind as u8, 0, 1);
        portrait.rect = [24.0, 52.0, 120.0, 160.0];
        nodes.push(portrait);
        let expected = portrait_cells_for(kind);
        let fits = overloaded.pixel_count + expected <= MAX_UI_PIXELS;
        let actual = extract(nodes, [0.0; 2], 0.0, true);
        assert_eq!(
            actual.pixel_count,
            overloaded.pixel_count + if fits { expected } else { 0 }
        );
        if !fits {
            for (before, after) in overloaded.batches.iter().zip(&actual.batches) {
                assert_eq!(before.transforms, after.transforms);
                assert_eq!(before.pixels, after.pixels);
            }
            assert_eq!(
                overloaded.batches.len(),
                actual.batches.len(),
                "overload cannot emit a partial body/face or consume a transform"
            );
        }
    }
}

#[test]
fn two_detail_portraits_share_the_original_cap_without_partial_layers() {
    // Exercise the real scene admission order, not a larger private quota.
    // Some two-portrait combinations genuinely cannot fit the60k interface cap.
    for first in 0..3 {
        for second in 0..3 {
            let first_node = licensed_portrait(first as u8, 0, 1);
            let first_frame = extract(vec![first_node], [0.0; 2], 0.0, true);
            let actual = extract(
                vec![
                    licensed_portrait(first as u8, 0, 1),
                    licensed_portrait(second as u8, 0, 1),
                ],
                [0.0; 2],
                0.0,
                true,
            );
            let first_count = portrait_cells_for(first);
            let second_count = portrait_cells_for(second);
            let fits = first_count + second_count <= MAX_UI_PIXELS;
            if fits {
                assert_eq!(actual.pixel_count, first_count + second_count);
                assert_eq!(actual.batches[0].transforms.len(), 6);
                assert!(actual.batches[0].pixels.iter().all(|p| p.color >> 24 != 0));
            } else {
                same(&first_frame, &actual);
                assert_eq!(actual.batches[0].transforms.len(), 3);
            }
            assert!(actual.pixel_count <= MAX_UI_PIXELS);
            println!(
                "pair{first}/{second}: requested{}, admitted{}, complete portraits{}",
                first_count + second_count,
                actual.pixel_count,
                if fits { 2 } else { 1 }
            );
        }
    }
}

#[test]
fn invalid_layouts_are_closed_and_nonfinite_time_or_corporation_defaults_are_safe() {
    for kind in ["portrait", "licensed-ship"] {
        let value = if kind == "portrait" { 8.0 } else { 0.0 };
        for rect in [
            [0.0, 0.0, f32::NAN, 32.0],
            [0.0, 0.0, f32::INFINITY, 32.0],
            [0.0, 0.0, 0.0, 32.0],
            [0.0, 0.0, -1.0, 32.0],
        ] {
            assert_eq!(
                extract(vec![node(kind, value, rect)], [0.0; 2], 0.0, true).pixel_count,
                0
            );
        }
        for clip in [[0.0, 0.0, f32::NAN, 32.0], [0.0, 0.0, 0.0, 32.0]] {
            let mut n = node(kind, value, [7.0, 9.0, 48.0, 64.0]);
            n.clip = Some(clip);
            assert_eq!(extract(vec![n], [0.0; 2], 0.0, true).pixel_count, 0);
        }
    }
    let base = extract(
        vec![node("licensed-ship", 0.0, [7.0, 9.0, 48.0, 64.0])],
        [0.0; 2],
        0.0,
        true,
    );
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        same(
            &base,
            &extract(
                vec![node("licensed-ship", value, [7.0, 9.0, 48.0, 64.0])],
                [0.0; 2],
                0.0,
                true,
            ),
        );
    }
    let base = extract(
        vec![node("portrait", 8.0, [7.0, 9.0, 48.0, 64.0])],
        [0.0; 2],
        0.0,
        false,
    );
    for time in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        same(
            &base,
            &extract(
                vec![node("portrait", 8.0, [7.0, 9.0, 48.0, 64.0])],
                [0.0; 2],
                time,
                false,
            ),
        );
    }
}

#[test]
fn meishi_card_all_profile_types_factory_hulls_and_eight_earned_patches_fit_60k() {
    // Authored occupancy is an independent oracle; scene admission is exercised
    // by the actual draw path. This is a logical fixture, not measured DOM QA.
    let patches: serde_json::Value = serde_json::from_str(include_str!(
        "../../../assets/ui/champion-patches-atlas.json"
    ))
    .unwrap();
    let patch_counts: [usize; 8] = std::array::from_fn(|index| {
        patches["sprites"][format!("campaign_patch_{index}")]["indices"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|value| value.as_u64().unwrap() != 0)
            .count()
    });
    assert_eq!(patch_counts.iter().sum::<usize>(), 2056);
    assert_eq!(MAX_UI_PIXELS, 60_000, "the shared cap must stay unchanged");

    let text = |value: &str, rect| UiNode {
        text: value.into(),
        font: 0.75,
        ..node("text", 0.0, rect)
    };
    let packet = |frame: &RenderFrame| {
        frame
            .batches
            .iter()
            .flat_map(|batch| {
                batch.pixels.iter().map(|pixel| {
                    let transform = batch.transforms[pixel.transform_slot()];
                    (pixel.local, pixel.color, transform.a, transform.b)
                })
            })
            .collect::<Vec<_>>()
    };
    for kind in 0..3 {
        for (main, corporation) in [Corporation::Pan, Corporation::Renko, Corporation::Hugo]
            .into_iter()
            .enumerate()
        {
            let mut profile = licensed_portrait(kind as u8, main as u8, ((main + 1) % 3) as u8);
            // Actual96CSS portrait at scale2.4, continuous source-cell fitting.
            profile.rect = [20.25, 60.5, 40.0, 128.0 / 2.4];
            let hull = node("licensed-ship", main as f32, [20.25, 129.5, 32.0, 32.0]);
            let mut nodes = vec![
                node("panel", 0.0, [5.0, 5.0, 614.0, 340.0]),
                node("close-icon", 0.0, [581.0, 14.0, 28.0, 28.0]),
                profile,
                text("MEISHI CARD", [78.25, 60.5, 360.0, 12.0]),
                text("APR#0123456789ABCDEF", [78.25, 74.5, 360.0, 12.0]),
                text(
                    "DORA-PAN-RENKO-0123456789ABCDEF",
                    [78.25, 88.5, 360.0, 12.0],
                ),
                hull,
                text(
                    "CURRENT FACTORY RAICHO / LV 1 / XP 7 / NEXT 20",
                    [66.25, 129.5, 510.0, 12.0],
                ),
                text("HULL 73/100 / SHIELD 0/0", [66.25, 143.5, 510.0, 12.0]),
            ];
            for (index, value) in [
                "OFFENSE / PULSE LENS / R1 EFFECTIVE",
                "DEFENSE / GUARD COIL / R1 EFFECTIVE",
                "PASSIVE / FLUX DRIVE / R1 EFFECTIVE",
                "RENKO OFFENSIVE 13 / HUGO DEFENSIVE 29 / PAN PASSIVE 47",
                "ARENA PATCHES 8/8",
            ]
            .into_iter()
            .enumerate()
            {
                nodes.push(text(
                    value,
                    [20.25, 167.5 + index as f32 * 12.0, 568.0, 10.0],
                ));
            }
            for index in 0..8 {
                let x = 20.25 + index as f32 * 65.0;
                let mut patch = node("campaign-patch", index as f32, [x, 232.5, 24.0, 24.0]);
                patch.active = true;
                nodes.push(patch);
                nodes.push(text(
                    &format!("{:02} EARNED", index + 1),
                    [x, 260.5, 60.0, 12.0],
                ));
            }
            for (index, label) in ["MEISHI", "CARGO", "HULLS", "ABILITIES", "STATS"]
                .into_iter()
                .enumerate()
            {
                nodes.push(UiNode {
                    text: label.into(),
                    font: 0.75,
                    selected: index == 0,
                    ..node(
                        "nav-tab",
                        0.0,
                        [20.25 + index as f32 * 116.0, 307.5, 104.0, 22.0],
                    )
                });
            }
            nodes.push(node("settings-icon", 0.0, [548.0, 14.0, 28.0, 28.0]));

            // Every isolated component must appear byte-for-byte in the mixed
            // scene: a capped portrait, absent late patch or missing caption
            // cannot be hidden by merely checking an upper-bound total.
            let mut parts = Vec::new();
            let mut joined = Vec::new();
            for node in nodes {
                let mut scene = UiScene {
                    groups: vec![UiGroup {
                        position: GROUP,
                        nodes: vec![node],
                    }],
                };
                let mut frame = FrameBuilder::new();
                draw(&mut frame, &scene, [0.0; 2], VIEW, 0.0, true);
                parts.push(frame.finish());
                joined.push(scene.groups[0].nodes.remove(0));
            }
            assert_eq!(parts[2].pixel_count, portrait_cells_for(kind));
            assert_eq!(
                assert_rect(&parts[2], joined[2].rect, [0.0; 2], 0.0001),
                portrait_cells_for(kind)
            );
            assert_eq!(
                parts[6].pixel_count,
                starter_body(corporation).occupied_count()
            );
            for (index, count) in patch_counts.into_iter().enumerate() {
                let part = &parts[14 + index * 2];
                assert_eq!(part.pixel_count, count);
                assert_eq!(
                    assert_rect(part, joined[14 + index * 2].rect, [0.0; 2], 0.0001),
                    count
                );
            }
            assert!(parts.iter().all(|part| part.pixel_count > 0));
            let expected: Vec<_> = parts.iter().flat_map(packet).collect();
            let actual = extract(joined, [0.0; 2], 0.0, true);
            assert!(actual.pixel_count < MAX_UI_PIXELS);
            assert_eq!(actual.pixel_count, expected.len());
            assert_eq!(packet(&actual), expected);
            println!(
                "MEISHI kind{kind}/corp{main}: profile{} + hull{} + eight patches2056 + chrome/text{} = {} /60000",
                portrait_cells_for(kind),
                starter_body(corporation).occupied_count(),
                actual.pixel_count
                    - portrait_cells_for(kind)
                    - starter_body(corporation).occupied_count()
                    - 2056,
                actual.pixel_count,
            );
        }
    }
}
