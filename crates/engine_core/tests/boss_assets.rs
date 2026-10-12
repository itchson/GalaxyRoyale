use engine_core::{
    FrameBuilder, PixelBody, PixelRig, PixelRigPart, PixelRigState, Transform, Vec2,
};
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};

#[derive(Deserialize)]
struct Atlas {
    version: u8,
    palette: Vec<u32>,
    sprites: BTreeMap<String, Sprite>,
    rigs: BTreeMap<String, RigData>,
}
#[derive(Deserialize)]
struct Sprite {
    width: usize,
    height: usize,
    indices: Vec<u8>,
}
#[derive(Deserialize)]
struct RigData {
    roles: Vec<u8>,
    parts: Vec<PixelRigPart>,
    core_position: [f32; 2],
    core_radius: f32,
}
fn atlas() -> Atlas {
    serde_json::from_str(include_str!("../../../assets/pixels/boss-atlas.json")).unwrap()
}
fn body(sprite: &Sprite, palette: &[u32]) -> PixelBody {
    let rows: Vec<String> = sprite
        .indices
        .chunks(sprite.width)
        .map(|row| {
            row.iter()
                .map(|&i| char::from_u32(0xe000 + i as u32).unwrap())
                .collect()
        })
        .collect();
    let colors: Vec<_> = palette
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, &c)| (char::from_u32(0xe000 + i as u32).unwrap(), c))
        .collect();
    PixelBody::from_ascii(
        &rows.iter().map(String::as_str).collect::<Vec<_>>(),
        &colors,
    )
}

#[test]
fn fifteen_heads_have_distinct_normalized_silhouettes_and_occupied_local_cores() {
    let atlas = atlas();
    assert_eq!(atlas.version, 1);
    assert_eq!(atlas.palette.len(), 64);
    assert_eq!(atlas.sprites.len(), 20);
    for family in ["dart", "bastion", "coil"] {
        let mut outlines = HashSet::new();
        for generation in 1..=5 {
            let key = format!("boss_{family}_{generation}");
            let sprite = &atlas.sprites[&key];
            let metadata = &atlas.rigs[&key];
            assert_eq!(sprite.width, 56 + generation * 8);
            assert_eq!(sprite.indices.len(), sprite.width * sprite.height);
            let body = body(sprite, &atlas.palette);
            assert!(body.occupied_count() <= 6500, "{key}");
            assert_eq!(metadata.parts.len(), 7);
            let core = Vec2::from_array(metadata.core_position);
            assert!(body.hit_segment(core, core).is_some(), "{key}");
            let mut role_counts = [0usize; 7];
            for (index, local, color) in body.cells_indexed() {
                assert_eq!(color.to_le_bytes()[3], 255);
                let role = metadata.roles[index] as usize;
                role_counts[role] += 1;
                if role == 4 {
                    assert!(
                        local.distance(core) <= metadata.core_radius + 0.001,
                        "{key}"
                    );
                }
            }
            assert!(
                role_counts.iter().all(|&count| count > 0),
                "{key}: {role_counts:?}"
            );
            let mut outline = vec![false; 32 * 32];
            for y in 0..32 {
                for x in 0..32 {
                    outline[y * 32 + x] =
                        body.is_occupied(x * sprite.width / 32, y * sprite.height / 32);
                }
            }
            assert!(
                outlines.insert(outline),
                "{key} only rescales a previous silhouette"
            );
        }
    }
}

#[test]
fn all_asset_rigs_preserve_damage_holes_under_nested_nonuniform_sheared_groups() {
    let atlas = atlas();
    let group = Transform {
        position: Vec2::new(31.0, -15.0),
        rotation: 0.7,
        scale: Vec2::new(2.1, 0.6),
    };
    let actor = Transform {
        position: Vec2::new(-12.0, 9.0),
        rotation: -0.4,
        scale: Vec2::new(-0.8, 1.4),
    };
    let parent = group.compose_affine(actor);
    assert!(parent.basis_x.dot(parent.basis_y).abs() > 0.1);
    for (key, sprite) in &atlas.sprites {
        let data = &atlas.rigs[key];
        let rig = PixelRig::new(
            sprite.width,
            sprite.height,
            data.roles.clone(),
            data.parts.clone(),
        )
        .unwrap();
        let mut body = body(sprite, &atlas.palette);
        let core = Vec2::from_array(data.core_position);
        assert!(!body.damage(core, 2.4).is_empty(), "{key}");
        assert!(body.hit_segment(core, core).is_none(), "{key}");
        let mut frame = FrameBuilder::new();
        assert_eq!(
            rig.draw_affine(
                &mut frame,
                &body,
                parent,
                PixelRigState {
                    charge: 1.0,
                    core_open: true,
                    time: 0.2,
                    hit_flash: 0.3
                }
            ),
            body.occupied_count()
        );
        let mut visited = HashSet::new();
        for batch in &frame.frame().batches {
            for transform in &batch.transforms {
                assert!(transform.b[2] >= 0.0 && transform.b[2] <= 1.0);
                assert!(transform.b[3] >= 0.0 && transform.b[3] <= 0.6);
            }
            for pixel in &batch.pixels {
                let world =
                    batch.transforms[pixel.transform_slot()].point(Vec2::from_array(pixel.local));
                let local = parent.inverse_point(world);
                let grid = local + Vec2::new(sprite.width as f32, sprite.height as f32) * 0.5;
                let center = grid.floor() + Vec2::splat(0.5);
                assert!(grid.distance(center) < 0.001, "{key}: {grid:?}");
                let x = grid.x.floor() as usize;
                let y = grid.y.floor() as usize;
                assert!(body.is_occupied(x, y), "restored removed cell in {key}");
                assert!(visited.insert((x, y)), "duplicated surviving cell in {key}");
            }
        }
        assert_eq!(visited.len(), body.occupied_count());
    }
}

#[test]
fn deepest_coil_assembly_has_measured_bounded_quad_and_transform_cost() {
    let atlas = atlas();
    let mut frame = FrameBuilder::new();
    for (key, count) in [("boss_coil_5", 1), ("boss_coil_5_segment", 19)] {
        let sprite = &atlas.sprites[key];
        let data = &atlas.rigs[key];
        let rig = PixelRig::new(
            sprite.width,
            sprite.height,
            data.roles.clone(),
            data.parts.clone(),
        )
        .unwrap();
        let body = body(sprite, &atlas.palette);
        for index in 0..count {
            rig.draw(
                &mut frame,
                &body,
                Transform {
                    position: Vec2::new(index as f32 * 27.0, 0.0),
                    ..Transform::default()
                },
                PixelRigState::default(),
            );
        }
    }
    assert_eq!(frame.frame().pixel_count, 13_233);
    assert_eq!(frame.frame().batches.len(), 1);
    assert_eq!(frame.frame().batches[0].transforms.len(), 45);
}
