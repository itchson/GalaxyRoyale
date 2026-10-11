//! Actual semantic-region packets, complete font names and invalid-input guards.
use super::*;
use engine_core::RenderFrame;

const VIEW: [f32; 2] = [640.0, 360.0];
fn node(kind: &str) -> UiNode {
    UiNode {
        kind: kind.into(),
        rect: [20.25, 30.5, 180.0, 56.0],
        text: "Master volume".into(),
        font: 1.0,
        label_rect: Some([28.25, 38.5, 164.0, 9.0]),
        control_rect: Some([28.25, 55.5, 164.0, 23.0]),
        ..Default::default()
    }
}
fn extract(node: UiNode, camera: [f32; 2], time: f32, reduced: bool) -> RenderFrame {
    let mut frame = FrameBuilder::new();
    draw(
        &mut frame,
        &UiScene {
            groups: vec![UiGroup {
                position: [0.0; 2],
                nodes: vec![node],
            }],
        },
        camera,
        VIEW,
        time,
        reduced,
    );
    frame.finish()
}
fn point(transform: engine_core::GpuTransform, p: Vec2, camera: [f32; 2]) -> Vec2 {
    transform.point(p) - Vec2::from_array(camera) + Vec2::from_array(VIEW) * 0.5
}
fn corners(p: Vec2) -> [Vec2; 4] {
    [
        p + Vec2::splat(-0.5),
        p + Vec2::new(0.5, -0.5),
        p + Vec2::new(-0.5, 0.5),
        p + Vec2::splat(0.5),
    ]
}
fn inside(p: Vec2, r: [f32; 4], tolerance: f32) -> bool {
    p.x >= r[0] - tolerance
        && p.y >= r[1] - tolerance
        && p.x <= r[0] + r[2] + tolerance
        && p.y <= r[1] + r[3] + tolerance
}

#[test]
fn settings_regions_deserialize_when_missing_null_or_measured() {
    for json in [
        r#"{"kind":"slider"}"#,
        r#"{"kind":"toggle","label_rect":null,"control_rect":null}"#,
    ] {
        let n: UiNode = serde_json::from_str(json).unwrap();
        assert!(n.label_rect.is_none() && n.control_rect.is_none());
    }
    let n: UiNode = serde_json::from_str(
        r#"{"kind":"slider","label_rect":[1,2,3,4],"control_rect":[5,6,7,8],"disabled":true}"#,
    )
    .unwrap();
    assert_eq!(n.label_rect, Some([1.0, 2.0, 3.0, 4.0]));
    assert_eq!(n.control_rect, Some([5.0, 6.0, 7.0, 8.0]));
    assert!(n.disabled);
}

#[test]
fn actual_slider_thumb_uses_measured_input_and_keeps_both_endpoint_corners_inside() {
    for camera in [[0.0; 2], [999_000.0, -999_000.0]] {
        for value in [-1.0, 0.0, 0.5, 1.0, 9.0, f32::NAN] {
            let mut n = node("slider");
            n.value = value;
            let control = n.control_rect.unwrap();
            let label = n.label_rect.unwrap();
            let card = n.rect;
            let frame = extract(n, camera, 0.0, false);
            let mut thumb = None;
            for batch in &frame.batches {
                for pixel in &batch.pixels {
                    let t = batch.transforms[pixel.transform as usize];
                    for corner in corners(Vec2::from_array(pixel.local)) {
                        let p = point(t, corner, camera);
                        assert!(inside(p, card, 0.0625));
                        if pixel.color == BUTTON_TEXT {
                            assert!(inside(p, label, 0.0625) || inside(p, control, 0.0625));
                        }
                        if pixel.color == CYAN {
                            assert!(inside(p, control, 0.0625));
                        }
                    }
                    if pixel.color == CYAN && t.a[2] == 6.0 && t.b[1] == 10.0 {
                        thumb = Some(point(t, Vec2::ZERO, camera));
                    }
                }
            }
            let v = if value.is_finite() {
                value.clamp(0.0, 1.0)
            } else {
                0.0
            };
            let expected = Vec2::new(
                control[0] + 3.0 + (control[2] - 6.0) * v,
                control[1] + control[3] * 0.5,
            );
            assert!((thumb.unwrap() - expected).abs().max_element() <= 0.0625);
        }
    }
}

#[test]
fn full_reduced_cosmic_detail_caption_is_rendered_at_real_font_width_and_compact_wraps() {
    let name = "Reduced cosmic detail";
    assert_eq!(
        (name.chars().count() as f32 * RoyaleMono::ADVANCE - 1.0),
        125.0
    );
    let expected_cells = name
        .chars()
        .map(|c| RoyaleMono::body(c).occupied_count())
        .sum::<usize>();
    // Wide2x requested font cannot fit the first label. The native1x fallback
    // renders the entire caption. Compact fixtures contain two or three lines.
    for (label, requested, lines) in [
        ([28.25, 38.5, 130.0, 9.0], 2.0, 1),
        ([28.25, 38.5, 86.0, 17.0], 1.0, 2),
        ([28.25, 38.5, 50.0, 25.0], 1.0, 3),
    ] {
        let mut n = node("toggle");
        n.text = name.into();
        n.font = requested;
        n.label_rect = Some(label);
        n.control_rect = Some([166.25, 48.5, 18.0, 18.0]);
        let font = settings_font(label, name, requested);
        assert_eq!(font, 1.0);
        assert_eq!(
            wrap(name, ((label[2] + font) / (6.0 * font)).floor() as usize).len(),
            lines
        );
        let frame = extract(n, [0.0; 2], 0.0, false);
        let mut glyphs = 0;
        for batch in &frame.batches {
            for pixel in &batch.pixels {
                let t = batch.transforms[pixel.transform as usize];
                // Card/check geometry has other colors. Disabled is tested below.
                if pixel.color == BUTTON_TEXT {
                    glyphs += 1;
                    for corner in corners(Vec2::from_array(pixel.local)) {
                        assert!(inside(point(t, corner, [0.0; 2]), label, 0.0001));
                    }
                }
            }
        }
        assert_eq!(
            glyphs, expected_cells,
            "complete name, not a truncated success"
        );
    }
}

#[test]
fn toggle_state_is_confined_to_real_checkbox_and_disabled_has_no_active_cyan() {
    let mut off = node("toggle");
    off.label_rect = Some([28.25, 38.5, 120.0, 24.0]);
    off.control_rect = Some([166.25, 48.5, 18.0, 18.0]);
    let mut on = node("toggle");
    on.label_rect = off.label_rect;
    on.control_rect = off.control_rect;
    on.value = 1.0;
    let checkbox = on.control_rect.unwrap();
    let a = extract(off, [0.0; 2], 0.0, false);
    let b = extract(on, [0.0; 2], 0.0, false);
    assert_eq!(b.pixel_count - a.pixel_count, 6);
    for batch in &b.batches {
        for p in &batch.pixels {
            if p.color == CYAN {
                for corner in corners(Vec2::from_array(p.local)) {
                    assert!(inside(
                        point(batch.transforms[p.transform as usize], corner, [0.0; 2]),
                        checkbox,
                        0.0001
                    ));
                }
            }
        }
    }
    let mut disabled = node("toggle");
    disabled.label_rect = Some([28.25, 38.5, 120.0, 24.0]);
    disabled.control_rect = Some(checkbox);
    disabled.disabled = true;
    disabled.active = true;
    disabled.value = 1.0;
    let f = extract(disabled, [0.0; 2], 0.0, false);
    assert!(
        f.batches
            .iter()
            .flat_map(|b| &b.pixels)
            .all(|p| p.color != CYAN)
    );
}

#[test]
fn malformed_escape_overlapping_or_nonfinite_regions_fail_closed_before_any_cells() {
    for control in [
        [f32::NAN, 55.5, 10.0, 10.0],
        [19.0, 55.5, 10.0, 10.0],
        [28.25, 55.5, 10.0, -1.0],
        [28.25, 38.5, 40.0, 8.0],
        [28.25, 55.5, 164.0, f32::INFINITY],
    ] {
        let mut n = node("slider");
        n.control_rect = Some(control);
        assert_eq!(extract(n, [0.0; 2], 0.0, false).pixel_count, 0);
    }
    let mut n = node("slider");
    n.label_rect = Some([28.25, 38.5, 9000.0, 9.0]);
    assert_eq!(extract(n, [0.0; 2], 0.0, false).pixel_count, 0);
}

#[test]
fn legacy_regions_and_scrolling_clip_remain_bounded_static_and_unlit() {
    for kind in ["slider", "toggle"] {
        let mut n = node(kind);
        n.label_rect = None;
        n.control_rect = None;
        n.rect = [20.25, 30.5, 180.0, 56.0];
        let f = extract(n, [0.0; 2], 0.0, false);
        assert!(f.pixel_count > 0);
        assert!(
            f.batches
                .iter()
                .all(|b| b.transforms.iter().all(|t| t.b[2..] == [0.0, 0.0]))
        );
        let mut n = node(kind);
        n.rect = [20.25, 30.5, 180.0, 56.0];
        n.clip = Some([22.0, 40.0, 90.0, 32.0]);
        let a = extract(n, [0.0; 2], 0.0, false);
        let mut n = node(kind);
        n.rect = [20.25, 30.5, 180.0, 56.0];
        n.clip = Some([22.0, 40.0, 90.0, 32.0]);
        let b = extract(n, [0.0; 2], 999.0, true);
        assert_eq!(a.pixel_count, b.pixel_count);
        for (a, b) in a.batches.iter().zip(&b.batches) {
            assert_eq!(a.pixels, b.pixels);
            assert_eq!(a.transforms, b.transforms);
        }
        for batch in &a.batches {
            for p in &batch.pixels {
                if p.color >> 24 != 0 {
                    assert!(inside(
                        point(
                            batch.transforms[p.transform as usize],
                            Vec2::from_array(p.local),
                            [0.0; 2]
                        ),
                        [22.0, 40.0, 90.0, 32.0],
                        0.0001
                    ));
                }
            }
        }
    }
}
