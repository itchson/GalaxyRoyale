//! Deterministic region network, discovery sites and bounded cosmetic scenery.
use engine_core::{FrameBuilder, PixelBody, Transform, Vec2};
use serde::Serialize;
use std::f32::consts::TAU;
use std::sync::LazyLock;

pub const REGION_SIZE: f32 = 4200.0;
pub const COSMETIC_PIXEL_BUDGET: usize = 6000;
const HUB_RADIUS: f32 = 1400.0;

/// Free space between complete node envelopes, including their asteroid bodies.
pub const NODE_CLEARANCE: f32 = 256.0;
pub const BELT_INNER_OFFSET: f32 = 240.0;
pub const BELT_OUTER_OFFSET: f32 = 520.0;
/// All streamed asteroid bodies at the configured maximum scale fit this disc.
pub const BELT_BODY_PADDING: f32 = 32.0;
// Above the maximum 0.0625-unit f32 world-coordinate ULP. Keep the advertised
// gap after both independently rounded centers and envelope sums are encoded.
const NODE_ROUNDING_MARGIN: f32 = 0.25;
const WORLD_LIMIT: f32 = 1_000_000.0;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Courier,
    Bounty,
    ChampionArena,
    RoyaleArena,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodePolicy {
    pub min_radius: f32,
    pub max_radius: f32,
}

impl NodeKind {
    pub const fn policy(self) -> NodePolicy {
        let (min_radius, max_radius) = match self {
            Self::Courier => (160.0, 220.0),
            Self::Bounty => (180.0, 260.0),
            Self::ChampionArena => (340.0, 420.0),
            Self::RoyaleArena => (400.0, 480.0),
        };
        NodePolicy {
            min_radius,
            max_radius,
        }
    }
}

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
    pub kind: NodeKind,
    pub position: [f32; 2],
    pub radius: f32,
    pub family: usize,
    pub target_kills: u32,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub struct ZoneBelt {
    pub inner: f32,
    pub outer: f32,
    pub material_tier: u8,
    pub recommended_level: u32,
}

/// Canonical remote rings stay clear of the Haven tutorial and world boundary.
/// Identity and material tier belong to this seeded site, never the camera.
pub fn site_belt(seed: u32, site: &Site) -> Option<ZoneBelt> {
    let canonical = site_by_id(seed, &site.id)?;
    if canonical != *site {
        return None;
    }
    let center = Vec2::from_array(site.position);
    let outer = site.radius + BELT_OUTER_OFFSET;
    if center.length() - site.envelope_radius() < HUB_RADIUS
        || center.abs().max_element() + site.envelope_radius() > WORLD_LIMIT
    {
        return None;
    }
    let tier = (1 + ((center.length() - HUB_RADIUS).max(0.0) / 1600.0) as u32).min(5) as u8;
    Some(ZoneBelt {
        inner: site.radius + BELT_INNER_OFFSET,
        outer,
        material_tier: tier,
        recommended_level: [1, 3, 5, 10, 12][tier as usize - 1],
    })
}

impl Site {
    /// Reserved birth envelope, not a bound on asteroid drift after impulses.
    /// The physical sanctuary still uses `radius`, never this cosmetic/belt disc.
    pub fn envelope_radius(&self) -> f32 {
        self.radius + BELT_OUTER_OFFSET + BELT_BODY_PADDING
    }

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
    let h = hash(seed, grid, 0x7369_7465);
    let courier = h & 1 == 0;
    let kind = if grid == [0, 2] {
        NodeKind::RoyaleArena
    } else if crate::campaign::CHAMPIONS.iter().any(|c| c.grid == grid) {
        NodeKind::ChampionArena
    } else if courier {
        NodeKind::Courier
    } else {
        NodeKind::Bounty
    };
    let policy = kind.policy();
    let radius = policy.min_radius + unit(mix(h)) * (policy.max_radius - policy.min_radius);
    let position = node_position(seed, grid, radius)?;
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
        kind,
        position: position.to_array(),
        radius,
        family: (mix(h ^ 37) % 3) as usize,
        target_kills: 8 + mix(h ^ 71) % 5,
    })
}

/// Each center is resolved independently inside its own cell. Insetting by the
/// complete belt/body envelope plus half the clearance proves separation even
/// when adjacent nodes select opposite extrema. No neighbor queries or retries.
fn node_position(seed: u32, grid: [i32; 2], radius: f32) -> Option<Vec2> {
    let (low, high) = node_bounds(grid, radius)?;
    let fraction = Vec2::new(
        unit(hash(seed, grid, 0x6e6f_6478)),
        unit(hash(seed, grid, 0x6e6f_6479)),
    );
    Some(low + (high - low) * fraction)
}

fn node_bounds(grid: [i32; 2], radius: f32) -> Option<(Vec2, Vec2)> {
    let envelope = radius + BELT_OUTER_OFFSET + BELT_BODY_PADDING;
    let inset = envelope + NODE_CLEARANCE * 0.5 + NODE_ROUNDING_MARGIN;
    let cell = Vec2::new(grid[0] as f32, grid[1] as f32) * REGION_SIZE;
    let mut low = cell.max(Vec2::splat(-WORLD_LIMIT)) + Vec2::splat(inset);
    let mut high =
        (cell + Vec2::splat(REGION_SIZE)).min(Vec2::splat(WORLD_LIMIT)) - Vec2::splat(inset);
    // The four cells touching Haven use far-corner rectangles. This leaves the
    // full tutorial belt/corridor clear without deleting a mandatory champion.
    if grid.iter().all(|coordinate| (-1..=0).contains(coordinate)) {
        let home_inset = HUB_RADIUS + inset;
        for axis in 0..2 {
            if grid[axis] == 0 {
                low[axis] = low[axis].max(home_inset);
            } else {
                high[axis] = high[axis].min(-home_inset);
            }
        }
    }
    if low.cmpgt(high).any() {
        return None; // Partial boundary cells cannot hold a complete group.
    }
    Some((low, high))
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
            let grain = surface_noise(Vec2::new(x * 0.7, y * 2.0), 0x7269_6e67);
            let radial = ((radius - 0.92) / 0.095).clamp(0.0, 1.0);
            // Lift the observed ellipse into its circular ring plane. Positive
            // y is the foreground half, so it is nearer the observer in z.
            // The existing sphere/light casts a curved shadow, not an unrelated
            // x/y rectangle that severs the exposed right tip of the ring.
            let position = [x, y, y * ((58.0_f32 / 18.0).powi(2) - 1.0).sqrt()];
            let light = [-0.52_f32, -0.32, 0.79];
            let light_squared = light.iter().map(|v| v * v).sum::<f32>();
            let toward_light = position.iter().zip(light).map(|(p, l)| p * l).sum::<f32>();
            let transmission = if toward_light >= 0.0 {
                1.0
            } else {
                let distance = (position.iter().map(|v| v * v).sum::<f32>()
                    - toward_light * toward_light / light_squared)
                    .max(0.0)
                    .sqrt();
                // A narrow five-source-unit penumbra around the unchanged
                // radius37.5 sphere; cached eight-shade quantization stays crisp.
                let t = ((distance - 35.0) / 5.0).clamp(0.0, 1.0);
                t * t * (3.0 - 2.0 * t)
            };
            let tone = ((2.0 + grain * 2.0 + (radial * std::f32::consts::PI).sin()) * transmission)
                .round() as usize;
            Some(['a', 'e', 'b', 'f', 'c', 'g', 'd', 'h'][tone.min(7)])
        })
    })
});
// The same authored surface at half resolution, generated once on first use.
// Doubled extraction scale retains its centered nominal world dimensions.
static LOW_PLANETS: LazyLock<[PixelBody; 4]> =
    LazyLock::new(|| std::array::from_fn(|i| half_body(&PLANETS[i])));
static LOW_RINGS: LazyLock<[PixelBody; 2]> =
    LazyLock::new(|| std::array::from_fn(|i| half_body(&RINGS[i])));

fn half_body(body: &PixelBody) -> PixelBody {
    assert!(body.width.is_multiple_of(2) && body.height.is_multiple_of(2));
    let rows: Vec<String> = (0..body.height / 2)
        .map(|y| {
            (0..body.width / 2)
                .map(|x| {
                    let samples: [Option<u32>; 4] =
                        std::array::from_fn(|i| body.color_at(x * 2 + (i & 1), y * 2 + (i >> 1)));
                    // Existing majority shade; first row-major sample wins ties.
                    // Any occupied source cell keeps its 2x2 block occupied.
                    let mut selected = None;
                    let mut most = 0;
                    for shade in samples.into_iter().flatten() {
                        let count = samples.iter().filter(|&&s| s == Some(shade)).count();
                        if count > most {
                            most = count;
                            selected = Some(shade);
                        }
                    }
                    selected.map_or(' ', |shade| char::from_u32('a' as u32 + shade).unwrap())
                })
                .collect()
        })
        .collect();
    PixelBody::from_ascii(
        &rows.iter().map(String::as_str).collect::<Vec<_>>(),
        &std::array::from_fn::<_, 8, _>(|i| {
            (char::from_u32('a' as u32 + i as u32).unwrap(), i as u32)
        }),
    )
}
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
// All texture generation is cached. It never consumes Game's combat RNG.
fn surface_noise(p: Vec2, seed: u32) -> f32 {
    let q = p / 5.0;
    let cell = q.floor().as_ivec2();
    let f = q - q.floor();
    let f = f * f * (Vec2::splat(3.0) - f * 2.0);
    let n = |dx, dy| unit(hash(seed, [cell.x + dx, cell.y + dy], 7));
    let a = n(0, 0) + (n(1, 0) - n(0, 0)) * f.x;
    let b = n(0, 1) + (n(1, 1) - n(0, 1)) * f.x;
    a + (b - a) * f.y
}

const SHADE_CODES: [char; 8] = ['a', 'e', 'b', 'f', 'c', 'g', 'd', 'h'];
// Sample materials in spherical coordinates. Screen-space texture would stretch
// evenly across the disc and make clouds look like flat stripes or camouflage.
fn sphere_surface(p: Vec2, z: f32, radius: f32) -> Vec2 {
    Vec2::new(
        p.x.atan2(z) * radius,
        (p.y / radius).clamp(-1.0, 1.0).asin() * radius,
    )
}

fn planet_albedo(kind: usize, surface: Vec2) -> f32 {
    let broad = surface_noise(surface * 0.45, 0x7375_7266 ^ kind as u32);
    let fine = surface_noise(surface * 1.9, 0x6669_6e65 ^ kind as u32);
    match kind {
        0 => 0.66 + broad * 0.25 + fine * 0.09,
        1 => {
            // Latitude bands are interrupted by a warped turbulent flow. A
            // compact elliptical vortex carries its own curved cloud wakes.
            let flow = surface.y + (surface.x * 0.075).sin() * 2.5 + broad * 3.0;
            let storm = Vec2::new((surface.x + 13.0) / 8.5, (surface.y - 7.0) / 4.5);
            let radius = storm.length();
            let wake = (storm.y.atan2(storm.x) * 2.0 - radius * 5.0).sin()
                * (-radius * radius * 0.65).exp();
            0.77 + (flow * 0.34).sin() * 0.11 + fine * 0.09 + wake * 0.11
                - (-radius * radius * 2.0).exp() * 0.13
        }
        _ => {
            let shelf = surface_noise(surface * 0.6 + Vec2::new(7.0, -3.0), 0x6963_6566);
            let ridge = (shelf - 0.46).abs();
            0.64 + broad * 0.15
                + fine * 0.08
                + shelf * 0.12
                + if ridge < 0.018 { 0.13 } else { 0.0 }
        }
    }
}
fn planet(kind: usize) -> PixelBody {
    if kind == 3 {
        return stellar_remnant();
    }
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
        let p = Vec2::new(x, y);
        let squared = p.length_squared();
        if squared > RADIUS * RADIUS {
            return None;
        }
        let z = (RADIUS * RADIUS - squared).sqrt();
        let light = ((-0.52 * x - 0.32 * y + 0.79 * z) / RADIUS).clamp(0.0, 1.0);
        let surface = sphere_surface(p, z, RADIUS);
        let mut albedo = planet_albedo(kind, surface);
        if kind == 0 {
            for (center, radius) in &craters {
                let delta = surface - *center;
                let distance = delta.length();
                if distance < *radius - 0.6 {
                    albedo *= if delta.x + delta.y < 0.0 { 0.67 } else { 0.86 };
                } else if (distance - radius).abs() < 0.65 {
                    albedo *= if delta.x + delta.y < 0.0 { 1.12 } else { 0.80 };
                }
            }
        }
        let tone = (light.powf(0.72) * albedo.clamp(0.0, 1.0) * 7.0).round() as usize;
        Some(SHADE_CODES[tone.min(7)])
    })
}

fn stellar_remnant() -> PixelBody {
    template(96, 96, |x, y| {
        let p = Vec2::new(x, y);
        let r = Vec2::new(x * 0.93, y * 1.13).length();
        let angle = y.atan2(x);
        if angle > -0.22 && angle < 0.40 {
            return None;
        }
        let cloud = surface_noise(p * 1.4, 0x7265_6d6e);
        let edge = 28.0 + (angle * 2.0 + 0.5).sin() * 3.2 + (angle * 3.0).cos() * 1.4;
        let distance = (r - edge + (cloud - 0.5) * 2.6).abs();
        let tone = if distance < 0.85 && cloud > 0.22 {
            Some((3.0 + cloud * 4.0).round() as usize)
        } else if distance < 2.5 && cloud > 0.66 {
            Some(if r < edge { 2 } else { 1 })
        } else if r < edge - 4.0 && r > edge - 5.1 && angle < -1.0 && cloud > 0.64 {
            Some(2)
        } else if (p - Vec2::new(-3.0, 1.0)).length() < 1.5 {
            Some(7)
        } else {
            None
        };
        tone.map(|i| SHADE_CODES[i.min(7)])
    })
}

// Type owns the surface material; biome contributes only a restrained hue bias.
// The ramp starts near black so terrain cannot erase a physical terminator.
const PLANET_RAMPS: [[[u8; 3]; 8]; 4] = [
    [
        [4, 5, 7],
        [14, 14, 16],
        [30, 28, 27],
        [48, 44, 39],
        [68, 60, 50],
        [90, 79, 64],
        [117, 104, 83],
        [147, 135, 111],
    ],
    [
        [4, 5, 7],
        [15, 15, 17],
        [33, 31, 29],
        [53, 47, 39],
        [75, 65, 50],
        [100, 86, 64],
        [127, 111, 83],
        [157, 140, 107],
    ],
    [
        [4, 5, 8],
        [12, 17, 21],
        [24, 35, 40],
        [40, 54, 59],
        [58, 74, 77],
        [79, 96, 97],
        [103, 120, 118],
        [135, 149, 142],
    ],
    [
        [18, 17, 22],
        [35, 29, 35],
        [54, 38, 45],
        [77, 49, 54],
        [101, 64, 64],
        [130, 85, 77],
        [161, 109, 92],
        [196, 146, 112],
    ],
];
fn planet_shade(kind: usize, palette: [u32; 4], shade: u32, alpha: u8) -> u32 {
    let index = [0, 2, 4, 6, 1, 3, 5, 7][shade as usize];
    let base = PLANET_RAMPS[kind][index];
    let bias = palette[2].to_le_bytes();
    let rgb: [u8; 3] =
        std::array::from_fn(|i| (base[i] as f32 * (0.90 + bias[i] as f32 / 255.0 * 0.16)) as u8);
    u32::from_le_bytes([rgb[0], rgb[1], rgb[2], alpha])
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
    fn planet_group(
        &mut self,
        kind: usize,
        mut transform: Transform,
        palette: [u32; 4],
        reduced: bool,
    ) -> usize {
        let planets = if reduced { &*LOW_PLANETS } else { &*PLANETS };
        let rings = (kind == 1).then(|| if reduced { &*LOW_RINGS } else { &*RINGS });
        let required = planets[kind].occupied_count()
            + rings.map_or(0, |rings| rings.iter().map(PixelBody::occupied_count).sum());
        if required > self.remaining {
            return 0;
        }
        if reduced {
            transform.scale *= 2.0;
        }
        if let Some(rings) = rings {
            self.planet_body(&rings[0], transform, 0, palette, 120);
        }
        self.planet_body(&planets[kind], transform, kind, palette, 230);
        if let Some(rings) = rings {
            self.planet_body(&rings[1], transform, 0, palette, 145);
        }
        required
    }
    fn planet_body(
        &mut self,
        body: &PixelBody,
        transform: Transform,
        kind: usize,
        palette: [u32; 4],
        alpha: u8,
    ) {
        self.frame.push_body_colored(body, transform, |_, c| {
            planet_shade(kind, palette, c, alpha)
        });
        self.remaining -= body.occupied_count();
    }
    fn nova_group(&mut self, position: Vec2, pulse: f32, reduced: bool) -> usize {
        let body = if reduced {
            &LOW_PLANETS[3]
        } else {
            &PLANETS[3]
        };
        let required = body.occupied_count();
        if required > self.remaining {
            return 0;
        }
        // A stable broken remnant, not a flashing disc with equally spaced spokes.
        self.planet_body(
            body,
            Transform {
                position,
                scale: Vec2::splat(0.52 * if reduced { 2.0 } else { 1.0 }),
                ..Transform::default()
            },
            3,
            PALETTES[4],
            (65.0 + pulse * 8.0) as u8,
        );
        required
    }
}

/// Stable analytic gas cells: seed/grid identifies a cell for the entire session;
/// time only moves its curl and opacity. No random respawns or simulation bodies.
fn gas_wisps(scene: &mut Scenery<'_>, seed: u32, time: f32, reduced: bool) -> usize {
    crate::cosmic_background::render_gas(
        scene.frame,
        crate::cosmic_background::SceneInput {
            seed,
            camera: scene.camera,
            viewport: scene.half * 2.0,
            time,
            reduced,
            reduced_motion: false, // Game supplies its already-frozen cosmetic time.
        },
        &mut scene.remaining,
    )
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
        let margin = if layer == 2 {
            crate::shooting_star::STREAM_RADIUS
        } else {
            180.0
        };
        let min = tile(sample - scene.half - Vec2::splat(margin), spacing);
        let max = tile(sample + scene.half + Vec2::splat(margin), spacing);
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
                    if h.is_multiple_of(11) {
                        let pulse = 0.5 + 0.5 * (time * 0.035 + unit(mix(h ^ 131)) * TAU).sin();
                        scene.nova_group(position, pulse, reduced);
                    }
                } else if layer == 1 && h.is_multiple_of(3) && scene.visible(position, 135.0) {
                    if biome.kind == 4 {
                        // Origin has not encoded parallax; helper encodes it once.
                        let input = crate::isotope_scar::SceneInput {
                            origin,
                            camera: scene.camera,
                            viewport: scene.half * 2.0,
                            time, // Game already forwards its frozen cosmetic_time().
                            rotation: unit(mix(h ^ 93)) * 0.6,
                            scale: Vec2::splat(0.64 + unit(mix(h ^ 37)) * 0.08),
                            reduced,
                            reduced_motion: false,
                        };
                        let _scar_stats =
                            crate::isotope_scar::extract(scene.frame, &input, &mut scene.remaining);
                        continue; // Replace the old planet/rings, even on budget rejection.
                    }
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
                    scene.planet_group(kind, transform, biome.palette, reduced);
                } else if layer == 2 {
                    crate::shooting_star::extract(
                        scene.frame,
                        &crate::shooting_star::SceneInput {
                            origin,
                            camera: scene.camera,
                            viewport: scene.half * 2.0,
                            time,
                            birth_hash: h,
                            tint: biome.palette,
                            reduced,
                            reduced_motion: false,
                        },
                        &mut scene.remaining,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "isotope_region_tests.rs"]
mod isotope_tests;

#[cfg(test)]
#[path = "region_immersion_tests.rs"]
mod immersion_tests;

#[cfg(test)]
#[path = "regions_lod_tests.rs"]
mod lod_tests;

#[cfg(test)]
#[path = "region_node_tests.rs"]
mod node_tests;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rich_cached_planets_and_split_rings_keep_fine_geometry() {
        for planet in PLANETS.iter().take(3) {
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
        assert!((1..=800).contains(&count));
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
                .all(|transform| transform.b[2] == 0.0 && transform.b[3] == -1.0)
        );
        assert!(
            a.batches
                .iter()
                .flat_map(|batch| &batch.pixels)
                .all(|pixel| (pixel.color >> 24) <= 100)
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
            let policy = s.kind.policy();
            assert!((policy.min_radius..=policy.max_radius).contains(&s.radius));
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
