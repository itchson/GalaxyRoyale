//! Detailed overhead shooting-star art. Pure extraction; no Game/save/ECS state.
use engine_core::{AffineTransform, FrameBuilder, PixelBody, PixelMaterial, Transform, Vec2};
use serde::Deserialize;
use std::{collections::HashMap, f32::consts::PI, sync::LazyLock};

pub const PARALLAX: f32 = 0.48;
pub const HEAD_CELL_SCALE: f32 = 0.25;
pub const LOCAL_RADIUS: f32 = 96.0;
/// Covers the unchanged +/-198 head travel plus the particulate tail.
pub const STREAM_RADIUS: f32 = 300.0;
pub const TAIL_CELLS: usize = 64;

#[derive(Deserialize)]
struct Sprite {
    width: usize,
    height: usize,
    indices: Vec<u8>,
}
#[derive(Deserialize)]
struct Atlas {
    version: u32,
    palette: Vec<u32>,
    sprites: HashMap<String, Sprite>,
}
struct Art {
    head: PixelBody,
    coma: PixelBody,
}
static ART: LazyLock<Art> = LazyLock::new(|| {
    let atlas: Atlas = serde_json::from_str(include_str!(
        "../../../assets/pixels/shooting-star/atlas.json"
    ))
    .expect("validated shooting-star art");
    assert_eq!(atlas.version, 1);
    assert_eq!(atlas.palette[0], 0);
    assert!((2..=64).contains(&atlas.palette.len()));
    assert!(atlas.palette[1..].iter().all(|c| c >> 24 == 255));
    let key = |i: usize| char::from_u32(0xe000 + i as u32).unwrap();
    let palette: Vec<_> = atlas
        .palette
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, &c)| (key(i), c))
        .collect();
    let body = |name: &str, size: [usize; 2]| {
        let s = &atlas.sprites[name];
        assert_eq!([s.width, s.height], size);
        assert_eq!(s.indices.len(), s.width * s.height);
        assert!(
            s.indices
                .iter()
                .all(|&i| (i as usize) < atlas.palette.len())
        );
        let rows: Vec<String> = s
            .indices
            .chunks_exact(s.width)
            .map(|row| {
                row.iter()
                    .map(|&i| if i == 0 { ' ' } else { key(i as usize) })
                    .collect()
            })
            .collect();
        PixelBody::from_ascii(
            &rows.iter().map(String::as_str).collect::<Vec<_>>(),
            &palette,
        )
    };
    Art {
        head: body("nucleus", [48, 32]),
        coma: body("coma", [64, 44]),
    }
});

#[derive(Debug, Clone, Copy)]
pub struct SceneInput {
    /// Existing layer-2 birth origin, before camera/parallax encoding.
    pub origin: Vec2,
    pub camera: Vec2,
    /// Actual world extent, already divided by eased zoom.
    pub viewport: Vec2,
    pub time: f32,
    /// Existing regions.rs field hash `h`, not Game's mutable combat RNG.
    pub birth_hash: u32,
    pub tint: [u32; 4],
    pub reduced: bool,
    pub reduced_motion: bool,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    pub quads: usize,
    pub tail_cells: usize,
    pub transforms: usize,
    pub inactive: bool,
    pub culled: bool,
    pub budget_skipped: bool,
}

// The existing region cycle's exact mixer and unit mapping, used only on birth_hash.
fn mix(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846ca68b);
    x ^ (x >> 16)
}
fn unit(x: u32) -> f32 {
    (x >> 8) as f32 / 16_777_216.0
}
fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
fn time(input: &SceneInput) -> f32 {
    if input.reduced_motion {
        0.0
    } else {
        input.time.max(0.0)
    }
}
fn phase(input: &SceneInput) -> f32 {
    (time(input) + unit(mix(input.birth_hash ^ 303)) * 80.0).rem_euclid(80.0)
}
fn angle(input: &SceneInput) -> f32 {
    -0.55 + unit(mix(input.birth_hash ^ 107)) * 1.1
}
fn parent(input: &SceneInput, phase: f32) -> AffineTransform {
    let angle = angle(input);
    let direction = Vec2::new(angle.cos(), angle.sin());
    Transform {
        position: input.origin
            + input.camera * (1.0 - PARALLAX)
            + direction * ((phase - 9.0) * 22.0),
        rotation: angle,
        scale: Vec2::ONE,
    }
    .affine()
}
fn envelope(phase: f32) -> f32 {
    smooth(phase / 0.8) * smooth((18.0 - phase) / 0.8)
}
fn tinted(color: u32, tint: u32, alpha: u8) -> u32 {
    let a = color.to_le_bytes();
    let b = tint.to_le_bytes();
    let c = std::array::from_fn(|i| {
        if i == 3 {
            alpha
        } else {
            ((a[i] as u16 * 4 + b[i] as u16) / 5) as u8
        }
    });
    u32::from_le_bytes(c)
}
fn valid(input: &SceneInput) -> bool {
    input.origin.is_finite()
        && input.camera.is_finite()
        && input.viewport.is_finite()
        && input.viewport.min_element() > 0.0
        && input.time.is_finite()
        && input.origin.abs().max_element() <= 1_000_000.0
        && input.camera.abs().max_element() <= 1_000_000.0
}
/// Analytic tail cells have stable seeded identities and bounded local age/orbit.
/// Arrays are stack-owned. No per-frame particle Vec or simulation entity exists.
fn particles(input: &SceneInput, phase: f32) -> [(Vec2, u32); TAIL_CELLS] {
    // Reset relative dust only at the seeded head cycle's invisible boundary.
    let clock = phase;
    let gain = envelope(phase);
    std::array::from_fn(|i| {
        let h = mix(input.birth_hash ^ (i as u32 + 1).wrapping_mul(7919));
        let age = (unit(h) + clock * 0.19).fract();
        let distance = 6.0 + age * 83.0;
        let lane = if i % 3 == 0 {
            -1.0
        } else if i % 3 == 1 {
            1.0
        } else {
            0.0
        };
        let side = lane * (0.4 + age * 2.7)
            + (clock * 0.9 + unit(mix(h ^ 71)) * PI * 2.0).sin() * (0.12 + age * 0.55);
        let lift = (age * PI).sin() * (1.0 - age);
        let alpha = ((if i < 32 { 102.0 } else { 68.0 }) * lift * gain) as u8;
        let base = if i < 32 {
            u32::from_le_bytes([163, 199, 208, 255])
        } else {
            u32::from_le_bytes([107, 140, 163, 255])
        };
        (
            Vec2::new(-distance, side),
            tinted(base, input.tint[3], alpha),
        )
    })
}

/// One whole visible head/coma/tail group, charged once to the shared budget.
/// Caller retains original field density/cycle gates and supplies frozen time.
pub fn extract(frame: &mut FrameBuilder, input: &SceneInput, remaining: &mut usize) -> Stats {
    if !valid(input) {
        return Stats::default();
    }
    let phase = phase(input);
    if phase >= 18.0 {
        return Stats {
            inactive: true,
            ..Stats::default()
        };
    }
    let parent = parent(input, phase);
    let relative = parent.position - input.camera;
    if (relative.abs() - input.viewport.min(Vec2::splat(8192.0)) * 0.5).max_element() > LOCAL_RADIUS
    {
        return Stats {
            culled: true,
            ..Stats::default()
        };
    }
    let tail_count = if input.reduced {
        TAIL_CELLS / 2
    } else {
        TAIL_CELLS
    };
    let required = ART.head.occupied_count() + ART.coma.occupied_count() + tail_count;
    if required > *remaining {
        return Stats {
            budget_skipped: true,
            ..Stats::default()
        };
    }
    let gain = envelope(phase);
    let cells = particles(input, phase);
    // Dust first; all child positions remain relative to the same live head.
    for (group, size, emission) in [(0, 0.42, 0.26), (1, 0.75, 0.18)] {
        let child = Transform {
            scale: Vec2::splat(size),
            ..Transform::default()
        }
        .affine();
        frame.push_soft_cells_affine_material(
            parent.compose(child),
            PixelMaterial {
                light_response: 0.0,
                emission,
            },
            cells
                .iter()
                .copied()
                .enumerate()
                .filter(move |(i, _)| i / 32 == group && (!input.reduced || i % 2 == 0))
                .map(|(_, (p, c))| (p / size, c)),
        );
    }
    let detail = parent.compose(
        Transform {
            scale: Vec2::splat(HEAD_CELL_SCALE),
            ..Transform::default()
        }
        .affine(),
    );
    frame.push_soft_cells_affine_material(
        detail,
        PixelMaterial {
            light_response: 0.0,
            emission: 0.20,
        },
        ART.coma
            .cells()
            .map(|(p, c)| (p, tinted(c, input.tint[3], (45.0 * gain) as u8))),
    );
    frame.push_body_affine_colored_material(
        &ART.head,
        detail,
        PixelMaterial {
            light_response: 0.0,
            emission: 0.18,
        },
        |_, c| tinted(c, input.tint[3], (190.0 * gain) as u8),
    );
    *remaining -= required;
    Stats {
        quads: required,
        tail_cells: tail_count,
        transforms: 4,
        ..Stats::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::{CameraView, GpuTransform};
    type Packet = ([f32; 2], u32, GpuTransform);
    fn input() -> SceneInput {
        SceneInput {
            origin: Vec2::ZERO,
            camera: Vec2::ZERO,
            viewport: Vec2::new(533.0, 300.0),
            time: 0.0,
            birth_hash: 0,
            tint: [0xff_b8aea0; 4],
            reduced: false,
            reduced_motion: false,
        }
    }
    fn active() -> SceneInput {
        let mut i = input();
        i.time = (9.0 - unit(mix(i.birth_hash ^ 303)) * 80.0).rem_euclid(80.0);
        i
    }
    fn packets(f: &FrameBuilder) -> Vec<Packet> {
        f.frame()
            .batches
            .iter()
            .flat_map(|b| {
                b.pixels
                    .iter()
                    .map(|p| (p.local, p.color, b.transforms[p.transform as usize]))
            })
            .collect()
    }
    fn draw(i: SceneInput) -> (FrameBuilder, Stats) {
        let mut f = FrameBuilder::new();
        let mut b = 6000;
        let s = extract(&mut f, &i, &mut b);
        assert_eq!(f.frame().pixel_count, s.quads);
        assert_eq!(6000 - b, s.quads);
        (f, s)
    }
    #[test]
    fn cached_overhead_head_has_facet_crater_rim_detail_and_no_new_lod() {
        assert_eq!((ART.head.width, ART.head.height), (48, 32));
        assert!((450..850).contains(&ART.head.occupied_count()));
        assert!(
            ART.head
                .cells()
                .map(|(_, c)| c)
                .collect::<std::collections::HashSet<_>>()
                .len()
                >= 8
        );
        assert!(ART.head.is_occupied(25, 18));
        assert!(!ART.head.is_occupied(0, 0));
        let (_, full) = draw(active());
        let (_, low) = draw(SceneInput {
            reduced: true,
            ..active()
        });
        assert_eq!(full.quads - low.quads, TAIL_CELLS / 2);
        assert_eq!((full.transforms, low.transforms), (4, 4));
    }
    #[test]
    fn seeded_cycle_origin_heading_and_speed_match_original_exactly() {
        for birth_hash in [0, 1, 17, 0xdead_beef, u32::MAX] {
            let a = SceneInput {
                birth_hash,
                ..active()
            };
            let b = SceneInput {
                time: a.time + 0.1,
                ..a
            };
            assert_eq!(
                phase(&a),
                (a.time + unit(mix(birth_hash ^ 303)) * 80.0).rem_euclid(80.0)
            );
            assert_eq!(angle(&a), -0.55 + unit(mix(birth_hash ^ 107)) * 1.1);
            let d = Vec2::new(angle(&a).cos(), angle(&a).sin());
            let pa = parent(&a, phase(&a));
            let pb = parent(&b, phase(&b));
            assert!(
                (pb.position - pa.position - d * ((phase(&b) - phase(&a)) * 22.0)).length()
                    < 0.0001
            );
        }
        let a = active();
        let outside = SceneInput {
            time: a.time + 10.0,
            ..a
        };
        let (f, s) = draw(outside);
        assert!(s.inactive);
        assert!(f.frame().batches.is_empty());
    }
    #[test]
    fn exact_shared_budget_preflight_has_no_partial_head_or_orphan_tail() {
        let (_, s) = draw(active());
        let dot = PixelBody::from_ascii(&["#"], &[('#', 0xffffffff)]);
        for n in [s.quads - 1, s.quads, s.quads + 7] {
            let mut f = FrameBuilder::new();
            f.push_body(&dot, Transform::default());
            let before = packets(&f);
            let mut budget = n;
            let result = extract(&mut f, &active(), &mut budget);
            if n < s.quads {
                assert!(result.budget_skipped);
                assert_eq!(budget, n);
                assert_eq!(packets(&f), before)
            } else {
                assert_eq!(result.quads, s.quads);
                assert_eq!(budget, n - s.quads);
                assert_eq!(f.frame().pixel_count, s.quads + 1)
            }
        }
    }
    #[test]
    fn faded_zero_alpha_cells_still_consume_the_atomic_submitted_budget() {
        let a = active();
        let input = SceneInput {
            time: (80.0 - unit(mix(a.birth_hash ^ 303)) * 80.0).rem_euclid(80.0),
            ..a
        };
        assert_eq!(phase(&input), 0.0);
        let (frame, stats) = draw(input);
        assert_eq!(stats.quads, 884);
        assert!(
            frame
                .frame()
                .batches
                .iter()
                .flat_map(|b| &b.pixels)
                .all(|p| p.color >> 24 == 0)
        );
    }
    #[test]
    fn reduced_density_removes_a_stable_tail_half_and_preserves_every_head_packet() {
        let (full, _) = draw(active());
        let (low, _) = draw(SceneInput {
            reduced: true,
            ..active()
        });
        let full = packets(&full);
        let low = packets(&low);
        assert_eq!(&full[64..], &low[32..]);
        assert_eq!(
            full[..64].iter().step_by(2).copied().collect::<Vec<_>>(),
            low[..32]
        );
    }
    #[test]
    fn relative_tail_is_bounded_emissive_and_fractional_head_keeps_all_cells() {
        let (f, s) = draw(active());
        assert_eq!(s.tail_cells, 64);
        for batch in &f.frame().batches {
            for t in &batch.transforms {
                assert!(t.a.into_iter().chain(t.b).all(f32::is_finite));
                assert_eq!(t.b[2], 0.0);
                assert!(if t.b[3] < 0.0 {
                    (-1.4..=-1.0).contains(&t.b[3])
                } else {
                    (0.0..=0.4).contains(&t.b[3])
                })
            }
        }
        let head_transform = *f.frame().batches[0].transforms.last().unwrap();
        assert!(
            (Vec2::new(head_transform.a[2], head_transform.a[3]).length() - 0.25).abs() < 0.000001
        );
        assert_eq!(
            f.frame().batches[0]
                .pixels
                .iter()
                .filter(|p| p.transform == 3)
                .count(),
            ART.head.occupied_count()
        );
        for (p, _) in particles(&active(), phase(&active())) {
            assert!(p.x >= -89.0 && p.x <= -6.0);
            assert!(p.y.abs() < 4.0)
        }
    }
    #[test]
    fn camera_parallax_is_encoded_once_and_zoom_only_changes_projection() {
        let a = active();
        let b = SceneInput {
            camera: Vec2::new(30.0, -15.0),
            ..a
        };
        let pa = parent(&a, phase(&a)).position - a.camera;
        let pb = parent(&b, phase(&b)).position - b.camera;
        assert!((pb - pa + Vec2::new(30.0, -15.0) * PARALLAX).length() < 0.0001);
        let (f, s) = draw(a);
        for zoom in [0.6, 1.0, 1.6] {
            let view = CameraView::new(a.camera, Vec2::new(533.0, 300.0), zoom).unwrap();
            let (g, gs) = draw(SceneInput {
                viewport: view.world_extent(),
                ..a
            });
            assert_eq!(gs, s);
            assert_eq!(packets(&f), packets(&g));
        }
    }
    #[test]
    fn visible_tail_stays_continuous_across_absolute_clock_wrap() {
        // Actual even layer-2 birth for seed 871238449, grid [-1, -3].
        // Its seeded phase is mid-pass when absolute time crosses 80 seconds.
        let a = SceneInput {
            birth_hash: 3_887_264_754,
            time: 79.999,
            ..input()
        };
        let b = SceneInput { time: 80.001, ..a };
        let pa = phase(&a);
        let pb = phase(&b);
        assert!((2.0..16.0).contains(&pa) && (2.0..16.0).contains(&pb));
        assert!((parent(&b, pb).position - parent(&a, pa).position).length() < 0.1);
        let first = particles(&a, pa);
        let second = particles(&b, pb);
        for reduced in [false, true] {
            let visible = first
                .iter()
                .zip(&second)
                .enumerate()
                .filter(|(i, ((_, ca), (_, cb)))| {
                    (!reduced || i % 2 == 0) && ca >> 24 >= 8 && cb >> 24 >= 8
                })
                .collect::<Vec<_>>();
            assert!(visible.len() >= 8, "continuity check needs visible dust");
            for (i, ((before, _), (after, _))) in visible {
                assert!(
                    (*after - *before).length() < 0.1,
                    "visible dust {i} teleported across absolute clock wrap: {before:?} -> {after:?}"
                );
            }
            let (_, sa) = draw(SceneInput { reduced, ..a });
            let (_, sb) = draw(SceneInput { reduced, ..b });
            assert_eq!(sa, sb);
            let frozen_a = draw(SceneInput {
                reduced,
                reduced_motion: true,
                ..a
            });
            let frozen_b = draw(SceneInput {
                reduced,
                reduced_motion: true,
                ..b
            });
            assert!(frozen_a.1.quads > 0);
            assert_eq!(packets(&frozen_a.0), packets(&frozen_b.0));
        }
    }
    #[test]
    fn seeded_cycle_reset_is_invisible_and_keeps_fade_submissions_bounded() {
        let birth_hash = 3_887_264_754;
        let reset = 80.0 - unit(mix(birth_hash ^ 303)) * 80.0;
        let at = SceneInput {
            birth_hash,
            time: reset,
            ..input()
        };
        assert_eq!(phase(&at), 0.0);
        for reduced in [false, true] {
            let before = draw(SceneInput {
                reduced,
                time: reset - 0.001,
                ..at
            });
            assert!(before.1.inactive && before.0.frame().pixel_count == 0);
            for time in [reset, reset + 0.001] {
                let (frame, stats) = draw(SceneInput {
                    reduced,
                    time,
                    ..at
                });
                assert_eq!(stats.quads, if reduced { 852 } else { 884 });
                assert_eq!(stats.transforms, 4);
                assert!(packets(&frame).iter().all(|(_, color, _)| color >> 24 == 0));
            }
            let (visible, _) = draw(SceneInput {
                reduced,
                time: reset + 0.8,
                ..at
            });
            assert!(
                packets(&visible)
                    .iter()
                    .any(|(_, color, _)| color >> 24 > 0)
            );
        }
    }
    #[test]
    fn reduced_motion_freezes_head_coma_particles_and_colors() {
        let birth_hash = (0..1000)
            .find(|&h| unit(mix(h ^ 303)) * 80.0 > 1.0 && unit(mix(h ^ 303)) * 80.0 < 17.0)
            .unwrap();
        let a = SceneInput {
            reduced_motion: true,
            birth_hash,
            ..active()
        };
        let b = SceneInput { time: 543.0, ..a };
        let first = draw(a);
        assert!(
            first.1.quads > 0,
            "freeze proof must contain an actual admitted star"
        );
        assert_eq!(packets(&first.0), packets(&draw(b).0));
        let a = active();
        let b = SceneInput {
            time: a.time + 0.25,
            ..a
        };
        assert_ne!(packets(&draw(a).0), packets(&draw(b).0));
    }
    #[test]
    fn invalid_distant_and_wrap_cases_add_nothing_or_stay_finite() {
        for bad in [
            SceneInput {
                time: f32::NAN,
                ..active()
            },
            SceneInput {
                origin: Vec2::splat(f32::INFINITY),
                ..active()
            },
            SceneInput {
                viewport: Vec2::ZERO,
                ..active()
            },
            SceneInput {
                camera: Vec2::splat(1_000_001.0),
                ..active()
            },
            SceneInput {
                origin: Vec2::splat(999_000.0),
                ..active()
            },
        ] {
            assert_eq!(draw(bad).0.frame().pixel_count, 0)
        }
        for center in [
            Vec2::ZERO,
            Vec2::new(999_000.0, -999_000.0),
            Vec2::new(-999_000.0, 999_000.0),
        ] {
            let i = SceneInput {
                origin: center * PARALLAX,
                camera: center,
                ..active()
            };
            let (f, s) = draw(i);
            assert!(s.quads > 0);
            let view = CameraView::new(center, Vec2::new(533.0, 300.0), 0.6).unwrap();
            for b in &f.frame().batches {
                for p in &b.pixels {
                    let t = view.relative_transform(b.transforms[p.transform as usize]);
                    for c in [
                        Vec2::splat(-0.5),
                        Vec2::new(0.5, -0.5),
                        Vec2::new(-0.5, 0.5),
                        Vec2::splat(0.5),
                    ] {
                        let corner = t.point(Vec2::from_array(p.local) + c);
                        assert!(corner.is_finite() && corner.length() < LOCAL_RADIUS + 0.25)
                    }
                }
            }
        }
    }
    #[test]
    fn batches_cross_128_without_changing_material_or_group_accounting() {
        let mut f = FrameBuilder::new();
        let dot = PixelBody::from_ascii(&["#"], &[('#', 0xff000000)]);
        for _ in 0..127 {
            f.push_body(&dot, Transform::default())
        }
        let s = extract(&mut f, &active(), &mut 6000);
        assert_eq!(f.frame().pixel_count, 127 + s.quads);
        assert_eq!(f.frame().batches.len(), 2);
        for b in &f.frame().batches {
            assert!(b.transforms.len() <= 128);
            assert!(
                b.pixels
                    .iter()
                    .all(|p| (p.transform as usize) < b.transforms.len())
            )
        }
    }
}
