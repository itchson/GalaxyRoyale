//! Quiet row hierarchy without altering the measured controls or labels.
use super::*;
use engine_core::{GpuTransform, RenderFrame};

const VIEW: [f32; 2] = [640.0, 360.0];
const ROW: [f32; 4] = [20.25, 30.5, 180.0, 56.0];
const LABEL: [f32; 4] = [28.25, 38.5, 164.0, 12.0];
const CONTROL: [f32; 4] = [28.25, 58.5, 164.0, 16.0];

fn extract(text: &str, active: bool, disabled: bool) -> RenderFrame {
    let mut frame = FrameBuilder::new();
    draw(
        &mut frame,
        &UiScene {
            groups: vec![UiGroup {
                nodes: vec![UiNode {
                    kind: "slider".into(),
                    rect: ROW,
                    label_rect: Some(LABEL),
                    control_rect: Some(CONTROL),
                    text: text.into(),
                    font: 1.5,
                    value: 0.5,
                    active,
                    disabled,
                    ..Default::default()
                }],
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
fn corners(t: GpuTransform, local: [f32; 2]) -> [Vec2; 4] {
    let p = Vec2::from_array(local);
    [
        p + Vec2::splat(-0.5),
        p + Vec2::new(0.5, -0.5),
        p + Vec2::new(-0.5, 0.5),
        p + Vec2::splat(0.5),
    ]
    .map(|p| t.point(p) + Vec2::from_array(VIEW) * 0.5)
}
fn inside(p: Vec2, r: [f32; 4]) -> bool {
    p.x >= r[0] - 0.0001
        && p.y >= r[1] - 0.0001
        && p.x <= r[0] + r[2] + 0.0001
        && p.y <= r[1] + r[3] + 0.0001
}
fn control_packets(frame: &RenderFrame) -> Vec<(GpuTransform, [f32; 2], u32)> {
    frame
        .batches
        .iter()
        .flat_map(|b| {
            b.pixels.iter().filter_map(|p| {
                let t = b.transforms[p.transform as usize];
                corners(t, p.local)
                    .into_iter()
                    .all(|p| inside(p, CONTROL))
                    .then_some((t, p.local, p.color))
            })
        })
        .collect()
}

#[test]
fn idle_settings_leave_the_row_open_except_for_control_and_bottom_separator() {
    let frame = extract("", false, false);
    let mut separator = 0;
    for batch in &frame.batches {
        for p in &batch.pixels {
            let corners = corners(batch.transforms[p.transform as usize], p.local);
            assert!(corners.into_iter().all(|p| inside(p, ROW)));
            if !corners.into_iter().all(|p| inside(p, CONTROL)) {
                assert!(corners.into_iter().all(|p| p.y >= ROW[1] + ROW[3] - 0.5001));
                separator += 1;
            }
        }
    }
    assert_eq!(separator, 1, "no top/side rails or repetitive filled cards");
    assert!(!control_packets(&frame).is_empty());
}

#[test]
fn focus_marks_the_row_without_moving_or_recoloring_actual_slider_control() {
    let idle = extract("Master volume", false, false);
    let focused = extract("Master volume", true, false);
    assert_eq!(control_packets(&idle), control_packets(&focused));
    let mut leading_accent = 0;
    for batch in &focused.batches {
        for p in &batch.pixels {
            let t = batch.transforms[p.transform as usize];
            let corners = corners(t, p.local);
            assert!(corners.into_iter().all(|p| inside(p, ROW)));
            if p.color == CYAN && !corners.into_iter().all(|p| inside(p, CONTROL)) {
                assert!(corners.into_iter().all(|p| p.x < LABEL[0]));
                assert!(t.a[2] <= 1.0 && t.b[1] <= 12.0);
                leading_accent += 1;
            }
        }
    }
    assert_eq!(leading_accent, 1);
    assert_eq!(
        focused.pixel_count - idle.pixel_count,
        2,
        "only background and leading accent are added"
    );
}

#[test]
fn disabled_focus_stays_quiet_and_keeps_complete_readable_label() {
    let idle = extract("Master volume", false, true);
    let focused = extract("Master volume", true, true);
    assert_eq!(idle.pixel_count, focused.pixel_count);
    for (a, b) in idle.batches.iter().zip(&focused.batches) {
        assert_eq!(a.pixels, b.pixels);
        assert_eq!(a.transforms, b.transforms);
    }
    let mut label_cells = 0;
    for batch in &focused.batches {
        for p in &batch.pixels {
            assert_ne!(p.color, CYAN);
            let t = batch.transforms[p.transform as usize];
            if p.color == BUTTON_DISABLED_TEXT && t.a[2] == 1.5 {
                assert!(corners(t, p.local).into_iter().all(|p| inside(p, LABEL)));
                label_cells += 1;
            }
        }
    }
    assert_eq!(
        label_cells,
        "Master volume"
            .chars()
            .map(|c| RoyaleMono::body(c).occupied_count())
            .sum::<usize>()
    );
}
