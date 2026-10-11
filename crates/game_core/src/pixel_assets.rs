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
    asteroids: [PixelBody; 4],
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
        .map(|(name, sprite)| (name.clone(), expand(sprite, &atlas.palette)))
        .collect();
    let asteroid = atlas.sprites.get("asteroid").expect("asteroid artwork");
    let asteroids = std::array::from_fn(|variant| {
        if variant > 0 {
            return procedural_asteroid(variant);
        }
        let palette: Vec<_> = atlas
            .palette
            .iter()
            .map(|&color| mineral_color(color, variant))
            .collect();
        expand(asteroid, &palette)
    });
    Assets {
        bodies,
        asteroids,
        palette: atlas.palette,
    }
});

/// Original bounded silhouettes: forged metal, split ice and porous basalt.
/// The rock template retains the generated faceted artwork. All four templates
/// are expanded once; every streamed asteroid still owns destructible cells.
fn procedural_asteroid(style: usize) -> PixelBody {
    let palettes = [
        [[0, 0, 0]; 5],
        [
            [27, 38, 48],
            [50, 70, 82],
            [83, 111, 122],
            [139, 163, 164],
            [203, 211, 183],
        ],
        [
            [22, 48, 66],
            [41, 88, 115],
            [74, 133, 157],
            [122, 182, 193],
            [190, 225, 211],
        ],
        [
            [38, 28, 43],
            [66, 49, 68],
            [95, 78, 87],
            [131, 110, 105],
            [173, 147, 118],
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
            let px = x as f32 - 19.5;
            let py = y as f32 - 19.5;
            let hash = crate::mix((x as u32 * 73) ^ (y as u32 * 193) ^ (style as u32 * 991));
            let occupied = match style {
                1 => px.abs() < 17.5 && py.abs() < 14.5 && px.abs() + py.abs() * 0.8 < 26.0,
                2 => {
                    px.abs() * 0.72 + py.abs() < 18.8
                        && px.abs() < 15.0
                        && !(px > 1.0 && px < 3.0 && py > -8.0 && py < 11.0)
                }
                _ => {
                    px * px / 280.0 + py * py / 250.0 < 1.0
                        && ![
                            (-7.0, -5.0, 3.5),
                            (6.0, 4.0, 4.0),
                            (4.0, -10.0, 2.2),
                            (-5.0, 10.0, 2.5),
                        ]
                        .iter()
                        .any(|&(cx, cy, r)| (px - cx).powi(2) + (py - cy).powi(2) < r * r)
                }
            };
            if !occupied {
                row.push(' ');
                continue;
            }
            let shade = match style {
                1 => {
                    if py < -10.0 || px < -14.0 {
                        4
                    } else if ((x + y * 2) / 7).is_multiple_of(3) {
                        3
                    } else if px > 10.0 || py > 9.0 {
                        1
                    } else {
                        2
                    }
                }
                2 => {
                    if px + py * 0.7 < -6.0 {
                        4
                    } else if py > 6.0 {
                        1
                    } else if (x * 2 + y) / 6 % 2 == 0 {
                        3
                    } else {
                        2
                    }
                }
                _ => {
                    if hash % 17 < 3 {
                        4
                    } else if px + py > 11.0 {
                        0
                    } else if ((x + y) / 5).is_multiple_of(3) {
                        3
                    } else {
                        2
                    }
                }
            };
            row.push((b'a' + shade) as char);
        }
        rows.push(row);
    }
    let borrowed: Vec<_> = rows.iter().map(String::as_str).collect();
    PixelBody::from_ascii(&borrowed, &colors)
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

fn mineral_color(color: u32, variant: usize) -> u32 {
    if color == 0 || variant == 0 {
        return color;
    }
    let [r, g, b, a] = color.to_le_bytes();
    let tint = match variant {
        1 => [0.85, 0.98, 1.12],
        2 => [1.08, 0.92, 0.82],
        _ => [1.05, 0.86, 1.13],
    };
    u32::from_le_bytes([
        (r as f32 * tint[0]).min(255.0) as u8,
        (g as f32 * tint[1]).min(255.0) as u8,
        (b as f32 * tint[2]).min(255.0) as u8,
        a,
    ])
}

pub(super) fn body(name: &str) -> PixelBody {
    ASSETS
        .bodies
        .get(name)
        .expect("known compiled sprite")
        .clone()
}

pub(super) fn asteroid(seed: u32) -> PixelBody {
    ASSETS.asteroids[(seed as usize >> 5) & 3].clone()
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
    fn four_authored_types_keep_real_cavities_conservative_collision_and_damaged_clone_isolation() {
        let types: Vec<_> = (0..4).map(|style| asteroid(style * 32)).collect();
        for (i, body) in types.iter().enumerate() {
            assert_eq!((body.width, body.height), (40, 40));
            assert!((500..=1067).contains(&body.occupied_count()));
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
        }
        assert!(!types[2].is_occupied(21, 20)); // split ice's real empty fissure
        assert!(!types[3].is_occupied(26, 24)); // porous basalt's real crater hole
        assert!(types[1].is_occupied(26, 24));
        assert!(
            types[3]
                .hit_segment(Vec2::new(5.5, 3.5), Vec2::new(6.5, 4.5))
                .is_none()
        );
        assert!(
            types[1]
                .hit_segment(Vec2::new(5.5, 3.5), Vec2::new(6.5, 4.5))
                .is_some()
        );
    }
}
