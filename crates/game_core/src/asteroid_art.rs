//! Seeded, solid asteroid geology. Generated once at birth, then damaged in ECS.
//! The fixed palette table is cached; instance bodies never enter a growing cache.
use engine_core::{PixelBody, Vec2};
use std::{collections::VecDeque, f32::consts::TAU, sync::LazyLock};

pub(super) const GRID: usize = 64;
/// A 64-cell canvas occupies the same nominal 40-unit local square as the old art.
pub(super) const CELL_SCALE: f32 = 40.0 / GRID as f32;
pub(super) const MAX_SIZE: f32 = 1.10;
pub(super) const MAX_CELLS: usize = 3_000;

const BASE: [[[u8; 3]; 8]; 4] = [
    [
        [32, 25, 24],
        [53, 40, 32],
        [76, 57, 43],
        [99, 76, 54],
        [127, 101, 71],
        [154, 129, 94],
        [187, 162, 122],
        [218, 198, 158],
    ],
    [
        [21, 30, 39],
        [35, 47, 57],
        [51, 66, 75],
        [73, 91, 99],
        [97, 117, 123],
        [129, 149, 151],
        [167, 184, 178],
        [209, 220, 204],
    ],
    [
        [17, 34, 50],
        [27, 55, 73],
        [40, 79, 100],
        [58, 106, 126],
        [81, 138, 155],
        [116, 173, 182],
        [162, 205, 208],
        [213, 233, 223],
    ],
    [
        [31, 23, 34],
        [48, 35, 47],
        [69, 50, 60],
        [91, 68, 74],
        [115, 87, 88],
        [144, 111, 101],
        [174, 139, 118],
        [205, 171, 144],
    ],
];
const MINERAL: [[u8; 3]; 4] = [
    [120, 141, 157],
    [183, 159, 107],
    [139, 213, 216],
    [171, 116, 84],
];
static PALETTES: LazyLock<[[u32; 12]; 20]> = LazyLock::new(|| {
    std::array::from_fn(|i| {
        let family = i / 5;
        let tier = (i % 5) as f32;
        std::array::from_fn(|shade| {
            let color = if shade < 8 {
                BASE[family][shade]
            } else {
                MINERAL[family]
            };
            let highlight = if shade < 8 {
                1.0
            } else {
                [0.48, 0.72, 0.93, 1.12][shade - 8]
            };
            let grade = match family {
                0 => [tier * 1.2, tier * 0.4, tier * 2.0],
                1 => [tier * 1.6, tier * 1.0, -tier * 0.8],
                2 => [-tier * 0.4, tier * 0.5, tier * 1.8],
                _ => [tier * 2.1, -tier * 0.4, tier * 0.8],
            };
            u32::from_le_bytes([
                (color[0] as f32 * highlight + grade[0])
                    .round()
                    .clamp(0.0, 240.0) as u8,
                (color[1] as f32 * highlight + grade[1])
                    .round()
                    .clamp(0.0, 240.0) as u8,
                (color[2] as f32 * highlight + grade[2])
                    .round()
                    .clamp(0.0, 240.0) as u8,
                255,
            ])
        })
    })
});

fn sample(seed: u32, salt: u32) -> f32 {
    crate::unit(crate::mix(seed ^ salt))
}
pub(super) fn variety(seed: u32) -> u8 {
    ((seed >> 5) & 3) as u8
}

/// Physical-size factor, separate from the finer cell's actor transform scale.
pub(super) fn size(seed: u32, tier: u8, belt: bool) -> f32 {
    let tier = tier.clamp(1, 5) as f32 - 1.0;
    let low = if belt { 0.60 } else { 0.55 };
    (low + tier * 0.025 + sample(seed, 43) * 0.40).min(MAX_SIZE)
}
/// Belts retain gentle orbital response; outer fields may drift slightly faster.
pub(super) fn speed(seed: u32, tier: u8, belt: bool) -> f32 {
    let tier = tier.clamp(1, 5) as f32 - 1.0;
    if belt {
        6.0 + sample(seed, 123) * 6.0 + tier * 0.5
    } else {
        3.0 + sample(seed, 123) * 9.0 + tier * 0.8
    }
}

#[derive(Clone, Copy)]
struct Crater {
    center: Vec2,
    radius: f32,
}
#[derive(Clone, Copy)]
struct Mineral {
    center: Vec2,
    angle: f32,
    radii: Vec2,
}
struct Geology {
    center: Vec2,
    angle: f32,
    stretch: Vec2,
    planes: [(Vec2, f32); 10],
    plane_count: usize,
    craters: [Crater; 5],
    minerals: [Mineral; 4],
    facet_phase: f32,
}
impl Geology {
    fn at(seed: u32, family: usize) -> Self {
        let center =
            Vec2::new(sample(seed, 0x6365_6e78), sample(seed, 0x6365_6e79)) * 2.0 - Vec2::ONE;
        let angle = sample(seed, 0x7368_6170) * TAU;
        let axis = 0.84 + sample(seed, 0x6178_6973) * 0.20;
        let stretch = match family {
            1 => Vec2::new(1.0, axis * 0.88),
            2 => Vec2::new(axis * 0.77, 1.0),
            _ => Vec2::new(1.0, axis),
        };
        let plane_count = [10, 8, 6, 10][family];
        let radius = 24.0 + sample(seed, 0x7261_6469) * 3.0;
        let planes = std::array::from_fn(|i| {
            let theta = i as f32 * TAU / plane_count as f32;
            let normal = Vec2::new(theta.cos(), theta.sin());
            let distance = radius * (0.85 + sample(seed, 0xface_0000 + i as u32 * 131) * 0.14);
            (normal, distance)
        });
        let craters = std::array::from_fn(|i| {
            let salt = 0x6372_0000 + i as u32 * 197;
            let theta = sample(seed, salt) * TAU;
            let reach = 5.0 + sample(seed, salt ^ 71) * 9.0;
            Crater {
                center: Vec2::new(theta.cos(), theta.sin()) * reach,
                radius: 2.5 + sample(seed, salt ^ 123) * if family == 3 { 4.5 } else { 3.5 },
            }
        });
        // Separated compact deposits: no long streak crosses the rock surface.
        let minerals = std::array::from_fn(|i| {
            let salt = 0x6d69_0000 + i as u32 * 211;
            let theta = i as f32 * TAU / 4.0 + (sample(seed, salt) - 0.5) * 0.24;
            let reach = 9.0 + sample(seed, salt ^ 79) * 3.0;
            Mineral {
                center: Vec2::new(theta.cos(), theta.sin()) * reach,
                angle: sample(seed, salt ^ 113) * TAU,
                radii: Vec2::new(
                    2.3 + sample(seed, salt ^ 53) * 0.8,
                    2.2 + sample(seed, salt ^ 179) * 0.8,
                ),
            }
        });
        Self {
            center,
            angle,
            stretch,
            planes,
            plane_count,
            craters,
            minerals,
            facet_phase: sample(seed, 0x6661_6365) * TAU,
        }
    }
    fn local(&self, p: Vec2) -> Vec2 {
        let p = p - self.center;
        let (s, c) = self.angle.sin_cos();
        Vec2::new(c * p.x + s * p.y, -s * p.x + c * p.y) / self.stretch
    }
    fn occupied(&self, p: Vec2) -> bool {
        let p = self.local(p);
        self.planes[..self.plane_count]
            .iter()
            .all(|(n, d)| p.dot(*n) < *d)
    }
    fn mineral_shade(&self, q: Vec2) -> Option<usize> {
        self.minerals.iter().find_map(|mineral| {
            let d = q - mineral.center;
            let (s, c) = mineral.angle.sin_cos();
            let p = Vec2::new(c * d.x + s * d.y, -s * d.x + c * d.y) / mineral.radii;
            // One beveled irregular nodule with a lit facet and dark outer lip.
            if p.length_squared() < 1.0 && p.x + p.y * 0.6 < 0.95 && p.y - p.x * 0.4 < 0.92 {
                Some(if p.length_squared() > 0.64 {
                    8
                } else if d.x + d.y < -0.7 {
                    11
                } else if d.x + d.y < 0.7 {
                    10
                } else {
                    9
                })
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
pub(super) fn mineral_colors(seed: u32, tier: u8) -> [u32; 4] {
    let palette = PALETTES[variety(seed) as usize * 5 + tier.clamp(1, 5) as usize - 1];
    palette[8..].try_into().unwrap()
}

/// The central component removes only rasterized diagonal-only tip fragments.
/// Convex planes and solid row spans do not author transparent interior pits.
fn solid_mask(geology: &Geology) -> [bool; GRID * GRID] {
    let occupied = std::array::from_fn::<_, { GRID * GRID }, _>(|i| {
        geology.occupied(Vec2::new(
            (i % GRID) as f32 - 31.5,
            (i / GRID) as f32 - 31.5,
        ))
    });
    let mut mask = [false; GRID * GRID];
    let start = 32 * GRID + 32;
    debug_assert!(occupied[start]);
    mask[start] = true;
    let mut queue = VecDeque::from([start]);
    while let Some(i) = queue.pop_front() {
        let x = i % GRID;
        let y = i / GRID;
        for (nx, ny) in [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ] {
            if nx < GRID && ny < GRID {
                let n = ny * GRID + nx;
                if occupied[n] && !mask[n] {
                    mask[n] = true;
                    queue.push_back(n);
                }
            }
        }
    }
    mask
}

pub(super) fn body(seed: u32, tier: u8) -> PixelBody {
    let family = variety(seed) as usize;
    let tier = tier.clamp(1, 5);
    let geology = Geology::at(seed, family);
    let mask = solid_mask(&geology);
    let colors = PALETTES[family * 5 + tier as usize - 1];
    let palette: [_; 12] = std::array::from_fn(|i| ((b'a' + i as u8) as char, colors[i]));
    let mut rows = Vec::with_capacity(GRID);
    for y in 0..GRID {
        let mut row = String::with_capacity(GRID);
        for x in 0..GRID {
            if !mask[y * GRID + x] {
                row.push(' ');
                continue;
            }
            let p = Vec2::new(x as f32 - 31.5, y as f32 - 31.5);
            let q = geology.local(p);
            let edge = [
                (x.wrapping_sub(1), y),
                (x + 1, y),
                (x, y.wrapping_sub(1)),
                (x, y + 1),
            ]
            .iter()
            .any(|&(nx, ny)| nx >= GRID || ny >= GRID || !mask[ny * GRID + nx]);
            // Broad planes share one light direction; fine strata form clusters.
            let light = (-p.x * 0.65 - p.y * 0.75) / 25.0;
            let mut shade = (3.4 + light * 2.0).round().clamp(1.0, 6.0) as usize;
            let stratum = (q.x * 0.22 + q.y * 0.13 + geology.facet_phase).sin();
            if stratum > 0.50 {
                shade = (shade + 1).min(6);
            }
            if stratum < -0.70 {
                shade = shade.saturating_sub(1).max(1);
            }
            let crater_count = if family == 2 {
                2
            } else {
                3 + usize::from(tier >= 3) + usize::from(family == 3)
            };
            for crater in &geology.craters[..crater_count] {
                let d = q - crater.center;
                let r = d.length();
                if r < crater.radius {
                    shade = if r > crater.radius - 1.25 && d.x + d.y < 0.0 {
                        6
                    } else if r < crater.radius * 0.55 {
                        0
                    } else {
                        1
                    };
                } else if r < crater.radius + 0.9 && d.x + d.y > 0.0 {
                    shade = 4;
                }
            }
            if !edge && let Some(mineral) = geology.mineral_shade(q) {
                shade = mineral;
            }
            if edge {
                shade = if p.x + p.y < 0.0 { 6 } else { 0 };
            }
            row.push((b'a' + shade as u8) as char);
        }
        rows.push(row);
    }
    let borrowed: Vec<_> = rows.iter().map(String::as_str).collect();
    let body = PixelBody::from_ascii(&borrowed, &palette);
    // Fail a changed authoring rule explicitly instead of silently truncating
    // live collision cells or admitting an unexpectedly expensive body.
    assert!(body.occupied_count() <= MAX_CELLS);
    body
}
