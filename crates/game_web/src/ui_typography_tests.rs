//! Actual fractional font packets, measured fit, and complete-cell clipping.
use super::*;
use engine_core::RenderFrame;

const VIEW: [f32; 2] = [640.0, 360.0];
fn extract(node: UiNode) -> RenderFrame {
    let mut frame = FrameBuilder::new();
    draw(
        &mut frame,
        &UiScene {
            groups: vec![UiGroup {
                nodes: vec![node],
                ..Default::default()
            }],
        },
        [0.0; 2],
        VIEW,
        0.0,
        true,
    );
    frame.finish()
}
fn glyph_count(text: &str) -> usize {
    text.chars()
        .map(|c| RoyaleMono::body(c).occupied_count())
        .sum()
}
fn corners(p: Vec2) -> [Vec2; 4] {
    [
        p + Vec2::splat(-0.5),
        p + Vec2::new(0.5, -0.5),
        p + Vec2::new(-0.5, 0.5),
        p + Vec2::splat(0.5),
    ]
}
fn inside(p: Vec2, r: [f32; 4]) -> bool {
    p.x >= r[0] - 0.0001
        && p.y >= r[1] - 0.0001
        && p.x <= r[0] + r[2] + 0.0001
        && p.y <= r[1] + r[3] + 0.0001
}
fn assert_glyphs(frame: &RenderFrame, color: u32, count: usize, font: f32, rect: [f32; 4]) {
    let mut visible = 0;
    for batch in &frame.batches {
        for pixel in &batch.pixels {
            if pixel.color != color {
                continue;
            }
            visible += 1;
            let t = batch.transforms[pixel.transform_slot()];
            assert!((t.a[2] - font).abs() < 0.0001);
            assert!((t.b[1] - font).abs() < 0.0001);
            assert_eq!(t.b[2..], [0.0, 0.0]);
            for corner in corners(Vec2::from_array(pixel.local)) {
                assert!(inside(t.point(corner) + Vec2::from_array(VIEW) * 0.5, rect));
            }
        }
    }
    assert_eq!(visible, count, "the complete occupied text must be visible");
}

#[test]
fn css_twelve_px_at_scale_one_keeps_one_point_five_native_cells() {
    let rect = [10.25, 20.5, 120.0, 16.0];
    let text = "BODY";
    let frame = extract(UiNode {
        kind: "text".into(),
        rect,
        text: text.into(),
        font: 12.0 / 8.0,
        ..Default::default()
    });
    assert_glyphs(&frame, WHITE, glyph_count(text), 1.5, rect);
}

#[test]
fn fractional_two_line_pitch_and_exact_single_glyph_fit_keep_every_cell() {
    let rect = [10.25, 20.5, 30.0, 23.0];
    let frame = extract(UiNode {
        kind: "text".into(),
        rect,
        text: "AA\nBB".into(),
        font: 1.5,
        ..Default::default()
    });
    assert_glyphs(&frame, WHITE, glyph_count("AABB"), 1.5, rect);
    let transforms = &frame.batches[0].transforms;
    assert_eq!(transforms[2].a[1] - transforms[0].a[1], 12.0);
    let rect = [10.25, 20.5, 6.25, 8.75];
    let frame = extract(UiNode {
        kind: "text".into(),
        rect,
        text: "A".into(),
        font: 1.5,
        ..Default::default()
    });
    assert_glyphs(&frame, WHITE, glyph_count("A"), 1.25, rect);
}

#[test]
fn button_and_input_keep_fractional_requested_font_inside_measured_controls() {
    let frame = extract(UiNode {
        kind: "primary-button".into(),
        rect: [10.25, 20.5, 100.0, 44.0],
        text: "LAUNCH".into(),
        font: 1.5,
        ..Default::default()
    });
    assert_glyphs(
        &frame,
        BUTTON_TEXT,
        glyph_count("LAUNCH"),
        1.5,
        [16.25, 22.5, 88.0, 40.0],
    );
    let tight = [10.25, 20.5, 64.5, 20.0];
    let frame = extract(UiNode {
        kind: "primary-button".into(),
        rect: tight,
        text: "LAUNCH".into(),
        font: 1.5,
        ..Default::default()
    });
    assert_glyphs(&frame, BUTTON_TEXT, glyph_count("LAUNCH"), 1.5, tight);
    let rect = [10.25, 20.5, 120.0, 22.0];
    let frame = extract(UiNode {
        kind: "input".into(),
        rect,
        text: "HELP".into(),
        font: 1.5,
        value: 4.0,
        ..Default::default()
    });
    assert_glyphs(&frame, WHITE, glyph_count("> HELP"), 1.5, rect);
}

#[test]
fn settings_preserve_requested_fraction_then_fit_complete_name_without_integer_snap() {
    for (text, label, requested, expected) in [
        ("Master volume", [18.0, 26.0, 120.0, 12.0], 1.5, 1.5),
        ("Reduced cosmic detail", [18.0, 26.0, 130.0, 9.0], 2.0, 1.04),
    ] {
        assert!((settings_font(label, text, requested) - expected).abs() < 0.0001);
        let frame = extract(UiNode {
            kind: "slider".into(),
            rect: [10.0, 20.0, 160.0, 48.0],
            label_rect: Some(label),
            control_rect: Some([18.0, 48.0, 144.0, 12.0]),
            text: text.into(),
            font: requested,
            ..Default::default()
        });
        // A slider's small thumb highlight also uses BUTTON_TEXT; isolate text
        // by its complete fractional transform rather than assuming a color.
        let mut cells = 0;
        for batch in &frame.batches {
            for pixel in &batch.pixels {
                let t = batch.transforms[pixel.transform_slot()];
                if pixel.color == BUTTON_TEXT && (t.a[2] - expected).abs() < 0.0001 {
                    assert!((t.b[1] - expected).abs() < 0.0001);
                    for corner in corners(Vec2::from_array(pixel.local)) {
                        assert!(inside(
                            t.point(corner) + Vec2::from_array(VIEW) * 0.5,
                            label
                        ));
                    }
                    cells += 1;
                }
            }
        }
        assert_eq!(cells, glyph_count(text));
    }
}

#[test]
fn fractional_scroll_clip_checks_all_four_cell_corners_and_charges_hidden_cells() {
    let text = "ABCD";
    let clip = [11.15, 21.1, 21.4, 8.8];
    let frame = extract(UiNode {
        kind: "text".into(),
        rect: [10.25, 20.5, 60.0, 16.0],
        text: text.into(),
        font: 1.5,
        clip: Some(clip),
        ..Default::default()
    });
    assert_eq!(frame.pixel_count, glyph_count(text));
    let mut shown = 0;
    let mut hidden = 0;
    for batch in &frame.batches {
        for pixel in &batch.pixels {
            let t = batch.transforms[pixel.transform_slot()];
            if pixel.color >> 24 == 0 {
                hidden += 1;
            } else {
                shown += 1;
                for corner in corners(Vec2::from_array(pixel.local)) {
                    assert!(inside(t.point(corner) + Vec2::from_array(VIEW) * 0.5, clip));
                }
            }
        }
    }
    assert!(shown > 0 && hidden > 0);
}

#[test]
fn fractional_glyphs_preserve_reflected_nonuniform_parent_and_atomic_glyph_budget() {
    let parent = Transform {
        position: Vec2::new(123.25, -43.5),
        rotation: 0.73,
        scale: Vec2::new(-1.1, 0.7),
    }
    .affine();
    let cells = glyph_count("A");
    for (remaining, work, expected) in [
        (cells - 1, cells, 0),
        (cells, cells - 1, 0),
        (cells, cells, cells),
    ] {
        let mut frame = FrameBuilder::new();
        let mut painter = Painter {
            frame: &mut frame,
            parent,
            remaining,
            work_remaining: work,
            viewport: Vec2::from_array(VIEW),
            clip: None,
        };
        painter.text([10.25, 20.5, 60.0, 16.0], "AA", 1.5, WHITE, false);
        assert_eq!(painter.remaining, remaining - expected);
        assert_eq!(painter.work_remaining, work - expected);
        let frame = frame.finish();
        assert_eq!(frame.pixel_count, expected);
        if expected > 0 {
            let text = Transform {
                position: Vec2::new(10.25, 20.5),
                scale: Vec2::splat(1.5),
                ..Default::default()
            }
            .affine();
            let expected = parent.compose(RoyaleMono::letter_transform(text, 0));
            let actual = frame.batches[0].transforms[0];
            for (cell, _) in RoyaleMono::body('A').cells() {
                for corner in corners(cell) {
                    assert!(actual.point(corner).distance(expected.point(corner)) < 0.0001);
                }
            }
        }
    }
}

#[test]
fn invalid_or_extreme_requested_fonts_stay_finite_and_bounded() {
    for (requested, expected) in [
        (f32::NAN, 1.0),
        (f32::INFINITY, 1.0),
        (f32::NEG_INFINITY, 1.0),
        (-3.0, 1.0),
        (0.0, 1.0),
        (0.1, 0.25),
        (0.25, 0.25),
        (0.833_333_3, 0.833_333_3),
        (1.375, 1.375),
        (19.0, 4.0),
    ] {
        let rect = [10.25, 20.5, 150.0, 40.0];
        let frame = extract(UiNode {
            kind: "text".into(),
            rect,
            text: "FIT".into(),
            font: requested,
            ..Default::default()
        });
        assert_glyphs(&frame, WHITE, glyph_count("FIT"), expected, rect);
    }
    for rect in [[0.0, 0.0, 1.24, 1.75], [0.0, 0.0, 1.25, 1.74]] {
        let frame = extract(UiNode {
            kind: "text".into(),
            rect,
            text: "A".into(),
            font: 1.5,
            ..Default::default()
        });
        assert_eq!(
            frame.pixel_count, 0,
            "undersized boxes cannot fit a complete glyph"
        );
    }
}

#[test]
fn complete_fractional_settings_names_survive_compact_width_height_and_roundoff_matrix() {
    let mut complete_cases = 0;
    for text in [
        "Master volume",
        "Reduced cosmic detail",
        "Reduced motion",
        "Camera zoom",
    ] {
        for width in [50.0, 86.0, 130.0] {
            for height in [15.0, 17.0, 23.0, 25.0, 36.0] {
                for requested in [1.0, 1.125, 1.375, 1.5, 1.75, 2.0] {
                    let rect = [10.25, 20.5, width, height];
                    let font = settings_font(rect, text, requested);
                    assert!(font.is_finite() && font >= MIN_FONT_SCALE && font <= requested);
                    let columns = ((width + font) / (6.0 * font)).floor() as usize;
                    let lines = wrap(text, columns);
                    if (lines.len() as f32 * 8.0 - 1.0) * font > height {
                        continue; // The documented minimum font may not fit every name.
                    }
                    let mut frame = FrameBuilder::new();
                    let mut painter = Painter {
                        frame: &mut frame,
                        parent: Transform {
                            position: -Vec2::from_array(VIEW) * 0.5,
                            ..Default::default()
                        }
                        .affine(),
                        remaining: MAX_UI_PIXELS,
                        work_remaining: MAX_UI_PIXELS * 2,
                        viewport: Vec2::from_array(VIEW),
                        clip: None,
                    };
                    painter.settings_label(rect, text, requested, WHITE);
                    let frame = frame.finish();
                    let visible = frame
                        .batches
                        .iter()
                        .flat_map(|b| &b.pixels)
                        .filter(|p| p.color == WHITE)
                        .count();
                    assert_eq!(
                        visible,
                        glyph_count(text),
                        "{text}: width {width} height {height} requested {requested} chosen {font}"
                    );
                    assert_glyphs(&frame, WHITE, glyph_count(text), font, rect);
                    complete_cases += 1;
                }
            }
        }
    }
    assert!(complete_cases > 250);
    println!("Verified {complete_cases} complete labels in the 360-case compact settings matrix");
}
