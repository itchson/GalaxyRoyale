use super::*;
use crate::{
    co_op::{self, PROTOCOL, ParticipantSpec},
    *,
};
use serde_json::{Value, json};

#[path = "pan_material_replica_tests.rs"]
mod pan_material;

#[path = "co_op_fx_tests.rs"]
mod nova_fx;

fn id(n: u32) -> ParticipantId {
    ParticipantId::parse(&format!("{n:032x}")).unwrap()
}
fn fixture(players: u32) -> (Game, u32) {
    let mut game = Game::new(Profile {
        seed: 812345,
        ..Profile::default()
    });
    game.begin_co_op(PROTOCOL, id(100), id(1)).unwrap();
    let mut epoch = 0;
    for n in 2..=players {
        let profile =
            crate::identity::create_profile("talo", "hugo", "renko", &format!("{n:016x}")).unwrap();
        let a = game
            .join_co_op(
                id(100),
                ParticipantSpec {
                    id: id(n),
                    local_seed: 98765 + n,
                    identity: profile.identity,
                },
            )
            .unwrap();
        if n == 2 {
            epoch = a.epoch;
        }
    }
    // Explicit small ECS fixture, retaining real admitted player components.
    let remove: Vec<_> = game
        .world
        .query::<(hecs::Entity, Option<&co_op::Pilot>)>()
        .iter()
        .filter(|(_, pilot)| pilot.is_none())
        .map(|(e, _)| e)
        .collect();
    for e in remove {
        game.world.despawn(e).unwrap();
    }
    game.spawn_timer = 10000.0;
    (game, epoch)
}
fn object(game: &mut Game) -> hecs::Entity {
    game.world.spawn((
        Motion::new(Vec2::new(310.0, 40.0), Vec2::new(4.0, 2.0), 0.3),
        Actor {
            kind: Kind::Asteroid,
            health: 10.0,
            max_health: 10.0,
            scale: 0.625,
            radius: 2.0,
            damage: 0.0,
        },
        PixelBody::from_indexed_rgba(
            3,
            2,
            &[
                [0, rgba(20, 30, 40, 255)],
                [2, rgba(50, 60, 70, 0)],
                [4, rgba(80, 90, 100, 255)],
            ],
            6,
        )
        .unwrap(),
    ))
}
#[derive(Clone)]
struct Data {
    header: Header,
    entities: Vec<EntityPage>,
    bodies: Vec<BodyPage>,
}
fn dto<T: serde::de::DeserializeOwned>(value: &impl Serialize) -> T {
    serde_json::from_value(serde_json::to_value(value).unwrap()).unwrap()
}
fn capture(game: &mut Game) -> Data {
    let view = game.acquire_co_op_publication(id(100)).unwrap();
    let token = view.acquisition;
    let rev = view.snapshot.revision;
    let header = dto(&view);
    let published = game.co_op_publication(id(100), token, rev).unwrap();
    let entities = (0..published.counts().entities)
        .step_by(64)
        .map(|offset| dto(&published.entity_page(offset).unwrap()))
        .collect();
    let mut bodies = Vec::new();
    for row in published.entities() {
        for offset in (0..row.occupied).step_by(1024) {
            bodies.push(dto(&published.body_page(&row.id, offset).unwrap()));
        }
    }
    Data {
        header,
        entities,
        bodies,
    }
}
fn replica(epoch: u32) -> Replica {
    Replica::new(id(100), id(2), epoch, 812345).unwrap()
}
fn stage(replica: &mut Replica, data: &Data, with_bodies: bool) {
    replica.stage(data.header.clone()).unwrap();
    for page in &data.entities {
        replica
            .entities(data.header.acquisition, page.clone())
            .unwrap();
    }
    if with_bodies {
        for page in &data.bodies {
            replica.body(data.header.acquisition, page.clone()).unwrap();
        }
    }
}
fn commit(replica: &mut Replica, data: &Data) {
    replica
        .commit(data.header.acquisition, data.header.snapshot.revision)
        .unwrap();
}
fn key(e: hecs::Entity) -> String {
    format!("{:016x}", e.to_bits().get())
}
type CellGeometry = ([f32; 2], u32, [f32; 2], [f32; 2], [f32; 2]);
fn geometry(frame: &engine_core::RenderFrame) -> Vec<CellGeometry> {
    frame
        .batches
        .iter()
        .flat_map(|b| {
            b.pixels.iter().map(|p| {
                let t = b.transforms[p.transform_slot()];
                (
                    t.point(Vec2::from_array(p.local)).to_array(),
                    p.color,
                    [t.a[2], t.a[3]],
                    [t.b[0], t.b[1]],
                    [t.b[2], t.b[3]],
                )
            })
        })
        .collect()
}
fn view() -> ViewInput {
    ViewInput {
        camera: Vec2::ZERO,
        viewport: Vec2::new(800.0, 600.0),
        alpha: 0.5,
        reduced: false,
        reduced_motion: true,
    }
}

#[test]
fn actual_two_and_eight_player_publications_keep_holes_and_transparent_occupancy() {
    for players in [2, 8] {
        let (mut game, epoch) = fixture(players);
        let target = object(&mut game);
        let data = capture(&mut game);
        let mut r = replica(epoch);
        stage(&mut r, &data, true);
        commit(&mut r, &data);
        let scene = r.scene().unwrap();
        assert_eq!(scene.header().snapshot.participants.len(), players as usize);
        assert_eq!(scene.entities().len(), players as usize + 1);
        let expected = game
            .co_op_publication(
                id(100),
                data.header.acquisition,
                data.header.snapshot.revision,
            )
            .unwrap();
        for (row, body) in scene.entities() {
            let source = expected.body(&row.id).unwrap();
            assert_eq!(
                body.cells_indexed()
                    .map(|(i, _, c)| (i, c))
                    .collect::<Vec<_>>(),
                source
                    .cells_indexed()
                    .map(|(i, _, c)| (i, c))
                    .collect::<Vec<_>>()
            );
            assert_eq!(body_signature(body), row.body_signature);
        }
        let body = &scene
            .entities()
            .iter()
            .find(|(e, _)| e.id == key(target))
            .unwrap()
            .1;
        assert_eq!(body.occupied_count(), 3);
        assert_eq!(
            body.cells_indexed()
                .find(|(i, _, _)| *i == 2)
                .unwrap()
                .2
                .to_le_bytes()[3],
            0
        );
        assert!(!body.cells_indexed().any(|(i, _, _)| i == 1));
    }
}
#[test]
fn acquisition_session_seed_revision_and_membership_bind_before_any_replacement() {
    let (mut game, epoch) = fixture(2);
    let first = capture(&mut game);
    let mut r = replica(epoch);
    stage(&mut r, &first, true);
    commit(&mut r, &first);
    let second = capture(&mut game); // same native tick/revision, new acquisition.
    r.stage(second.header.clone()).unwrap();
    assert_eq!(
        r.entities(first.header.acquisition, first.entities[0].clone()),
        Err("stale_page")
    );
    for change in 0..6 {
        let mut bad = second.header.clone();
        bad.acquisition += 1;
        match change {
            0 => bad.snapshot.session = id(200),
            1 => bad.snapshot.world_seed = 98767,
            2 => bad.snapshot.participants[1].epoch += 1,
            3 => bad.snapshot.progression = true,
            4 => bad.snapshot.participants[1].id = bad.snapshot.participants[0].id,
            _ => bad.snapshot.participants[1].slot = 0,
        }
        assert!(r.stage(bad).is_err());
        assert_eq!(
            r.scene().unwrap().header().acquisition,
            first.header.acquisition
        );
    }
    for p in &second.entities {
        r.entities(second.header.acquisition, p.clone()).unwrap();
    }
    commit(&mut r, &second); // exact matching old bodies can be retained.
    assert_eq!(r.stage(first.header.clone()), Err("invalid_publication"));
    assert!(Replica::new(id(100), id(2), 0, 812345).is_err());
    assert!(Replica::new(id(100), id(2), epoch, 0).is_err());
}
#[test]
fn failed_or_incomplete_transfer_keeps_committed_scene_and_can_resume() {
    let (mut game, epoch) = fixture(2);
    let target = object(&mut game);
    let first = capture(&mut game);
    let mut r = replica(epoch);
    stage(&mut r, &first, true);
    commit(&mut r, &first);
    *game.world.get::<&mut PixelBody>(target).unwrap() = PixelBody::from_indexed_rgba(
        3,
        2,
        &[
            [1, rgba(1, 2, 3, 255)],
            [2, rgba(4, 5, 6, 0)],
            [4, rgba(7, 8, 9, 255)],
        ],
        6,
    )
    .unwrap();
    let second = capture(&mut game);
    stage(&mut r, &second, false);
    assert_eq!(
        r.commit(second.header.acquisition, second.header.snapshot.revision),
        Err("missing_body")
    );
    assert_eq!(
        r.scene().unwrap().header().acquisition,
        first.header.acquisition
    );
    let page = second.bodies.iter().find(|p| p.id == key(target)).unwrap();
    let mut corrupt = page.clone();
    corrupt.cells[0][1] ^= 1;
    r.body(second.header.acquisition, corrupt).unwrap();
    assert_eq!(
        r.commit(second.header.acquisition, second.header.snapshot.revision),
        Err("body_signature_mismatch")
    );
    // New acquisition discards its rejected staging buffers; old view survives.
    let third = capture(&mut game);
    stage(&mut r, &third, false);
    let page = third.bodies.iter().find(|p| p.id == key(target)).unwrap();
    let mut prefix = page.clone();
    prefix.cells.truncate(1);
    prefix.next_offset = Some(1);
    r.body(third.header.acquisition, prefix).unwrap();
    assert_eq!(
        r.commit(third.header.acquisition, third.header.snapshot.revision),
        Err("incomplete_body")
    );
    let mut rest = page.clone();
    rest.offset = 1;
    rest.cells.remove(0);
    r.body(third.header.acquisition, rest).unwrap();
    commit(&mut r, &third);
    let body = &r
        .scene()
        .unwrap()
        .entities()
        .iter()
        .find(|(e, _)| e.id == key(target))
        .unwrap()
        .1;
    assert_eq!(body.cells_indexed().next().unwrap().0, 1);
}
#[test]
fn exact_cache_reuse_drops_removed_entities_and_invalidates_same_count_color_changes() {
    let (mut game, epoch) = fixture(2);
    let target = object(&mut game);
    let first = capture(&mut game);
    let mut r = replica(epoch);
    stage(&mut r, &first, true);
    commit(&mut r, &first);
    let original = Rc::clone(
        &r.scene()
            .unwrap()
            .entities()
            .iter()
            .find(|(e, _)| e.id == key(target))
            .unwrap()
            .1,
    );
    let second = capture(&mut game);
    stage(&mut r, &second, false);
    commit(&mut r, &second);
    assert!(Rc::ptr_eq(
        &original,
        &r.scene()
            .unwrap()
            .entities()
            .iter()
            .find(|(e, _)| e.id == key(target))
            .unwrap()
            .1
    ));
    *game.world.get::<&mut PixelBody>(target).unwrap() = PixelBody::from_indexed_rgba(
        3,
        2,
        &[
            [0, rgba(21, 30, 40, 255)],
            [2, rgba(50, 60, 70, 0)],
            [4, rgba(80, 90, 100, 255)],
        ],
        6,
    )
    .unwrap();
    let third = capture(&mut game);
    stage(&mut r, &third, false);
    assert_eq!(
        r.commit(third.header.acquisition, third.header.snapshot.revision),
        Err("missing_body")
    );
    let page = third.bodies.iter().find(|p| p.id == key(target)).unwrap();
    r.body(third.header.acquisition, page.clone()).unwrap();
    commit(&mut r, &third);
    assert!(!Rc::ptr_eq(
        &original,
        &r.scene()
            .unwrap()
            .entities()
            .iter()
            .find(|(e, _)| e.id == key(target))
            .unwrap()
            .1
    ));
    game.world.despawn(target).unwrap();
    let fourth = capture(&mut game);
    stage(&mut r, &fourth, false);
    commit(&mut r, &fourth);
    assert!(
        r.scene()
            .unwrap()
            .entities()
            .iter()
            .all(|(e, _)| e.id != key(target))
    );
}
#[test]
fn empty_bodies_use_exact_dimensions_and_verify_signature_without_a_page() {
    let (mut game, epoch) = fixture(2);
    let target = object(&mut game);
    *game.world.get::<&mut PixelBody>(target).unwrap() =
        PixelBody::from_indexed_rgba(3, 2, &[], 6).unwrap();
    let data = capture(&mut game);
    assert!(data.bodies.iter().all(|p| p.id != key(target)));
    let mut r = replica(epoch);
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    let body = &r
        .scene()
        .unwrap()
        .entities()
        .iter()
        .find(|(e, _)| e.id == key(target))
        .unwrap()
        .1;
    assert_eq!((body.width, body.height, body.occupied_count()), (3, 2, 0));
    let mut wrong = data.clone();
    wrong.header.acquisition += 1;
    wrong.entities[0]
        .entities
        .iter_mut()
        .find(|e| e.id == key(target))
        .unwrap()
        .body_signature = "0000000000000000".into();
    stage(&mut r, &wrong, false);
    assert_eq!(
        r.commit(wrong.header.acquisition, wrong.header.snapshot.revision),
        Err("body_signature_mismatch")
    );
}
#[test]
fn scalar_and_nested_json_guards_fail_without_admitting_a_profile() {
    let (mut game, epoch) = fixture(2);
    let data = capture(&mut game);
    let mut r = replica(epoch);
    for change in 0..8 {
        let mut h = data.header.clone();
        match change {
            0 => h.counts.entities = MAX_ENTITIES + 1,
            1 => h.counts.storage_cells = MAX_STORAGE_CELLS + 1,
            2 => h.counts.occupied_cells = h.counts.storage_cells + 1,
            3 => h.counts.entity_pages += 1,
            4 => {
                h.snapshot.projectiles = vec![
                    Projectile {
                        id: "0000000000000001".into(),
                        owner: None,
                        owner_epoch: None,
                        position: [0.0; 2],
                        previous: [0.0; 2],
                        velocity: [0.0; 2],
                        damage: 1.0,
                        life: 1.0,
                        remaining_hits: 1
                    };
                    181
                ]
            }
            5 => h.snapshot.participants[0].heading = f32::NAN,
            6 => h.snapshot.participants[1].position[0] = 1_000_001.0,
            _ => h.snapshot.participants[1].selected_ship = 9,
        }
        assert!(r.stage(h).is_err());
        assert!(r.scene().is_none());
        assert!(r.pending.is_none());
    }
    let mut encoded = serde_json::to_value(&data.header).unwrap();
    encoded["snapshot"]["participants"][1]["identity"]["currency"] = json!([999, 999, 999]);
    assert!(serde_json::from_value::<Header>(encoded).is_err());
    let mut encoded = serde_json::to_value(&data.header).unwrap();
    encoded["snapshot"]["profile"] = json!({"seed":1});
    assert!(serde_json::from_value::<Header>(encoded).is_err());
}
#[test]
fn entity_guards_keep_full_cell_transforms_and_reject_false_pilot_metadata() {
    let (mut game, epoch) = fixture(2);
    let target = object(&mut game);
    let data = capture(&mut game);
    let token = data.header.acquisition;
    for change in 0..7 {
        let mut r = replica(epoch);
        r.stage(data.header.clone()).unwrap();
        let mut page = data.entities[0].clone();
        let row = page
            .entities
            .iter_mut()
            .find(|e| e.id == key(target))
            .unwrap();
        match change {
            0 => row.scale = 0.0,
            1 => row.heading = f32::INFINITY,
            2 => row.width = 513,
            3 => row.hull = row.max_hull + 1.0,
            4 => row.participant = Some(id(2)),
            5 => row.body_signature = "bad".into(),
            _ => row.kind = "unknown".into(),
        };
        assert!(r.entities(token, page).is_err());
        assert!(r.pending.as_ref().unwrap().rows.is_empty());
    }
    let mut r = replica(epoch);
    let mut small = data.clone();
    small.entities[0]
        .entities
        .iter_mut()
        .find(|e| e.id == key(target))
        .unwrap()
        .scale = 0.00000001;
    stage(&mut r, &small, true);
    commit(&mut r, &small); // no authored-cell scale floor.
    let mut wrong = data.clone();
    wrong.header.acquisition += 1;
    wrong.entities[0]
        .entities
        .iter_mut()
        .find(|e| e.participant == Some(id(2)))
        .unwrap()
        .heading += 0.2;
    stage(&mut r, &wrong, true);
    assert_eq!(
        r.commit(wrong.header.acquisition, wrong.header.snapshot.revision),
        Err("pilot_mismatch")
    );
}
#[test]
fn body_pages_are_ordered_bounded_and_transactional() {
    let (mut game, epoch) = fixture(2);
    object(&mut game);
    let data = capture(&mut game);
    let token = data.header.acquisition;
    for change in 0..6 {
        let mut r = replica(epoch);
        stage(&mut r, &data, false);
        let mut page = data.bodies[0].clone();
        match change {
            0 => page.cells[1][0] = page.cells[0][0],
            1 => page.cells[0][0] = (page.width * page.height) as u32,
            2 => page.offset = 1,
            3 => page.width += 1,
            4 => page.next_offset = Some(42),
            _ => page.cells = vec![[0, 0]; 1025],
        };
        assert!(r.body(token, page).is_err());
        assert!(r.pending.as_ref().unwrap().cells.is_empty());
        assert_eq!(r.body(token + 1, data.bodies[0].clone()), Err("stale_page"));
        assert!(r.body(token, data.bodies[0].clone()).is_ok());
    }
}
#[test]
fn segment_unknown_maximum_is_narrow_and_real_page_compatible() {
    let (mut game, epoch) = fixture(2);
    game.spawn_enemy(2, false, Vec2::new(1800.0, 0.0), 1);
    let data = capture(&mut game);
    let mut r = replica(epoch);
    assert!(
        data.entities
            .iter()
            .flat_map(|p| &p.entities)
            .any(|e| e.kind == "segment" && e.hull > 0.0 && e.max_hull == 0.0)
    );
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    let mut wrong = data.entities[0].clone();
    let segment = wrong
        .entities
        .iter_mut()
        .find(|e| e.kind == "segment")
        .unwrap();
    segment.max_hull = 0.5;
    let mut next = data.header.clone();
    next.acquisition += 1;
    r.stage(next.clone()).unwrap();
    assert_eq!(r.entities(next.acquisition, wrong), Err("invalid_entity"));
}
#[test]
fn native_cells_match_real_actor_rotation_and_interpolation_at_every_corner() {
    let (mut game, epoch) = fixture(2);
    let target = object(&mut game);
    {
        let mut m = game.world.get::<&mut Motion>(target).unwrap();
        m.previous = Vec2::new(300.0, 30.0);
        m.previous_heading = 3.0;
        m.heading = -3.0;
    }
    let data = capture(&mut game);
    let mut r = replica(epoch);
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    let scene = r.scene().unwrap();
    for alpha in [0.0, 0.5, 1.0] {
        let mut frame = FrameBuilder::new();
        scene.append_occupied(&mut frame, alpha);
        let row = scene
            .entities()
            .iter()
            .find(|(e, _)| e.id == key(target))
            .unwrap();
        let expected = Transform {
            position: Vec2::from_array(row.0.previous)
                .lerp(Vec2::from_array(row.0.position), alpha),
            rotation: crate::interpolate_heading(3.0, -3.0, alpha) + FRAC_PI_2,
            scale: Vec2::splat(0.625),
        };
        let batch = &frame.frame().batches[0];
        let transform = batch.transforms.last().unwrap();
        for (_, cell, color) in row.1.cells_indexed() {
            let actual = batch.pixels.iter().find(|p| p.color == color).unwrap();
            for offset in [
                Vec2::new(-0.5, -0.5),
                Vec2::new(0.5, -0.5),
                Vec2::new(-0.5, 0.5),
                Vec2::new(0.5, 0.5),
            ] {
                assert!(
                    transform
                        .point(Vec2::from_array(actual.local) + offset)
                        .distance(expected.point(cell + offset))
                        < 0.00005
                );
            }
        }
    }
}
#[test]
fn host_seed_scenery_home_and_reduced_motion_match_the_real_native_prefix() {
    let (mut game, epoch) = fixture(2);
    game.set_viewport(800.0, 600.0);
    game.set_reduced_motion(true);
    let data = capture(&mut game);
    let mut r = replica(epoch);
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    let scene = r.scene().unwrap();
    let mut actual = FrameBuilder::new();
    scene
        .render_into_with_home(&mut actual, view(), |_, _, _| false)
        .unwrap();
    let mut bodies = FrameBuilder::new();
    scene.append_occupied(&mut bodies, view().alpha);
    let prefix = actual.frame().pixel_count - bodies.frame().pixel_count;
    let expected = game.render(view().alpha);
    assert_eq!(
        &geometry(actual.frame())[..prefix],
        &geometry(&expected)[..prefix]
    );
    assert_ne!(
        scene.world_seed(),
        scene.local_pilot(id(2)).unwrap().local_seed
    );
    let mut callback = 0;
    scene
        .render_into_with_home(&mut actual, view(), |_, station, time| {
            callback += 1;
            assert_eq!(station.scale, Vec2::splat(DOCK_SCALE));
            assert_eq!(time, 0.0);
            false
        })
        .unwrap();
    assert_eq!(callback, 1);
    let saved = geometry(actual.frame());
    let mut invalid = view();
    invalid.camera.x = f32::NAN;
    assert_eq!(
        scene.render_into_with_home(&mut actual, invalid, |_, _, _| false),
        Err("invalid_view")
    );
    assert_eq!(geometry(actual.frame()), saved);
}
#[test]
fn factory_guest_hud_clears_all_solo_progression_and_keeps_original_profile() {
    let (mut game, epoch) = fixture(2);
    let data = capture(&mut game);
    let mut r = replica(epoch);
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    let mut profile = Profile {
        currency: [21, 22, 23],
        ..Profile::default()
    };
    profile.campaign.cargo_materials = [3; 5];
    profile.campaign.stored_materials = [4; 5];
    profile.campaign.ship_xp = [999; 10];
    profile.campaign.patches = [true; 8];
    let solo = Game::new(profile);
    let original = serde_json::to_value(solo.profile()).unwrap();
    let before = solo.hud();
    let hud = r
        .scene()
        .unwrap()
        .guest_hud(id(2), Vec2::new(17.0, 11.0), before.clone())
        .unwrap();
    assert_eq!(hud.currency, [0; 3]);
    assert_eq!(hud.material_stash, [0; 5]);
    assert_eq!(hud.material_cargo, [0; 5]);
    assert_eq!(hud.ship_xp, 0);
    assert!(
        hud.quest_log.is_empty()
            && hud.campaign.champions.is_empty()
            && hud.components.is_empty()
            && hud.upgrades.is_empty()
            && hud.abilities.is_empty()
    );
    assert_eq!(
        hud.identity.unwrap().corporation,
        identity::Corporation::Hugo
    );
    assert_eq!(hud.loadout[1].as_ref().unwrap().index, 1);
    assert_eq!(
        (hud.hull, hud.max_hull, hud.max_shield),
        (100.0, 100.0, 30.0)
    );
    assert_eq!(hud.ship_stats.top_speed, 100.0);
    assert_eq!(hud.camera, [17.0, 11.0]);
    assert_eq!(hud.navigation.home, [0.0; 2]);
    assert!(hud.navigation.quest.is_none());
    assert_eq!(
        hud.biome,
        regions::at(game.profile.seed, Vec2::from_array(hud.position))
    );
    assert_eq!(serde_json::to_value(solo.profile()).unwrap(), original);
    assert_eq!(before.currency, [21, 22, 23]);
    assert_eq!(
        r.scene()
            .unwrap()
            .guest_hud(id(1), Vec2::ZERO, before)
            .unwrap_err(),
        "unknown_pilot"
    );
}
#[test]
fn end_invalidates_all_pages_cache_and_scene_without_touching_solo() {
    let (mut game, epoch) = fixture(2);
    let data = capture(&mut game);
    let mut r = replica(epoch);
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    assert_eq!(r.end(id(101)), Err("stale_session"));
    assert!(r.scene().is_some());
    r.end(id(100)).unwrap();
    assert!(r.scene().is_none() && r.pending.is_none());
    assert_eq!(r.stage(data.header.clone()), Err("invalid_publication"));
    assert_eq!(r.end(id(100)), Err("stale_session"));
    let mut next = Replica::new(id(101), id(2), epoch, 812345).unwrap();
    assert!(next.stage(data.header).is_err());
}
#[test]
fn completed_captures_blend_real_multitick_motion_without_extrapolation_or_header_mutation() {
    let (mut game, epoch) = fixture(2);
    let first = capture(&mut game);
    let mut r = replica(epoch);
    stage(&mut r, &first, true);
    commit(&mut r, &first);
    let old = first
        .header
        .snapshot
        .participants
        .iter()
        .find(|p| p.id == id(2))
        .unwrap()
        .position;
    game.submit_guest_co_op_input(
        id(100),
        id(2),
        epoch,
        1,
        InputFrame {
            thrust: Some([800.0, 0.0]),
            ..Default::default()
        },
    )
    .unwrap();
    for _ in 0..6 {
        game.tick_co_op(id(100)).unwrap();
    }
    let second = capture(&mut game);
    stage(&mut r, &second, true);
    commit(&mut r, &second);
    let scene = r.scene().unwrap();
    let current = scene.local_pilot(id(2)).unwrap();
    assert_ne!(old, current.previous);
    assert_ne!(old, current.position);
    assert_eq!(scene.blend_seconds(), 0.1);
    assert_eq!(scene.focus(id(2), 0.0).unwrap().player.to_array(), old);
    assert_eq!(
        scene.focus(id(2), 1.0).unwrap().player.to_array(),
        current.position
    );
    let half = Vec2::from_array(old).lerp(Vec2::from_array(current.position), 0.5);
    assert_eq!(scene.focus(id(2), 0.5).unwrap().player, half);
    let header = serde_json::to_value(scene.header()).unwrap();
    let mut frame = FrameBuilder::new();
    scene.append_occupied(&mut frame, 0.5);
    let index = scene
        .entities()
        .iter()
        .position(|(e, _)| e.participant == Some(id(2)))
        .unwrap();
    assert_eq!(
        [
            frame.frame().batches[0].transforms[index].a[0],
            frame.frame().batches[0].transforms[index].a[1]
        ],
        half.to_array()
    );
    scene.append_occupied(&mut FrameBuilder::new(), 2.0); // clamp; never predict beyond latest capture.
    assert_eq!(serde_json::to_value(scene.header()).unwrap(), header);
    assert!(scene.focus(id(2), 1.1).is_none());
}
#[test]
fn projectiles_keep_ownership_trails_and_only_explicit_bounded_lights() {
    let (mut game, epoch) = fixture(2);
    game.world.spawn((
        Motion::new(Vec2::new(30.0, 0.0), Vec2::X * 100.0, 0.0),
        Bullet {
            player: true,
            damage: 12.0,
            life: 0.5,
            remaining_hits: 1,
            previous_hit: None,
        },
        co_op::ShotOwner { id: id(2), epoch },
    ));
    game.world.spawn((
        Motion::new(Vec2::new(40.0, 0.0), -Vec2::X * 100.0, 0.0),
        Bullet {
            player: false,
            damage: 8.0,
            life: 0.5,
            remaining_hits: 1,
            previous_hit: None,
        },
    ));
    let data = capture(&mut game);
    let mut r = replica(epoch);
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    let scene = r.scene().unwrap();
    let mut frame = FrameBuilder::new();
    scene.append_occupied(&mut frame, 0.5);
    assert_eq!(
        frame.frame().pixel_count,
        data.header.counts.occupied_cells + 6
    );
    assert!(
        geometry(frame.frame())
            .iter()
            .any(|(p, c, _, _, _)| *p == [30.0, 0.0] && *c == rgba(255, 237, 156, 255))
    );
    assert!(
        geometry(frame.frame())
            .iter()
            .any(|(p, c, _, _, _)| *p == [40.0, 0.0] && *c == rgba(255, 93, 129, 255))
    );
    let lights = scene.light_sources(view());
    assert_eq!(lights.len(), 3);
    assert_eq!(lights[0].position, [30.0, 0.0]);
    assert_eq!(lights[1].position, [40.0, 0.0]);
    assert_eq!(lights[2].position, [0.0; 2]);
    assert!(scene.focus(id(2), 0.5).is_some());
    assert!(scene.focus(id(2), f32::NAN).is_none());
}

#[test]
#[ignore = "Explicit bounded fixture writer, not part of ordinary test execution"]
fn export_native_replica_operations() {
    let directory =
        std::env::var("GALAXY_REPLICA_FIXTURES").expect("explicit output path required");
    let directory = std::path::Path::new(&directory);
    std::fs::create_dir_all(directory).unwrap();
    let (mut game, epoch) = fixture(2);
    object(&mut game);
    let data = capture(&mut game);
    let token = data.header.acquisition;
    let revision = data.header.snapshot.revision;
    let mut operations = Vec::<Value>::new();
    let mut record = |request: Value, result: Result<()>| {
        operations.push(json!({"request":request,"response":match result {Ok(())=>json!({"v":1,"ok":true,"value":null}),Err(error)=>json!({"v":1,"ok":false,"error":error})}}));
    };
    let mut r = replica(epoch);
    record(
        json!({"v":1,"op":"replica_begin","session":id(100),"peer":id(2),"epoch":epoch,"world_seed":812345}),
        Ok(()),
    );
    record(
        json!({"v":1,"op":"replica_stage","header":data.header}),
        r.stage(data.header.clone()),
    );
    for page in &data.entities {
        record(
            json!({"v":1,"op":"replica_entities","acquisition":token,"page":page}),
            r.entities(token, page.clone()),
        );
    }
    for page in &data.bodies {
        record(
            json!({"v":1,"op":"replica_body","acquisition":token,"page":page}),
            r.body(token, page.clone()),
        );
    }
    record(
        json!({"v":1,"op":"replica_commit","acquisition":token,"revision":revision}),
        r.commit(token, revision),
    );
    assert!(r.scene().is_some());
    let mut frame = FrameBuilder::new();
    r.scene().unwrap().append_occupied(&mut frame, 1.0);
    let body_quads = frame.frame().pixel_count;
    let body_transforms: usize = frame
        .frame()
        .batches
        .iter()
        .map(|b| b.transforms.len())
        .sum();
    r.scene()
        .unwrap()
        .render_into_with_home(&mut frame, view(), |_, _, _| false)
        .unwrap();
    let environment_quads = frame.frame().pixel_count - body_quads;
    record(
        json!({"v":1,"op":"replica_end","session":id(100)}),
        r.end(id(100)),
    );
    let maximum_request = operations
        .iter()
        .map(|o| serde_json::to_vec(&o["request"]).unwrap().len())
        .max()
        .unwrap();
    assert!(maximum_request <= 65536);
    let fixture = json!({"scope":"Real native Game ECS admitted two-pilot fixture; scene includes one arranged three-cell asteroid. Native operation Result mapped to agreed wrapper response; not executed WASM/browser wrapper.","operations":operations});
    std::fs::write(
        directory.join("native-replica-operations.json"),
        serde_json::to_string_pretty(&fixture).unwrap() + "\n",
    )
    .unwrap();
    println!(
        "native replica operations={} headerBytes={} entities={} occupied={} storage={} bodyQuads={} bodyTransforms={} environmentQuads={} maximumRequestBytes={}",
        fixture["operations"].as_array().unwrap().len(),
        serde_json::to_vec(&data.header).unwrap().len(),
        data.header.counts.entities,
        data.header.counts.occupied_cells,
        data.header.counts.storage_cells,
        body_quads,
        body_transforms,
        environment_quads,
        maximum_request
    );
}
