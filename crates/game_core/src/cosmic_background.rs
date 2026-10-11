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
const SYSTEM_TILE: f32 = 768.0;
const SYSTEM_PARALLAX: f32 = 0.27;
const GAS_LIMIT: usize = 800;
const GAS_TILE: f32 = 256.0;
const GAS_SIZES: [f32; 4] = [0.45, 0.7, 1.05, 1.55];
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
    pub systems: usize,
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
            rgba(43, 37, 58, 30),
            rgba(75, 57, 93, 58),
            rgba(127, 85, 131, 96),
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
        &[rgba(48, 60, 71, 32), rgba(88, 111, 116, 58)],
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

/// A seeded stellar neighborhood is born once in field space. Its nodes and
/// connecting dotted asterism never depend on camera, quality or animation.
fn stellar_system(seed: u32, grid: [i32; 2]) -> Option<(Vec2, [Vec2; 7], u32)> {
    let h = hash3(seed ^ 0x7379_7374, grid[0], grid[1], 71);
    if !h.is_multiple_of(17) {
        return None;
    }
    let origin = (Vec2::new(grid[0] as f32, grid[1] as f32)
        + Vec2::new(0.2 + unit(h) * 0.6, 0.2 + unit(mix(h ^ 31)) * 0.6))
        * SYSTEM_TILE;
    let angle = unit(mix(h ^ 137)) * TAU;
    let axis = Vec2::new(angle.cos(), angle.sin());
    let side = Vec2::new(-axis.y, axis.x);
    let nodes = std::array::from_fn(|i| {
        let q = mix(h ^ (i as u32 + 1).wrapping_mul(7919));
        let x = (i as f32 - 3.0) * 20.0 + (unit(q) - 0.5) * 10.0;
        let y = (unit(mix(q ^ 83)) - 0.5) * 55.0;
        axis * x + side * y
    });
    Some((origin, nodes, h))
}

fn draw_system(
    frame: &mut FrameBuilder,
    scene: &SceneInput,
    origin: Vec2,
    nodes: [Vec2; 7],
    h: u32,
    remaining: usize,
) -> usize {
    let relative = origin - scene.camera * SYSTEM_PARALLAX;
    if (relative.abs() - scene.viewport * 0.5).max_element() > 90.0 {
        return 0;
    }
    let tint = Atmosphere::at(scene.seed, origin / SYSTEM_PARALLAX).tint;
    let time = if scene.reduced_motion {
        0.0
    } else {
        scene.time
    };
    let brightness = 0.94 + 0.06 * (time * 0.13 + unit(h) * TAU).sin();
    let mut links = [None; 96];
    let mut count = 0;
    for edge in 0..6 {
        let a = nodes[edge];
        let b = nodes[edge + 1];
        let steps = ((b - a).length() / 3.2).ceil() as usize;
        for step in 1..steps {
            if (scene.reduced && step % 2 == 1) || count == links.len() {
                continue;
            }
            let p = a.lerp(b, step as f32 / steps as f32);
            links[count] = Some((
                p / 0.55,
                rgba(tint[0] as u8, tint[1] as u8, tint[2] as u8, 38),
            ));
            count += 1;
        }
    }
    // Whole asterism admission keeps seven real nodes with their visible links.
    if remaining < count + nodes.len() {
        return 0;
    }
    let parent = origin + scene.camera * (1.0 - SYSTEM_PARALLAX);
    frame.push_cells_affine_material(
        Transform {
            position: parent,
            scale: Vec2::splat(0.55),
            ..Transform::default()
        }
        .affine(),
        PixelMaterial::default(),
        links.into_iter().flatten(),
    );
    frame.push_cells_affine_material(
        Transform {
            position: parent,
            scale: Vec2::splat(1.15),
            ..Transform::default()
        }
        .affine(),
        PixelMaterial::default(),
        nodes.into_iter().enumerate().map(|(i, p)| {
            let warm = if (h >> i) & 1 == 0 {
                [229, 211, 179]
            } else {
                [172, 211, 227]
            };
            (
                p / 1.15,
                rgba(warm[0], warm[1], warm[2], (190.0 * brightness) as u8),
            )
        }),
    );
    count + nodes.len()
}

/// Region-owned gas quota: no physics body or combat RNG, and every emitted
/// soft cell is charged against the caller's existing cosmetic quota. Field
/// translations are encoded once; curl remains small local affine geometry.
pub(super) fn render_gas(
    frame: &mut FrameBuilder,
    scene: SceneInput,
    remaining: &mut usize,
) -> usize {
    if !scene.camera.is_finite()
        || !scene.viewport.is_finite()
        || scene.viewport.min_element() <= 0.0
        || !scene.time.is_finite()
    {
        return 0;
    }
    let scene = SceneInput {
        camera: scene
            .camera
            .clamp(Vec2::splat(-1_000_000.0), Vec2::splat(1_000_000.0)),
        ..scene
    };
    let limit = (if scene.reduced {
        GAS_LIMIT / 2
    } else {
        GAS_LIMIT
    })
    .min(*remaining);
    let mut emitted = 0;
    let time = if scene.reduced_motion {
        0.0
    } else {
        scene.time
    };
    let half = scene.viewport.min(Vec2::splat(8192.0)) * 0.5;
    for (layer, factor) in [(0_u32, 0.16), (1, 0.36)] {
        let sample = scene.camera * factor;
        let min = ((sample - half - Vec2::splat(116.0)) / GAS_TILE)
            .floor()
            .as_ivec2();
        let max = ((sample + half + Vec2::splat(116.0)) / GAS_TILE)
            .floor()
            .as_ivec2();
        for y in min.y..=max.y {
            for x in min.x..=max.x {
                let field = hash3(scene.seed ^ 0x6761_7300, x, y, layer);
                let origin = (Vec2::new(x as f32, y as f32)
                    + Vec2::new(0.15 + unit(field) * 0.7, 0.15 + unit(mix(field ^ 29)) * 0.7))
                    * GAS_TILE;
                let blend = FieldBlend::new(
                    scene.seed,
                    Vec2::new(x as f32, y as f32) * GAS_TILE,
                    GAS_TILE,
                    factor,
                );
                let atmosphere = blend.sample(Vec2::splat(0.5), origin / factor);
                let angle = unit(mix(field ^ 19)) * TAU;
                let direction = Vec2::new(angle.cos(), angle.sin());
                let side = Vec2::new(-direction.y, direction.x);
                let biome = regions::at(scene.seed, origin / factor);
                let palette = biome.palette[2].to_le_bytes();
                let haven_gain =
                    smooth((((origin / factor).length() - 500.0) / 2000.0).clamp(0.0, 1.0));
                for (size_index, size) in GAS_SIZES.into_iter().enumerate() {
                    let cells = (0..96_u32).filter_map(|i| {
                        let h = mix(field ^ i.wrapping_mul(7919) ^ 37);
                        if h as usize & 3 != size_index
                            || (scene.reduced && h & 4 != 0)
                            || unit(mix(h ^ 131)) > atmosphere.density * 0.9 + 0.1
                        {
                            return None;
                        }
                        let along = unit(h) * 2.0 - 1.0;
                        let phase =
                            time * (0.07 + unit(mix(h ^ 41)) * 0.035) + unit(mix(h ^ 11)) * TAU;
                        let curl = (along * 5.2 + phase * 0.8).sin() * 13.0;
                        let local = direction * (along * 99.0 + phase.sin() * 7.0)
                            + side
                                * (curl
                                    + (unit(mix(h ^ 47)) - 0.5) * 14.0
                                    + (phase * 0.73).cos() * 4.0);
                        let relative = origin - scene.camera * factor + local;
                        // Four world units conservatively cover tiny soft raster
                        // expansion for the supported mobile/desktop extents.
                        if (relative.abs() - half).max_element() > size + 4.0 {
                            return None;
                        }
                        let envelope = (1.0 - along.abs() * 0.55) * (0.82 + 0.18 * phase.sin());
                        let alpha = ((34.0 + unit(mix(h ^ 71)) * 66.0)
                            * envelope
                            * (0.07 + 0.93 * haven_gain)) as u8;
                        let tint: [u8; 3] = std::array::from_fn(|j| {
                            ((atmosphere.tint[j] * 0.42 + palette[j] as f32 * 0.55)
                                * (0.8 + 0.2 * envelope))
                                .min(190.0) as u8
                        });
                        Some((local / size, rgba(tint[0], tint[1], tint[2], alpha)))
                    });
                    let count = frame.push_soft_cells_affine_material(
                        Transform {
                            position: origin + scene.camera * (1.0 - factor),
                            scale: Vec2::splat(size),
                            ..Transform::default()
                        }
                        .affine(),
                        PixelMaterial::default(),
                        cells.take(limit - emitted),
                    );
                    emitted += count;
                    if emitted == limit {
                        *remaining -= emitted;
                        return emitted;
                    }
                }
            }
        }
    }
    *remaining -= emitted;
    emitted
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
    let sample = scene.camera * SYSTEM_PARALLAX;
    let min = ((sample - half - Vec2::splat(90.0)) / SYSTEM_TILE)
        .floor()
        .as_ivec2();
    let max = ((sample + half + Vec2::splat(90.0)) / SYSTEM_TILE)
        .floor()
        .as_ivec2();
    for y in min.y..=max.y {
        for x in min.x..=max.x {
            if let Some((origin, nodes, h)) = stellar_system(scene.seed, [x, y]) {
                let count = draw_system(frame, &scene, origin, nodes, h, star_limit - stats.stars);
                stats.stars += count;
                stats.systems += usize::from(count > 0);
                stats.transforms += usize::from(count > 0) * 2;
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
    fn nebula_visibility_keeps_authored_cells_unlit_and_within_existing_group_budget() {
        let seed = crate::Profile::default().seed;
        let m = (-16..=16)
            .flat_map(|y| (-16..=16).filter_map(move |x| motif(seed, [x, y])))
            .filter(|m| m.kind == MotifKind::Nebula)
            .min_by(|a, b| {
                a.origin
                    .length_squared()
                    .total_cmp(&b.origin.length_squared())
            })
            .unwrap();
        let camera = m.origin / MOTIF_PARALLAX;
        assert_eq!(camera, Vec2::new(29410.637, 11408.639));
        for reduced in [false, true] {
            let a = ART.nebula.body(reduced);
            let b = ART.nebula_thread.body(reduced);
            let expected = a.occupied_count() + b.occupied_count();
            for time in [0.0, 15.0, 47.0] {
                let scene = SceneInput {
                    seed,
                    camera,
                    reduced,
                    time,
                    ..input()
                };
                let mut frame = FrameBuilder::new();
                let mut budget = expected;
                assert_eq!(draw_motif(&mut frame, m, &scene, &mut budget), expected);
                assert_eq!(budget, 0);
                assert_eq!(frame.frame().pixel_count, expected);
                assert_eq!(frame.frame().batches[0].transforms.len(), 2);
                assert!(
                    frame.frame().batches[0]
                        .transforms
                        .iter()
                        .all(|t| t.b[2..] == [0.0, 0.0])
                );
                let submitted: Vec<_> = frame.frame().batches[0].pixels.iter().collect();
                let source: Vec<_> = a.cells().chain(b.cells()).collect();
                assert!(
                    submitted
                        .iter()
                        .zip(source)
                        .all(|(pixel, (local, _))| pixel.local == local.to_array())
                );
                let min_alpha = submitted
                    .iter()
                    .map(|p| p.color.to_le_bytes()[3])
                    .min()
                    .unwrap();
                let max_alpha = submitted
                    .iter()
                    .map(|p| p.color.to_le_bytes()[3])
                    .max()
                    .unwrap();
                assert!(min_alpha >= 21 && (69..=96).contains(&max_alpha));
                eprintln!(
                    "Actual nebula reduced={reduced} time={time}: {expected} cells/2 transforms, alpha {min_alpha}..{max_alpha}"
                );
                let mut rejected = FrameBuilder::new();
                let mut short = expected - 1;
                assert_eq!(draw_motif(&mut rejected, m, &scene, &mut short), 0);
                assert_eq!(short, expected - 1);
                assert!(rejected.frame().batches.is_empty());
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

    #[test]
    fn stellar_neighborhoods_keep_real_nodes_and_exact_seeded_links_under_quality_and_parallax() {
        let seed = crate::Profile::default().seed;
        let (origin, nodes, h) = (-8..=8)
            .flat_map(|y| (-8..=8).filter_map(move |x| stellar_system(seed, [x, y])))
            .min_by(|a, b| a.0.length_squared().total_cmp(&b.0.length_squared()))
            .unwrap();
        let camera = origin / SYSTEM_PARALLAX;
        for reduced in [false, true] {
            let scene = SceneInput {
                seed,
                camera,
                reduced,
                ..input()
            };
            let mut frame = FrameBuilder::new();
            let required = draw_system(&mut frame, &scene, origin, nodes, h, 200);
            assert!((7..=103).contains(&required));
            assert_eq!(frame.frame().pixel_count, required);
            assert_eq!(
                frame
                    .frame()
                    .batches
                    .iter()
                    .map(|b| b.transforms.len())
                    .sum::<usize>(),
                2
            );
            assert!(
                frame
                    .frame()
                    .batches
                    .iter()
                    .flat_map(|b| &b.transforms)
                    .all(|t| t.b[2..] == [0.0, 0.0])
            );
            let mut rejected = FrameBuilder::new();
            assert_eq!(
                draw_system(&mut rejected, &scene, origin, nodes, h, required - 1),
                0
            );
            assert_eq!(rejected.frame().pixel_count, 0);
            for zoom in [0.6, 1.0, 1.6] {
                let viewport = Vec2::new(533.0, 300.0) / zoom;
                let (actual, stats) = draw(SceneInput { viewport, ..scene });
                assert!(stats.systems > 0);
                assert!(
                    packets(&actual)
                        .windows(required)
                        .any(|p| p == packets(frame.frame()))
                );
            }
            let mut frozen = FrameBuilder::new();
            let mut again = FrameBuilder::new();
            draw_system(
                &mut frozen,
                &SceneInput {
                    time: 0.0,
                    reduced_motion: true,
                    ..scene
                },
                origin,
                nodes,
                h,
                200,
            );
            draw_system(
                &mut again,
                &SceneInput {
                    time: 901.0,
                    reduced_motion: true,
                    ..scene
                },
                origin,
                nodes,
                h,
                200,
            );
            assert_eq!(packets(frozen.frame()), packets(again.frame()));
        }
        eprintln!("actual seeded system camera={camera:?},7nodes,hash={h}");
    }

    #[test]
    fn soft_seeded_gas_uses_local_fractional_cells_and_charges_actual_shared_quota() {
        for reduced in [false, true] {
            for camera in [
                Vec2::ZERO,
                Vec2::new(27000.0, -19000.0),
                Vec2::new(999000.0, -999000.0),
            ] {
                for zoom in [0.6, 1.0, 1.6] {
                    let scene = SceneInput {
                        camera,
                        reduced,
                        viewport: Vec2::new(533.0, 300.0) / zoom,
                        ..input()
                    };
                    let mut frame = FrameBuilder::new();
                    frame.push_pixel(Vec2::ZERO, 1.0, 0xffffffff);
                    let mut remaining = if reduced { 3000 } else { 6000 };
                    let initial = remaining;
                    let count = render_gas(&mut frame, scene, &mut remaining);
                    assert!(count > 0 && count <= if reduced { 400 } else { 800 });
                    assert_eq!(remaining, initial - count);
                    assert_eq!(frame.frame().pixel_count, count + 1);
                    assert!(
                        frame
                            .frame()
                            .batches
                            .iter()
                            .all(|b| b.transforms.len() <= 128)
                    );
                    let soft: Vec<_> = frame
                        .frame()
                        .batches
                        .iter()
                        .flat_map(|b| &b.transforms)
                        .filter(|t| t.b[3] < 0.0)
                        .collect();
                    assert!(!soft.is_empty());
                    assert!(soft.iter().all(|t| t.b[2..] == [0.0, -1.0]));
                    assert!(
                        soft.iter()
                            .all(|t| GAS_SIZES.iter().any(|s| (t.a[2] - *s).abs() < 0.00001))
                    );
                    let mut limited = FrameBuilder::new();
                    let mut quota = 31;
                    let actual = render_gas(&mut limited, scene, &mut quota);
                    assert_eq!(actual, limited.frame().pixel_count);
                    assert_eq!(31 - actual, quota);
                    assert!(actual <= 31);
                    let view = CameraView::new(camera, Vec2::new(533.0, 300.0), zoom).unwrap();
                    for b in &frame.frame().batches {
                        for p in &b.pixels {
                            let transform = b.transforms[p.transform as usize];
                            if transform.b[3] >= 0.0 {
                                continue;
                            }
                            let relative = view.relative_transform(transform);
                            let point = relative.point(Vec2::from_array(p.local));
                            assert!(point.is_finite());
                            assert!((point.abs() - scene.viewport * 0.5).max_element() < 6.0);
                        }
                    }
                    eprintln!(
                        "gas reduced{reduced} camera{camera:?} zoom{zoom}: submitted{count},remaining{remaining},softgroups{}",
                        soft.len()
                    );
                }
            }
        }
    }

    #[test]
    fn gas_curl_is_repeatable_and_reduced_motion_freezes_all_submitted_cells() {
        let render = |time, reduced_motion| {
            let mut frame = FrameBuilder::new();
            let mut quota = 6000;
            render_gas(
                &mut frame,
                SceneInput {
                    camera: Vec2::new(27000.0, -19000.0),
                    time,
                    reduced_motion,
                    ..input()
                },
                &mut quota,
            );
            frame.finish()
        };
        let a = render(20.0, false);
        assert_eq!(packets(&a), packets(&render(20.0, false)));
        assert_ne!(packets(&a), packets(&render(21.0, false)));
        assert_eq!(packets(&render(20.0, true)), packets(&render(901.0, true)));
        let mut frame = FrameBuilder::new();
        let mut quota = 6000;
        assert_eq!(
            render_gas(
                &mut frame,
                SceneInput {
                    time: f32::NAN,
                    ..input()
                },
                &mut quota
            ),
            0
        );
        assert_eq!(quota, 6000);
        assert_eq!(frame.frame().pixel_count, 0);
    }
}
