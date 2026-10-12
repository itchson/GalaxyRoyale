//! Actual facility glyph extraction; no facility-art or pointer changes.
use super::*;
use engine_core::RenderFrame;

const VIEW: [f32; 2] = [533.0, 300.0];
const GROUP: [f32; 2] = [13.25, 8.5];

fn extract(value: f32, font: f32, rect: [f32; 4]) -> RenderFrame {
    let mut frame = FrameBuilder::new();
    draw(
        &mut frame,
        &UiScene {
            groups: vec![UiGroup {
                position: GROUP,
                nodes: vec![UiNode {
                    kind: "facility-label".into(),
                    text: "ARENAS".into(),
                    value,
                    font,
                    rect,
                    ..Default::default()
                }],
            }],
        },
        [0.0; 2],
        VIEW,
        0.5,
        false,
    );
    frame.finish()
}

fn visible_points(frame: &RenderFrame) -> Vec<Vec2> {
    frame
        .batches
        .iter()
        .flat_map(|batch| {
            batch
                .pixels
                .iter()
                .filter(|pixel| pixel.color >> 24 != 0)
                .map(|pixel| {
                    let transform = batch.transforms[pixel.transform_slot()];
                    assert_eq!(transform.b[2..], [0.0, 0.0], "UI stays unlit");
                    assert_eq!([transform.a[2], transform.a[3]], [1.0, 0.0]);
                    assert_eq!([transform.b[0], transform.b[1]], [0.0, 1.0]);
                    transform.point(Vec2::from_array(pixel.local)) + Vec2::from_array(VIEW) * 0.5
                        - Vec2::from_array(GROUP)
                })
        })
        .collect()
}

fn same_packet(a: &RenderFrame, b: &RenderFrame) {
    assert_eq!(a.pixel_count, b.pixel_count);
    assert_eq!(a.batches.len(), b.batches.len());
    for (a, b) in a.batches.iter().zip(&b.batches) {
        assert_eq!(a.transforms, b.transforms);
        assert_eq!(a.pixels, b.pixels);
    }
}

#[test]
fn facility_labels_keep_font_one_and_fit_the_selected_full_label_rectangle() {
    let rect = [120.25, 130.75, 18.0, 22.5];
    for (value, y) in [(0.0, rect[1] + rect[3] + 2.0), (1.0, rect[1] - 12.0)] {
        let frame = extract(value, 4.0, rect);
        let points = visible_points(&frame);
        assert!(!points.is_empty());
        let x = rect[0] + rect[2] * 0.5 - 50.0;
        for p in points {
            assert!(p.x - 0.5 >= x - 0.001 && p.x + 0.5 <= x + 100.001);
            assert!(p.y - 0.5 >= y - 0.001 && p.y + 0.5 <= y + 10.001);
        }
        same_packet(&frame, &extract(value, 1.0, rect));
    }
    let below = visible_points(&extract(0.0, 1.0, rect));
    let above = visible_points(&extract(1.0, 1.0, rect));
    assert_eq!(below.len(), above.len());
    for (a, b) in below.iter().zip(above) {
        assert_eq!(a.x, b.x);
        assert!((a.y - b.y - rect[3] - 14.0).abs() < 0.001);
    }
}

#[test]
fn facility_hidden_state_emits_nothing_and_nonfinite_values_retain_default() {
    let rect = [160.0, 140.0, 20.0, 20.0];
    let hidden = extract(2.0, 1.0, rect);
    assert_eq!(hidden.pixel_count, 0);
    assert!(
        hidden
            .batches
            .iter()
            .all(|b| b.pixels.is_empty() && b.transforms.is_empty())
    );
    let default = extract(0.0, 1.0, rect);
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0] {
        same_packet(&default, &extract(value, 1.0, rect));
    }
}

#[test]
fn facility_above_choice_clears_the_footer_that_below_would_overlap() {
    // A projected hit area near the bottom must not move to repair its label.
    let rect = [310.0, 253.0, 20.0, 20.0];
    let footer_top = VIEW[1] - 28.0;
    let above = visible_points(&extract(1.0, 1.0, rect));
    assert!(!above.is_empty());
    assert!(above.iter().all(|p| p.y + 0.5 < footer_top));
    let below = visible_points(&extract(0.0, 1.0, rect));
    assert!(below.iter().any(|p| p.y - 0.5 >= footer_top));
}
