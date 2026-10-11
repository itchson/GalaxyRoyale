//! The generated atlas is compiled into WASM and expanded once into real cells.
//! Rendering and damage always use PixelBody; no sprite texture is involved.
use engine_core::PixelBody;
use serde::Deserialize;
use std::{collections::HashMap, sync::LazyLock};

#[derive(Deserialize)]
struct Atlas {
    version: u32,
    palette: Vec<u32>,
    sprites: HashMap<String, Sprite>,
}

#[derive(Deserialize)]
struct Sprite {
    width: usize,
    height: usize,
    indices: Vec<u8>,
}

struct Assets {
    bodies: HashMap<String, PixelBody>,
    asteroids: [PixelBody; 16],
    palette: Vec<u32>,
}

static ASSETS: LazyLock<Assets> = LazyLock::new(|| {
    let atlas: Atlas = serde_json::from_str(include_str!("../../../assets/pixels/atlas.json"))
        .expect("compiled pixel atlas must be valid JSON");
    assert_eq!(atlas.version, 1, "unsupported compiled pixel atlas version");
    assert_eq!(atlas.palette.len(), 64);
    assert_eq!(atlas.palette[0], 0, "index zero must be transparent");
    assert!(atlas.palette[1..].iter().all(|color| color >> 24 == 255));
    let bodies = atlas
        .sprites
        .iter()
        .map(|(name, sprite)| {
            let body = expand(sprite, &atlas.palette);
            let body = if name == "dock" {
                detail_dock(&body, &atlas.palette)
            } else {
                body
            };
            (name.clone(), body)
        })
        .collect();
    let asteroid = atlas.sprites.get("asteroid").expect("asteroid artwork");
    // Four mineral families, each with four cached deterministic surface fields.
    // The silhouettes are fixed: recoloring cannot alter orbit/collision radii.
    let asteroids = std::array::from_fn(|index| {
        let style = index & 3;
        let geology = index >> 2;
        if style == 0 {
            let body = expand(asteroid, &atlas.palette);
            if geology == 0 {
                body
            } else {
                map_colors(&body, |x, y, original| {
                    let grain =
                        crate::mix((x as u32 * 73) ^ (y as u32 * 193) ^ (geology as u32 * 991));
                    let [r, g, b, a] = original.to_le_bytes();
                    let band = ((x + y * (geology + 1)) / 7) % 4;
                    let factor = if band == 0 {
                        0.82
                    } else if grain.is_multiple_of(29) {
                        1.13
                    } else {
                        1.0
                    };
                    u32::from_le_bytes([
                        (r as f32 * factor).min(255.0) as u8,
                        (g as f32 * factor).min(255.0) as u8,
                        (b as f32 * factor).min(255.0) as u8,
                        a,
                    ])
                })
            }
        } else {
            procedural_asteroid(style, geology)
        }
    });
    Assets {
        bodies,
        asteroids,
        palette: atlas.palette,
    }
});

/// Intact material is solid. Cracks, pits and ice cleavages are opaque shades;
/// only actual damage may turn an interior cell into a collision hole.
fn procedural_asteroid(style: usize, geology: usize) -> PixelBody {
    let palettes = [
        [[0, 0, 0]; 8],
        [
            [20, 31, 41],
            [35, 48, 59],
            [51, 69, 79],
            [72, 94, 104],
            [98, 125, 132],
            [133, 158, 161],
            [176, 194, 183],
            [221, 228, 205],
        ],
        [
            [17, 36, 54],
            [26, 57, 79],
            [39, 81, 105],
            [56, 110, 133],
            [81, 144, 164],
            [118, 180, 190],
            [165, 213, 214],
            [215, 238, 224],
        ],
        [
            [30, 22, 35],
            [47, 35, 51],
            [67, 51, 67],
            [87, 70, 80],
            [112, 90, 94],
            [143, 115, 106],
            [175, 143, 118],
            [203, 174, 136],
        ],
    ];
    let colors: Vec<_> = palettes[style]
        .iter()
        .enumerate()
        .map(|(i, c)| {
            (
                (b'a' + i as u8) as char,
                u32::from_le_bytes([c[0], c[1], c[2], 255]),
            )
        })
        .collect();
    let mut rows = Vec::with_capacity(40);
    for y in 0..40usize {
        let mut row = String::with_capacity(40);
        for x in 0..40usize {
            let p = engine_core::Vec2::new(x as f32 - 19.5, y as f32 - 19.5);
            if !rock_mask(style, p.x, p.y) {
                row.push(' ');
                continue;
            }
            let grain = crate::mix(
                (x as u32 * 73) ^ (y as u32 * 193) ^ (style as u32 * 991) ^ (geology as u32 * 1777),
            );
            let mut shade = if p.x * 0.6 + p.y * 0.8 < -5.0 {
                5
            } else if p.x + p.y > 12.0 {
                2
            } else {
                3
            };
            // Directional faceted strata avoid random bright speckle/noisy holes.
            if ((x + y * (style + 1) + geology * 3) / 7).is_multiple_of(4) {
                shade += 1;
            }
            if grain.is_multiple_of(37) {
                shade = (shade + 1).min(6);
            }
            let edge = [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)]
                .iter()
                .any(|&(dx, dy)| !rock_mask(style, p.x + dx, p.y + dy));
            if edge {
                shade = if p.x + p.y < 0.0 { 6 } else { 1 };
            }
            let shift = geology as f32 * 1.4;
            for (cx, cy, r) in [
                (-7.0 + shift, -5.0, 3.5),
                (6.0, 4.0 - shift, 4.0),
                (4.0, -10.0, 2.2),
                (-5.0, 10.0, 2.5),
            ] {
                let dx = p.x - cx;
                let dy = p.y - cy;
                let d = dx * dx + dy * dy;
                if d < r * r {
                    // Opaque crater floor and a light-facing inner lip.
                    shade = if d > (r - 1.0) * (r - 1.0) && dx + dy < 0.0 {
                        5
                    } else if d < r * r * 0.45 {
                        0
                    } else {
                        1
                    };
                } else if d < (r + 0.8) * (r + 0.8) && dx + dy > 0.0 {
                    shade = 4;
                }
            }
            let seam = match style {
                1 => (p.y * 0.35 + p.x - 4.0 + shift).abs() < 0.65 && p.y.abs() < 10.0,
                2 => (p.x - (1.5 + p.y * 0.12 - shift)).abs() < 0.75 && p.y > -9.0 && p.y < 11.0,
                _ => (p.y + p.x * 0.45 + 3.0 - shift).abs() < 0.65 && p.x.abs() < 12.0,
            };
            if seam && !edge {
                shade = 0;
            }
            row.push((b'a' + shade) as char);
        }
        rows.push(row);
    }
    let borrowed: Vec<_> = rows.iter().map(String::as_str).collect();
    PixelBody::from_ascii(&borrowed, &colors)
}

fn rock_mask(style: usize, x: f32, y: f32) -> bool {
    match style {
        1 => x.abs() < 17.5 && y.abs() < 14.5 && x.abs() + y.abs() * 0.8 < 26.0,
        2 => x.abs() * 0.72 + y.abs() < 18.8 && x.abs() < 15.0,
        _ => x * x / 280.0 + y * y / 250.0 < 1.0,
    }
}

/// Initializer-only palette mapping; it preserves every occupied cell exactly.
fn map_colors(body: &PixelBody, mut map: impl FnMut(usize, usize, u32) -> u32) -> PixelBody {
    let mut keys = HashMap::new();
    let mut palette = Vec::new();
    let mut rows = Vec::with_capacity(body.height);
    for y in 0..body.height {
        let mut row = String::new();
        for x in 0..body.width {
            let Some(original) = body.color_at(x, y) else {
                row.push(' ');
                continue;
            };
            let color = map(x, y, original);
            let key = *keys.entry(color).or_insert_with(|| {
                let key = char::from_u32(0xe000 + palette.len() as u32).unwrap();
                palette.push((key, color));
                key
            });
            row.push(key);
        }
        rows.push(row);
    }
    let borrowed: Vec<_> = rows.iter().map(String::as_str).collect();
    PixelBody::from_ascii(&borrowed, &palette)
}

/// A strictly overhead deck treatment on the exact accepted docking footprint.
/// Roof seams, radial berths and flush signal sockets replace elevation shading;
/// the open bay, actor radius and every existing occupied cell stay unchanged.
fn detail_dock(body: &PixelBody, palette: &[u32]) -> PixelBody {
    map_colors(body, |x, y, _| {
        let p = engine_core::Vec2::new(x as f32 - 31.5, y as f32 - 31.5);
        let a = p.abs();
        let r = p.length();
        let edge = [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ]
        .iter()
        .any(|&(nx, ny)| !body.is_occupied(nx, ny));
        let rgb = if edge {
            if p.x + p.y < 0.0 {
                [139, 169, 175]
            } else {
                [27, 43, 54]
            }
        } else if (a.x - 19.5).abs() < 0.1 && (a.y - 9.5).abs() < 0.1 {
            [209, 226, 218]
        } else if (17.0..=25.0).contains(&r) && (x + y * 3).is_multiple_of(11) {
            // These exact palette signals use the existing host's object-local
            // cosmetic wave and therefore respect its Reduced Motion clock.
            return palette[if x.is_multiple_of(2) { 6 } else { 4 }];
        } else if a.max_element() >= 26.0 {
            if a.min_element() < 0.7 {
                [16, 33, 46]
            } else if ((a.max_element() as usize) / 3).is_multiple_of(2) {
                [61, 91, 107]
            } else {
                [36, 58, 75]
            }
        } else if (a.x - a.y).abs() < 0.7 || (21.0..=22.0).contains(&r) {
            [17, 31, 44]
        } else if (16.5..=19.0).contains(&r) {
            [81, 166, 184]
        } else if (24.0..=26.0).contains(&r) {
            if (x + y).is_multiple_of(4) {
                [196, 151, 79]
            } else {
                [74, 94, 108]
            }
        } else if a.min_element() < 1.0 && r >= 23.0 {
            [102, 186, 196]
        } else {
            [48, 69, 84]
        };
        u32::from_le_bytes([rgb[0], rgb[1], rgb[2], 255])
    })
}

fn expand(sprite: &Sprite, colors: &[u32]) -> PixelBody {
    assert!((1..=128).contains(&sprite.width));
    assert!((1..=128).contains(&sprite.height));
    assert_eq!(sprite.indices.len(), sprite.width * sprite.height);
    assert!(
        sprite
            .indices
            .iter()
            .all(|&index| (index as usize) < colors.len())
    );
    // The engine's palette constructor accepts character keys. Private-use keys
    // preserve every atlas index without depending on printable ASCII glyphs.
    let key = |index: usize| char::from_u32(0xe000 + index as u32).unwrap();
    let palette: Vec<_> = colors
        .iter()
        .enumerate()
        .skip(1)
        .map(|(index, &color)| (key(index), color))
        .collect();
    let rows: Vec<String> = sprite
        .indices
        .chunks_exact(sprite.width)
        .map(|row| {
            row.iter()
                .map(|&index| if index == 0 { ' ' } else { key(index as usize) })
                .collect()
        })
        .collect();
    let borrowed: Vec<_> = rows.iter().map(String::as_str).collect();
    PixelBody::from_ascii(&borrowed, &palette)
}

pub(super) fn body(name: &str) -> PixelBody {
    ASSETS
        .bodies
        .get(name)
        .expect("known compiled sprite")
        .clone()
}

pub(super) fn asteroid(seed: u32) -> PixelBody {
    let style = asteroid_variety(seed) as usize;
    let geology = (seed as usize >> 11) & 3;
    ASSETS.asteroids[style + geology * 4].clone()
}
/// Immutable physical variety shares the exact selector used by authored cells.
pub(super) fn asteroid_variety(seed: u32) -> u8 {
    ((seed >> 5) & 3) as u8
}

pub(super) fn color(index: usize) -> u32 {
    ASSETS.palette[index]
}

/// A conservative circle containing every occupied unit square, including its
/// corners. Scales remain actor-local and never resize or rasterize the cells.
pub(super) fn radius(body: &PixelBody, scale: f32) -> f32 {
    body.cells()
        .map(|(cell, _)| (cell.length() + std::f32::consts::FRAC_1_SQRT_2) * scale)
        .fold(0.0, f32::max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::Vec2;

    #[test]
    fn generated_assets_keep_exact_occupancy_dimensions_and_hard_palette() {
        for (name, size, occupied) in [
            ("player", 48, 931),
            ("dart", 32, 426),
            ("bastion", 40, 938),
            ("coil_head", 32, 584),
            ("coil_segment", 16, 87),
            ("asteroid", 40, 1067),
            ("dock", 64, 1454),
            ("dart_boss", 64, 2045),
        ] {
            let body = body(name);
            assert_eq!((body.width, body.height), (size, size), "{name}");
            assert_eq!(body.occupied_count(), occupied, "{name}");
            assert!(body.cells().all(|(_, color)| color >> 24 == 255));
        }
    }

    #[test]
    fn dock_center_is_open_and_damage_never_changes_the_pristine_template() {
        let dock = body("dock");
        assert!(!dock.is_occupied(32, 32));
        assert!(
            dock.hit_segment(Vec2::new(-3.0, 0.0), Vec2::new(3.0, 0.0))
                .is_none()
        );
        let mut ship = body("player");
        assert!(!ship.damage(Vec2::ZERO, 3.0).is_empty());
        assert!(ship.occupied_count() < body("player").occupied_count());
        assert_eq!(body("player").occupied_count(), 931);
    }

    #[test]
    fn asteroid_types_have_distinct_live_masks_and_deterministic_facets() {
        let a = asteroid(0);
        let b = asteroid(32);
        assert_ne!(a.occupied_count(), b.occupied_count());
        assert_ne!(
            a.cells().map(|(p, _)| p).collect::<Vec<_>>(),
            b.cells().map(|(p, _)| p).collect::<Vec<_>>()
        );
        assert_ne!(
            a.cells().map(|(_, c)| c).collect::<Vec<_>>(),
            b.cells().map(|(_, c)| c).collect::<Vec<_>>()
        );
        assert_eq!(
            asteroid(32).cells().collect::<Vec<_>>(),
            b.cells().collect::<Vec<_>>()
        );
    }

    #[test]
    fn sixteen_solid_surface_fields_preserve_family_radii_and_clone_isolation() {
        let types: Vec<_> = (0..4).map(|style| asteroid(style * 32)).collect();
        for (i, body) in types.iter().enumerate() {
            assert_eq!((body.width, body.height), (40, 40));
            assert!((500..=1600).contains(&body.occupied_count()));
            assert!(body.cells().all(|(_, color)| color >> 24 == 255));
            for scale in [0.55, 1.0] {
                let r = radius(body, scale);
                for (p, _) in body.cells() {
                    for offset in [
                        Vec2::new(-0.5, -0.5),
                        Vec2::new(-0.5, 0.5),
                        Vec2::new(0.5, -0.5),
                        Vec2::new(0.5, 0.5),
                    ] {
                        assert!((p + offset).length() * scale <= r + 0.00001);
                    }
                }
            }
            let pristine = body.cells().collect::<Vec<_>>();
            let mut damaged = asteroid(i as u32 * 32);
            let cell = damaged.cells().next().unwrap().0;
            assert!(!damaged.damage(cell, 2.0).is_empty());
            assert!(damaged.occupied_count() < pristine.len());
            assert_eq!(
                asteroid(i as u32 * 32).cells().collect::<Vec<_>>(),
                pristine
            );
            for other in types.iter().skip(i + 1) {
                assert_ne!(
                    body.cells().map(|(p, _)| p).collect::<Vec<_>>(),
                    other.cells().map(|(p, _)| p).collect::<Vec<_>>()
                );
            }
            let positions: Vec<_> = body.cells().map(|(p, _)| p).collect();
            let mut colors = std::collections::HashSet::new();
            for geology in 0..4 {
                let variant = asteroid(i as u32 * 32 + geology * 2048);
                assert_eq!(
                    variant.cells().map(|(p, _)| p).collect::<Vec<_>>(),
                    positions
                );
                assert_eq!(radius(&variant, 0.55), radius(body, 0.55));
                assert_eq!(
                    variant.cells().collect::<Vec<_>>(),
                    asteroid(i as u32 * 32 + geology * 2048)
                        .cells()
                        .collect::<Vec<_>>()
                );
                assert!(colors.insert(variant.cells().map(|(_, c)| c).collect::<Vec<_>>()));
                assert_solid_connected(&variant);
            }
        }
        assert!(types[2].is_occupied(21, 20)); // Opaque ice cleavage.
        assert!(types[3].is_occupied(26, 24)); // Opaque crater floor.
        assert!(types[1].is_occupied(26, 24));
        assert!(
            types[3]
                .hit_segment(Vec2::new(5.5, 3.5), Vec2::new(6.5, 4.5))
                .is_some()
        );
        assert!(
            types[1]
                .hit_segment(Vec2::new(5.5, 3.5), Vec2::new(6.5, 4.5))
                .is_some()
        );
    }

    fn assert_solid_connected(body: &PixelBody) {
        use std::collections::{HashSet, VecDeque};
        let neighbors = |x: usize, y: usize| {
            [
                (x.wrapping_sub(1), y),
                (x + 1, y),
                (x, y.wrapping_sub(1)),
                (x, y + 1),
            ]
        };
        let flood = |starts: Vec<(usize, usize)>, filled: bool| {
            let mut seen: HashSet<_> = starts.iter().copied().collect();
            let mut todo = VecDeque::from(starts);
            while let Some((x, y)) = todo.pop_front() {
                for p in neighbors(x, y) {
                    if p.0 < body.width
                        && p.1 < body.height
                        && body.is_occupied(p.0, p.1) == filled
                        && seen.insert(p)
                    {
                        todo.push_back(p);
                    }
                }
            }
            seen
        };
        let first = (0..body.height)
            .flat_map(|y| (0..body.width).map(move |x| (x, y)))
            .find(|&(x, y)| body.is_occupied(x, y))
            .unwrap();
        assert_eq!(
            flood(vec![first], true).len(),
            body.occupied_count(),
            "intact rock must be one connected body"
        );
        let boundary = (0..body.height)
            .flat_map(|y| (0..body.width).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                (x == 0 || y == 0 || x == body.width - 1 || y == body.height - 1)
                    && !body.is_occupied(x, y)
            })
            .collect();
        assert_eq!(
            flood(boundary, false).len() + body.occupied_count(),
            body.width * body.height,
            "initial rock must have no enclosed transparent cavities"
        );
    }

    #[test]
    fn actual_damage_opens_a_previously_opaque_crater_without_changing_other_rocks() {
        let mut basalt = asteroid(96);
        let start = Vec2::new(5.5, 3.5);
        let end = Vec2::new(6.5, 4.5);
        assert!(basalt.hit_segment(start, end).is_some());
        let before = basalt.occupied_count();
        let removed = basalt.damage(Vec2::new(6.0, 4.0), 2.5);
        assert!(!removed.is_empty());
        assert_eq!(before - basalt.occupied_count(), removed.len());
        assert!(basalt.hit_segment(start, end).is_none());
        assert!(
            basalt
                .hit_segment(Vec2::new(-20.0, 0.0), Vec2::ZERO)
                .is_some()
        );
        assert!(asteroid(96).hit_segment(start, end).is_some());
    }

    #[test]
    fn filled_interiors_keep_the_previously_published_outer_collision_radii() {
        for style in 1..=3 {
            let mut previous_radius = 0.0f32;
            for y in 0..40 {
                for x in 0..40 {
                    let p = Vec2::new(x as f32 - 19.5, y as f32 - 19.5);
                    // Published pre-change material mask. Interior fissures/craters
                    // were absent, but the occupied outer perimeter was the same.
                    let old = match style {
                        1 => {
                            p.x.abs() < 17.5
                                && p.y.abs() < 14.5
                                && p.x.abs() + p.y.abs() * 0.8 < 26.0
                        }
                        2 => {
                            p.x.abs() * 0.72 + p.y.abs() < 18.8
                                && p.x.abs() < 15.0
                                && !(p.x > 1.0 && p.x < 3.0 && p.y > -8.0 && p.y < 11.0)
                        }
                        _ => {
                            p.x * p.x / 280.0 + p.y * p.y / 250.0 < 1.0
                                && ![
                                    (-7.0, -5.0, 3.5),
                                    (6.0, 4.0, 4.0),
                                    (4.0, -10.0, 2.2),
                                    (-5.0, 10.0, 2.5),
                                ]
                                .iter()
                                .any(|&(cx, cy, r)| (p.x - cx).powi(2) + (p.y - cy).powi(2) < r * r)
                        }
                    };
                    if old {
                        previous_radius =
                            previous_radius.max(p.length() + std::f32::consts::FRAC_1_SQRT_2);
                    }
                }
            }
            assert_eq!(radius(&asteroid(style * 32), 1.0), previous_radius);
        }
    }

    #[test]
    fn authored_dock_panel_detail_keeps_exact_generated_open_hull() {
        let atlas: Atlas =
            serde_json::from_str(include_str!("../../../assets/pixels/atlas.json")).unwrap();
        let old = expand(&atlas.sprites["dock"], &atlas.palette);
        let detailed = body("dock");
        assert_eq!(
            old.cells().map(|(p, _)| p).collect::<Vec<_>>(),
            detailed.cells().map(|(p, _)| p).collect::<Vec<_>>()
        );
        assert_eq!(radius(&old, 1.0), radius(&detailed, 1.0));
        let changed = old
            .cells()
            .zip(detailed.cells())
            .filter(|(a, b)| a.1 != b.1)
            .count();
        assert!(changed > old.occupied_count() * 3 / 4 && changed <= old.occupied_count());
        assert!(detailed.cells().any(|(_, color)| color == atlas.palette[6]));
        assert!(detailed.cells().any(|(_, color)| color == atlas.palette[4]));
    }
}
