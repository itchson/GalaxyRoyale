//! Deterministic region network, discovery sites and bounded cosmetic scenery.
use engine_core::{FrameBuilder, PixelBody, Transform, Vec2};
use serde::Serialize;
use std::f32::consts::TAU;
use std::sync::LazyLock;

pub const REGION_SIZE: f32 = 4200.0;
pub const COSMETIC_PIXEL_BUDGET: usize = 6000;
const HUB_RADIUS: f32 = 1400.0;

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub struct Biome {
    pub kind: u8,
    pub name: &'static str,
    pub grid: [i32; 2],
    /// Dark atmosphere, shadow, body color, highlight (packed RGBA).
    pub palette: [u32; 4],
    /// Relative Dart / Bastion / Coil spawn weights; not an extra spawn system.
    pub spawn_weights: [u8; 3],
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Site {
    pub id: String,
    pub name: &'static str,
    pub event: &'static str,
    pub grid: [i32; 2],
    pub position: [f32; 2],
    pub radius: f32,
    pub family: usize,
    pub target_kills: u32,
}

impl Site {
    /// Lets gameplay reuse its nearby-site cache during projectile safety checks.
    pub fn intersects_segment(&self, from: Vec2, to: Vec2) -> bool {
        if !from.is_finite() || !to.is_finite() {
            return false;
        }
        let delta = to - from;
        let center = Vec2::from_array(self.position);
        let denominator = delta.length_squared();
        if !denominator.is_finite() {
            return false;
        }
        let t = if denominator > 0.0 {
            ((center - from).dot(delta) / denominator).clamp(0.0, 1.0)
        } else {
            0.0
        };
        (from + delta * t).distance_squared(center) <= self.radius * self.radius
    }
}

const NAMES: [&str; 6] = [
    "Haven Expanse",
    "Amethyst Drift",
    "Aurora Reach",
    "Iron Belt",
    "Radial Wastes",
    "Cryo Rift",
];
const PALETTES: [[u32; 4]; 6] = [
    [0xff_382018, 0xff_65422d, 0xff_a9764d, 0xff_f1db9a],
    [0xff_381b2b, 0xff_733b65, 0xff_bb67a5, 0xff_efd0ed],
    [0xff_312e12, 0xff_62622b, 0xff_8dba52, 0xff_d2f3bb],
    [0xff_242232, 0xff_51415d, 0xff_797eac, 0xff_bce0f3],
    [0xff_211735, 0xff_36286c, 0xff_6167c3, 0xff_95c3ff],
    [0xff_3c291e, 0xff_79533c, 0xff_c79666, 0xff_fae8b1],
];
const WEIGHTS: [[u8; 3]; 6] = [
    [4, 3, 3],
    [2, 3, 6],
    [5, 2, 4],
    [2, 7, 2],
    [7, 3, 2],
    [3, 3, 6],
];

fn mix(mut n: u32) -> u32 {
    n ^= n >> 16;
    n = n.wrapping_mul(0x7feb_352d);
    n ^= n >> 15;
    n = n.wrapping_mul(0x846c_a68b);
    n ^ (n >> 16)
}
fn hash(seed: u32, grid: [i32; 2], salt: u32) -> u32 {
    mix(seed
        ^ (grid[0] as u32).wrapping_mul(0x9e37_79b9)
        ^ (grid[1] as u32).wrapping_mul(0x85eb_ca6b)
        ^ salt)
}
fn unit(n: u32) -> f32 {
    (n >> 8) as f32 / 16_777_216.0
}
fn bounded(position: Vec2) -> Vec2 {
    if position.is_finite() {
        position.clamp(Vec2::splat(-1_000_000.0), Vec2::splat(1_000_000.0))
    } else {
        Vec2::ZERO
    }
}
fn tile(position: Vec2, size: f32) -> [i32; 2] {
    let p = (bounded(position) / size).floor();
    [p.x as i32, p.y as i32]
}
fn center(seed: u32, grid: [i32; 2]) -> Vec2 {
    let jitter = Vec2::new(unit(hash(seed, grid, 41)), unit(hash(seed, grid, 97))) * 0.44
        + Vec2::splat(0.28);
    (Vec2::new(grid[0] as f32, grid[1] as f32) + jitter) * REGION_SIZE
}

pub fn at(seed: u32, position: Vec2) -> Biome {
    let position = bounded(position);
    let base = tile(position, REGION_SIZE);
    let mut closest = base;
    let mut distance = f32::INFINITY;
    for y in -1..=1 {
        for x in -1..=1 {
            let candidate = [base[0] + x, base[1] + y];
            let next = center(seed, candidate).distance_squared(position);
            if next < distance {
                distance = next;
                closest = candidate;
            }
        }
    }
    let kind = if position.length_squared() <= HUB_RADIUS * HUB_RADIUS {
        0
    } else {
        (hash(seed, closest, 0x6269_6f6d) % 6) as u8
    };
    Biome {
        kind,
        name: NAMES[kind as usize],
        grid: closest,
        palette: PALETTES[kind as usize],
        spawn_weights: WEIGHTS[kind as usize],
    }
}

fn site(seed: u32, grid: [i32; 2]) -> Option<Site> {
    let position = center(seed, grid);
    let h = hash(seed, grid, 0x7369_7465);
    let radius = 160.0 + unit(mix(h)) * 70.0;
    // Leave the entire initial combat/belt hub corridor unoccupied by remote sites.
    if position.length() < HUB_RADIUS + radius + 120.0
        || position.abs().max_element() + radius > 1_000_000.0
    {
        return None;
    }
    let courier = h & 1 == 0;
    let names = if courier {
        ["Relay Anchorage", "Wayfarer Station", "Signal Sanctuary"]
    } else {
        ["Sentinel Refuge", "Outrider Beacon", "Frontier Bastion"]
    };
    Some(Site {
        id: format!("{}:{}", grid[0], grid[1]),
        name: names[(mix(h ^ 11) % 3) as usize],
        event: if courier { "courier" } else { "bounty" },
        grid,
        position: position.to_array(),
        radius,
        family: (mix(h ^ 37) % 3) as usize,
        target_kills: 8 + mix(h ^ 71) % 5,
    })
}

/// Only the adjacent 3×3 network is materialized; IDs do not depend on query order.
pub fn sites_near(seed: u32, position: Vec2) -> Vec<Site> {
    let base = tile(position, REGION_SIZE);
    let mut sites = Vec::with_capacity(9);
    for y in -1..=1 {
        for x in -1..=1 {
            if let Some(site) = site(seed, [base[0] + x, base[1] + y]) {
                sites.push(site);
            }
        }
    }
    sites
}

pub fn protected(seed: u32, position: Vec2) -> Option<Site> {
    if !position.is_finite() {
        return None;
    }
    sites_near(seed, position).into_iter().find(|site| {
        Vec2::from_array(site.position).distance_squared(position) <= site.radius * site.radius
    })
}

/// Canonical IDs support persisted discovery validation without trusting save data.
pub fn site_by_id(seed: u32, id: &str) -> Option<Site> {
    let (x, y) = id.split_once(':')?;
    let grid = [x.parse::<i32>().ok()?, y.parse::<i32>().ok()?];
    if grid.iter().any(|v| !(-239..=239).contains(v)) || id != format!("{}:{}", grid[0], grid[1]) {
        return None;
    }
    site(seed, grid)
}

/// Short gameplay segments query one neighborhood. Longer segments sample a
/// bounded sequence of neighborhoods, then use exact circle/segment overlap.
pub fn segment_protected(seed: u32, from: Vec2, to: Vec2) -> bool {
    if !from.is_finite() || !to.is_finite() {
        return false;
    }
    let from = bounded(from);
    let to = bounded(to);
    let steps = (from.distance(to) / REGION_SIZE).ceil().max(1.0) as usize;
    (0..steps).any(|i| {
        let midpoint = from.lerp(to, (i as f32 + 0.5) / steps as f32);
        sites_near(seed, midpoint)
            .iter()
            .any(|site| site.intersects_segment(from, to))
    })
}

// Small local-cell templates are generated once, rather than a huge background bitmap.
static PLANETS: LazyLock<[PixelBody; 4]> = LazyLock::new(|| std::array::from_fn(planet));
static RINGS: LazyLock<[PixelBody; 2]> = LazyLock::new(|| {
    std::array::from_fn(|front| {
        template(128, 48, |x, y| {
            if (y >= 0.0) != (front == 1) {
                return None;
            }
            let radius = (x * x / (58.0 * 58.0) + y * y / (18.0 * 18.0)).sqrt();
            if !(0.92..1.015).contains(&radius) {
                return None;
            }
            let band = ((radius - 0.92) * 95.0).floor() as i32;
            Some(if x > 17.0 && y < 5.0 {
                'a'
            } else if band % 3 == 0 {
                'd'
            } else if band % 3 == 1 {
                'f'
            } else {
                'b'
            })
        })
    })
});
static NEBULA: LazyLock<PixelBody> = LazyLock::new(|| {
    template(32, 20, |x, y| {
        let wave = (x * 0.29).sin() * 3.0;
        if (y - wave).abs() < 4.0 && x.abs() < 14.0 && ((x + y) as i32).rem_euclid(3) != 0 {
            Some(if y > wave { 'b' } else { 'c' })
        } else {
            None
        }
    })
});
fn template(width: usize, height: usize, sample: impl Fn(f32, f32) -> Option<char>) -> PixelBody {
    let rows: Vec<String> = (0..height)
        .map(|y| {
            (0..width)
                .map(|x| {
                    sample(
                        x as f32 + 0.5 - width as f32 * 0.5,
                        y as f32 + 0.5 - height as f32 * 0.5,
                    )
                    .unwrap_or(' ')
                })
                .collect()
        })
        .collect();
    let borrowed: Vec<_> = rows.iter().map(String::as_str).collect();
    PixelBody::from_ascii(
        &borrowed,
        &[
            ('a', 0),
            ('b', 1),
            ('c', 2),
            ('d', 3),
            ('e', 4),
            ('f', 5),
            ('g', 6),
            ('h', 7),
        ],
    )
}
fn planet(kind: usize) -> PixelBody {
    const RADIUS: f32 = 37.5;
    let craters: Vec<_> = (0..23)
        .map(|index| {
            let h = hash(0x6372_6174, [kind as i32, index], 71);
            let angle = unit(h) * TAU;
            let distance = unit(mix(h ^ 17)).sqrt() * 31.0;
            (
                Vec2::new(angle.cos(), angle.sin()) * distance,
                1.5 + unit(mix(h ^ 43)) * 6.0,
            )
        })
        .collect();
    template(96, 96, |x, y| {
        let distance_squared = x * x + y * y;
        if distance_squared > RADIUS * RADIUS {
            return None;
        }
        let z = (RADIUS * RADIUS - distance_squared).sqrt();
        let illumination = ((-0.52 * x - 0.32 * y + 0.79 * z) / RADIUS).clamp(0.0, 1.0);
        let mut tone = (illumination * 7.5).min(7.0) as usize;
        match kind {
            0 => {
                for (center, radius) in &craters {
                    let delta = Vec2::new(x, y) - *center;
                    let distance = delta.length();
                    if distance < *radius - 0.6 {
                        tone = tone.saturating_sub(if delta.x + delta.y < 0.0 { 3 } else { 1 });
                    } else if (distance - radius).abs() < 0.8 {
                        tone = if delta.x + delta.y < 0.0 {
                            (tone + 2).min(7)
                        } else {
                            tone.saturating_sub(2)
                        };
                    }
                }
            }
            1 => {
                let latitude = y + (x * 0.09).sin() * 2.3 + (z / RADIUS) * 3.0;
                let band = (latitude / 3.1).floor() as i32;
                if band.rem_euclid(5) == 0 {
                    tone = tone.saturating_sub(2);
                }
                if band.rem_euclid(5) == 2 {
                    tone = (tone + 1).min(7);
                }
                let storm = ((x + 11.0) / 7.0).powi(2) + ((y - 6.0) / 3.4).powi(2);
                if (0.5..1.3).contains(&storm) {
                    tone = tone.saturating_sub(3);
                }
                if storm < 0.5 {
                    tone = (tone + 1).min(7);
                }
            }
            2 => {
                let vein = (x * 0.11 + y * 0.075).sin() * 8.0 + y * 0.31;
                if (x - vein - 5.0).abs() < 0.9
                    || (y + x * 0.42 - (x * 0.17).sin() * 2.0).abs() < 0.7
                {
                    tone = if illumination > 0.3 { 7 } else { 2 };
                }
                let plate = hash(91, [(x / 8.0).floor() as i32, (y / 7.0).floor() as i32], 9);
                if plate & 3 == 0 {
                    tone = tone.saturating_sub(1);
                }
            }
            _ => {
                let filament = (x * 0.24 + y * 0.17).sin() + (y * 0.33 - x * 0.1).cos();
                tone = if filament > 0.75 {
                    7
                } else if filament < -0.6 {
                    3
                } else {
                    5
                };
                if distance_squared > 34.0 * 34.0 {
                    tone = tone.saturating_sub(2);
                }
            }
        }
        Some(['a', 'e', 'b', 'f', 'c', 'g', 'd', 'h'][tone])
    })
}

fn palette_shade(palette: [u32; 4], shade: u32, alpha: u8) -> u32 {
    let color = if shade < 4 {
        palette[shade as usize]
    } else {
        let (a, b) = match shade {
            4 => (0, 1),
            5 => (1, 2),
            6 => (2, 3),
            _ => (3, 3),
        };
        let left = palette[a].to_le_bytes();
        let right = palette[b].to_le_bytes();
        u32::from_le_bytes(std::array::from_fn(|channel| {
            if channel == 3 {
                255
            } else if shade == 7 {
                ((left[channel] as u16 * 3 + 255) / 4) as u8
            } else {
                ((left[channel] as u16 + right[channel] as u16) / 2) as u8
            }
        }))
    };
    (color & 0x00ff_ffff) | ((alpha as u32) << 24)
}

struct Scenery<'a> {
    frame: &'a mut FrameBuilder,
    camera: Vec2,
    half: Vec2,
    remaining: usize,
}
impl Scenery<'_> {
    fn visible(&self, position: Vec2, radius: f32) -> bool {
        let d = (position - self.camera).abs();
        d.x <= self.half.x + radius && d.y <= self.half.y + radius
    }
    fn body(&mut self, body: &PixelBody, transform: Transform, palette: [u32; 4], alpha: u8) {
        if body.occupied_count() > self.remaining {
            return;
        }
        self.frame.push_body_colored(body, transform, |_, color| {
            palette_shade(palette, color, alpha)
        });
        self.remaining -= body.occupied_count();
    }
    fn pixel(&mut self, p: Vec2, size: f32, color: u32) {
        if self.remaining > 0 && self.visible(p, size) {
            self.frame.push_pixel(p, size, color);
            self.remaining -= 1;
        }
    }
}

/// Stable analytic gas cells: seed/grid identifies a cell for the entire session;
/// time only moves its curl and opacity. No random respawns or simulation bodies.
fn gas_wisps(scene: &mut Scenery<'_>, seed: u32, time: f32, reduced: bool) -> usize {
    if scene.remaining == 0 {
        return 0;
    }
    let limit = if reduced { 400 } else { 800 };
    let mut emitted = 0;
    for (layer, factor) in [(0_u32, 0.16), (1, 0.36)] {
        let sample = scene.camera * factor;
        let min = tile(sample - scene.half - Vec2::splat(100.0), 128.0);
        let max = tile(sample + scene.half + Vec2::splat(100.0), 128.0);
        for y in min[1]..=max[1] {
            for x in min[0]..=max[0] {
                let grid = [x, y];
                let field = hash(seed, grid, 0x6761_7300 + layer);
                let angle = unit(mix(field ^ 19)) * TAU;
                let direction = Vec2::new(angle.cos(), angle.sin());
                let side = Vec2::new(-direction.y, direction.x);
                let origin = (Vec2::new(x as f32, y as f32)
                    + Vec2::new(unit(field), unit(mix(field))))
                    * 128.0;
                let violet = field & 1 == 0;
                for index in 0..48 {
                    if reduced && index % 2 == 1 {
                        continue;
                    }
                    let h = hash(field, grid, index * 7919 + 37);
                    let along = unit(h) * 2.0 - 1.0;
                    let phase = time * 0.09 + unit(mix(h ^ 11)) * TAU;
                    let drift =
                        direction * (phase.sin() * 9.0) + side * ((phase * 0.73).cos() * 3.0);
                    let curl = (along * 4.2 + time * 0.065 + unit(field) * TAU).sin() * 10.0;
                    let jitter = (unit(mix(h ^ 47)) - 0.5) * 10.0;
                    let position = origin
                        + direction * along * 87.0
                        + side * (curl + jitter)
                        + drift
                        + scene.camera * (1.0 - factor);
                    let size = [0.65, 0.9, 1.15, 1.4][(h & 3) as usize];
                    if !scene.visible(position, size) {
                        continue;
                    }
                    let softness = (1.0 - along.abs() * 0.6) * (0.84 + 0.16 * phase.sin());
                    let alpha = ((25.0 + unit(mix(h ^ 71)) * 47.0) * softness) as u32;
                    let shimmer = (time * 0.06 + along * 2.0 + unit(field) * TAU).sin() * 7.0;
                    let color = if violet {
                        [(151.0 + shimmer) as u8, (90.0 - shimmer * 0.5) as u8, 183]
                    } else {
                        [57, (147.0 + shimmer * 0.5) as u8, (177.0 + shimmer) as u8]
                    };
                    let packed = u32::from_le_bytes([color[0], color[1], color[2], alpha as u8]);
                    scene.pixel(position, size, packed);
                    emitted += 1;
                    if emitted >= limit || scene.remaining == 0 {
                        return emitted;
                    }
                }
            }
        }
    }
    emitted
}

/// Pure seeded parallax scenery. Adds at most 6,000 quads, even for oversized viewports.
/// Radiation stars/supernovae are cosmetic; gameplay hazards require explicit rules.
pub fn background(frame: &mut FrameBuilder, seed: u32, camera: Vec2, viewport: Vec2, time: f32) {
    background_with_density(frame, seed, camera, viewport, time, false);
}

/// Reduced density selects a stable half of fields, preserving their positions.
pub fn background_with_density(
    frame: &mut FrameBuilder,
    seed: u32,
    camera: Vec2,
    viewport: Vec2,
    time: f32,
    reduced: bool,
) {
    if !camera.is_finite()
        || !viewport.is_finite()
        || viewport.min_element() <= 0.0
        || !time.is_finite()
    {
        return;
    }
    let camera = bounded(camera);
    let viewport = viewport.min(Vec2::splat(8192.0));
    let mut scene = Scenery {
        frame,
        camera,
        half: viewport * 0.5,
        remaining: if reduced {
            COSMETIC_PIXEL_BUDGET / 2
        } else {
            COSMETIC_PIXEL_BUDGET
        },
    };
    gas_wisps(&mut scene, seed, time, reduced);
    for (layer, factor, spacing) in [(0_u32, 0.075, 1800.0), (1, 0.22, 860.0), (2, 0.48, 1300.0)] {
        let sample = camera * factor;
        let min = tile(sample - scene.half - Vec2::splat(180.0), spacing);
        let max = tile(sample + scene.half + Vec2::splat(180.0), spacing);
        for y in min[1]..=max[1] {
            for x in min[0]..=max[0] {
                let grid = [x, y];
                let h = hash(seed, grid, layer.wrapping_mul(7919) ^ 0x706c_616e);
                if reduced && h & 1 != 0 {
                    continue;
                }
                let origin =
                    (Vec2::new(x as f32, y as f32) + Vec2::new(unit(h), unit(mix(h)))) * spacing;
                let position = origin + camera * (1.0 - factor);
                let biome = at(seed, origin / factor.max(0.075));
                if layer == 0 && scene.visible(position, 180.0) {
                    if h.is_multiple_of(5) {
                        let transform = Transform {
                            position,
                            rotation: unit(mix(h ^ 51)) * TAU,
                            scale: Vec2::splat(2.4),
                        };
                        let alpha = (20.0 + 8.0 * (time * 0.3 + unit(h) * TAU).sin()) as u8;
                        scene.body(&NEBULA, transform, biome.palette, alpha);
                    }
                    if h.is_multiple_of(11) {
                        let phase = (time + unit(mix(h ^ 131)) * 90.0).rem_euclid(90.0);
                        let pulse = if phase < 12.0 {
                            (phase / 12.0 * std::f32::consts::PI).sin()
                        } else {
                            0.0
                        };
                        let palette = PALETTES[4];
                        scene.body(
                            &PLANETS[3],
                            Transform {
                                position,
                                scale: Vec2::splat(0.32 + pulse * 0.2),
                                ..Transform::default()
                            },
                            palette,
                            (90.0 + pulse * 100.0) as u8,
                        );
                        for i in 0..24 {
                            let angle = i as f32 * TAU / 24.0;
                            let direction = Vec2::new(angle.cos(), angle.sin());
                            scene.pixel(
                                position + direction * (15.0 + pulse * 19.0),
                                0.8 + pulse * 0.7,
                                0x80_8fcaff,
                            );
                        }
                    }
                } else if layer == 1 && h.is_multiple_of(3) && scene.visible(position, 135.0) {
                    let kind = (mix(h ^ 12) % 3) as usize;
                    let transform = Transform {
                        position,
                        rotation: if kind == 1 {
                            -0.2
                        } else {
                            unit(mix(h ^ 93)) * 0.6
                        },
                        scale: Vec2::splat(0.32 + unit(mix(h ^ 37)) * 0.4),
                    };
                    if kind == 1 {
                        scene.body(&RINGS[0], transform, biome.palette, 150);
                    }
                    scene.body(&PLANETS[kind], transform, biome.palette, 190);
                    if kind == 1 {
                        scene.body(&RINGS[1], transform, biome.palette, 190);
                    }
                } else if layer == 2 {
                    let phase = (time + unit(mix(h ^ 303)) * 80.0).rem_euclid(80.0);
                    if phase < 18.0 {
                        let angle = -0.55 + unit(mix(h ^ 107)) * 1.1;
                        let d = Vec2::new(angle.cos(), angle.sin());
                        let head = position + d * ((phase - 9.0) * 22.0);
                        for i in (0..18).rev() {
                            let alpha = (190 - i * 9) as u32;
                            scene.pixel(
                                head - d * i as f32 * 5.0,
                                if i < 3 { 2.0 } else { 1.0 },
                                (biome.palette[3] & 0x00ff_ffff) | (alpha << 24),
                            );
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rich_cached_planets_and_split_rings_keep_fine_geometry() {
        for planet in PLANETS.iter() {
            assert_eq!((planet.width, planet.height), (96, 96));
            assert!((4_300..4_500).contains(&planet.occupied_count()));
            assert!(planet.cells().all(|(_, shade)| shade < 8));
            assert!(!planet.is_occupied(0, 0));
        }
        assert_ne!(
            PLANETS[0].cells().collect::<Vec<_>>(),
            PLANETS[1].cells().collect::<Vec<_>>()
        );
        assert_eq!((RINGS[0].width, RINGS[0].height), (128, 48));
        assert!(RINGS[0].cells().all(|(position, _)| position.y < 0.0));
        assert!(RINGS[1].cells().all(|(position, _)| position.y > 0.0));
        assert!(RINGS[0].occupied_count() + RINGS[1].occupied_count() < 800);
    }

    #[test]
    fn gas_cells_persist_animate_and_respect_full_and_reduced_limits() {
        let draw = |time, reduced| {
            let mut frame = FrameBuilder::new();
            let count = gas_wisps(
                &mut Scenery {
                    frame: &mut frame,
                    camera: Vec2::ZERO,
                    half: Vec2::new(400.0, 180.0),
                    remaining: COSMETIC_PIXEL_BUDGET,
                },
                17,
                time,
                reduced,
            );
            (frame.finish(), count)
        };
        let (a, count) = draw(20.0, false);
        let (b, repeated) = draw(20.0, false);
        let (later, _) = draw(21.0, false);
        let (reduced, sparse_count) = draw(20.0, true);
        assert!((400..=800).contains(&count));
        assert_eq!(count, repeated);
        assert_eq!(a.pixel_count, count);
        assert!(sparse_count <= 400 && sparse_count < count);
        assert_eq!(reduced.pixel_count, sparse_count);
        let pixels = |frame: &engine_core::RenderFrame| {
            frame
                .batches
                .iter()
                .flat_map(|batch| batch.pixels.iter().copied())
                .collect::<Vec<_>>()
        };
        assert_eq!(pixels(&a), pixels(&b));
        assert_ne!(pixels(&a), pixels(&later));
        assert!(
            a.batches
                .iter()
                .flat_map(|batch| &batch.transforms)
                .all(|transform| transform.b[2] == 0.0 && transform.b[3] == 0.0)
        );
        assert!(
            a.batches
                .iter()
                .flat_map(|batch| &batch.pixels)
                .all(|pixel| (pixel.color >> 24) <= 72)
        );
    }
    #[test]
    fn nearest_regions_are_stable_and_hub_is_forced_haven() {
        let seed = 0x4752_4f59;
        let mut kinds = [false; 6];
        for y in -8..=8 {
            for x in -8..=8 {
                let p = center(seed, [x, y]);
                let a = at(seed, p);
                assert_eq!(a, at(seed, p + Vec2::splat(0.25)));
                assert_eq!(a.grid, [x, y]);
                kinds[a.kind as usize] = true;
            }
        }
        assert!(kinds.into_iter().all(|seen| seen));
        for p in [Vec2::ZERO, Vec2::new(1399.0, 0.0), Vec2::new(-900.0, 800.0)] {
            assert_eq!(at(seed, p).kind, 0);
        }
    }
    #[test]
    fn discovery_ids_safe_radii_and_overlap_queries_are_stable() {
        let seed = 17;
        let sites = sites_near(seed, Vec2::new(4500.0, -6000.0));
        assert!(sites.len() <= 9);
        for s in &sites {
            assert_eq!(s.id, format!("{}:{}", s.grid[0], s.grid[1]));
            assert!((160.0..=230.0).contains(&s.radius));
            assert!((8..=12).contains(&s.target_kills));
            assert!(s.family < 3 && Vec2::from_array(s.position).length() > HUB_RADIUS + s.radius);
            assert_eq!(
                protected(seed, Vec2::from_array(s.position)),
                Some(s.clone())
            );
            assert_eq!(site_by_id(seed, &s.id), Some(s.clone()));
            for other in &sites {
                if s.id != other.id {
                    assert!(
                        Vec2::from_array(s.position).distance(Vec2::from_array(other.position))
                            > s.radius + other.radius
                    );
                }
            }
        }
        assert!(protected(seed, Vec2::ZERO).is_none());
        assert_eq!(sites, sites_near(seed, Vec2::new(4501.0, -6001.0)));
        for id in ["+1:2", "01:2", "1:02", "1:2:3", "999999:0", "x:y", "-0:1"] {
            assert!(site_by_id(seed, id).is_none());
        }
        let s = &sites[0];
        let p = Vec2::from_array(s.position);
        assert!(segment_protected(
            seed,
            p - Vec2::X * (s.radius + 20.0),
            p + Vec2::X * (s.radius + 20.0)
        ));
        assert!(!segment_protected(
            seed,
            p + Vec2::Y * (s.radius + 1.0),
            p + Vec2::Y * (s.radius + 1.0) + Vec2::X * 20.0
        ));
    }
    #[test]
    fn background_is_deterministic_bounded_and_has_valid_tables() {
        for viewport in [Vec2::new(640.0, 360.0), Vec2::new(8192.0, 8192.0)] {
            for camera in [
                Vec2::ZERO,
                Vec2::new(-9200.0, 11700.0),
                Vec2::splat(1_000_000.0),
            ] {
                let mut a = FrameBuilder::new();
                let mut b = FrameBuilder::new();
                background(&mut a, 17, camera, viewport, 9.5);
                background(&mut b, 17, camera, viewport, 9.5);
                assert!(a.frame().pixel_count <= COSMETIC_PIXEL_BUDGET);
                assert_eq!(a.frame().pixel_count, b.frame().pixel_count);
                for (a, b) in a.frame().batches.iter().zip(&b.frame().batches) {
                    assert_eq!(a.pixels, b.pixels);
                    assert_eq!(a.transforms, b.transforms);
                    assert!(a.transforms.len() <= 128);
                    assert!(
                        a.pixels
                            .iter()
                            .all(|p| (p.transform as usize) < a.transforms.len())
                    );
                }
            }
        }
    }
}
