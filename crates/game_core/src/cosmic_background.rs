//! Original authored cosmic cells: sparse stars, spiral galaxies, torn gas and
//! small supernova remnants. Pure presentation, independent of combat RNG.
use crate::{hash3, mix, regions, unit};
use engine_core::{AffineTransform, FrameBuilder, PixelBody, PixelMaterial, Transform, Vec2};
use std::{f32::consts::TAU, sync::LazyLock};

const BUDGET: usize = 8_000;
const STAR_LIMIT: usize = 1_800;
const STAR_TILE: f32 = 384.0;
const STAR_CANDIDATES: u32 = 32;
const MOTIF_TILE: f32 = 1200.0;
const MOTIF_PARALLAX: f32 = 0.08;
const MOTIF_RADIUS: f32 = 180.0;
const DENSITIES: [f32; 6] = [0.14, 0.54, 0.80, 0.29, 0.43, 0.65];
const TINTS: [[f32; 3]; 6] = [
    [137.0, 141.0, 150.0],
    [173.0, 139.0, 184.0],
    [139.0, 178.0, 158.0],
    [155.0, 143.0, 133.0],
    [166.0, 163.0, 126.0],
    [141.0, 166.0, 185.0],
];

#[derive(Clone, Copy)]
pub(super) struct SceneInput {
    pub seed: u32,
    /// Interpolated unshaken camera. The renderer subtracts its camera once.
    pub camera: Vec2,
    /// Actual world extent, already divided by eased zoom.
    pub viewport: Vec2,
    pub time: f32,
    pub reduced: bool,
    pub reduced_motion: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Stats {
    pub stars: usize,
    pub motif_quads: usize,
    pub motifs: usize,
    pub transforms: usize,
}

#[derive(Clone, Copy)]
struct Atmosphere {
    density: f32,
    tint: [f32; 3],
}
impl Atmosphere {
    fn at(seed: u32, world: Vec2) -> Self {
        let kind = regions::at(seed, world).kind as usize;
        Self {
            density: DENSITIES[kind],
            tint: TINTS[kind],
        }
    }
    fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            density: self.density + (other.density - self.density) * t,
            tint: std::array::from_fn(|i| self.tint[i] + (other.tint[i] - self.tint[i]) * t),
        }
    }
}

/// Four birth-space corner samples per streamed field. Adjacent fields share
/// corners, and smoothstep makes density/color continuous across their edges.
struct FieldBlend([Atmosphere; 4]);
impl FieldBlend {
    fn new(seed: u32, origin: Vec2, spacing: f32, factor: f32) -> Self {
        Self(std::array::from_fn(|i| {
            let offset = Vec2::new((i & 1) as f32, (i >> 1) as f32) * spacing;
            Atmosphere::at(seed, (origin + offset) / factor)
        }))
    }
    fn sample(&self, uv: Vec2, world: Vec2) -> Atmosphere {
        let uv = uv.clamp(Vec2::ZERO, Vec2::ONE);
        let u = smooth(uv.x);
        let v = smooth(uv.y);
        let mut result = self.0[0]
            .lerp(self.0[1], u)
            .lerp(self.0[2].lerp(self.0[3], u), v);
        // A smooth birth-space Haven falloff, never a camera-global reroll.
        let outer = smooth(((world.length() - 1000.0) / 2000.0).clamp(0.0, 1.0));
        result.density = 0.105 + (result.density - 0.105) * outer;
        result.tint = std::array::from_fn(|i| TINTS[0][i] + (result.tint[i] - TINTS[0][i]) * outer);
        result
    }
}
fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MotifKind {
    Galaxy,
    Nebula,
    Supernova,
}
#[derive(Clone, Copy)]
struct Motif {
    kind: MotifKind,
    origin: Vec2,
    hash: u32,
    scale: f32,
}
fn motif(seed: u32, grid: [i32; 2]) -> Option<Motif> {
    let h = hash3(seed ^ 0x636f_736d, grid[0], grid[1], 83);
    let kind = match h % 31 {
        0 => MotifKind::Galaxy,
        1 => MotifKind::Nebula,
        2 => MotifKind::Supernova,
        _ => return None,
    };
    let origin = (Vec2::new(grid[0] as f32, grid[1] as f32)
        + Vec2::new(unit(mix(h ^ 31)), unit(mix(h ^ 71))))
        * MOTIF_TILE;
    Some(Motif {
        kind,
        origin,
        hash: h,
        scale: 0.8 + unit(mix(h ^ 109)) * 0.5,
    })
}

struct Pair {
    full: PixelBody,
    low: PixelBody,
}
impl Pair {
    fn new(body: PixelBody) -> Self {
        let width = body.width / 2;
        let height = body.height / 2;
        let mut palette = Vec::new();
        let rows: Vec<String> = (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| {
                        let color = (0..4)
                            .filter_map(|i| body.color_at(x * 2 + (i & 1), y * 2 + (i >> 1)))
                            .max_by_key(|c| c.to_le_bytes()[3]);
                        if let Some(color) = color {
                            let index = palette
                                .iter()
                                .position(|&(_, c)| c == color)
                                .unwrap_or_else(|| {
                                    let index = palette.len();
                                    palette.push((
                                        char::from_u32(0xe000 + index as u32).unwrap(),
                                        color,
                                    ));
                                    index
                                });
                            palette[index].0
                        } else {
                            ' '
                        }
                    })
                    .collect()
            })
            .collect();
        let low = PixelBody::from_ascii(
            &rows.iter().map(String::as_str).collect::<Vec<_>>(),
            &palette,
        );
        Self { full: body, low }
    }
    fn body(&self, reduced: bool) -> &PixelBody {
        if reduced { &self.low } else { &self.full }
    }
}
struct Art {
    galaxy_arms: Pair,
    galaxy_core: Pair,
    nebula: Pair,
    nebula_thread: Pair,
    nova_shell: Pair,
    nova_core: Pair,
}
static ART: LazyLock<Art> = LazyLock::new(|| {
    // These equations author occupancy once. No bitmap, runtime random fill or
    // giant translucent rectangle is used: vacancies remain actual empty cells.
    let galaxy_arms = template(
        96,
        96,
        |p| {
            let r = p.length();
            if !(5.0..44.0).contains(&r) {
                return None;
            }
            let phase = p.y.atan2(p.x) - r * 0.092;
            let ridge = (phase * 2.0).sin().abs();
            let noise = unit(hash3(
                0x7370_6972,
                p.x.floor() as i32,
                p.y.floor() as i32,
                0,
            ));
            if ridge < 0.17 && noise > 0.16 {
                Some(if ridge < 0.075 { 2 } else { 1 })
            } else if ridge < 0.36 && noise > 0.79 {
                Some(0)
            } else {
                None
            }
        },
        &[
            rgba(69, 61, 86, 20),
            rgba(115, 89, 135, 34),
            rgba(166, 138, 172, 53),
        ],
    );
    let galaxy_core = template(
        24,
        24,
        |p| {
            let r = (p.x * p.x + p.y * p.y * 1.1).sqrt();
            if r < 4.0 {
                Some(2)
            } else if r < 7.0 {
                Some(1)
            } else if r < 10.0 && ((p.x + p.y) as i32).rem_euclid(3) != 0 {
                Some(0)
            } else {
                None
            }
        },
        &[
            rgba(130, 112, 134, 26),
            rgba(186, 167, 153, 46),
            rgba(231, 213, 180, 90),
        ],
    );
    let nebula = template(
        80,
        48,
        |p| {
            let curve = (p.x * 0.11).sin() * 6.0 + (p.x * 0.032).cos() * 3.0;
            let distance = (p.y - curve).abs();
            let noise = unit(hash3(
                0x746f_726e,
                p.x.floor() as i32,
                p.y.floor() as i32,
                7,
            ));
            if p.x.abs() > 36.0 || distance > 8.0 || noise < 0.30 {
                return None;
            }
            if distance < 1.5 && noise > 0.45 {
                Some(2)
            } else if distance < 4.0 {
                Some(1)
            } else if noise > 0.64 {
                Some(0)
            } else {
                None
            }
        },
        &[
            rgba(43, 37, 58, 13),
            rgba(75, 57, 93, 23),
            rgba(127, 85, 131, 38),
        ],
    );
    let nebula_thread = template(
        48,
        24,
        |p| {
            let distance = (p.y - (p.x * 0.15).sin() * 3.0).abs();
            if p.x.abs() < 21.0 && distance < 2.0 && ((p.x * 3.0 + p.y) as i32).rem_euclid(4) != 0 {
                Some(usize::from(distance < 0.75))
            } else {
                None
            }
        },
        &[rgba(48, 60, 71, 18), rgba(88, 111, 116, 32)],
    );
    let nova_shell = template(
        64,
        64,
        |p| {
            let radius = p.length();
            let angle = p.y.atan2(p.x);
            let edge = 21.0 + (angle * 5.0).sin() * 2.3 + (angle * 3.0).cos();
            let noise = unit(hash3(
                0x656a_6563,
                p.x.floor() as i32,
                p.y.floor() as i32,
                17,
            ));
            if (radius - edge).abs() < 1.8 && noise > 0.26 {
                Some(1)
            } else if (radius - edge).abs() < 4.0 && noise > 0.86 {
                Some(0)
            } else {
                None
            }
        },
        &[rgba(94, 69, 93, 18), rgba(161, 112, 104, 35)],
    );
    let nova_core = template(
        12,
        12,
        |p| {
            if p.x.abs() + p.y.abs() < 3.0 {
                Some(1)
            } else if (p.x.abs() < 0.6 || p.y.abs() < 0.6) && p.length() < 5.0 {
                Some(0)
            } else {
                None
            }
        },
        &[rgba(162, 131, 139, 45), rgba(230, 196, 165, 95)],
    );
    Art {
        galaxy_arms: Pair::new(galaxy_arms),
        galaxy_core: Pair::new(galaxy_core),
        nebula: Pair::new(nebula),
        nebula_thread: Pair::new(nebula_thread),
        nova_shell: Pair::new(nova_shell),
        nova_core: Pair::new(nova_core),
    }
});
fn template(
    width: usize,
    height: usize,
    sample: impl Fn(Vec2) -> Option<usize>,
    colors: &[u32],
) -> PixelBody {
    let rows: Vec<String> = (0..height)
        .map(|y| {
            (0..width)
                .map(|x| {
                    let p = Vec2::new(
                        x as f32 + 0.5 - width as f32 * 0.5,
                        y as f32 + 0.5 - height as f32 * 0.5,
                    );
                    sample(p).map_or(' ', |i| (b'a' + i as u8) as char)
                })
                .collect()
        })
        .collect();
    let palette: Vec<_> = colors
        .iter()
        .enumerate()
        .map(|(i, &c)| ((b'a' + i as u8) as char, c))
        .collect();
    PixelBody::from_ascii(
        &rows.iter().map(String::as_str).collect::<Vec<_>>(),
        &palette,
    )
}
fn rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    u32::from_le_bytes([r, g, b, a])
}

fn colored(color: u32, tint: [f32; 3], gain: f32, alpha_gain: f32) -> u32 {
    let channels = color.to_le_bytes();
    let rgb: [u8; 3] = std::array::from_fn(|i| {
        (channels[i] as f32 * (0.72 + tint[i] / 255.0 * 0.45) * gain).clamp(0.0, 255.0) as u8
    });
    rgba(
        rgb[0],
        rgb[1],
        rgb[2],
        (channels[3] as f32 * alpha_gain).clamp(0.0, 150.0) as u8,
    )
}
fn parent(m: Motif, scene: &SceneInput) -> AffineTransform {
    Transform {
        position: m.origin + scene.camera * (1.0 - MOTIF_PARALLAX),
        rotation: unit(mix(m.hash ^ 47)) * TAU,
        scale: Vec2::splat(m.scale),
    }
    .affine()
}
fn draw_motif(
    frame: &mut FrameBuilder,
    m: Motif,
    scene: &SceneInput,
    remaining: &mut usize,
) -> usize {
    let relative = m.origin - scene.camera * MOTIF_PARALLAX;
    if (relative.abs() - scene.viewport * 0.5).max_element() > MOTIF_RADIUS {
        return 0;
    }
    let reduced = scene.reduced;
    let time = if scene.reduced_motion {
        0.0
    } else {
        scene.time
    };
    let phase = unit(m.hash) * TAU;
    let parent = parent(m, scene);
    let double = if reduced { 2.0 } else { 1.0 };
    let (a, b, first, second, alpha) = match m.kind {
        MotifKind::Galaxy => (
            &ART.galaxy_arms,
            &ART.galaxy_core,
            Transform {
                rotation: time * 0.008,
                scale: Vec2::new(double, double * 0.70),
                ..Transform::default()
            },
            Transform {
                scale: Vec2::splat(double),
                ..Transform::default()
            },
            0.93 + 0.07 * (time * 0.16 + phase).sin(),
        ),
        MotifKind::Nebula => (
            &ART.nebula,
            &ART.nebula_thread,
            Transform {
                rotation: (time * 0.04 + phase).sin() * 0.035,
                scale: Vec2::splat(double * 1.1),
                ..Transform::default()
            },
            Transform {
                position: Vec2::new(7.0, -10.0 + (time * 0.08 + phase).sin() * 1.4),
                rotation: -0.2,
                scale: Vec2::splat(double),
            },
            0.86 + 0.14 * (time * 0.10 + phase).sin(),
        ),
        MotifKind::Supernova => {
            let pulse = smooth(0.5 + 0.5 * (time * 0.075 + phase).sin());
            (
                &ART.nova_shell,
                &ART.nova_core,
                Transform {
                    rotation: time * 0.003,
                    scale: Vec2::splat(double * (0.70 + pulse * 0.45)),
                    ..Transform::default()
                },
                Transform {
                    scale: Vec2::splat(double * (0.75 + pulse * 0.25)),
                    ..Transform::default()
                },
                0.40 + 0.60 * pulse,
            )
        }
    };
    let a = a.body(reduced);
    let b = b.body(reduced);
    let required = a.occupied_count() + b.occupied_count();
    if required > *remaining {
        return 0;
    }
    let tint = Atmosphere::at(scene.seed, m.origin / MOTIF_PARALLAX).tint;
    frame.push_body_affine_colored(a, parent.compose(first.affine()), |p, c| {
        colored(
            c,
            tint,
            0.98 + 0.02 * (time * 0.09 + p.x * 0.1 + phase).sin(),
            alpha,
        )
    });
    frame.push_body_affine_colored(b, parent.compose(second.affine()), |_, c| {
        colored(c, tint, 1.0, alpha)
    });
    *remaining -= required;
    required
}

fn star_cells(
    seed: u32,
    grid: [i32; 2],
    layer: usize,
    factor: f32,
    size: f32,
    scene: &SceneInput,
) -> [Option<(Vec2, u32)>; STAR_CANDIDATES as usize] {
    let corner = Vec2::new(grid[0] as f32, grid[1] as f32) * STAR_TILE;
    let blend = FieldBlend::new(seed, corner, STAR_TILE, factor);
    let time = if scene.reduced_motion {
        0.0
    } else {
        scene.time
    };
    std::array::from_fn(|i| {
        let h = hash3(
            seed ^ (0x7374_6172 + layer as u32 * 7919),
            grid[0],
            grid[1],
            i as u32,
        );
        if scene.reduced && h & 1 != 0 {
            return None;
        }
        let uv = Vec2::new(unit(h), unit(mix(h ^ 59)));
        let local = (uv - Vec2::splat(0.5)) * STAR_TILE;
        let origin = corner + Vec2::splat(STAR_TILE * 0.5);
        let relative = origin - scene.camera * factor + local;
        if (relative.abs() - scene.viewport * 0.5).max_element() > size {
            return None;
        }
        let birth = (corner + uv * STAR_TILE) / factor;
        let atmosphere = blend.sample(uv, birth);
        if unit(mix(h ^ 131)) > atmosphere.density {
            return None;
        }
        let twinkle =
            0.89 + 0.11 * (time * (0.12 + unit(mix(h ^ 211)) * 0.14) + unit(h) * TAU).sin();
        let white = if h.is_multiple_of(7) { 0.50 } else { 0.18 };
        let rgb: [u8; 3] =
            std::array::from_fn(|j| (atmosphere.tint[j] * (1.0 - white) + 210.0 * white) as u8);
        let alpha = ((125.0 + layer as f32 * 29.0) * twinkle) as u8;
        Some((local / size, rgba(rgb[0], rgb[1], rgb[2], alpha)))
    })
}

pub(super) fn render(frame: &mut FrameBuilder, input: SceneInput) -> Stats {
    if !input.camera.is_finite()
        || !input.viewport.is_finite()
        || input.viewport.min_element() <= 0.0
        || !input.time.is_finite()
    {
        return Stats::default();
    }
    let scene = SceneInput {
        camera: input
            .camera
            .clamp(Vec2::splat(-1_000_000.0), Vec2::splat(1_000_000.0)),
        viewport: input.viewport.min(Vec2::splat(8192.0)),
        ..input
    };
    let budget = if scene.reduced { BUDGET / 2 } else { BUDGET };
    let star_limit = if scene.reduced {
        STAR_LIMIT / 2
    } else {
        STAR_LIMIT
    };
    let mut remaining = budget - star_limit;
    let mut stats = Stats::default();
    let half = scene.viewport * 0.5;
    let sample = scene.camera * MOTIF_PARALLAX;
    let min = ((sample - half - Vec2::splat(MOTIF_RADIUS)) / MOTIF_TILE)
        .floor()
        .as_ivec2();
    let max = ((sample + half + Vec2::splat(MOTIF_RADIUS)) / MOTIF_TILE)
        .floor()
        .as_ivec2();
    for y in min.y..=max.y {
        for x in min.x..=max.x {
            if let Some(m) = motif(scene.seed, [x, y]) {
                let count = draw_motif(frame, m, &scene, &mut remaining);
                if count > 0 {
                    stats.motif_quads += count;
                    stats.motifs += 1;
                    stats.transforms += 2;
                }
            }
        }
    }
    for (layer, (factor, size)) in [(0.09, 0.70), (0.19, 0.85), (0.38, 1.0), (0.68, 1.25)]
        .into_iter()
        .enumerate()
    {
        let sample = scene.camera * factor;
        let min = ((sample - half - Vec2::splat(size)) / STAR_TILE)
            .floor()
            .as_ivec2();
        let max = ((sample + half + Vec2::splat(size)) / STAR_TILE)
            .floor()
            .as_ivec2();
        for y in min.y..=max.y {
            for x in min.x..=max.x {
                if stats.stars >= star_limit {
                    return stats;
                }
                let origin = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) * STAR_TILE;
                let transform = Transform {
                    position: origin + scene.camera * (1.0 - factor),
                    scale: Vec2::splat(size),
                    ..Transform::default()
                };
                let cells = star_cells(scene.seed, [x, y], layer, factor, size, &scene);
                let count = frame.push_cells_affine_material(
                    transform.affine(),
                    PixelMaterial::default(),
                    cells.into_iter().flatten().take(star_limit - stats.stars),
                );
                stats.stars += count;
                stats.transforms += usize::from(count > 0);
            }
        }
    }
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::{CameraView, GpuTransform, RenderFrame};
    fn input() -> SceneInput {
        SceneInput {
            seed: 17,
            camera: Vec2::ZERO,
            viewport: Vec2::new(533.0, 300.0),
            time: 18.0,
            reduced: false,
            reduced_motion: false,
        }
    }
    fn packets(frame: &RenderFrame) -> Vec<([f32; 2], u32, GpuTransform)> {
        frame
            .batches
            .iter()
            .flat_map(|b| {
                b.pixels
                    .iter()
                    .map(|p| (p.local, p.color, b.transforms[p.transform as usize]))
            })
            .collect()
    }
    fn draw(i: SceneInput) -> (RenderFrame, Stats) {
        let mut frame = FrameBuilder::new();
        let stats = render(&mut frame, i);
        (frame.finish(), stats)
    }
    #[test]
    fn cosmic_seed_revisit_is_exact_and_changes_only_presentation() {
        let a = draw(input());
        let b = draw(input());
        assert_eq!(a.1, b.1);
        assert_eq!(packets(&a.0), packets(&b.0));
        assert!(a.0.pixel_count > 0);
        let mut game = crate::Game::new(crate::Profile::default());
        let before = serde_json::to_vec(game.profile()).unwrap();
        let rng = game.rng;
        let events = game.events.len();
        game.render(0.5);
        game.set_cosmetic_density(true);
        game.set_reduced_motion(true);
        game.render(0.5);
        assert_eq!(serde_json::to_vec(game.profile()).unwrap(), before);
        assert_eq!(game.rng, rng);
        assert_eq!(game.events.len(), events);
    }
    #[test]
    fn cosmic_field_blends_are_continuous_at_adjacent_birth_space_edges() {
        for factor in [0.09, 0.19, 0.38, 0.68] {
            for grid in [[0, 0], [7, -4], [-2100, 1900]] {
                let origin = Vec2::new(grid[0] as f32, grid[1] as f32) * STAR_TILE;
                let left = FieldBlend::new(17, origin, STAR_TILE, factor);
                let right = FieldBlend::new(17, origin + Vec2::X * STAR_TILE, STAR_TILE, factor);
                for v in [0.0, 0.23, 0.7, 1.0] {
                    let world = (origin + Vec2::new(1.0, v) * STAR_TILE) / factor;
                    let a = left.sample(Vec2::new(1.0, v), world);
                    let b = right.sample(Vec2::new(0.0, v), world);
                    assert!((a.density - b.density).abs() < 0.00001);
                    for i in 0..3 {
                        assert!((a.tint[i] - b.tint[i]).abs() < 0.0001);
                    }
                }
            }
        }
    }
    #[test]
    fn cosmic_haven_is_measurably_sparser_than_outer_biomes() {
        let mut haven = 0;
        let mut outer = 0;
        for seed in 1..=32 {
            let mut i = input();
            i.seed = seed;
            haven += draw(i).1.stars;
            i.camera = Vec2::new(27_000.0, -19_000.0);
            outer += draw(i).1.stars;
        }
        assert!(
            outer > haven * 2,
            "actual stars: Haven{haven}, outer{outer}"
        );
        eprintln!(
            "32 declared seeds, 533x300 world view: Haven stars={haven}, outer stars={outer}"
        );
    }
    #[test]
    fn cosmic_reduced_motion_freezes_all_authored_cells_and_group_transforms() {
        for reduced in [false, true] {
            for kind in [MotifKind::Galaxy, MotifKind::Nebula, MotifKind::Supernova] {
                let m = Motif {
                    kind,
                    origin: Vec2::ZERO,
                    hash: 93,
                    scale: 1.1,
                };
                let mut i = input();
                i.reduced = reduced;
                i.reduced_motion = true;
                let mut a = FrameBuilder::new();
                let mut b = FrameBuilder::new();
                draw_motif(&mut a, m, &i, &mut 8000);
                i.time = 901.0;
                draw_motif(&mut b, m, &i, &mut 8000);
                assert_eq!(packets(a.frame()), packets(b.frame()));
                assert_eq!(
                    packets(&draw(i).0),
                    packets(&draw(SceneInput { time: 0.0, ..i }).0)
                );
            }
        }
    }
    #[test]
    fn cosmic_motifs_actually_animate_and_never_swap_identity_with_view_or_quality() {
        for kind in [MotifKind::Galaxy, MotifKind::Nebula, MotifKind::Supernova] {
            let m = Motif {
                kind,
                origin: Vec2::ZERO,
                hash: 93,
                scale: 1.1,
            };
            let mut a = FrameBuilder::new();
            let mut b = FrameBuilder::new();
            let i = input();
            assert!(draw_motif(&mut a, m, &i, &mut 8000) > 0);
            draw_motif(
                &mut b,
                m,
                &SceneInput {
                    time: i.time + 7.0,
                    ..i
                },
                &mut 8000,
            );
            assert_eq!(a.frame().pixel_count, b.frame().pixel_count);
            assert_ne!(packets(a.frame()), packets(b.frame()));
        }
    }
    #[test]
    fn cosmic_real_seeded_motifs_keep_their_authored_packets_across_view_and_quality_changes() {
        let seed = crate::Profile::default().seed;
        for kind in [MotifKind::Galaxy, MotifKind::Nebula, MotifKind::Supernova] {
            let m = (-16..=16)
                .flat_map(|y| (-16..=16).filter_map(move |x| motif(seed, [x, y])))
                .filter(|m| m.kind == kind)
                .min_by(|a, b| {
                    a.origin
                        .length_squared()
                        .total_cmp(&b.origin.length_squared())
                })
                .expect("declared seed contains each rare authored motif");
            let camera = m.origin / MOTIF_PARALLAX;
            eprintln!(
                "{kind:?}: actual default-seed camera={camera:?}, hash={}, scale={}",
                m.hash, m.scale
            );
            for reduced in [false, true] {
                for zoom in [0.6, 1.0, 1.6] {
                    for movement in [Vec2::ZERO, Vec2::new(4.0, -3.0)] {
                        let i = SceneInput {
                            seed,
                            camera: camera + movement,
                            viewport: Vec2::new(533.0, 300.0) / zoom,
                            reduced,
                            ..input()
                        };
                        let (actual, _) = draw(i);
                        let mut expected = FrameBuilder::new();
                        assert!(draw_motif(&mut expected, m, &i, &mut 8000) > 0);
                        let needle = packets(expected.frame());
                        assert!(
                            packets(&actual)
                                .windows(needle.len())
                                .any(|window| window == needle),
                            "seeded{kind:?} switched/omitted in declared uncrowded view at zoom{zoom}"
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn cosmic_groups_reject_atomically_and_all_packets_respect_custom_caps() {
        for reduced in [false, true] {
            for kind in [MotifKind::Galaxy, MotifKind::Nebula, MotifKind::Supernova] {
                let m = Motif {
                    kind,
                    origin: Vec2::ZERO,
                    hash: 93,
                    scale: 1.1,
                };
                let i = SceneInput { reduced, ..input() };
                let mut accepted = FrameBuilder::new();
                let required = draw_motif(&mut accepted, m, &i, &mut 8000);
                let mut rejected = FrameBuilder::new();
                rejected.push_pixel(Vec2::ZERO, 1.0, 0xffaabbcc);
                let mut short = required - 1;
                assert_eq!(draw_motif(&mut rejected, m, &i, &mut short), 0);
                assert_eq!(short, required - 1);
                assert_eq!(rejected.frame().pixel_count, 1);
            }
            for camera in [Vec2::ZERO, Vec2::new(999_000.0, -999_000.0)] {
                for zoom in [0.6, 1.0, 1.6] {
                    let (frame, stats) = draw(SceneInput {
                        camera,
                        reduced,
                        viewport: Vec2::splat(8192.0) / zoom,
                        ..input()
                    });
                    assert!(frame.pixel_count <= if reduced { BUDGET / 2 } else { BUDGET });
                    assert_eq!(frame.pixel_count, stats.stars + stats.motif_quads);
                    assert!(frame.batches.iter().all(|b| b.transforms.len() <= 128));
                    assert!(
                        frame
                            .batches
                            .iter()
                            .flat_map(|b| &b.transforms)
                            .all(|t| t.b[2..] == [0.0, 0.0])
                    );
                }
            }
        }
    }
    #[test]
    fn cosmic_far_world_stars_keep_local_spacing_before_camera_subtraction() {
        for camera in [
            Vec2::new(999_000.0, -999_000.0),
            Vec2::new(-999_000.0, 999_000.0),
        ] {
            let (frame, _) = draw(SceneInput {
                camera,
                viewport: Vec2::new(1200.0, 700.0),
                ..input()
            });
            for zoom in [0.6, 1.0, 1.6] {
                let view = CameraView::new(camera, Vec2::new(1200.0, 700.0), zoom).unwrap();
                for b in &frame.batches {
                    for pixel in &b.pixels {
                        let t = view.relative_transform(b.transforms[pixel.transform as usize]);
                        let p = t.point(Vec2::from_array(pixel.local));
                        assert!(p.is_finite());
                        let width = t.point(Vec2::from_array(pixel.local) + Vec2::X) - p;
                        assert!(width.length() > 0.4 && width.length() < 4.3);
                    }
                }
            }
        }
    }
    #[test]
    fn cosmic_invalid_inputs_preserve_prior_frame_and_authored_negative_space() {
        let mut frame = FrameBuilder::new();
        frame.push_pixel(Vec2::ZERO, 1.0, 0xffaabbcc);
        for bad in [
            SceneInput {
                time: f32::NAN,
                ..input()
            },
            SceneInput {
                camera: Vec2::splat(f32::INFINITY),
                ..input()
            },
            SceneInput {
                viewport: Vec2::ZERO,
                ..input()
            },
        ] {
            assert_eq!(render(&mut frame, bad), Stats::default());
            assert_eq!(frame.frame().pixel_count, 1);
        }
        assert!(ART.galaxy_arms.full.occupied_count() < 96 * 96 / 3);
        assert!(!ART.galaxy_arms.full.is_occupied(0, 0));
        assert!(ART.nebula.full.occupied_count() < 80 * 48 / 3);
        assert!(
            ART.galaxy_core
                .full
                .cells()
                .all(|(_, c)| c.to_le_bytes()[3] <= 90)
        );
    }
}
