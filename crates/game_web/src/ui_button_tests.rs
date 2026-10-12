//! Actual extraction tests for semantic controls; no GPU or DOM substitutes.
use super::*;
use engine_core::RenderFrame;

const KINDS: [&str; 6] = [
    "nav-tab",
    "primary-button",
    "exit-button",
    "offense-button",
    "defense-button",
    "passive-button",
];
const GROUP: [f32; 2] = [20.25, 10.5];
const VIEW: [f32; 2] = [640.0, 360.0];

fn node(kind: &str, rect: [f32; 4], text: &str) -> UiNode {
    UiNode {
        kind: kind.into(),
        rect,
        text: text.into(),
        font: 2.0,
        ..Default::default()
    }
}
fn extract(nodes: Vec<UiNode>, camera: [f32; 2], time: f32, reduced: bool) -> RenderFrame {
    let mut builder = FrameBuilder::new();
    draw(
        &mut builder,
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
    builder.finish()
}
fn same(a: &RenderFrame, b: &RenderFrame) {
    assert_eq!(a.pixel_count, b.pixel_count);
    assert_eq!(a.batches.len(), b.batches.len());
    for (a, b) in a.batches.iter().zip(&b.batches) {
        assert_eq!(a.transforms, b.transforms);
        assert_eq!(a.pixels, b.pixels);
    }
}
fn in_rect(point: Vec2, r: [f32; 4]) -> bool {
    point.x >= r[0] - 0.001
        && point.x <= r[0] + r[2] + 0.001
        && point.y >= r[1] - 0.001
        && point.y <= r[1] + r[3] + 0.001
}
fn local_screen(transform: engine_core::GpuTransform, p: Vec2, camera: [f32; 2]) -> Vec2 {
    transform.point(p) - Vec2::from_array(camera) + Vec2::from_array(VIEW) * 0.5
        - Vec2::from_array(GROUP)
}
fn corners(local: Vec2) -> [Vec2; 4] {
    [
        local + Vec2::new(-0.5, -0.5),
        local + Vec2::new(0.5, -0.5),
        local + Vec2::new(-0.5, 0.5),
        local + Vec2::new(0.5, 0.5),
    ]
}

#[test]
fn persistent_selection_survives_hover_of_another_tab() {
    let mut selected = node("nav-tab", [0.0, 0.0, 100.0, 24.0], "SETTINGS");
    selected.selected = true;
    let selected_frame = extract(vec![selected], [0.0; 2], 0.0, true);
    let mut selected = node("nav-tab", [0.0, 0.0, 100.0, 24.0], "SETTINGS");
    selected.selected = true;
    let mut hovered = node("nav-tab", [110.0, 0.0, 100.0, 24.0], "HANGAR");
    hovered.active = true;
    let both = extract(vec![selected, hovered], [0.0; 2], 0.0, true);
    let a = &selected_frame.batches[0];
    let b = &both.batches[0];
    assert_eq!(&b.transforms[..a.transforms.len()], &a.transforms);
    assert_eq!(&b.pixels[..a.pixels.len()], &a.pixels);
    assert_eq!(
        a.transforms
            .iter()
            .filter(|t| t.a[2] == 94.0 && t.b[1] == 2.0)
            .count(),
        1,
        "one independent selected bar keeps its exact original bounds"
    );
    let mut hovered_selected = node("nav-tab", [0.0, 0.0, 100.0, 24.0], "SETTINGS");
    hovered_selected.selected = true;
    hovered_selected.active = true;
    same(
        &selected_frame,
        &extract(vec![hovered_selected], [0.0; 2], 9.0, false),
    );
    let idle = extract(
        vec![node("nav-tab", [0.0, 0.0, 100.0, 24.0], "SETTINGS")],
        [0.0; 2],
        0.0,
        true,
    );
    assert_ne!(selected_frame.batches[0].pixels, idle.batches[0].pixels);
}

#[test]
fn semantic_actions_have_distinct_unlit_opaque_colors_and_no_motion() {
    let mut accents = std::collections::HashSet::new();
    for kind in KINDS {
        assert!(accents.insert(button_accent(kind)));
        let mut n = node(kind, [0.0, 0.0, 120.0, 24.0], "LAUNCH");
        n.active = true;
        let frame = extract(vec![n], [0.0; 2], 0.0, false);
        assert!(
            frame
                .batches
                .iter()
                .flat_map(|b| &b.pixels)
                .any(|p| p.color == button_accent(kind))
        );
        assert!(
            frame
                .batches
                .iter()
                .flat_map(|b| &b.transforms)
                .all(|t| t.b[2..] == [0.0, 0.0])
        );
        assert!(
            frame
                .batches
                .iter()
                .flat_map(|b| &b.pixels)
                .all(|p| p.color.to_le_bytes()[3] == 255)
        );
        let mut n = node(kind, [0.0, 0.0, 120.0, 24.0], "LAUNCH");
        n.active = true;
        same(&frame, &extract(vec![n], [0.0; 2], 999.0, true));
    }
}

#[test]
fn disabled_controls_remain_opaque_readable_and_ignore_hover() {
    for kind in KINDS {
        let mut n = node(kind, [0.0, 0.0, 100.0, 24.0], "LOCKED");
        n.disabled = true;
        n.active = true;
        let frame = extract(vec![n], [0.0; 2], 0.0, true);
        let colors: Vec<_> = frame
            .batches
            .iter()
            .flat_map(|b| &b.pixels)
            .map(|p| p.color)
            .collect();
        assert!(colors.contains(&BUTTON_DISABLED_TEXT));
        assert!(colors.iter().all(|c| c.to_le_bytes()[3] == 255));
        assert!(!colors.contains(&button_accent(kind)));
        let mut n = node(kind, [0.0, 0.0, 100.0, 24.0], "LOCKED");
        n.disabled = true;
        same(&frame, &extract(vec![n], [0.0; 2], 0.0, true));
    }
}

#[test]
fn category_headers_share_their_action_colors_without_hover_dependency() {
    for (kind, button) in [
        ("offense-text", "offense-button"),
        ("defense-text", "defense-button"),
        ("passive-text", "passive-button"),
    ] {
        let frame = extract(
            vec![node(kind, [0.0, 0.0, 120.0, 20.0], "CATEGORY")],
            [0.0; 2],
            0.0,
            true,
        );
        assert!(frame.pixel_count > 0);
        assert!(
            frame
                .batches
                .iter()
                .flat_map(|b| &b.pixels)
                .all(|p| p.color == button_accent(button))
        );
        let mut hovered = node(kind, [0.0, 0.0, 120.0, 20.0], "CATEGORY");
        hovered.active = true;
        same(&frame, &extract(vec![hovered], [0.0; 2], 0.0, true));
    }
}

#[test]
fn labels_fit_inner_bounds_at_narrow_fractional_and_large_font_extents() {
    for kind in KINDS.into_iter().chain(["button"]) {
        for width in [24.0, 39.875, 40.0, 80.25, 120.0] {
            for height in [12.0, 18.125, 24.0] {
                for font in [1.0, 2.0, 4.0] {
                    for text in [
                        "RETURN TO FLIGHT",
                        "UPGRADE\nSIGNATURE",
                        "É★界 LONG LABEL",
                        "A",
                    ] {
                        let rect = [5.25, 9.125, width, height];
                        let mut n = node(kind, rect, text);
                        n.font = font;
                        let frame = extract(vec![n], [0.0; 2], 0.0, true);
                        let padding = if width < 40.0 { 2.0 } else { 6.0 };
                        let inner = [
                            rect[0] + padding,
                            rect[1] + 2.0,
                            width - padding * 2.0,
                            height - if kind == "nav-tab" { 5.0 } else { 4.0 },
                        ];
                        let label_color = if kind == "button" { WHITE } else { BUTTON_TEXT };
                        let mut checked = 0;
                        for batch in &frame.batches {
                            for p in batch.pixels.iter().filter(|p| p.color == label_color) {
                                let t = batch.transforms[p.transform_slot()];
                                // Neutral atlas pixels never use WHITE exactly; these are cached glyphs.
                                for c in corners(Vec2::from_array(p.local)) {
                                    let screen = local_screen(t, c, [0.0; 2]);
                                    assert!(
                                        in_rect(screen, inner),
                                        "{kind} {width}x{height} font{font}: {screen:?} {inner:?}"
                                    );
                                }
                                checked += 1;
                            }
                        }
                        // RoyaleMono deliberately keeps unsupported Unicode
                        // glyphs empty; do not redefine the existing font here.
                        if text.is_ascii() {
                            assert!(
                                checked > 0,
                                "supported button keeps a label: {kind} {width}x{height}"
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn too_short_semantic_rects_keep_frames_without_crossing_glyphs() {
    for height in [2.0, 7.0, 10.0] {
        let r = [5.0, 6.0, 80.0, height];
        let frame = extract(
            vec![node("primary-button", r, "LAUNCH")],
            [0.0; 2],
            0.0,
            true,
        );
        assert!(frame.pixel_count > 0);
        for b in &frame.batches {
            for p in &b.pixels {
                assert_ne!(p.color, BUTTON_TEXT);
                for c in corners(Vec2::from_array(p.local)) {
                    assert!(in_rect(
                        local_screen(b.transforms[p.transform_slot()], c, [0.0; 2]),
                        r
                    ));
                }
            }
        }
    }
}

#[test]
fn legacy_settings_text_stays_inside_the_padded_nonoverlapping_regions() {
    for kind in ["toggle", "slider"] {
        for width in [40.0, 64.0, 100.0, 140.25] {
            for height in [20.0, 24.0, 36.0] {
                let r = [5.25, 7.5, width, height];
                let mut n = node(kind, r, if kind == "toggle" { "ON" } else { "VOL" });
                n.font = 4.0;
                n.value = 1.0;
                let label = setting_regions(&n).unwrap().0;
                let frame = extract(vec![n], [0.0; 2], 0.0, true);
                let mut glyphs = 0;
                for b in &frame.batches {
                    for p in b.pixels.iter().filter(|p| p.color == BUTTON_TEXT) {
                        let t = b.transforms[p.transform_slot()];
                        if t.a[2] != t.b[1] || t.a[2] > 4.0 {
                            continue;
                        } // slider knob
                        for c in corners(Vec2::from_array(p.local)) {
                            assert!(
                                in_rect(local_screen(t, c, [0.0; 2]), label),
                                "{kind}: {label:?}"
                            );
                        }
                        glyphs += 1;
                    }
                }
                assert!(glyphs > 0);
            }
        }
    }
}

#[test]
fn horizontal_and_vertical_clips_apply_to_frames_and_label_centers() {
    for kind in KINDS.into_iter().chain(["slider", "toggle"]) {
        let mut n = node(kind, [5.25, 8.5, 120.0, 32.0], "LONG LABEL ACROSS EDGE");
        let clip = [19.0, 15.0, 42.0, 12.0];
        n.clip = Some(clip);
        n.selected = true;
        let frame = extract(vec![n], [0.0; 2], 0.0, true);
        assert!(frame.pixel_count > 0);
        for b in &frame.batches {
            for p in b.pixels.iter().filter(|p| p.color >> 24 != 0) {
                let position = local_screen(
                    b.transforms[p.transform_slot()],
                    Vec2::from_array(p.local),
                    [0.0; 2],
                );
                assert!(position.x >= clip[0] && position.x < clip[0] + clip[2]);
                assert!(position.y >= clip[1] && position.y < clip[1] + clip[3]);
            }
        }
    }
}

#[test]
fn logo_intro_settle_never_leaves_a_tightly_fitted_mobile_rect() {
    for r in [
        [5.0, 9.0, 256.0, 104.0],
        [5.0, 9.0, 192.0, 78.0],
        [5.0, 9.0, 128.0, 52.0],
    ] {
        for time in [0.0, 0.1, 2.0] {
            let frame = extract(vec![node("logo", r, "")], [0.0; 2], time, false);
            assert!(frame.pixel_count > 10_000);
            for b in &frame.batches {
                for p in &b.pixels {
                    for c in corners(Vec2::from_array(p.local)) {
                        assert!(in_rect(
                            local_screen(b.transforms[p.transform_slot()], c, [0.0; 2]),
                            r
                        ));
                    }
                }
            }
        }
    }
}

#[test]
fn semantic_scene_caps_and_pool_reuse_preserve_real_transform_indices() {
    let make = |count| UiScene {
        groups: vec![UiGroup {
            position: GROUP,
            nodes: (0..count)
                .map(|i| {
                    let mut n = node(
                        KINDS[i % KINDS.len()],
                        [0.0, 0.0, 600.0, 24.0],
                        &"A".repeat(256),
                    );
                    n.selected = true;
                    n
                })
                .collect(),
        }],
    };
    let large = make(600);
    let small = make(1);
    let mut builder = FrameBuilder::new();
    draw(&mut builder, &large, [0.0; 2], VIEW, 0.0, false);
    assert!(builder.frame().pixel_count <= MAX_UI_PIXELS);
    assert!(builder.frame().pixel_count > 50_000);
    assert!(builder.frame().batches.iter().all(|b| {
        b.transforms.len() <= 128
            && b.pixels
                .iter()
                .all(|p| (p.transform_slot()) < b.transforms.len())
    }));
    builder.clear();
    draw(&mut builder, &small, [0.0; 2], VIEW, 0.0, true);
    let fresh = extract(
        small.groups.into_iter().next().unwrap().nodes,
        [0.0; 2],
        0.0,
        true,
    );
    same(builder.frame(), &fresh);
}

#[test]
fn semantic_groups_keep_screen_position_when_camera_moves() {
    let a = extract(
        vec![node(
            "offense-button",
            [5.25, 9.125, 100.0, 24.0],
            "UPGRADE",
        )],
        [0.0; 2],
        0.0,
        true,
    );
    let camera = [999_000.25, -998_000.5];
    let b = extract(
        vec![node(
            "offense-button",
            [5.25, 9.125, 100.0, 24.0],
            "UPGRADE",
        )],
        camera,
        0.0,
        true,
    );
    assert_eq!(a.pixel_count, b.pixel_count);
    for (a, b) in a.batches.iter().zip(&b.batches) {
        assert_eq!(a.pixels, b.pixels);
        for (ap, bp) in a.pixels.iter().zip(&b.pixels) {
            let pa = local_screen(
                a.transforms[ap.transform_slot()],
                Vec2::from_array(ap.local),
                [0.0; 2],
            );
            let pb = local_screen(
                b.transforms[bp.transform_slot()],
                Vec2::from_array(bp.local),
                camera,
            );
            assert!(
                pa.distance(pb) <= 0.1,
                "far-world f32 quantization: {pa:?} {pb:?}"
            );
        }
    }
}

#[test]
fn legacy_serialized_nodes_default_to_unselected() {
    let n: UiNode =
        serde_json::from_str(r#"{"kind":"nav-tab","rect":[0,0,80,24],"active":true}"#).unwrap();
    assert!(!n.selected);
    assert!(n.active);
    let n: UiNode = serde_json::from_str(r#"{"kind":"nav-tab","selected":true}"#).unwrap();
    assert!(n.selected);
    assert!(!n.active);
}
