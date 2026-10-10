//! Original distant Isotope Scar accent. Packed cells only; no game/ECS state.
use engine_core::{AffineTransform, FrameBuilder, PixelBody, Transform, Vec2};
use serde::Deserialize;
use std::{collections::HashMap, sync::LazyLock};

pub const PARALLAX: f32 = 0.22;
pub const LOCAL_RADIUS: f32 = 80.0;

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
    surface: PixelBody,
    low: PixelBody,
    back: PixelBody,
    front: PixelBody,
    fragment: PixelBody,
}
static ART: LazyLock<Art> = LazyLock::new(|| {
    let atlas: Atlas = serde_json::from_str(include_str!(
        "../../../assets/pixels/isotope-scar-atlas.json"
    ))
    .unwrap();
    assert_eq!(atlas.version, 1);
    assert_eq!(atlas.palette.len(), 64);
    assert_eq!(atlas.palette[0], 0);
    assert!(atlas.palette[1..].iter().all(|color| color >> 24 == 255));
    let body = |name: &str| {
        let sprite = atlas.sprites.get(name).unwrap();
        assert!((1..=128).contains(&sprite.width) && (1..=128).contains(&sprite.height));
        assert_eq!(sprite.indices.len(), sprite.width * sprite.height);
        assert!(sprite.indices.iter().all(|&i| i < 64));
        let key = |i: usize| char::from_u32(0xe000 + i as u32).unwrap();
        let palette: Vec<_> = atlas
            .palette
            .iter()
            .enumerate()
            .skip(1)
            .map(|(i, &c)| (key(i), c))
            .collect();
        let rows: Vec<String> = sprite
            .indices
            .chunks_exact(sprite.width)
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
        surface: body("scar_surface"),
        low: body("scar_surface_low"),
        back: body("scar_ion_back"),
        front: body("scar_ion_front"),
        fragment: body("scar_fragment"),
    }
});

/// `origin` is the layer's seed/grid anchor before camera-relative encoding.
/// `viewport` is the actual world extent, already divided by eased zoom.
#[derive(Debug, Clone, Copy)]
pub struct SceneInput {
    pub origin: Vec2,
    pub camera: Vec2,
    pub viewport: Vec2,
    pub time: f32,
    pub rotation: f32,
    pub scale: Vec2,
    pub reduced: bool,
    pub reduced_motion: bool,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    pub quads: usize,
    pub transforms: usize,
    pub culled: bool,
    pub budget_skipped: bool,
}

fn valid(input: &SceneInput) -> bool {
    input.origin.is_finite()
        && input.camera.is_finite()
        && input.viewport.is_finite()
        && input.viewport.min_element() > 0.0
        && input.time.is_finite()
        && input.rotation.is_finite()
        && input.scale.is_finite()
        && input.scale.abs().min_element() >= 0.3
        && input.scale.abs().max_element() <= 0.9
        && input.origin.abs().max_element() <= 1_000_000.0
        && input.camera.abs().max_element() <= 1_000_000.0
}
fn parent(input: &SceneInput) -> AffineTransform {
    Transform {
        position: input.origin + input.camera * (1.0 - PARALLAX),
        rotation: input.rotation,
        scale: input.scale,
    }
    .affine()
}
fn layers(input: &SceneInput) -> [(Option<&'static PixelBody>, Transform, u8); 4] {
    let time = if input.reduced_motion {
        0.0
    } else {
        input.time.max(0.0)
    };
    let slow = (time * 0.08).sin();
    let sway = (time * 0.053).cos();
    [
        (
            (!input.reduced).then_some(&ART.back),
            Transform {
                position: Vec2::new(-6.0 + slow * 1.6, -22.0 + sway * 0.7),
                rotation: -0.18,
                scale: Vec2::new(0.94, 1.05),
            },
            48,
        ),
        (
            Some(if input.reduced {
                &ART.low
            } else {
                &ART.surface
            }),
            Transform {
                scale: Vec2::splat(if input.reduced { 2.0 } else { 1.0 }),
                ..Transform::default()
            },
            184,
        ),
        (
            Some(&ART.fragment),
            Transform {
                position: Vec2::new(40.0, -27.0),
                rotation: -0.45,
                scale: Vec2::splat(0.74),
            },
            156,
        ),
        (
            Some(&ART.front),
            Transform {
                position: Vec2::new(8.0 - slow * 1.1, 17.0 + sway * 0.6),
                rotation: -0.12,
                scale: Vec2::new(0.93, 0.88),
            },
            66,
        ),
    ]
}

fn color(color: u32, alpha: u8, local: Vec2, cosmetic_time: f32) -> u32 {
    let [r, g, b, _] = color.to_le_bytes();
    // Only authored mineral-green cells shimmer; lithosphere and silhouette stay stable.
    let mineral = g as i16 - r as i16 > 10 && g > b;
    let factor = if mineral {
        1.0 + (cosmetic_time * 0.14 + local.x * 0.025 + local.y * 0.018).sin() * 0.055
    } else {
        1.0
    };
    u32::from_le_bytes([
        (r as f32 * factor).round().clamp(0.0, 255.0) as u8,
        (g as f32 * factor).round().clamp(0.0, 255.0) as u8,
        (b as f32 * factor).round().clamp(0.0, 255.0) as u8,
        alpha,
    ])
}

/// Adds one whole visible motif within the caller's existing cosmetic budget.
/// It receives no Game, world, RNG, save, physics, audio or gameplay material.
pub fn extract(frame: &mut FrameBuilder, input: &SceneInput, remaining: &mut usize) -> Stats {
    if !valid(input) {
        return Stats::default();
    }
    let relative = input.origin - input.camera * PARALLAX;
    let half = input.viewport.min(Vec2::splat(8192.0)) * 0.5;
    let radius = LOCAL_RADIUS * input.scale.abs().max_element();
    if relative.abs().x > half.x + radius || relative.abs().y > half.y + radius {
        return Stats {
            culled: true,
            ..Stats::default()
        };
    }
    let layers = layers(input);
    let quads: usize = layers
        .iter()
        .filter_map(|(body, _, _)| *body)
        .map(PixelBody::occupied_count)
        .sum();
    if quads > *remaining {
        return Stats {
            budget_skipped: true,
            ..Stats::default()
        };
    }
    let parent = parent(input);
    let time = if input.reduced_motion {
        0.0
    } else {
        input.time.max(0.0)
    };
    let mut transforms = 0;
    for (body, child, alpha) in layers {
        if let Some(body) = body {
            frame.push_body_affine_colored(body, parent.compose(child.affine()), |local, c| {
                color(c, alpha, local, time)
            });
            transforms += 1;
        }
    }
    *remaining -= quads;
    Stats {
        quads,
        transforms,
        ..Stats::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> SceneInput {
        SceneInput {
            origin: Vec2::new(120.0, -37.0),
            camera: Vec2::new(500.0, -180.0),
            viewport: Vec2::new(640.0, 360.0),
            time: 13.0,
            rotation: -0.24,
            scale: Vec2::splat(0.68),
            reduced: false,
            reduced_motion: false,
        }
    }
    fn packed(frame: &FrameBuilder) -> Vec<(Vec2, u32)> {
        frame
            .frame()
            .batches
            .iter()
            .flat_map(|batch| {
                batch.pixels.iter().map(|pixel| {
                    (
                        batch.transforms[pixel.transform as usize]
                            .point(Vec2::from_array(pixel.local)),
                        pixel.color,
                    )
                })
            })
            .collect()
    }
    #[test]
    fn authored_fields_have_real_holes_detail_and_bounded_full_reduced_counts() {
        assert_eq!((ART.surface.width, ART.surface.height), (96, 96));
        assert_eq!((ART.low.width, ART.low.height), (48, 48));
        assert!(!ART.surface.is_occupied(0, 0));
        assert!(
            !ART.surface.is_occupied(80, 29),
            "authored bitten limb stays empty"
        );
        assert!(ART.surface.is_occupied(45, 45));
        let colors: std::collections::HashSet<_> = ART.surface.cells().map(|(_, c)| c).collect();
        assert!(colors.len() >= 15);
        let mut frame = FrameBuilder::new();
        let mut budget = 6000;
        let full = extract(&mut frame, &input(), &mut budget);
        assert_eq!(full.quads, frame.frame().pixel_count);
        assert!(full.quads <= 5100 && full.quads > 4000);
        assert_eq!(full.transforms, 4);
        let mut smaller = input();
        smaller.reduced = true;
        frame.clear();
        budget = 3000;
        let reduced = extract(&mut frame, &smaller, &mut budget);
        assert!(reduced.quads <= 1700 && reduced.quads > 1000);
        assert_eq!(reduced.transforms, 3);
        assert!(reduced.quads * 2 < full.quads);
    }
    #[test]
    fn extraction_exactly_composes_all_live_cell_centers_and_unlit_materials() {
        let mut i = input();
        i.scale = Vec2::new(-0.72, 0.43);
        i.rotation = 0.71;
        let mut frame = FrameBuilder::new();
        let mut remaining = 6000;
        extract(&mut frame, &i, &mut remaining);
        let actual = packed(&frame);
        let expected: Vec<_> = layers(&i)
            .into_iter()
            .flat_map(|(body, child, alpha)| {
                body.into_iter().flat_map(move |b| {
                    b.cells().map(move |(local, c)| {
                        (
                            parent(&i).point(child.point(local)),
                            color(c, alpha, local, i.time),
                        )
                    })
                })
            })
            .collect();
        assert_eq!(actual.len(), expected.len());
        for ((a, ac), (e, ec)) in actual.iter().zip(expected.iter()) {
            assert!((*a - *e).length() < 0.0001);
            assert_eq!(ac, ec);
        }
        assert!(
            frame
                .frame()
                .batches
                .iter()
                .flat_map(|b| &b.transforms)
                .all(|t| t.b[2..] == [0.0, 0.0])
        );
    }
    #[test]
    fn parallax_movement_is_encoded_once_and_zoom_changes_only_projection() {
        let a = input();
        let mut b = a;
        b.camera += Vec2::new(20.0, -10.0);
        let relative = |i: &SceneInput| parent(i).position - i.camera;
        let observed = relative(&b) - relative(&a);
        assert!((observed + Vec2::new(20.0, -10.0) * PARALLAX).length() < 0.0001);
        for zoom in [0.6, 1.0, 1.6] {
            let mut i = a;
            i.viewport = Vec2::new(533.0, 300.0) / zoom;
            let mut frame = FrameBuilder::new();
            let mut remaining = 6000;
            let stats = extract(&mut frame, &i, &mut remaining);
            assert!(stats.quads > 0);
            assert_eq!(stats.quads, 6000 - remaining);
        }
    }
    #[test]
    fn slow_cosmetic_animation_preserves_occupancy_and_reduced_motion_freezes_it() {
        let a = input();
        let mut b = a;
        b.time += 18.0;
        let mut f = FrameBuilder::new();
        let mut g = FrameBuilder::new();
        extract(&mut f, &a, &mut 6000);
        extract(&mut g, &b, &mut 6000);
        assert_eq!(f.frame().pixel_count, g.frame().pixel_count);
        assert!(
            packed(&f)
                .iter()
                .zip(packed(&g))
                .any(|((p, c), (q, d))| p != &q || *c != d)
        );
        let mut a = a;
        a.reduced_motion = true;
        b.reduced_motion = true;
        f.clear();
        g.clear();
        extract(&mut f, &a, &mut 6000);
        extract(&mut g, &b, &mut 6000);
        assert_eq!(packed(&f), packed(&g));
        assert_eq!(
            ART.surface.occupied_count(),
            f.frame().batches[0]
                .pixels
                .iter()
                .filter(|p| p.transform == 1)
                .count()
        );
    }
    #[test]
    fn shared_budget_is_atomic_and_invalid_or_distant_inputs_emit_nothing() {
        let mut frame = FrameBuilder::new();
        let mut remaining = 6000;
        let full = extract(&mut frame, &input(), &mut remaining);
        frame.clear();
        let mut small = full.quads - 1;
        let rejected = extract(&mut frame, &input(), &mut small);
        assert!(rejected.budget_skipped);
        assert_eq!(small, full.quads - 1);
        assert_eq!(frame.frame().pixel_count, 0);
        for variant in 0..6 {
            let mut i = input();
            match variant {
                0 => i.time = f32::NAN,
                1 => i.origin.x = f32::INFINITY,
                2 => i.viewport.y = 0.0,
                3 => i.scale.x = 0.0,
                4 => i.camera.x = 1_000_001.0,
                _ => i.origin.x = 999_000.0,
            }
            let mut remaining = 6000;
            extract(&mut frame, &i, &mut remaining);
            assert_eq!(remaining, 6000);
            assert_eq!(frame.frame().pixel_count, 0);
        }
    }
    #[test]
    fn far_world_projection_and_all_quad_corners_stay_finite_within_cull_radius() {
        for p in [
            Vec2::ZERO,
            Vec2::new(999_000.0, -999_000.0),
            Vec2::new(-999_000.0, 999_000.0),
        ] {
            for rotation in [-0.7, 0.0, 0.83] {
                let mut i = input();
                i.camera = p;
                i.origin = p * PARALLAX;
                i.rotation = rotation;
                let mut frame = FrameBuilder::new();
                extract(&mut frame, &i, &mut 6000);
                assert!(frame.frame().pixel_count > 0);
                for batch in &frame.frame().batches {
                    for pixel in &batch.pixels {
                        let transform = batch.transforms[pixel.transform as usize];
                        for corner in [
                            Vec2::new(-0.5, -0.5),
                            Vec2::new(0.5, -0.5),
                            Vec2::new(0.5, 0.5),
                            Vec2::new(-0.5, 0.5),
                        ] {
                            let world = transform.point(Vec2::from_array(pixel.local) + corner);
                            assert!(world.is_finite());
                            assert!(
                                (world - parent(&i).position).length()
                                    <= LOCAL_RADIUS * i.scale.max_element() + 0.125
                            );
                        }
                    }
                }
            }
        }
    }
}
