//! Actual tiled PixelRig proof; no runtime/Game asset registration.
use engine_core::{
    AffineTransform, CameraView, FrameBuilder, PixelBody, PixelRig, PixelRigPart, PixelRigState,
    RenderFrame, Vec2,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Deserialize)]
struct Pack {
    palette: Vec<u32>,
    sprites: HashMap<String, Sprite>,
    pixel_rig_tiles: Tiled,
}
#[derive(Deserialize)]
struct Sprite {
    width: usize,
    height: usize,
    indices: Vec<u8>,
}
#[derive(Deserialize)]
struct Tiled {
    tiles: Vec<Tile>,
    lamp_cells_source: Vec<usize>,
}
#[derive(Deserialize)]
struct Tile {
    width: usize,
    height: usize,
    position: [f32; 2],
    roles: Vec<u8>,
    parts: Vec<PixelRigPart>,
    frame_sprites: Vec<String>,
}
fn pack() -> Pack {
    serde_json::from_str(include_str!(
        "../../../../../assets/pipeline/pan-coop-dock/native256/pack.json"
    ))
    .unwrap()
}
fn body(pack: &Pack, name: &str) -> PixelBody {
    let s = &pack.sprites[name];
    assert_eq!((s.width, s.height), (128, 128));
    let key = |i: usize| char::from_u32(0xe000 + i as u32).unwrap();
    let colors: Vec<_> = pack
        .palette
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, &c)| (key(i), c))
        .collect();
    let rows: Vec<String> = s
        .indices
        .chunks_exact(128)
        .map(|r| {
            r.iter()
                .map(|&i| if i == 0 { ' ' } else { key(i as usize) })
                .collect()
        })
        .collect();
    let rows: Vec<_> = rows.iter().map(String::as_str).collect();
    PixelBody::from_ascii(&rows, &colors)
}
fn rig(t: &Tile) -> PixelRig {
    PixelRig::new(t.width, t.height, t.roles.clone(), t.parts.clone()).unwrap()
}
fn source_parent() -> AffineTransform {
    AffineTransform {
        position: Vec2::ZERO,
        basis_x: Vec2::new(0.5, 0.),
        basis_y: Vec2::new(0., 0.5),
    }
}
fn composed(parent: AffineTransform, t: &Tile) -> AffineTransform {
    parent.compose(source_parent()).compose(AffineTransform {
        position: Vec2::from_array(t.position),
        basis_x: Vec2::X,
        basis_y: Vec2::Y,
    })
}
fn selected_frame(ms: u64, reduced: bool) -> usize {
    if reduced {
        0
    } else {
        ((ms / 700) % 3) as usize
    }
}
// Deliberately a local proof-host quota; FrameBuilder has no global cap/rollback.
fn append(
    builder: &mut FrameBuilder,
    pack: &Pack,
    bodies: &[PixelBody],
    parent: AffineTransform,
    pixels: usize,
    transforms: usize,
) -> Option<usize> {
    let count: usize = bodies.iter().map(PixelBody::occupied_count).sum();
    if bodies.len() != 4 || count > pixels || transforms < 16 {
        return None;
    }
    for (t, b) in pack.pixel_rig_tiles.tiles.iter().zip(bodies) {
        let p = composed(parent, t);
        if !b.cells().all(|(c, _)| {
            [
                Vec2::splat(-0.5),
                Vec2::new(0.5, -0.5),
                Vec2::splat(0.5),
                Vec2::new(-0.5, 0.5),
            ]
            .into_iter()
            .all(|d| {
                let v = p.point(c + d);
                v.is_finite() && v.abs().max_element() <= 1000.
            })
        }) {
            return None;
        }
    }
    let mut emitted = 0;
    for (t, b) in pack.pixel_rig_tiles.tiles.iter().zip(bodies) {
        emitted += rig(t).draw_affine(
            builder,
            b,
            composed(parent, t),
            PixelRigState {
                time: 0.,
                charge: 0.2,
                ..PixelRigState::default()
            },
        );
    }
    assert_eq!(count, emitted);
    Some(count)
}
fn draw(pack: &Pack, n: usize, parent: AffineTransform) -> RenderFrame {
    let bodies: Vec<_> = pack
        .pixel_rig_tiles
        .tiles
        .iter()
        .map(|t| body(pack, &t.frame_sprites[n]))
        .collect();
    let mut builder = FrameBuilder::new();
    assert_eq!(
        append(&mut builder, pack, &bodies, parent, 32000, 16),
        Some(28549)
    );
    builder.finish()
}
fn frame_json(frame: &RenderFrame, n: usize, view: Option<CameraView>) -> Value {
    let mut cells = Vec::new();
    for b in &frame.batches {
        for p in &b.pixels {
            let t = b.transforms[p.transform as usize];
            let c = Vec2::from_array(p.local);
            let corners = [
                Vec2::splat(-0.5),
                Vec2::new(0.5, -0.5),
                Vec2::splat(0.5),
                Vec2::new(-0.5, 0.5),
            ]
            .map(|d| {
                let v = t.point(c + d);
                view.map_or(v, |camera| camera.world_to_screen(v))
                    .to_array()
            });
            cells.push(json!({"corners":corners,"color":p.color,"light_response":t.b[2],"emission":t.b[3]}));
        }
    }
    let bound = |axis: usize, max: bool| {
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
    json!({"group":format!("tiled-rig-{n}"),"duration_ms":700,"occupied_cells":frame.pixel_count,"transforms":frame.batches.iter().map(|b|b.transforms.len()).sum::<usize>(),"batches":frame.batches.len(),"bounds":[bound(0,false),bound(1,false),bound(0,true),bound(1,true)],"cells":cells})
}
fn main() {
    let folder = std::env::args().nth(1).expect("output directory required");
    let folder = std::path::Path::new(&folder);
    let pack = pack();
    let frames: Vec<_> = (0..3)
        .map(|n| {
            draw(
                &pack,
                selected_frame(n * 700, false),
                AffineTransform::default(),
            )
        })
        .collect();
    let world = json!({"scope":"Actual4PixelRigs and FrameBuilder,16transforms; candidate only, no Game/GPU acceptance","source_cell_scale":0.5,"lamp_cells":pack.pixel_rig_tiles.lamp_cells_source.len(),"frames":frames.iter().enumerate().map(|(n,f)|frame_json(f,n,None)).collect::<Vec<_>>()});
    std::fs::write(
        folder.join("rig-extraction.json"),
        serde_json::to_string(&world).unwrap() + "\n",
    )
    .unwrap();
    let projected_frames: Vec<_> = [0.6, 1.0, 1.6]
        .into_iter()
        .enumerate()
        .map(|(n, z)| {
            let mut projected = frame_json(
                &frames[0],
                n,
                Some(CameraView::new(Vec2::ZERO, Vec2::new(320., 240.), z).unwrap()),
            );
            projected["group"] = format!("CameraView zoom{z:.1} /state0").into();
            projected
        })
        .collect();
    let zoom = json!({"scope":"ActualCameraView corner projection from actual28549cell rig frame; flat software review, not GPU sampling/lighting","logical_viewport":[320,240],"zooms":[0.6,1.,1.6],"frames":projected_frames});
    std::fs::write(
        folder.join("zoom-extraction.json"),
        serde_json::to_string(&zoom).unwrap() + "\n",
    )
    .unwrap();
    println!("Actual4rigs:3states and3CameraView zooms;28549cells/16transforms perframe");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tiled_rigs_emit_every_fine_cell_once_at_half_scale() {
        let p = pack();
        let f = draw(&p, 0, AffineTransform::default());
        assert_eq!(f.pixel_count, 28549);
        assert_eq!(
            f.batches.iter().map(|b| b.transforms.len()).sum::<usize>(),
            16
        );
        let mut seen = std::collections::HashSet::new();
        for b in f.batches {
            for q in b.pixels {
                let t = b.transforms[q.transform as usize];
                let c = t.point(Vec2::from_array(q.local));
                assert!(seen.insert(((c.x * 4.).round() as i32, (c.y * 4.).round() as i32)));
                assert_eq!(t.a[2], 0.5);
                assert_eq!(t.b[1], 0.5);
            }
        }
    }
    #[test]
    fn actual_nested_reflected_shear_corners_and_zoom_remain_finite_continuous() {
        let p = pack();
        let parent = AffineTransform {
            position: Vec2::new(11., -7.),
            basis_x: Vec2::new(-0.8, 0.13),
            basis_y: Vec2::new(0.21, 0.65),
        };
        let f = draw(&p, 0, parent);
        let mut offset = 0;
        for tile in &p.pixel_rig_tiles.tiles {
            let body = body(&p, &tile.frame_sprites[0]);
            for k in 0..4 {
                for (i, local, _) in body.cells_indexed().filter(|(i, _, _)| tile.roles[*i] == k) {
                    let actual = &f.batches[0].pixels[offset];
                    let transform = f.batches[0].transforms[actual.transform as usize];
                    for corner in [
                        Vec2::splat(-0.5),
                        Vec2::new(0.5, -0.5),
                        Vec2::splat(0.5),
                        Vec2::new(-0.5, 0.5),
                    ] {
                        let wanted =
                            parent.point((local + corner + Vec2::from_array(tile.position)) * 0.5);
                        let have = transform.point(Vec2::from_array(actual.local) + corner);
                        assert!(wanted.distance(have) < 0.0001, "cell{i}");
                        for z in [0.6, 1., 1.6] {
                            let v = CameraView::new(Vec2::ZERO, Vec2::new(320., 240.), z).unwrap();
                            assert!(v.world_to_screen(have).is_finite());
                            assert!(
                                v.screen_to_world(v.world_to_screen(have)).distance(have) < 0.0001
                            );
                        }
                    }
                    offset += 1;
                }
            }
        }
        assert_eq!(offset, 28549);
    }
    #[test]
    fn three_states_keep_geometry_and_only_lamp_colors_change() {
        let p = pack();
        let a = frame_json(&draw(&p, 0, AffineTransform::default()), 0, None);
        let b = frame_json(&draw(&p, 1, AffineTransform::default()), 1, None);
        let c = frame_json(&draw(&p, 2, AffineTransform::default()), 2, None);
        let mut offset = 0;
        let mut changed = [0, 0];
        for (n, t) in p.pixel_rig_tiles.tiles.iter().enumerate() {
            let body = body(&p, &t.frame_sprites[0]);
            let x0 = (n % 2) * 128;
            let y0 = (n / 2) * 128;
            for role in 0..4 {
                for (i, _, _) in body.cells_indexed().filter(|(i, _, _)| t.roles[*i] == role) {
                    let global = (y0 + i / 128) * 256 + x0 + i % 128;
                    for (state, o) in [&b, &c].into_iter().enumerate() {
                        assert_eq!(a["cells"][offset]["corners"], o["cells"][offset]["corners"]);
                        if a["cells"][offset]["color"] != o["cells"][offset]["color"] {
                            assert!(p.pixel_rig_tiles.lamp_cells_source.contains(&global));
                            changed[state] += 1;
                        }
                    }
                    offset += 1;
                }
            }
        }
        assert!(changed.iter().all(|c| *c > 1000));
        for ms in [0, 699, 700, 1400, 2100, u64::MAX] {
            assert_eq!(selected_frame(ms, true), 0);
        }
        assert_eq!(selected_frame(1400, false), 2);
    }
    #[test]
    fn whole_group_preflight_and_damage_never_recreate_missing_cells() {
        let p = pack();
        let mut bodies: Vec<_> = p
            .pixel_rig_tiles
            .tiles
            .iter()
            .map(|t| body(&p, &t.frame_sprites[0]))
            .collect();
        let mut builder = FrameBuilder::new();
        assert_eq!(
            append(
                &mut builder,
                &p,
                &bodies,
                AffineTransform::default(),
                28548,
                16
            ),
            None
        );
        assert_eq!(
            append(
                &mut builder,
                &p,
                &bodies,
                AffineTransform::default(),
                32000,
                15
            ),
            None
        );
        assert_eq!(builder.finish().pixel_count, 0);
        let mut lost = 0;
        for (t, b) in p.pixel_rig_tiles.tiles.iter().zip(&mut bodies) {
            lost += b.damage(-Vec2::from_array(t.position), 6.).len();
        }
        assert!(lost > 0);
        let mut builder = FrameBuilder::new();
        assert_eq!(
            append(
                &mut builder,
                &p,
                &bodies,
                AffineTransform::default(),
                32000,
                16
            ),
            Some(28549 - lost)
        );
        assert_eq!(builder.finish().pixel_count, 28549 - lost);
        let mut bad = AffineTransform::default();
        bad.position.x = f32::NAN;
        let mut builder = FrameBuilder::new();
        assert_eq!(append(&mut builder, &p, &bodies, bad, 32000, 16), None);
        assert_eq!(builder.finish().pixel_count, 0);
    }
    #[test]
    fn all_sixteen_transforms_cross_table_without_truncation() {
        let p = pack();
        let bodies: Vec<_> = p
            .pixel_rig_tiles
            .tiles
            .iter()
            .map(|t| body(&p, &t.frame_sprites[0]))
            .collect();
        let one = PixelBody::from_ascii(&["X"], &[('X', u32::MAX)]);
        let mut builder = FrameBuilder::new();
        for _ in 0..127 {
            builder.push_body_affine(&one, AffineTransform::default());
        }
        assert_eq!(
            append(
                &mut builder,
                &p,
                &bodies,
                AffineTransform::default(),
                32000,
                16
            ),
            Some(28549)
        );
        let f = builder.finish();
        assert_eq!(f.pixel_count, 28676);
        assert_eq!(f.batches.len(), 2);
        assert_eq!(
            f.batches.iter().map(|b| b.transforms.len()).sum::<usize>(),
            143
        );
    }
}
