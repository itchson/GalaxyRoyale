//! Original arena patch artwork. Immutable cached cells and UI-only color cues.
use engine_core::{AffineTransform, FrameBuilder, PixelBody, Transform, Vec2};
use serde::Deserialize;
use std::{collections::HashMap, sync::LazyLock};
pub const COUNT: usize = 9;
pub const SIZE: usize = 24;
#[derive(Deserialize)]
struct Sprite {
    width: usize,
    height: usize,
    indices: Vec<u8>,
    shimmer_cells: Vec<usize>,
}
#[derive(Deserialize)]
struct Atlas {
    version: u32,
    palette: Vec<u32>,
    sprites: HashMap<String, Sprite>,
}
struct Art {
    body: PixelBody,
    shimmer: Box<[bool]>,
}
static ART: LazyLock<[Art; COUNT]> = LazyLock::new(|| {
    let atlas: Atlas = serde_json::from_str(include_str!(
        "../../../assets/ui/champion-patches-atlas.json"
    ))
    .unwrap();
    assert_eq!(atlas.version, 1);
    assert_eq!(atlas.palette.len(), 64);
    assert_eq!(atlas.palette[0], 0);
    assert!(atlas.palette[1..].iter().all(|color| color >> 24 == 255));
    std::array::from_fn(|index| {
        let s = &atlas.sprites[&format!("campaign_patch_{index}")];
        assert_eq!((s.width, s.height), (SIZE, SIZE));
        assert_eq!(s.indices.len(), SIZE * SIZE);
        assert!(s.indices.iter().all(|&i| i < 64));
        assert!(s.shimmer_cells.len() <= 24);
        let mut shimmer = vec![false; SIZE * SIZE].into_boxed_slice();
        for &i in &s.shimmer_cells {
            assert!(i < SIZE * SIZE && s.indices[i] != 0 && !shimmer[i]);
            shimmer[i] = true;
        }
        let key = |i: usize| char::from_u32(0xe000 + i as u32).unwrap();
        let palette: Vec<_> = atlas
            .palette
            .iter()
            .enumerate()
            .skip(1)
            .map(|(i, &c)| (key(i), c))
            .collect();
        let rows: Vec<String> = s
            .indices
            .chunks_exact(SIZE)
            .map(|row| {
                row.iter()
                    .map(|&i| if i == 0 { ' ' } else { key(i as usize) })
                    .collect()
            })
            .collect();
        let body = PixelBody::from_ascii(
            &rows.iter().map(String::as_str).collect::<Vec<_>>(),
            &palette,
        );
        Art { body, shimmer }
    })
});
#[derive(Debug, Clone, Copy)]
pub struct PatchInput {
    pub index: usize,
    pub rect: [f32; 4],
    pub parent: AffineTransform,
    pub clip: Option<[f32; 4]>,
    /// Champion.patch (indices 0..7) or authoritative tournament_won (index 8).
    pub earned: bool,
    pub disabled: bool,
    pub time: f32,
    pub reduced_motion: bool,
}
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Stats {
    pub quads: usize,
    pub visible_cells: usize,
    pub shimmered_cells: usize,
    pub scale: f32,
}
fn valid_rect(rect: [f32; 4]) -> bool {
    rect.iter().all(|v| v.is_finite() && v.abs() <= 8192.0) && rect[2] > 0.0 && rect[3] > 0.0
}
fn valid_parent(parent: AffineTransform) -> bool {
    parent.position.is_finite()
        && parent.position.abs().max_element() <= 1_100_000.0
        && parent.basis_x.is_finite()
        && parent.basis_y.is_finite()
        && parent.basis_x.abs().max_element() <= 64.0
        && parent.basis_y.abs().max_element() <= 64.0
        && parent.basis_x.perp_dot(parent.basis_y).is_finite()
        && parent.basis_x.perp_dot(parent.basis_y) != 0.0
}
fn inside(point: Vec2, clip: Option<[f32; 4]>) -> bool {
    clip.is_none_or(|r| {
        point.x >= r[0] && point.x <= r[0] + r[2] && point.y >= r[1] && point.y <= r[1] + r[3]
    })
}
fn shade(input: &PatchInput, cell: Vec2, base: u32, eligible: bool) -> (u32, bool) {
    let [r, g, b, a] = base.to_le_bytes();
    if input.disabled {
        let gray = ((r as u16 + g as u16 + b as u16) / 4) as u8;
        return (u32::from_le_bytes([gray, gray, gray, 180]), false);
    }
    if !input.earned || !eligible {
        return (base, false);
    }
    let time = if input.time.is_finite() {
        input.time.max(0.0)
    } else {
        0.0
    };
    let phase = (time + input.index as f32 * 0.47).rem_euclid(12.0);
    // A brief quiet bevel sweep every 12 seconds; staggered per emblem. It never
    // changes geometry, adds external sparkle particles or animates unearned UI.
    let strength = if !input.reduced_motion && phase < 0.8 {
        let sweep = -13.0 + phase / 0.8 * 26.0;
        (1.0 - (cell.x - cell.y * 0.15 - sweep).abs() / 1.6).max(0.0) * 0.16
    } else {
        0.0
    };
    let lift = 4 + (strength * 100.0).round() as u8;
    (
        u32::from_le_bytes([
            r.saturating_add(lift),
            g.saturating_add(lift),
            b.saturating_add(lift),
            a,
        ]),
        strength > 0.0,
    )
}

/// Uses the existing Painter pixel/work budgets and continuous body fit. Clipped
/// cells remain real submitted quads and are charged even when their alpha=0.
pub fn extract(
    frame: &mut FrameBuilder,
    input: &PatchInput,
    remaining: &mut usize,
    work_remaining: &mut usize,
) -> Stats {
    if input.index >= COUNT
        || !valid_rect(input.rect)
        || !valid_parent(input.parent)
        || input.clip.is_some_and(|r| !valid_rect(r))
    {
        return Stats::default();
    }
    let art = &ART[input.index];
    let count = art.body.occupied_count();
    let scale = (input.rect[2] / SIZE as f32).min(input.rect[3] / SIZE as f32);
    if !scale.is_finite() || scale <= 0.0 || count > *remaining || count > *work_remaining {
        return Stats::default();
    }
    let local = Transform {
        position: Vec2::new(
            input.rect[0] + input.rect[2] * 0.5,
            input.rect[1] + input.rect[3] * 0.5,
        ),
        scale: Vec2::splat(scale),
        ..Transform::default()
    };
    let mut visible_cells = 0;
    let mut shimmered_cells = 0;
    frame.push_body_affine_colored(
        &art.body,
        input.parent.compose(local.affine()),
        |cell, base| {
            if ![
                Vec2::splat(-0.5),
                Vec2::new(0.5, -0.5),
                Vec2::new(-0.5, 0.5),
                Vec2::splat(0.5),
            ]
            .into_iter()
            .all(|corner| inside(local.point(cell + corner), input.clip))
            {
                return 0;
            }
            visible_cells += 1;
            let index = (cell.y + SIZE as f32 * 0.5) as usize * SIZE
                + (cell.x + SIZE as f32 * 0.5) as usize;
            let (color, shimmered) = shade(input, cell, base, art.shimmer[index]);
            shimmered_cells += usize::from(shimmered);
            color
        },
    );
    *remaining -= count;
    *work_remaining -= count;
    Stats {
        quads: count,
        visible_cells,
        shimmered_cells,
        scale,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input(index: usize) -> PatchInput {
        PatchInput {
            index,
            rect: [7.0, 9.0, 24.0, 24.0],
            parent: AffineTransform::default(),
            clip: None,
            earned: false,
            disabled: false,
            time: 0.0,
            reduced_motion: false,
        }
    }
    fn colors(frame: &FrameBuilder) -> Vec<u32> {
        frame
            .frame()
            .batches
            .iter()
            .flat_map(|b| b.pixels.iter().map(|p| p.color))
            .collect()
    }
    fn draw(input: &PatchInput) -> (FrameBuilder, Stats) {
        let mut frame = FrameBuilder::new();
        let mut remaining = 60000;
        let mut work = 120000;
        let stats = extract(&mut frame, input, &mut remaining, &mut work);
        assert_eq!(remaining, 60000 - stats.quads);
        assert_eq!(work, 120000 - stats.quads);
        (frame, stats)
    }
    #[test]
    fn nine_masks_are_distinct_detailed_real_cells_with_bounded_occupied_shimmer() {
        let mut seen = std::collections::HashSet::new();
        let expected = [165, 321, 275, 219, 225, 323, 326, 202, 303];
        for (index, art) in ART.iter().enumerate() {
            assert_eq!((art.body.width, art.body.height), (24, 24));
            assert_eq!(art.body.occupied_count(), expected[index]);
            let mask: Vec<_> = (0..576)
                .map(|i| art.body.is_occupied(i % 24, i / 24))
                .collect();
            assert!(seen.insert(mask.clone()));
            assert!(mask.iter().any(|b| !*b));
            assert!(
                art.body
                    .cells()
                    .map(|(_, c)| c)
                    .collect::<std::collections::HashSet<_>>()
                    .len()
                    >= 4
            );
            let shimmer = art.shimmer.iter().filter(|&&b| b).count();
            assert!((1..=24).contains(&shimmer));
            for (i, &enabled) in art.shimmer.iter().enumerate() {
                if enabled {
                    assert!(mask[i]);
                }
            }
        }
        assert_eq!(expected.iter().sum::<usize>(), 2359);
    }
    #[test]
    fn fractional_and_24_32_48_rects_retain_all_cells_with_continuous_unlit_fit() {
        for size in [0.125, 22.5, 24.0, 32.0, 48.0] {
            for index in 0..COUNT {
                let mut i = input(index);
                i.rect = [7.0, 9.0, size, size];
                let (frame, stats) = draw(&i);
                assert_eq!(stats.scale, size / 24.0);
                assert_eq!(stats.quads, frame.frame().pixel_count);
                assert_eq!(stats.visible_cells, stats.quads);
                assert_eq!(frame.frame().batches[0].transforms.len(), 1);
                for batch in &frame.frame().batches {
                    for t in &batch.transforms {
                        assert_eq!(t.b[2..], [0.0, 0.0]);
                    }
                    for pixel in &batch.pixels {
                        for corner in [
                            Vec2::splat(-0.5),
                            Vec2::new(0.5, -0.5),
                            Vec2::new(-0.5, 0.5),
                            Vec2::splat(0.5),
                        ] {
                            let p = batch.transforms[pixel.transform as usize]
                                .point(Vec2::from_array(pixel.local) + corner);
                            assert!(
                                p.x >= i.rect[0] - 0.00001
                                    && p.x <= i.rect[0] + size + 0.00001
                                    && p.y >= i.rect[1] - 0.00001
                                    && p.y <= i.rect[1] + size + 0.00001
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn earned_shimmer_only_changes_selected_colors_and_reduced_disabled_unearned_are_static() {
        for index in 0..COUNT {
            let mut i = input(index);
            i.earned = true;
            let (baseline, _) = draw(&PatchInput {
                reduced_motion: true,
                ..i
            });
            let baseline = colors(&baseline);
            let mut observed = false;
            for step in 0..240 {
                i.time = step as f32 * 0.05;
                let (frame, stats) = draw(&i);
                assert_eq!(stats.quads, baseline.len());
                assert!(stats.shimmered_cells <= 24);
                if stats.shimmered_cells > 0 {
                    observed = true;
                    assert_ne!(colors(&frame), baseline);
                }
                let actual = colors(&frame);
                for ((pixel_index, _, _), (&a, &b)) in ART[index]
                    .body
                    .cells_indexed()
                    .zip(actual.iter().zip(&baseline))
                {
                    if a != b {
                        assert!(ART[index].shimmer[pixel_index]);
                    }
                }
            }
            assert!(observed, "no real earned shimmer for index {index}");
            for (earned, disabled, reduced) in [
                (false, false, false),
                (true, true, false),
                (true, false, true),
            ] {
                i.earned = earned;
                i.disabled = disabled;
                i.reduced_motion = reduced;
                i.time = 0.1;
                let (a, sa) = draw(&i);
                i.time = 7.4;
                let (b, sb) = draw(&i);
                assert_eq!(colors(&a), colors(&b));
                assert_eq!(sa.shimmered_cells, 0);
                assert_eq!(sb.shimmered_cells, 0);
            }
        }
    }
    #[test]
    fn fractional_scroll_clip_charges_all_cells_and_contains_all_opaque_corners() {
        for index in 0..COUNT {
            let mut i = input(index);
            i.rect = [7.125, 9.375, 22.5, 22.5];
            i.clip = Some([7.125, 9.375, 11.1, 22.5]);
            let (frame, stats) = draw(&i);
            assert!(stats.visible_cells > 0 && stats.visible_cells < stats.quads);
            assert_eq!(stats.quads, ART[index].body.occupied_count());
            for batch in &frame.frame().batches {
                for pixel in &batch.pixels {
                    let contained = [
                        Vec2::splat(-0.5),
                        Vec2::new(0.5, -0.5),
                        Vec2::new(-0.5, 0.5),
                        Vec2::splat(0.5),
                    ]
                    .into_iter()
                    .all(|corner| {
                        inside(
                            batch.transforms[pixel.transform as usize]
                                .point(Vec2::from_array(pixel.local) + corner),
                            i.clip,
                        )
                    });
                    assert_eq!(pixel.color >> 24 != 0, contained);
                }
            }
        }
    }
    #[test]
    fn shared_pixel_and_work_budget_reject_whole_icons_and_invalid_layouts() {
        let i = input(1);
        let count = ART[1].body.occupied_count();
        for (mut remaining, mut work) in [(count - 1, 120000), (60000, count - 1)] {
            let before = (remaining, work);
            let mut frame = FrameBuilder::new();
            assert_eq!(
                extract(&mut frame, &i, &mut remaining, &mut work),
                Stats::default()
            );
            assert_eq!(frame.frame().pixel_count, 0);
            assert_eq!((remaining, work), before);
        }
        for variant in 0..8 {
            let mut i = i;
            match variant {
                0 => i.index = 9,
                1 => i.rect[2] = 0.0,
                2 => i.rect[0] = f32::NAN,
                3 => i.clip = Some([0.0, 0.0, 0.0, 20.0]),
                4 => i.parent.basis_x = Vec2::ZERO,
                5 => i.parent.position.x = f32::INFINITY,
                6 => i.parent.basis_x = Vec2::splat(f32::MAX),
                _ => i.parent.position.x = 1_100_001.0,
            }
            assert_eq!(draw(&i).1, Stats::default());
        }
    }
    #[test]
    fn independent_ui_projection_keeps_exact_group_geometry_across_world_zoom_and_far_camera() {
        use engine_core::CameraView;
        for camera in [Vec2::ZERO, Vec2::new(999000.0, -999000.0)] {
            for world_zoom in [0.6, 1.0, 1.6] {
                let viewport = Vec2::new(533.0, 300.0);
                let _world = CameraView::new(camera, viewport, world_zoom).unwrap();
                let ui = CameraView::new(camera, viewport, 1.0).unwrap();
                let mut i = input(8);
                i.parent = Transform {
                    position: camera - viewport * 0.5 + Vec2::new(23.0, 29.0),
                    rotation: 0.34,
                    scale: Vec2::new(-1.0, 0.7),
                }
                .affine();
                let (frame, _) = draw(&i);
                let local = Transform {
                    position: Vec2::new(19.0, 21.0),
                    ..Transform::default()
                };
                let expected = i.parent.compose(local.affine());
                let batch = &frame.frame().batches[0];
                let actual = ui.relative_transform(batch.transforms[0]);
                assert!(
                    (Vec2::new(actual.a[0], actual.a[1]) - (expected.position - camera)).length()
                        < 0.0001
                );
                assert_eq!(
                    [actual.a[2], actual.a[3]],
                    [expected.basis_x.x, expected.basis_x.y]
                );
                assert_eq!(
                    [actual.b[0], actual.b[1]],
                    [expected.basis_y.x, expected.basis_y.y]
                );
            }
        }
    }
}
