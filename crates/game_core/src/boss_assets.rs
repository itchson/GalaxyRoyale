//! Authored boss occupancy and rigid, hole-preserving nested visual rigs.
use crate::{PixelBody, Vec2};
use engine_core::{PixelRig, PixelRigPart};
use serde::Deserialize;
use std::{collections::HashMap, sync::LazyLock};

#[derive(Deserialize)]
struct Atlas {
    version: u32,
    palette: Vec<u32>,
    sprites: HashMap<String, Sprite>,
    rigs: HashMap<String, Rig>,
}
#[derive(Deserialize)]
struct Sprite {
    width: usize,
    height: usize,
    indices: Vec<u8>,
}
#[derive(Deserialize)]
struct Rig {
    roles: Vec<u8>,
    parts: Vec<PixelRigPart>,
    core_position: [f32; 2],
    core_radius: f32,
}
pub(super) struct BossArt {
    pub body: PixelBody,
    pub rig: PixelRig,
    pub core_position: Vec2,
    pub core_radius: f32,
}
struct Assets {
    heads: [[BossArt; 5]; 3],
    segments: [BossArt; 5],
}
static ASSETS: LazyLock<Assets> = LazyLock::new(|| {
    let mut atlas: Atlas =
        serde_json::from_str(include_str!("../../../assets/pixels/boss-atlas.json"))
            .expect("compiled boss atlas JSON");
    assert_eq!(atlas.version, 1);
    assert_eq!(atlas.palette.len(), 64);
    assert_eq!(atlas.palette[0], 0);
    assert!(atlas.palette[1..].iter().all(|c| c >> 24 == 255));
    let heads = std::array::from_fn(|family| {
        std::array::from_fn(|generation| {
            let name = format!(
                "boss_{}_{}",
                ["dart", "bastion", "coil"][family],
                generation + 1
            );
            let sprite = atlas.sprites.get(&name).expect("all fifteen boss heads");
            let rig = atlas.rigs.remove(&name).expect("head rig metadata");
            assert!(rig.core_radius.is_finite() && rig.core_radius > 0.0);
            assert!(rig.core_position.iter().all(|v| v.is_finite()));
            BossArt {
                body: expand(sprite, &atlas.palette),
                rig: PixelRig::new(sprite.width, sprite.height, rig.roles, rig.parts)
                    .expect("authored rigid rig is valid"),
                core_position: Vec2::from_array(rig.core_position),
                core_radius: rig.core_radius,
            }
        })
    });
    let segments = std::array::from_fn(|generation| {
        let name = format!("boss_coil_{}_segment", generation + 1);
        let sprite = atlas
            .sprites
            .get(&name)
            .expect("five destructible coil segment templates");
        let rig = atlas.rigs.remove(&name).expect("segment rig metadata");
        BossArt {
            body: expand(sprite, &atlas.palette),
            rig: PixelRig::new(sprite.width, sprite.height, rig.roles, rig.parts)
                .expect("authored segment rig"),
            core_position: Vec2::from_array(rig.core_position),
            core_radius: rig.core_radius,
        }
    });
    Assets { heads, segments }
});
fn expand(sprite: &Sprite, palette: &[u32]) -> PixelBody {
    assert!((1..=128).contains(&sprite.width) && (1..=128).contains(&sprite.height));
    assert_eq!(sprite.indices.len(), sprite.width * sprite.height);
    assert!(sprite.indices.iter().all(|i| (*i as usize) < palette.len()));
    let key = |index| char::from_u32(0xe000 + index as u32).unwrap();
    let colors: Vec<_> = palette
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, c)| (key(i), *c))
        .collect();
    let rows: Vec<String> = sprite
        .indices
        .chunks_exact(sprite.width)
        .map(|row| {
            row.iter()
                .map(|i| if *i == 0 { ' ' } else { key(*i as usize) })
                .collect()
        })
        .collect();
    PixelBody::from_ascii(
        &rows.iter().map(String::as_str).collect::<Vec<_>>(),
        &colors,
    )
}
pub(super) fn art(family: usize, generation: u8) -> &'static BossArt {
    &ASSETS.heads[family][generation.saturating_sub(1).min(4) as usize]
}
pub(super) fn segment(generation: u8) -> PixelBody {
    segment_art(generation).body.clone()
}
pub(super) fn segment_art(generation: u8) -> &'static BossArt {
    &ASSETS.segments[generation.saturating_sub(1).min(4) as usize]
}
