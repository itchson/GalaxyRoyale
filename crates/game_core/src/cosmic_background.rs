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
    fn new(
        width: usize,
        height: usize,
        sample: impl Fn(Vec2) -> Option<usize>,
        colors: &[u32],
    ) -> Self {
        // Both densities sample the same physical source field. Reduced detail
        // does not max-fill 2x2 blocks across the authored dark lanes.
        Self {
            full: template(width * 2, height * 2, 0.5, &sample, colors),
            low: template(width, height, 1.0, &sample, colors),
        }
    }
    fn body(&self, reduced: bool) -> &PixelBody {
        if reduced { &self.low } else { &self.full }
    }
}
struct Art {
    galaxy_disk: Pair,
    galaxy_arms: Pair,
    galaxy_core: Pair,
    nebula: Pair,
    nebula_thread: Pair,
    nova_shell: Pair,
    nova_core: Pair,
}

// These cells overlap as radially filtered volumes. The spacing is authored
// independently from each particle's footprint; it is never a raster minimum.
const VEIL_SPACING: f32 = 3.0;
const BILLOW_SPACING: f32 = 2.5;
struct GasArt {
    veil: Pair,
    billows: Pair,
}

fn noise(seed: u32, p: Vec2) -> f32 {
    let grid = p.floor().as_ivec2();
    let uv = p - p.floor();
    let u = smooth(uv.x);
    let v = smooth(uv.y);
    let a = unit(hash3(seed, grid.x, grid.y, 0));
    let b = unit(hash3(seed, grid.x + 1, grid.y, 0));
    let c = unit(hash3(seed, grid.x, grid.y + 1, 0));
    let d = unit(hash3(seed, grid.x + 1, grid.y + 1, 0));
    (a + (b - a) * u) + ((c + (d - c) * u) - (a + (b - a) * u)) * v
}

/// A fixed turbulent birth field gives the cloud unequal billows and isolated
/// dust pockets. Initialization samples it once at both densities.
fn gas_density(p: Vec2) -> f32 {
    let warp = Vec2::new(
        noise(0x7665_696c, p / 19.0) - 0.5,
        noise(0x6269_6c6c, p / 17.0) - 0.5,
    ) * 9.0;
    let q = p + warp;
    let billow = |center: Vec2, radii: Vec2| {
        smooth((1.0 - ((q - center) / radii).length_squared()).clamp(0.0, 1.0))
    };
    // Unequal overlapping volumes form one cloud with broad and narrow edges,
    // rather than two matching ribbons on either side of a continuous slit.
    let envelope = (billow(Vec2::new(-19.0, 1.0), Vec2::new(33.0, 25.0)) * 0.88
        + billow(Vec2::new(12.0, -7.0), Vec2::new(31.0, 20.0)) * 0.82
        + billow(Vec2::new(34.0, 12.0), Vec2::new(21.0, 17.0)) * 0.61
        + billow(Vec2::new(-39.0, -17.0), Vec2::new(17.0, 15.0)) * 0.46)
        .min(1.0);
    let turbulence = noise(0x6e65_6275, q / 12.0) * 0.58
        + noise(0x6c61_6e65, q / 5.5) * 0.29
        + noise(0x7269_6467, q / 2.4) * 0.13;
    let pocket = |center: Vec2, radii: Vec2, angle: f32| {
        let delta = q - center;
        let (s, c) = angle.sin_cos();
        let local = Vec2::new(c * delta.x - s * delta.y, s * delta.x + c * delta.y);
        let rough_edge = (noise(0x6475_7374, q / 3.4) - 0.5) * 0.25;
        smooth((((local / radii).length_squared() + rough_edge - 0.25) / 0.65).clamp(0.0, 1.0))
    };
    let transmission = pocket(Vec2::new(-17.0, 0.0), Vec2::new(15.0, 4.5), 0.35)
        .min(pocket(Vec2::new(17.0, -6.0), Vec2::new(10.0, 5.5), -0.6))
        .min(pocket(Vec2::new(-36.0, 13.0), Vec2::new(8.0, 7.0), -0.2));
    envelope * (0.35 + turbulence * 0.95) * transmission
}

fn gas_pair(width: usize, height: usize, spacing: f32, threshold: f32, colors: &[u32]) -> Pair {
    let sample = |p: Vec2| {
        let density = gas_density(p);
        (density > threshold).then(|| {
            // The upper-left illuminated ridge is part of the authored gas
            // material. It is not a foreground light or a new simulation event.
            let ridge =
                noise(0x6c69_7465, p / 7.0) * 0.3 + (0.5 - p.y / 80.0).clamp(0.0, 1.0) * 0.18;
            ((density * 0.72 + ridge) * (colors.len() - 1) as f32)
                .clamp(0.0, (colors.len() - 1) as f32) as usize
        })
    };
    Pair {
        full: template(width, height, spacing, &sample, colors),
        low: template(width / 2, height / 2, spacing * 2.0, &sample, colors),
    }
}

static GAS_ART: LazyLock<GasArt> = LazyLock::new(|| GasArt {
    veil: gas_pair(
        40,
        24,
        VEIL_SPACING,
        0.12,
        &[
            rgba(27, 24, 35, 43),
            rgba(35, 28, 44, 53),
            rgba(45, 34, 55, 64),
            rgba(57, 41, 65, 77),
            rgba(69, 48, 77, 88),
            rgba(82, 55, 88, 101),
            rgba(96, 66, 99, 113),
            rgba(113, 79, 110, 125),
        ],
    ),
    billows: gas_pair(
        48,
        32,
        BILLOW_SPACING,
        0.27,
        &[
            rgba(39, 28, 44, 60),
            rgba(49, 34, 51, 73),
            rgba(62, 41, 61, 86),
            rgba(77, 50, 69, 99),
            rgba(92, 59, 77, 112),
            rgba(107, 72, 86, 126),
            rgba(123, 87, 98, 138),
            rgba(141, 103, 111, 150),
        ],
    ),
});

/// Fixed two-unit birth clusters provide coherent ridges rather than independent
/// bright noise at every tiny cell. This only runs while initializing the cache.
fn cluster(seed: u32, p: Vec2) -> f32 {
    unit(hash3(
        seed,
        (p.x * 0.5).floor() as i32,
        (p.y * 0.5).floor() as i32,
        7,
    ))
}

// Smooth cache-only surface field. Fine cells form contiguous material clusters,
// rather than hard hash-block boundaries or independent point noise.
fn art_noise(seed: u32, p: Vec2) -> f32 {
    let q = p / 3.0;
    let cell = q.floor().as_ivec2();
    let f = q - q.floor();
    let x = smooth(f.x);
    let y = smooth(f.y);
    let n = |dx, dy| unit(hash3(seed, cell.x + dx, cell.y + dy, 7));
    let a = n(0, 0) + (n(1, 0) - n(0, 0)) * x;
    let b = n(0, 1) + (n(1, 1) - n(0, 1)) * x;
    a + (b - a) * y
}

fn spiral_arm(p: Vec2) -> Option<usize> {
    let r = p.length();
    if !(5.0..43.0).contains(&r) {
        return None;
    }
    let field = galactic_field(p);
    // Small correlated knots populate the disk, biased toward its broad arms.
    // There is no continuously traced arm edge or disconnected outer arc.
    let knot = art_noise(0x7370_6972, p * 2.1);
    let threshold = 0.90 - field.arm * 0.065 - field.density.min(0.7) * 0.065;
    if field.density < 0.05 || field.transmission < 0.42 || knot < threshold {
        return None;
    }
    Some(
        ((knot - threshold) * 28.0 + field.arm * 2.0 + field.density * 3.0).clamp(0.0, 6.0)
            as usize,
    )
}

fn galactic_bulge(p: Vec2) -> Option<usize> {
    let q = p - Vec2::new(0.4, -0.2);
    let r = Vec2::new(q.x * 0.9, q.y * 1.15).length();
    let light = (-(r / 4.2).powi(2)).exp() * (0.88 + art_noise(0x636f_7265, p) * 0.12);
    (light > 0.18).then(|| ((light - 0.18) * 8.0).min(6.0) as usize)
}

fn remnant_filament(p: Vec2) -> Option<usize> {
    let r = Vec2::new(p.x * 0.94, p.y * 1.1).length();
    let a = p.y.atan2(p.x);
    if a > -0.18 && a < 0.32 {
        return None;
    }
    let cloud = art_noise(0x656a_6563, p);
    let edge = 20.0 + (a * 2.0 + 0.5).sin() * 2.8 + (a * 3.0 - 1.0).sin() * 1.1;
    let d = (r - edge + (cloud - 0.5) * 2.0).abs();
    if d < 0.55 && cloud > 0.27 {
        Some(if cloud > 0.76 && a < 0.0 {
            7
        } else if cloud > 0.70 {
            6
        } else if cloud > 0.60 {
            5
        } else if cloud > 0.45 {
            4
        } else {
            3
        })
    } else if d < 2.3 && cloud > 0.70 {
        Some(if r < edge { 2 } else { 1 })
    } else if r < edge - 3.0 && r > edge - 4.1 && cloud > 0.72 && a < -1.0 {
        Some(2)
    } else if d < 3.4 && cloud > 0.87 {
        Some(0)
    } else {
        None
    }
}

fn remnant_source(p: Vec2) -> Option<usize> {
    let r = (p - Vec2::new(-1.2, 0.5)).length();
    let cloud = art_noise(0x7075_6c73, p);
    if r < 0.65 {
        Some(5)
    } else if r < 1.0 {
        Some(4)
    } else if r < 1.4 {
        Some(3)
    } else if r < 1.9 {
        Some(2)
    } else if r < 3.8 && cloud > 0.57 {
        Some(1)
    } else if r < 4.7 && cloud > 0.70 {
        Some(0)
    } else {
        None
    }
}

const GALAXY_DISK_SPACING: f32 = 2.5;
struct GalacticField {
    density: f32,
    arm: f32,
    transmission: f32,
}
fn galactic_field(p: Vec2) -> GalacticField {
    let r = p.length();
    let cloud = noise(0x6469_736b, p / 8.0);
    let phase = p.y.atan2(p.x) - 1.65 * ((r + 3.0) / 7.0).ln() + (cloud - 0.5) * 0.35;
    let cross = phase.sin() * r;
    let width = (5.5 + cloud * 2.5) * (1.0 - (r / 45.0).min(1.0) * 0.45);
    let arm = smooth((1.0 - cross.abs() / width).clamp(0.0, 1.0));
    // Dust winds off-center along the stellar arms. Its strength fades into
    // the warm bulge instead of dividing the entire galaxy into two leaves.
    let dust = (cross + (1.2 + cloud) * phase.cos()).abs();
    let lane = 1.0 - smooth((dust / (0.75 + cloud * 0.65)).clamp(0.0, 1.0));
    let lane_strength =
        smooth(((r - 6.0) / 10.0).clamp(0.0, 1.0)) * smooth(((43.0 - r) / 10.0).clamp(0.0, 1.0));
    let transmission = 1.0 - lane * lane_strength * 0.82;
    let boundary = 43.0 + (noise(0x6564_6765, p / 11.0) - 0.5) * 3.0;
    let edge = smooth(((boundary - r) / 15.0).clamp(0.0, 1.0));
    let population = 0.22 + (-(r / 25.0).powi(2)).exp() * 0.50;
    GalacticField {
        density: edge * population * (0.68 + cloud * 0.32) * (0.70 + arm * 0.30) * transmission,
        arm,
        transmission,
    }
}
fn galactic_disk_density(p: Vec2) -> f32 {
    galactic_field(p).density
}
fn galactic_disk() -> Pair {
    let sample = |p: Vec2| {
        let density = galactic_disk_density(p);
        (density > 0.05).then(|| (density * 8.0).min(5.0) as usize)
    };
    let colors = [
        rgba(62, 59, 70, 14),
        rgba(79, 73, 85, 21),
        rgba(98, 88, 102, 28),
        rgba(118, 104, 119, 35),
        rgba(140, 122, 136, 43),
        rgba(164, 145, 152, 51),
    ];
    Pair {
        full: template(36, 36, GALAXY_DISK_SPACING, &sample, &colors),
        low: template(18, 18, GALAXY_DISK_SPACING * 2.0, &sample, &colors),
    }
}

static ART: LazyLock<Art> = LazyLock::new(|| Art {
    galaxy_disk: galactic_disk(),
    galaxy_arms: Pair::new(
        96,
        96,
        spiral_arm,
        &[
            rgba(57, 54, 63, 18),
            rgba(73, 66, 80, 24),
            rgba(87, 77, 96, 29),
            rgba(103, 89, 110, 35),
            rgba(120, 105, 124, 41),
            rgba(143, 126, 145, 48),
            rgba(167, 151, 165, 56),
        ],
    ),
    galaxy_core: Pair::new(
        24,
        24,
        galactic_bulge,
        &[
            rgba(81, 65, 88, 14),
            rgba(113, 87, 110, 23),
            rgba(132, 105, 122, 29),
            rgba(153, 127, 133, 39),
            rgba(181, 157, 148, 50),
            rgba(202, 182, 162, 64),
            rgba(230, 211, 180, 87),
        ],
    ),
    nebula: Pair::new(
        80,
        48,
        |p| {
            let curve = (p.x * 0.11).sin() * 6.0 + (p.x * 0.032).cos() * 3.0;
            let side = p.y - curve;
            let distance = side.abs();
            let c = cluster(0x746f_726e, p);
            if p.x.abs() > 36.0 || distance > 8.0 {
                return None;
            }
            let lane = (p.x * 0.22).sin() * 0.4;
            if (side - lane).abs() < 0.6 {
                return None;
            }
            let ridge = (distance - (1.35 + (p.x * 0.17).cos() * 0.5)).abs();
            if ridge < 0.47 && c > 0.18 {
                Some(if side < 0.0 && c > 0.70 {
                    7
                } else if c > 0.55 {
                    6
                } else {
                    5
                })
            } else if distance < 4.6 && c > 0.65 {
                Some(if distance < 2.8 {
                    4
                } else if side < 0.0 {
                    3
                } else {
                    2
                })
            } else if distance < 7.5 && c > 0.94 {
                Some(if side < 0.0 { 1 } else { 0 })
            } else {
                None
            }
        },
        &[
            rgba(31, 28, 40, 14),
            rgba(39, 32, 48, 19),
            rgba(48, 39, 57, 25),
            rgba(58, 44, 69, 32),
            rgba(74, 52, 83, 43),
            rgba(90, 60, 100, 55),
            rgba(111, 74, 117, 69),
            rgba(132, 88, 135, 83),
        ],
    ),
    nebula_thread: Pair::new(
        48,
        24,
        |p| {
            let curve = (p.x * 0.15).sin() * 3.0;
            let d = (p.y - curve).abs();
            let c = cluster(0x7468_7264, p);
            if p.x.abs() >= 21.0 {
                return None;
            }
            if d < 0.42 && c > 0.18 {
                Some(if c > 0.78 {
                    5
                } else if c > 0.48 {
                    4
                } else {
                    3
                })
            } else if d < 1.5 && c > 0.84 {
                Some(if d < 0.8 {
                    2
                } else if p.y < curve {
                    1
                } else {
                    0
                })
            } else {
                None
            }
        },
        &[
            rgba(32, 36, 39, 12),
            rgba(41, 45, 49, 17),
            rgba(49, 55, 56, 23),
            rgba(62, 69, 69, 29),
            rgba(77, 85, 82, 37),
            rgba(96, 106, 99, 47),
        ],
    ),
    nova_shell: Pair::new(
        64,
        64,
        remnant_filament,
        &[
            rgba(49, 36, 45, 9),
            rgba(64, 43, 53, 13),
            rgba(79, 51, 62, 18),
            rgba(95, 63, 69, 23),
            rgba(110, 75, 77, 27),
            rgba(128, 88, 86, 31),
            rgba(144, 103, 94, 36),
            rgba(164, 119, 104, 42),
        ],
    ),
    nova_core: Pair::new(
        12,
        12,
        remnant_source,
        &[
            rgba(88, 64, 73, 20),
            rgba(117, 86, 91, 29),
            rgba(149, 112, 111, 40),
            rgba(178, 140, 130, 54),
            rgba(205, 172, 151, 69),
            rgba(230, 197, 162, 93),
        ],
    ),
});

fn template(
    width: usize,
    height: usize,
    cell_scale: f32,
    sample: &impl Fn(Vec2) -> Option<usize>,
    colors: &[u32],
) -> PixelBody {
    let rows: Vec<String> = (0..height)
        .map(|y| {
            (0..width)
                .map(|x| {
                    let p = Vec2::new(
                        x as f32 + 0.5 - width as f32 * 0.5,
                        y as f32 + 0.5 - height as f32 * 0.5,
                    ) * cell_scale;
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

#[derive(Clone, Copy)]
struct CellStyle {
    time: f32,
    phase: f32,
    cell_scale: f32,
    tint: [f32; 3],
    alpha: f32,
    curl: f32,
}

fn curled_local(local: Vec2, style: CellStyle) -> Vec2 {
    let physical = local * style.cell_scale;
    // A bounded, continuous field moves fine clusters by less than one full
    // source cell. Physical coordinates keep full/reduced motion coherent.
    let offset = Vec2::new(
        (physical.y * 0.13 + style.time * 0.07 + style.phase).sin(),
        (physical.x * 0.11 - style.time * 0.05 + style.phase).cos(),
    ) * style.curl;
    local + offset / style.cell_scale
}

fn draw_part(
    frame: &mut FrameBuilder,
    body: &PixelBody,
    transform: AffineTransform,
    style: CellStyle,
) {
    frame.push_cells_affine_material(
        transform,
        PixelMaterial::default(),
        body.cells().map(|(local, color)| {
            let physical = local * style.cell_scale;
            (
                curled_local(local, style),
                colored(
                    color,
                    style.tint,
                    0.98 + 0.02 * (style.time * 0.09 + physical.x * 0.1 + style.phase).sin(),
                    style.alpha,
                ),
            )
        }),
    );
}

fn gas_flow(physical: Vec2, time: f32, phase: f32, depth: usize) -> Vec2 {
    let t = time * if depth == 0 { 0.045 } else { -0.065 };
    let curl = Vec2::new(
        (physical.y * 0.09 + t + phase).sin() + (physical.x * 0.055 - t * 0.7 + phase).cos() * 0.28,
        (physical.x * 0.075 - t * 0.8 + phase).cos() + (physical.y * 0.12 + t * 0.6).sin() * 0.22,
    );
    curl * if depth == 0 { 1.15 } else { 0.72 }
}

fn draw_gas_part(
    frame: &mut FrameBuilder,
    body: &PixelBody,
    parent: AffineTransform,
    spacing: f32,
    style: CellStyle,
    depth: usize,
) {
    let footprint = spacing * if depth == 0 { 2.7 } else { 2.4 };
    let pose = Transform {
        position: if depth == 0 {
            Vec2::ZERO
        } else {
            Vec2::new(1.5, -1.0)
        },
        rotation: (style.time * 0.025 + style.phase).sin()
            * if depth == 0 { 0.018 } else { -0.012 },
        scale: Vec2::splat(footprint),
    };
    frame.push_soft_cells_affine_material(
        parent.compose(pose.affine()),
        PixelMaterial {
            light_response: 0.0,
            emission: if depth == 0 { 0.04 } else { 0.09 },
        },
        body.cells().map(|(local, color)| {
            let physical = local * spacing;
            let breathing =
                0.88 + 0.12 * (style.time * 0.075 + physical.x * 0.038 + style.phase).sin();
            (
                (physical + gas_flow(physical, style.time, style.phase, depth)) / footprint,
                colored(color, style.tint, 0.98, style.alpha * breathing),
            )
        }),
    );
}

fn motif_cost(kind: MotifKind, reduced: bool) -> usize {
    let parts = match kind {
        MotifKind::Galaxy => [&ART.galaxy_arms, &ART.galaxy_core],
        MotifKind::Nebula => [&ART.nebula, &ART.nebula_thread],
        MotifKind::Supernova => [&ART.nova_shell, &ART.nova_core],
    };
    let count: usize = parts.iter().map(|p| p.body(reduced).occupied_count()).sum();
    count
        + if kind == MotifKind::Galaxy {
            ART.galaxy_disk.body(reduced).occupied_count()
        } else {
            0
        }
        + if kind == MotifKind::Nebula {
            GAS_ART.veil.body(reduced).occupied_count()
                + GAS_ART.billows.body(reduced).occupied_count()
        } else {
            0
        }
}

fn motif_transforms(kind: MotifKind) -> usize {
    match kind {
        MotifKind::Galaxy => 3,
        MotifKind::Nebula => 4,
        MotifKind::Supernova => 2,
    }
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
    let cell_scale = if reduced { 1.0 } else { 0.5 };
    let (a, b, first, second, alpha) = match m.kind {
        MotifKind::Galaxy => (
            &ART.galaxy_arms,
            &ART.galaxy_core,
            Transform {
                scale: Vec2::new(cell_scale, cell_scale * 0.62),
                ..Transform::default()
            },
            Transform {
                scale: Vec2::new(cell_scale, cell_scale * 0.62),
                ..Transform::default()
            },
            0.95 + 0.05 * (time * 0.035 + phase).sin(),
        ),
        MotifKind::Nebula => (
            &ART.nebula,
            &ART.nebula_thread,
            Transform {
                rotation: (time * 0.04 + phase).sin() * 0.035,
                scale: Vec2::splat(cell_scale * 1.1),
                ..Transform::default()
            },
            Transform {
                position: Vec2::new(7.0, -10.0 + (time * 0.08 + phase).sin() * 1.4),
                rotation: -0.2,
                scale: Vec2::splat(cell_scale),
            },
            0.86 + 0.14 * (time * 0.10 + phase).sin(),
        ),
        MotifKind::Supernova => {
            let pulse = smooth(0.5 + 0.5 * (time * 0.075 + phase).sin());
            (
                &ART.nova_shell,
                &ART.nova_core,
                Transform {
                    scale: Vec2::splat(cell_scale * (0.99 + pulse * 0.02)),
                    ..Transform::default()
                },
                Transform {
                    scale: Vec2::splat(cell_scale * (0.99 + pulse * 0.02)),
                    ..Transform::default()
                },
                0.93 + 0.07 * pulse,
            )
        }
    };
    let a = a.body(reduced);
    let b = b.body(reduced);
    let required = motif_cost(m.kind, reduced);
    if required > *remaining {
        return 0;
    }
    let tint = Atmosphere::at(scene.seed, m.origin / MOTIF_PARALLAX).tint;
    let style = CellStyle {
        time,
        phase,
        cell_scale,
        tint,
        alpha,
        curl: match m.kind {
            MotifKind::Nebula => 0.14,
            MotifKind::Galaxy => 0.0,
            MotifKind::Supernova => 0.04,
        },
    };
    if m.kind == MotifKind::Galaxy {
        let spacing = GALAXY_DISK_SPACING * if reduced { 2.0 } else { 1.0 };
        let footprint = spacing * 2.4;
        let transform = parent.compose(
            Transform {
                scale: Vec2::new(footprint, footprint * 0.62),
                ..Transform::default()
            }
            .affine(),
        );
        frame.push_soft_cells_affine_material(
            transform,
            PixelMaterial::default(),
            ART.galaxy_disk
                .body(reduced)
                .cells()
                .map(|(p, c)| (p * (spacing / footprint), colored(c, tint, 0.98, alpha))),
        );
    }
    if m.kind == MotifKind::Nebula {
        let density_scale = if reduced { 2.0 } else { 1.0 };
        draw_gas_part(
            frame,
            GAS_ART.veil.body(reduced),
            parent,
            VEIL_SPACING * density_scale,
            style,
            0,
        );
        draw_gas_part(
            frame,
            GAS_ART.billows.body(reduced),
            parent,
            BILLOW_SPACING * density_scale,
            style,
            1,
        );
    }
    draw_part(frame, a, parent.compose(first.affine()), style);
    draw_part(
        frame,
        b,
        parent.compose(second.affine()),
        CellStyle {
            curl: if m.kind == MotifKind::Galaxy {
                0.0
            } else {
                0.07
            },
            ..style
        },
    );
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
    let time = if scene.reduced_motion {
        0.0
    } else {
        scene.time
    };
    let brightness = 0.94 + 0.06 * (time * 0.13 + unit(h) * TAU).sin();
    // A physical neighborhood is seven stars, not chart strokes in the universe.
    if remaining < nodes.len() {
        return 0;
    }
    let parent = origin + scene.camera * (1.0 - SYSTEM_PARALLAX);
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
    nodes.len()
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
                    stats.transforms += motif_transforms(m.kind);
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
                stats.transforms += usize::from(count > 0);
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
            let expected = motif_cost(MotifKind::Nebula, reduced);
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
                assert_eq!(frame.frame().batches[0].transforms.len(), 4);
                assert!(
                    frame.frame().batches[0]
                        .transforms
                        .iter()
                        .all(|t| t.b[2] == 0.0
                            && ((t.b[3] == 0.0 || t.b[3] == -1.0)
                                || (t.b[3] + 1.04).abs() < 0.00001
                                || (t.b[3] + 1.09).abs() < 0.00001))
                );
                let batch = &frame.frame().batches[0];
                let submitted: Vec<_> = batch
                    .pixels
                    .iter()
                    .filter(|p| batch.transforms[p.transform as usize].b[3] == 0.0)
                    .collect();
                let source: Vec<_> = a.cells().chain(b.cells()).collect();
                let cell_scale = if reduced { 1.0 } else { 0.5 };
                assert!(submitted.iter().zip(source).all(|(pixel, (local, _))| {
                    (Vec2::from_array(pixel.local) - local).abs().max_element() * cell_scale
                        <= 0.141
                }));
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
                assert!(min_alpha >= 8 && (59..=83).contains(&max_alpha));
                eprintln!(
                    "Actual nebula reduced={reduced} time={time}: {expected} cells/4 transforms, hard filament alpha {min_alpha}..{max_alpha}"
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
                            .all(|t| t.b[2] == 0.0
                                && ((t.b[3] == 0.0 || t.b[3] == -1.0)
                                    || (t.b[3] + 1.04).abs() < 0.00001
                                    || (t.b[3] + 1.09).abs() < 0.00001))
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
                        assert!(width.length() > 0.0 && width.length() < 4.3);
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
    fn stellar_neighborhoods_keep_exact_seeded_stars_without_chart_strokes_under_quality_and_parallax()
     {
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
            assert_eq!(required, 7);
            assert_eq!(frame.frame().pixel_count, required);
            assert_eq!(
                frame
                    .frame()
                    .batches
                    .iter()
                    .map(|b| b.transforms.len())
                    .sum::<usize>(),
                1
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

#[cfg(test)]
#[path = "cosmic_detail_tests.rs"]
mod detail_tests;

#[cfg(test)]
#[path = "deep_nebula_tests.rs"]
mod deep_nebula_tests;
