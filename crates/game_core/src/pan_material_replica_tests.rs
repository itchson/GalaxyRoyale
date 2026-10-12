use super::*;

fn pan_fixture() -> (Game, u32) {
    let mut profile = identity::create_profile("talo", "pan", "renko", "0123456789abcdef").unwrap();
    profile.seed = 812345;
    let mut game = Game::new(profile.clone());
    game.begin_co_op(PROTOCOL, id(100), id(1)).unwrap();
    let epoch = game
        .join_co_op(
            id(100),
            ParticipantSpec {
                id: id(2),
                local_seed: 9987,
                identity: profile.identity,
            },
        )
        .unwrap()
        .epoch;
    let remove: Vec<_> = game
        .world
        .query::<(hecs::Entity, Option<&co_op::Pilot>)>()
        .iter()
        .filter(|(_, p)| p.is_none())
        .map(|(e, _)| e)
        .collect();
    for e in remove {
        game.world.despawn(e).unwrap();
    }
    let poses: Vec<_> = game
        .world
        .query::<(hecs::Entity, &co_op::Pilot)>()
        .iter()
        .map(|(e, p)| (e, p.admission.slot))
        .collect();
    for (e, slot) in poses {
        *game.world.get::<&mut Motion>(e).unwrap() = Motion::new(
            Vec2::new(450.0, slot as f32 * 150.0),
            Vec2::ZERO,
            -std::f32::consts::FRAC_PI_2,
        );
    }
    game.camera = Vec2::new(450.0, 75.0);
    game.previous_camera = game.camera;
    game.viewport = Vec2::new(600.0, 400.0);
    game.reduced_motion = true;
    (game, epoch)
}
fn material_cells(frame: &engine_core::RenderFrame) -> Vec<([u32; 2], u32, u32, [u32; 4])> {
    let mut cells: Vec<_> = frame
        .batches
        .iter()
        .flat_map(|b| {
            b.pixels.iter().filter(|p| p.surface().is_some()).map(|p| {
                let t = b.transforms[p.transform_slot()];
                let world = t.point(Vec2::from_array(p.local));
                (
                    world.to_array().map(f32::to_bits),
                    p.color,
                    p.surface().unwrap().packed(),
                    [
                        t.a[2].to_bits(),
                        t.a[3].to_bits(),
                        t.b[2].to_bits(),
                        t.b[3].to_bits(),
                    ],
                )
            })
        })
        .collect();
    cells.sort_unstable();
    cells
}
#[test]
fn actual_host_and_complete_native_guest_render_reconstruct_identical_pan_fields() {
    let (mut game, epoch) = pan_fixture();
    let profile = serde_json::to_string(game.profile()).unwrap();
    let data = capture(&mut game);
    let mut r = replica(epoch);
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    let scene = r.scene().unwrap();
    assert!(scene.entities().iter().all(|(_, b)| b.surfaces().is_none()));
    let mut raw = FrameBuilder::new();
    scene.append_occupied(&mut raw, 1.0);
    let raw = raw.finish();
    assert_eq!(raw.pixel_count, 1324);
    assert!(
        raw.batches
            .iter()
            .flat_map(|b| &b.pixels)
            .all(|p| p.surface().is_none())
    );
    let mut guest = FrameBuilder::new();
    scene
        .render_into_with_home(
            &mut guest,
            ViewInput {
                camera: game.camera,
                viewport: game.viewport,
                alpha: 1.0,
                reduced: false,
                reduced_motion: true,
            },
            |_, _, _| false,
        )
        .unwrap();
    let host = game.render(1.0);
    let guest = guest.finish();
    assert_eq!(material_cells(&host).len(), 3734);
    assert_eq!(material_cells(&host), material_cells(&guest));
    assert_eq!(profile, serde_json::to_string(game.profile()).unwrap());
    assert!(
        !serde_json::to_string(&data.header)
            .unwrap()
            .contains("surface")
    );
}
#[test]
fn damaged_guest_import_keeps_real_coarse_holes_and_prunes_every_relative_material_cell() {
    let (mut game, epoch) = pan_fixture();
    let peer = game
        .world
        .query::<(hecs::Entity, &co_op::Pilot)>()
        .iter()
        .find(|(_, p)| p.admission.id == id(2))
        .unwrap()
        .0;
    let removed = {
        let mut body = game.world.get::<&mut PixelBody>(peer).unwrap();
        let hit = body
            .hit_segment(Vec2::new(-20.0, -8.0), Vec2::new(20.0, 8.0))
            .unwrap();
        body.damage(hit, 3.5)
    };
    assert!(!removed.is_empty());
    let data = capture(&mut game);
    let mut r = replica(epoch);
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    let scene = r.scene().unwrap();
    let mut guest = FrameBuilder::new();
    scene
        .render_into_with_home(
            &mut guest,
            ViewInput {
                camera: game.camera,
                viewport: game.viewport,
                alpha: 1.0,
                reduced: false,
                reduced_motion: true,
            },
            |_, _, _| false,
        )
        .unwrap();
    let host = game.render(1.0);
    let guest = guest.finish();
    assert!(material_cells(&guest).len() < 3734);
    assert_eq!(material_cells(&host), material_cells(&guest));
    let imported = &scene
        .entities()
        .iter()
        .find(|(e, _)| e.participant == Some(id(2)))
        .unwrap()
        .1;
    assert_eq!(imported.occupied_count(), 662 - removed.len());
    assert_eq!(
        body_signature(imported),
        data.entities
            .iter()
            .flat_map(|p| &p.entities)
            .find(|e| e.participant == Some(id(2)))
            .unwrap()
            .body_signature
    );
}
