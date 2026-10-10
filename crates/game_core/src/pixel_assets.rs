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
    fn asteroid_variants_preserve_facets_occupancy_and_determinism() {
        let a = asteroid(0);
        let b = asteroid(32);
        assert_eq!(a.occupied_count(), b.occupied_count());
        assert_eq!(
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
}
