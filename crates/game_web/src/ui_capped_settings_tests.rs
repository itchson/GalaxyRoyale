//! Regression packets from the actual 1280x720 capped-font Settings BCRs.
use super::*;
use engine_core::RenderFrame;

const SCALE: f64 = 2.4;
const ROOT: [f64; 2] = [260.0, 40.0];
const VIEW: [f32; 2] = [1280.0 / 2.4, 720.0 / 2.4];
// CSS pixels: row, independently measured text span, independently measured input.
type MeasuredLabel = (&'static str, [f64; 4], [f64; 4], [f64; 4]);
const LABELS: [MeasuredLabel; 9] = [
    (
        "Master",
        [
            289.6000061035156,
            171.60000610351562,
            323.6000061035156,
            59.20000076293945,
        ],
        [
            296.8000183105469,
            189.6000061035156,
            120.82500457763672,
            22.399999618530273,
        ],
        [
            436.8250122070313,
            178.8000030517578,
            169.16250610351562,
            44.0,
        ],
    ),
    (
        "Music",
        [
            289.6000061035156,
            238.8000030517578,
            323.6000061035156,
            59.20000076293945,
        ],
        [
            296.8000183105469,
            256.8000183105469,
            120.82500457763672,
            22.399999618530273,
        ],
        [436.8250122070313, 246.0, 169.16250610351562, 44.0],
    ),
    (
        "Effects",
        [
            289.6000061035156,
            306.0,
            323.6000061035156,
            59.20000076293945,
        ],
        [
            296.8000183105469,
            324.0,
            120.82500457763672,
            22.399999618530273,
        ],
        [
            436.8250122070313,
            313.20001220703125,
            169.16250610351562,
            44.0,
        ],
    ),
    (
        "Dialogue",
        [
            289.6000061035156,
            373.2000122070313,
            323.6000061035156,
            48.79999923706055,
        ],
        [
            296.8000183105469,
            386.0,
            246.8000030517578,
            22.399999618530273,
        ],
        [
            572.4000244140625,
            380.3999938964844,
            33.60000228881836,
            33.60000228881836,
        ],
    ),
    (
        "Voice",
        [
            289.6000061035156,
            430.0,
            323.6000061035156,
            59.20000076293945,
        ],
        [
            296.8000183105469,
            448.0,
            120.82500457763672,
            22.399999618530273,
        ],
        [
            436.8250122070313,
            437.2000122070313,
            169.16250610351562,
            44.0,
        ],
    ),
    (
        "Zoom",
        [
            642.0,
            171.60000610351562,
            323.6000061035156,
            59.20000076293945,
        ],
        [
            649.2000122070312,
            189.6000061035156,
            120.82500457763672,
            22.399999618530273,
        ],
        [
            789.2250366210938,
            178.8000030517578,
            169.16250610351562,
            44.0,
        ],
    ),
    (
        "Camera shake",
        [
            642.0,
            238.8000030517578,
            323.6000061035156,
            48.79999923706055,
        ],
        [
            649.2000122070312,
            251.6000061035156,
            246.8000030517578,
            22.399999618530273,
        ],
        [
            924.7999877929688,
            246.0,
            33.60000228881836,
            33.60000228881836,
        ],
    ),
    (
        "Reduced motion",
        [
            642.0,
            295.6000061035156,
            323.6000061035156,
            48.79999923706055,
        ],
        [
            649.2000122070312,
            308.3999938964844,
            246.8000030517578,
            22.399999618530273,
        ],
        [
            924.7999877929688,
            302.8000183105469,
            33.60000228881836,
            33.60000228881836,
        ],
    ),
    (
        "Lower cosmic detail",
        [
            642.0,
            352.3999938964844,
            323.6000061035156,
            48.79999923706055,
        ],
        [
            649.2000122070312,
            365.2000122070313,
            246.8000030517578,
            22.399999618530273,
        ],
        [
            924.7999877929688,
            359.6000061035156,
            33.60000228881836,
            33.60000228881836,
        ],
    ),
];
fn rect(css: [f64; 4]) -> [f32; 4] {
    [
        ((css[0] - ROOT[0]) / SCALE) as f32,
        ((css[1] - ROOT[1]) / SCALE) as f32,
        (css[2] / SCALE) as f32,
        (css[3] / SCALE) as f32,
    ]
}
fn count(text: &str) -> usize {
    text.chars()
        .map(|c| RoyaleMono::body(c).occupied_count())
        .sum()
}
fn text_packet(text: &str, label: [f32; 4], requested: f32) -> RenderFrame {
    let mut frame = FrameBuilder::new();
    let mut painter = Painter {
        frame: &mut frame,
        parent: Transform {
            position: Vec2::new((ROOT[0] / SCALE) as f32, (ROOT[1] / SCALE) as f32)
                - Vec2::from_array(VIEW) * 0.5,
            ..Default::default()
        }
        .affine(),
        remaining: MAX_UI_PIXELS,
        work_remaining: MAX_UI_PIXELS * 2,
        viewport: Vec2::from_array(VIEW),
        clip: None,
    };
    painter.settings_label(label, text, requested, BUTTON_TEXT);
    frame.finish()
}
fn assert_complete(frame: &RenderFrame, text: &str, expected_font: f32) {
    let mut visible = 0;
    for batch in &frame.batches {
        for pixel in &batch.pixels {
            assert_eq!(
                pixel.color, BUTTON_TEXT,
                "{text}: a complete glyph cell was hidden"
            );
            let transform = batch.transforms[pixel.transform as usize];
            assert!(
                (transform.a[2] - expected_font).abs() < 0.00001,
                "{text}: requested font changed"
            );
            assert!((transform.b[1] - expected_font).abs() < 0.00001);
            visible += 1;
        }
    }
    assert_eq!(
        visible,
        count(text),
        "{text}: complete word including leading columns and final DETAIL"
    );
}

#[test]
fn measured_capped_settings_all_nine_names_keep_every_source_cell() {
    let font = (16.0 / (SCALE * 8.0)) as f32;
    for (text, row, label, control) in LABELS {
        let label = rect(label);
        let node = UiNode {
            kind: if control[3] == 44.0 {
                "slider"
            } else {
                "toggle"
            }
            .into(),
            rect: rect(row),
            label_rect: Some(label),
            control_rect: Some(rect(control)),
            text: text.into(),
            font,
            ..Default::default()
        };
        let mut frame = FrameBuilder::new();
        draw(
            &mut frame,
            &UiScene {
                groups: vec![UiGroup {
                    position: [(ROOT[0] / SCALE) as f32, (ROOT[1] / SCALE) as f32],
                    nodes: vec![node],
                }],
            },
            [0.0; 2],
            VIEW,
            0.0,
            true,
        );
        let glyphs = frame
            .frame()
            .batches
            .iter()
            .flat_map(|b| {
                b.pixels.iter().filter(move |p| {
                    (b.transforms[p.transform as usize].a[2] - font).abs() < 0.00001
                        && (b.transforms[p.transform as usize].b[1] - font).abs() < 0.00001
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(
            glyphs.len(),
            count(text),
            "{text}: complete actual measured label"
        );
        assert!(
            glyphs.iter().all(|p| p.color == BUTTON_TEXT),
            "{text}: leading glyph columns clipped"
        );
        assert_complete(&text_packet(text, label, font), text, font);
    }
}

#[test]
fn integer_font_at_fractional_bcr_origins_keeps_leading_columns() {
    for (text, _, label, _) in LABELS.iter().take(8) {
        assert_complete(&text_packet(text, rect(*label), 1.0), text, 1.0);
    }
}

#[test]
fn capped_css_font_reaches_quarter_cells_without_logical_minimum() {
    for scale in [1.0_f32, 2.4, 4.0, 8.0] {
        let font = 16.0 / scale / 8.0;
        let text = "Lower cosmic detail";
        let label = [10.333_337, 24.666_668, 246.8 / scale, 22.4 / scale];
        assert_complete(&text_packet(text, label, font), text, font);
    }
}

#[test]
fn subunit_font_keeps_strict_scroll_clip_and_hidden_cell_budget() {
    let mut frame = FrameBuilder::new();
    let clip = [20.51, 30.01, 9.9, 5.9];
    let mut painter = Painter {
        frame: &mut frame,
        parent: AffineTransform::default(),
        remaining: 1000,
        work_remaining: 2000,
        viewport: Vec2::from_array(VIEW),
        clip: Some(clip),
    };
    let initial = (painter.remaining, painter.work_remaining);
    painter.text([20.0, 30.0, 40.0, 8.0], "MASTER", 0.833_333_3, WHITE, false);
    assert_eq!(initial.0 - painter.remaining, count("MASTER"));
    assert_eq!(initial.1 - painter.work_remaining, count("MASTER"));
    let packet = frame.finish();
    let mut shown = 0;
    let mut hidden = 0;
    for batch in &packet.batches {
        for pixel in &batch.pixels {
            if pixel.color >> 24 == 0 {
                hidden += 1;
                continue;
            }
            shown += 1;
            for offset in [
                Vec2::splat(-0.5),
                Vec2::new(0.5, -0.5),
                Vec2::new(-0.5, 0.5),
                Vec2::splat(0.5),
            ] {
                let p = batch.transforms[pixel.transform as usize]
                    .point(Vec2::from_array(pixel.local) + offset);
                assert!(
                    p.x >= clip[0]
                        && p.y >= clip[1]
                        && p.x <= clip[0] + clip[2]
                        && p.y <= clip[1] + clip[3]
                );
            }
        }
    }
    assert!(shown > 0 && hidden > 0);
}
