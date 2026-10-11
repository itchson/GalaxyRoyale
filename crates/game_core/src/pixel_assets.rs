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
    Assets {
        bodies,
        palette: atlas.palette,
    }
});

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
    asteroid_instance(seed, 1)
}
pub(super) fn asteroid_instance(seed: u32, material_tier: u8) -> PixelBody {
    crate::asteroid_art::body(seed, material_tier)
}
/// The immutable variety/drop selector remains unchanged by instance detailing.
pub(super) fn asteroid_variety(seed: u32) -> u8 {
    crate::asteroid_art::variety(seed)
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
    fn seeded_solid_bodies_keep_real_radius_and_independent_damage() {
        for family in 0..4 {
            let mut signatures = std::collections::HashSet::new();
            for instance in 0..12 {
                let seed = (crate::mix(instance * 1777) & !96) | (family * 32);
                let body = asteroid(seed);
                assert_eq!((body.width, body.height), (64, 64));
                assert!((700..=crate::asteroid_art::MAX_CELLS).contains(&body.occupied_count()));
                assert!(body.cells().all(|(_, color)| color >> 24 == 255));
                assert_solid_connected(&body);
                let intact = body.cells().collect::<Vec<_>>();
                let signature: Vec<_> = body.cells_indexed().map(|(i, _, c)| (i, c)).collect();
                assert!(signatures.insert(signature), "same-family instances repeat");
                let mut damaged = body.clone();
                let cell = damaged.cells().next().unwrap().0;
                assert!(!damaged.damage(cell, 2.0).is_empty());
                assert_eq!(asteroid(seed).cells().collect::<Vec<_>>(), intact);
                let scale = crate::asteroid_art::CELL_SCALE * crate::asteroid_art::MAX_SIZE;
                let r = radius(&body, scale);
                assert!(r < crate::regions::BELT_BODY_PADDING);
                for (p, _) in body.cells() {
                    for corner in [
                        Vec2::new(-0.5, -0.5),
                        Vec2::new(-0.5, 0.5),
                        Vec2::new(0.5, -0.5),
                        Vec2::new(0.5, 0.5),
                    ] {
                        assert!((p + corner).length() * scale <= r + 0.00001);
                    }
                }
            }
        }
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
