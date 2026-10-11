//! Candidate-only actual PixelRig proof. This is not a Game asset adapter.
use engine_core::{
    AffineTransform, FrameBuilder, PixelBody, PixelRig, PixelRigPart, PixelRigState, Vec2,
};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;

#[derive(Deserialize)]
struct Pack {
    palette: Vec<u32>,
    sprites: HashMap<String, Sprite>,
    pixel_rig: Rig,
}
#[derive(Deserialize)]
struct Sprite {
    width: usize,
    height: usize,
    indices: Vec<u8>,
}
#[derive(Deserialize)]
struct Rig {
    width: usize,
    height: usize,
    roles: Vec<u8>,
    parts: Vec<PixelRigPart>,
    frame_sprites: Vec<String>,
    lamp_cells: Vec<usize>,
}

fn pack() -> Pack {
    serde_json::from_str(include_str!(
        "../../../../../assets/pipeline/pan-coop-dock/pack.json"
    ))
    .unwrap()
}
fn body(pack: &Pack, name: &str) -> PixelBody {
    let sprite = &pack.sprites[name];
    assert_eq!(sprite.indices.len(), sprite.width * sprite.height);
    let key = |i: usize| char::from_u32(0xe000 + i as u32).unwrap();
    let palette: Vec<_> = pack
        .palette
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, &c)| (key(i), c))
        .collect();
    let rows: Vec<String> = sprite
        .indices
        .chunks_exact(sprite.width)
        .map(|row| {
            row.iter()
                .map(|&i| if i == 0 { ' ' } else { key(i as usize) })
                .collect()
        })
        .collect();
    let rows: Vec<_> = rows.iter().map(String::as_str).collect();
    PixelBody::from_ascii(&rows, &palette)
}
fn rig(pack: &Pack) -> PixelRig {
    PixelRig::new(
        pack.pixel_rig.width,
        pack.pixel_rig.height,
        pack.pixel_rig.roles.clone(),
        pack.pixel_rig.parts.clone(),
    )
    .unwrap()
}
// FrameBuilder itself has no pixel/transform quota or rollback. A host must do
// this complete local preflight before either draw; the candidate ceiling is
// not a new global engine limit.
fn admitted(body: &PixelBody, parent: AffineTransform, pixels: usize, transforms: usize) -> bool {
    if body.occupied_count() > pixels || transforms < 4 {
        return false;
    }
    body.cells().all(|(local, _)| {
        [
            Vec2::splat(-0.5),
            Vec2::new(0.5, -0.5),
            Vec2::splat(0.5),
            Vec2::new(-0.5, 0.5),
        ]
        .into_iter()
        .all(|corner| {
            let p = parent.point(local + corner);
            p.is_finite() && p.abs().max_element() <= 1000.0
        })
    })
}
fn parent() -> AffineTransform {
    // Continuous fractional fit; reflected/sheared and nonuniform parents are
    // composed as affine matrices, not decomposed back into lossy TRS.
    AffineTransform {
        position: Vec2::new(8.0, -6.0),
        basis_x: Vec2::new(-0.75, 0.12),
        basis_y: Vec2::new(0.18, 0.65),
    }
}
fn extract(pack: &Pack, n: usize) -> serde_json::Value {
    let body = body(pack, &pack.pixel_rig.frame_sprites[n]);
    let rig = rig(pack);
    let parent = parent();
    assert!(admitted(&body, parent, 8000, 4));
    let mut builder = FrameBuilder::new();
    let count = rig.draw_affine(
        &mut builder,
        &body,
        parent,
        PixelRigState {
            charge: 0.2,
            time: 0.0,
            ..PixelRigState::default()
        },
    );
    let frame = builder.finish();
    assert_eq!(count, body.occupied_count());
    assert_eq!(frame.pixel_count, count);
    let mut cells = Vec::new();
    for batch in &frame.batches {
        for p in &batch.pixels {
            let t = batch.transforms[p.transform as usize];
            let local = Vec2::from_array(p.local);
            let corners = [
                Vec2::splat(-0.5),
                Vec2::new(0.5, -0.5),
                Vec2::splat(0.5),
                Vec2::new(-0.5, 0.5),
            ]
            .map(|c| t.point(local + c).to_array());
            cells.push(json!({"corners":corners,"color":p.color,"light_response":t.b[2],"emission":t.b[3]}));
        }
    }
    let bounds = |axis: usize, max: bool| {
        cells
            .iter()
            .flat_map(|c| c["corners"].as_array().unwrap())
            .map(|p| p[axis].as_f64().unwrap())
            .fold(
                if max {
                    f64::NEG_INFINITY
                } else {
                    f64::INFINITY
                },
                if max { f64::max } else { f64::min },
            )
    };
    json!({"group":format!("actual-rig-{n}"),"duration_ms":700,"occupied_cells":count,"transforms":frame.batches.iter().map(|b|b.transforms.len()).sum::<usize>(),"batches":frame.batches.len(),"bounds":[bounds(0,false),bounds(1,false),bounds(0,true),bounds(1,true)],"cells":cells})
}
fn main() {
    let output = std::env::args().nth(1).expect("output path required");
    let pack = pack();
    let result = json!({"scope":"Actual PixelRig/PixelBody/FrameBuilder extraction; flat software review only; no GPU/Game/co-op acceptance", "lamp_cells":pack.pixel_rig.lamp_cells.len(), "state":{"time":0,"charge":0.2,"core_open":false,"hit_flash":0},"parent":{"position":[8,-6],"basis_x":[-0.75,0.12],"basis_y":[0.18,0.65]},"frames":(0..3).map(|n|extract(&pack,n)).collect::<Vec<_>>()});
    std::fs::write(output, serde_json::to_string(&result).unwrap() + "\n").unwrap();
    println!(
        "Actual rig:3 frames /7100 cells each /4 transforms; color states supplied by candidate host"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_rig_states_materials_geometry_and_lamp_only_changes() {
        let pack = pack();
        let frames: Vec<_> = (0..3).map(|n| extract(&pack, n)).collect();
        let expected = body(&pack, "dock-0");
        let rig = rig(&pack);
        let mut ordered = Vec::new();
        for part in 0..4 {
            ordered.extend(
                expected
                    .cells_indexed()
                    .filter(|(i, _, _)| pack.pixel_rig.roles[*i] == part),
            );
        }
        for (index, ((cell, local, _), actual)) in ordered
            .iter()
            .zip(frames[0]["cells"].as_array().unwrap())
            .enumerate()
        {
            for (n, offset) in [
                Vec2::splat(-0.5),
                Vec2::new(0.5, -0.5),
                Vec2::splat(0.5),
                Vec2::new(-0.5, 0.5),
            ]
            .into_iter()
            .enumerate()
            {
                let expected_corner = parent().point(*local + offset);
                let have = &actual["corners"][n];
                assert!((expected_corner.x - have[0].as_f64().unwrap() as f32).abs() < 0.0001);
                assert!((expected_corner.y - have[1].as_f64().unwrap() as f32).abs() < 0.0001);
            }
            for other in &frames[1..] {
                assert_eq!(actual["corners"], other["cells"][index]["corners"]);
                if actual["color"] != other["cells"][index]["color"] {
                    assert!(pack.pixel_rig.lamp_cells.contains(cell));
                }
            }
            let part = &rig.parts()[pack.pixel_rig.roles[*cell] as usize];
            if matches!(part.role, engine_core::PixelRigRole::Armor) {
                assert_eq!(actual["emission"].as_f64().unwrap(), 0.0);
            }
            if matches!(part.role, engine_core::PixelRigRole::Emitter) {
                assert!(actual["emission"].as_f64().unwrap() > 0.1);
            }
        }
        assert_ne!(frames[0]["cells"], frames[1]["cells"]);
        assert_ne!(frames[0]["cells"], frames[2]["cells"]);
        assert_eq!(expected.occupied_count(), 7100);
    }

    #[test]
    fn damaged_live_body_has_actual_holes_no_role_refill() {
        let pack = pack();
        let mut body = body(&pack, "dock-0");
        let lost = body.damage(Vec2::ZERO, 6.0);
        assert!(!lost.is_empty());
        let mut builder = FrameBuilder::new();
        let count = rig(&pack).draw_affine(&mut builder, &body, parent(), PixelRigState::default());
        assert_eq!(count, 7100 - lost.len());
        assert_eq!(builder.finish().pixel_count, count);
    }

    #[test]
    fn preflight_rejects_pixel_transform_and_finite_extent_before_append() {
        let pack = pack();
        let body = body(&pack, "dock-0");
        assert!(!admitted(&body, parent(), 7099, 4));
        assert!(!admitted(&body, parent(), 8000, 3));
        let mut bad = parent();
        bad.position.x = f32::NAN;
        assert!(!admitted(&body, bad, 8000, 4));
        bad = parent();
        bad.position.x = 1000.0;
        assert!(!admitted(&body, bad, 8000, 4));
        let frame = FrameBuilder::new().finish();
        assert_eq!(frame.pixel_count, 0);
    }

    #[test]
    fn transform_table_boundary_splits_batches_without_losing_group() {
        let pack = pack();
        let one = PixelBody::from_ascii(&["X"], &[('X', u32::MAX)]);
        let mut builder = FrameBuilder::new();
        for _ in 0..127 {
            builder.push_body_affine(&one, parent());
        }
        rig(&pack).draw_affine(
            &mut builder,
            &body(&pack, "dock-0"),
            parent(),
            PixelRigState::default(),
        );
        let frame = builder.finish();
        assert_eq!(frame.pixel_count, 7227);
        assert_eq!(frame.batches.len(), 2);
        for batch in frame.batches {
            assert!(batch.transforms.len() <= 128);
            assert!(
                batch
                    .pixels
                    .iter()
                    .all(|p| (p.transform as usize) < batch.transforms.len())
            );
        }
    }
}
