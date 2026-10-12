//! Actual native sample/capture/replica/render tests; no browser acceptance.
use super::*;
use crate::co_op_wake_fx::{MAX_PAGE_BYTES, MAX_POINTS, POSITION_LIMIT, Sample, WakePage};

fn wake_fixture(players: u32) -> (Game, u32) {
    let mut profile =
        crate::identity::create_profile("talo", "hugo", "pan", "0123456789abcdef").unwrap();
    profile.seed = 812345;
    profile.campaign.ship_xp[0] = 1560;
    profile.campaign.abilities[5] = crate::campaign::AbilityState {
        blueprint: true,
        rank: 5,
    };
    profile.campaign.loadouts[0].slots[2] = Some(5);
    for index in [3, 4] {
        profile.campaign.abilities[index] = crate::campaign::AbilityState {
            blueprint: true,
            rank: 5,
        };
    }
    profile.campaign.loadouts[0].slots[0] = Some(3);
    profile.campaign.loadouts[0].slots[1] = Some(4);
    profile.boss_generations[0] = [16, 8, 4, 2, 1];
    profile.boss_generations[1] = [16, 8, 4, 2, 1];
    profile.boss_generations[2] = [16, 8, 4, 2, 1];
    let mut game = Game::new(profile.clone());
    game.begin_co_op(PROTOCOL, id(100), id(1)).unwrap();
    let mut epoch = 0;
    for n in 2..=players {
        let owned = Game::new(profile.clone()).co_op_owned_loadout().unwrap();
        let a = game
            .join_co_op_owned(
                id(100),
                ParticipantSpec {
                    id: id(n),
                    local_seed: profile.seed,
                    identity: profile.identity.clone(),
                },
                owned,
            )
            .unwrap();
        if n == 2 {
            epoch = a.epoch;
        }
    }
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
    game.spawn_timer = 10000.0;
    let pilots: Vec<_> = game
        .world
        .query::<(hecs::Entity, &co_op::Pilot)>()
        .iter()
        .map(|(e, p)| (e, p.admission, p.stats.speed))
        .collect();
    for (e, a, speed) in pilots {
        *game.world.get::<&mut Motion>(e).unwrap() = Motion::new(
            Vec2::new(800.0, a.slot as f32 * 100.0),
            Vec2::X * speed * 0.75,
            0.0,
        );
        let input = InputFrame {
            joystick: Some([1.0, 0.0]),
            ..Default::default()
        };
        if a.slot == 0 {
            game.submit_host_co_op_input(id(100), 1, input).unwrap();
        } else {
            game.submit_guest_co_op_input(id(100), a.id, a.epoch, 1, input)
                .unwrap();
        }
    }
    game.tick_co_op(id(100)).unwrap();
    (game, epoch)
}
fn wake_capture(game: &mut Game) -> (Data, WakePage) {
    let data = capture(game);
    let page = game
        .published_co_op_wake_page(
            id(100),
            data.header.acquisition,
            data.header.snapshot.revision,
        )
        .unwrap();
    assert_eq!(
        usize::from(data.header.snapshot.wake_fx.unwrap().points),
        page.points.len()
    );
    (data, page)
}
fn transfer(r: &mut Replica, data: &Data, page: &WakePage, bodies: bool) {
    stage(r, data, bodies);
    if !page.points.is_empty() {
        r.wake(data.header.acquisition, page.clone()).unwrap();
    }
    commit(r, data);
}
fn dots(scene: &Scene, reduced_motion: bool) -> Vec<CellGeometry> {
    let mut f = FrameBuilder::new();
    let v = ViewInput {
        camera: Vec2::new(900.0, 350.0),
        viewport: Vec2::splat(8192.0),
        reduced: true,
        reduced_motion,
        ..view()
    };
    scene.append_visible(&mut f, 1.0, Some(v));
    geometry(&f.finish())
        .into_iter()
        .filter(|p| p.1 & 0x00ff_ffff == rgba(132, 255, 202, 0))
        .collect()
}

#[test]
fn initial_real_samples_match_host_and_solo_style_without_replaying_lights() {
    let (mut game, epoch) = wake_fixture(8);
    let (data, page) = wake_capture(&mut game);
    assert_eq!(page.points.len(), 8);
    assert!(
        page.points
            .iter()
            .all(|p| p.born_tick == data.header.snapshot.tick)
    );
    let particles = game.particles.len();
    let lights = game.transient_lights.len();
    let mut r = replica(epoch);
    transfer(&mut r, &data, &page, true);
    let rendered = dots(r.scene().unwrap(), false);
    let mut expected = FrameBuilder::new();
    crate::co_op_wake_fx::append(
        &mut expected,
        page.points.iter().map(|p| {
            (
                Vec2::new(p.position[0] as f32, p.position[1] as f32),
                p.life as f32,
            )
        }),
        data.header.snapshot.tick as f32 / 60.0,
        |_, _| true,
    );
    assert_eq!(rendered, geometry(&expected.finish()));
    assert_eq!(rendered.len(), 40);
    // Compare the actual solo and host frame paths, not only helper output.
    game.camera = Vec2::new(900.0, 350.0);
    game.viewport = Vec2::splat(8192.0);
    game.reduced_motion = false;
    let wake_geometry = |frame: &engine_core::RenderFrame| {
        geometry(frame)
            .into_iter()
            .filter(|p| p.1 & 0x00ff_ffff == rgba(132, 255, 202, 0))
            .collect::<Vec<_>>()
    };
    let mut host = FrameBuilder::new();
    game.render_into(1.0, &mut host);
    assert_eq!(rendered, wake_geometry(&host.finish()));
    let mut solo = Game::new(Profile::default());
    solo.camera = game.camera;
    solo.viewport = game.viewport;
    solo.time = data.header.snapshot.tick as f32 / 60.0;
    solo.wake = page
        .points
        .iter()
        .map(|p| {
            (
                Vec2::new(p.position[0] as f32, p.position[1] as f32),
                p.life as f32,
            )
        })
        .collect();
    let mut solo_frame = FrameBuilder::new();
    solo.render_into(1.0, &mut solo_frame);
    assert_eq!(rendered, wake_geometry(&solo_frame.finish()));
    assert_eq!(
        r.scene()
            .unwrap()
            .light_sources(view())
            .iter()
            .filter(|s| s.color == [0.23, 1.0, 0.62])
            .count(),
        0
    );
    assert_eq!(game.particles.len(), particles);
    assert_eq!(game.transient_lights.len(), lights);
    assert_ne!(rendered, dots(r.scene().unwrap(), true));
    assert_eq!(dots(r.scene().unwrap(), true).len(), 40);
    let mut offscreen = FrameBuilder::new();
    r.scene().unwrap().append_visible(
        &mut offscreen,
        1.0,
        Some(ViewInput {
            camera: Vec2::new(-WORLD_LIMIT, -WORLD_LIMIT),
            ..view()
        }),
    );
    assert!(geometry(&offscreen.finish()).is_empty());
    println!(
        "actual_initial_wakes=8 actual_guest_quads=40 shared_solo_style=true guessed_guest_birth_lights=0"
    );
}

#[test]
fn frozen_page_is_non_draining_and_membership_invalidates_its_acquisition() {
    let (mut game, epoch) = wake_fixture(2);
    let (data, page) = wake_capture(&mut game);
    let bytes = serde_json::to_value(&page).unwrap();
    for _ in 0..20 {
        game.tick_co_op(id(100)).unwrap();
    }
    let read = || {
        game.published_co_op_wake_page(
            id(100),
            data.header.acquisition,
            data.header.snapshot.revision,
        )
        .unwrap()
    };
    assert_eq!(serde_json::to_value(read()).unwrap(), bytes);
    assert_eq!(serde_json::to_value(read()).unwrap(), bytes);
    game.leave_co_op(id(100), id(2), epoch).unwrap();
    assert!(
        game.published_co_op_wake_page(
            id(100),
            data.header.acquisition,
            data.header.snapshot.revision
        )
        .is_err()
    );
}

#[test]
fn pending_missing_duplicate_bad_pages_do_not_move_visible_scene_or_cache() {
    let (mut game, epoch) = wake_fixture(2);
    let (initial, page) = wake_capture(&mut game);
    let mut r = replica(epoch);
    transfer(&mut r, &initial, &page, true);
    let before = dots(r.scene().unwrap(), true);
    let (next, next_page) = wake_capture(&mut game);
    stage(&mut r, &next, false); // body cache may bypass cells, never the wake page.
    assert_eq!(
        r.commit(next.header.acquisition, next.header.snapshot.revision),
        Err("incomplete_wake")
    );
    assert_eq!(dots(r.scene().unwrap(), true), before);
    let mut bad = next_page.clone();
    bad.points[0].epoch += 1;
    assert_eq!(
        r.wake(next.header.acquisition, bad),
        Err("unknown_wake_member")
    );
    let mut bad = next_page.clone();
    bad.points.pop();
    assert_eq!(
        r.wake(next.header.acquisition, bad),
        Err("invalid_wake_page")
    );
    let mut bad = next_page.clone();
    bad.revision += 1;
    assert_eq!(r.wake(next.header.acquisition, bad), Err("stale_page"));
    r.wake(next.header.acquisition, next_page.clone()).unwrap();
    assert_eq!(
        r.wake(next.header.acquisition, next_page),
        Err("invalid_wake_page")
    );
    commit(&mut r, &next);
    assert_eq!(dots(r.scene().unwrap(), true), before);
}

#[test]
fn captured_same_tick_sets_are_immutable_and_deadlines_age_during_assembly() {
    let (mut game, epoch) = wake_fixture(2);
    let (first, page) = wake_capture(&mut game);
    let mut r = replica(epoch);
    r.advance_presentation(10.0).unwrap();
    stage(&mut r, &first, true);
    r.wake(first.header.acquisition, page.clone()).unwrap();
    r.advance_presentation(13.5).unwrap();
    commit(&mut r, &first);
    assert!(dots(r.scene().unwrap(), true).is_empty());
    let (next, same) = wake_capture(&mut game);
    transfer(&mut r, &next, &same, false);
    assert!(dots(r.scene().unwrap(), true).is_empty());
    for mutate_life in [true, false] {
        let (changed, mut points) = wake_capture(&mut game);
        if mutate_life {
            points.points[0].life -= 0.1;
        } else {
            points.points[0].born_tick = 0;
        }
        stage(&mut r, &changed, false);
        r.wake(changed.header.acquisition, points).unwrap();
        assert_eq!(
            r.commit(changed.header.acquisition, changed.header.snapshot.revision),
            Err("same_tick_wake_mutation")
        );
        assert_eq!(
            r.scene().unwrap().header().acquisition,
            next.header.acquisition
        );
    }
    let (mut empty, _) = wake_capture(&mut game);
    empty.header.snapshot.wake_fx = Some(crate::co_op_wake_fx::WakeMeta { points: 0 });
    stage(&mut r, &empty, false);
    assert_eq!(
        r.commit(empty.header.acquisition, empty.header.snapshot.revision),
        Err("same_tick_wake_mutation")
    );
    assert_eq!(
        r.scene().unwrap().header().acquisition,
        next.header.acquisition
    );
    let (mut legacy, _) = wake_capture(&mut game);
    legacy.header.snapshot.wake_fx = None;
    stage(&mut r, &legacy, false);
    assert_eq!(
        r.commit(legacy.header.acquisition, legacy.header.snapshot.revision),
        Err("same_tick_wake_mutation")
    );
    // A fresh real fixed tick authoritatively revalidates a still-live sample.
    game.tick_co_op(id(100)).unwrap();
    let (newer, newer_page) = wake_capture(&mut game);
    transfer(&mut r, &newer, &newer_page, false);
    assert!(!dots(r.scene().unwrap(), true).is_empty());
    assert_eq!(
        r.advance_presentation(13.4),
        Err("invalid_presentation_clock")
    );
    r.advance_presentation(17.0).unwrap();
    for clock in [f64::NAN, f64::INFINITY, -1.0] {
        assert_eq!(
            r.advance_presentation(clock),
            Err("invalid_presentation_clock")
        );
    }
    assert!(dots(r.scene().unwrap(), true).is_empty());
    r.end(id(100)).unwrap();
    assert!(r.scene().is_none());
}

#[test]
fn raw_value_number_bounds_and_canonical_short_spellings_agree() {
    let (mut game, epoch) = wake_fixture(2);
    let (data, page) = wake_capture(&mut game);
    for life in [0.0, -1.0, 1e-46, 3.4000001] {
        let mut value = serde_json::to_value(&page).unwrap();
        value["points"][0]["life"] = json!(life);
        let mut r = replica(epoch);
        stage(&mut r, &data, true);
        assert_eq!(
            r.wake(
                data.header.acquisition,
                serde_json::from_value(value).unwrap()
            ),
            Err("invalid_wake_point"),
            "life={life}"
        );
    }
    for position in [1_000_014.01, -1_000_014.01, 1e40] {
        let mut value = serde_json::to_value(&page).unwrap();
        value["points"][0]["position"][0] = json!(position);
        let mut r = replica(epoch);
        stage(&mut r, &data, true);
        assert_eq!(
            r.wake(
                data.header.acquisition,
                serde_json::from_value(value).unwrap()
            ),
            Err("invalid_wake_point")
        );
    }
    let mut r = replica(epoch);
    transfer(&mut r, &data, &page, true);
    let (next, mut same) = wake_capture(&mut game);
    same.points[0].life = 3.4; // same captured f32, different raw f64 spelling.
    transfer(&mut r, &next, &same, false);
    let mut value = serde_json::to_value(&data.header).unwrap();
    value["snapshot"]["wake_fx"] = Value::Null;
    assert!(serde_json::from_value::<Header>(value).is_err());
    let mut value = serde_json::to_value(&page).unwrap();
    value["offset"] = json!(0);
    assert!(serde_json::from_value::<WakePage>(value).is_err());
}

#[test]
fn native_edge_stamp_keeps_legal_rear_anchor_halo() {
    let (mut game, epoch) = wake_fixture(2);
    let e = game
        .world
        .query::<(hecs::Entity, &co_op::Pilot)>()
        .iter()
        .find(|(_, p)| p.admission.id == id(2))
        .unwrap()
        .0;
    let speed = game.world.get::<&co_op::Pilot>(e).unwrap().stats.speed;
    *game.world.get::<&mut Motion>(e).unwrap() = Motion::new(
        Vec2::new(WORLD_LIMIT, 0.0),
        -Vec2::X * speed * 0.75,
        std::f32::consts::PI,
    );
    game.world.get::<&mut co_op::Pilot>(e).unwrap().wake_timer = 0.0;
    game.submit_guest_co_op_input(
        id(100),
        id(2),
        epoch,
        2,
        InputFrame {
            joystick: Some([-1.0, 0.0]),
            ..Default::default()
        },
    )
    .unwrap();
    game.tick_co_op(id(100)).unwrap();
    // Existing chunk objects beyond WORLD_LIMIT fail the older entity DTO.
    // Isolate this actual birth-anchor test without changing those bounds.
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
    let (data, page) = wake_capture(&mut game);
    let edge = page
        .points
        .iter()
        .find(|p| p.slot == 1 && p.born_tick == data.header.snapshot.tick)
        .unwrap();
    assert_eq!(edge.position[0], f64::from(POSITION_LIMIT));
    assert_eq!(f64::from(POSITION_LIMIT), 1_000_014.0);
    let mut r = replica(epoch);
    transfer(&mut r, &data, &page, true);
    println!("actual_native_edge_rear_anchor_x={}", edge.position[0]);
}

#[test]
fn epoch_leave_empty_new_tick_and_position_mutation_have_atomic_lifecycle() {
    let (mut game, epoch) = wake_fixture(3);
    let (first, page) = wake_capture(&mut game);
    let mut r = replica(epoch);
    transfer(&mut r, &first, &page, true);
    let (next, mut moved) = wake_capture(&mut game);
    moved.points[0].position[0] += 1.0;
    stage(&mut r, &next, false);
    r.wake(next.header.acquisition, moved).unwrap();
    assert_eq!(
        r.commit(next.header.acquisition, next.header.snapshot.revision),
        Err("same_tick_wake_mutation")
    );
    game.leave_co_op(id(100), id(3), 3).unwrap();
    let (left, left_page) = wake_capture(&mut game);
    transfer(&mut r, &left, &left_page, false);
    assert_eq!(dots(r.scene().unwrap(), true).len(), 10);
    // Explicit new real tick with no active samples clears every cached key.
    for p in game.world.query_mut::<&mut co_op::Pilot>() {
        p.wake.clear();
    }
    for m in game.world.query_mut::<&mut Motion>() {
        m.velocity = Vec2::ZERO;
    }
    game.tick_co_op(id(100)).unwrap();
    let (empty, empty_page) = wake_capture(&mut game);
    assert!(empty_page.points.is_empty());
    transfer(&mut r, &empty, &empty_page, false);
    assert!(dots(r.scene().unwrap(), true).is_empty());
}

#[test]
fn configured_maximum_point_state_stays_bounded_and_draws_at_most_960_quads() {
    let (mut game, epoch) = wake_fixture(8);
    // Explicit capacity fixture, not a claim that 24 points naturally coexist.
    for pilot in game.world.query_mut::<&mut co_op::Pilot>() {
        pilot.wake = (0..24)
            .map(|born_tick| Sample {
                position: Vec2::new(
                    800.0 + born_tick as f32,
                    pilot.admission.slot as f32 * 100.0,
                ),
                life: 3.4,
                born_tick,
            })
            .collect();
    }
    // Admit fixture birth range with a real fixed-step session watermark.
    for _ in 0..24 {
        game.tick_co_op(id(100)).unwrap();
    }
    for pilot in game.world.query_mut::<&mut co_op::Pilot>() {
        pilot.wake = (0..24)
            .map(|born_tick| Sample {
                position: Vec2::new(
                    800.0 + born_tick as f32,
                    pilot.admission.slot as f32 * 100.0,
                ),
                life: 3.4,
                born_tick,
            })
            .collect();
    }
    let (data, page) = wake_capture(&mut game);
    assert_eq!(page.points.len(), MAX_POINTS);
    let bytes = serde_json::to_vec(&serde_json::to_value(&page).unwrap())
        .unwrap()
        .len();
    assert!(bytes <= MAX_PAGE_BYTES);
    let mut r = replica(epoch);
    transfer(&mut r, &data, &page, true);
    assert_eq!(dots(r.scene().unwrap(), true).len(), 960);
    let (next, mut bad) = wake_capture(&mut game);
    bad.points.push(bad.points[0].clone());
    stage(&mut r, &next, false);
    assert_eq!(r.wake(next.header.acquisition, bad), Err("wake_page_bound"));
    println!("actual_capacity_fixture_points=192 actual_quads=960 actual_value_page_bytes={bytes}");
}

#[test]
fn same_tick_empty_baseline_cannot_gain_points_and_new_epoch_drops_old_owner() {
    let (mut game, epoch) = wake_fixture(3);
    let (mut empty, page) = wake_capture(&mut game);
    empty.header.snapshot.wake_fx = None;
    let mut r = replica(epoch);
    stage(&mut r, &empty, true);
    commit(&mut r, &empty);
    let (positive, _) = wake_capture(&mut game);
    stage(&mut r, &positive, false);
    r.wake(positive.header.acquisition, page.clone()).unwrap();
    assert_eq!(
        r.commit(
            positive.header.acquisition,
            positive.header.snapshot.revision
        ),
        Err("same_tick_wake_mutation")
    );
    assert!(dots(r.scene().unwrap(), true).is_empty());
    game.tick_co_op(id(100)).unwrap();
    let (live, live_page) = wake_capture(&mut game);
    transfer(&mut r, &live, &live_page, false);
    let previous_count = dots(r.scene().unwrap(), true).len();
    let old_member = live
        .header
        .snapshot
        .participants
        .iter()
        .find(|p| p.id == id(3))
        .unwrap()
        .clone();
    game.leave_co_op(id(100), id(3), old_member.epoch).unwrap();
    let build = old_member.owned_loadout.unwrap();
    let rejoined = game
        .join_co_op_owned(
            id(100),
            ParticipantSpec {
                id: id(3),
                local_seed: old_member.local_seed,
                identity: old_member.identity,
            },
            build,
        )
        .unwrap();
    assert_ne!(rejoined.epoch, old_member.epoch);
    let (new_epoch, page) = wake_capture(&mut game);
    transfer(&mut r, &new_epoch, &page, true);
    assert_eq!(dots(r.scene().unwrap(), true).len(), previous_count - 5);
    assert!(page.points.iter().all(|p| p.epoch != old_member.epoch));
}

#[test]
fn changed_positions_on_newer_ticks_and_malformed_order_never_replace_cache() {
    let (mut game, epoch) = wake_fixture(2);
    let (first, page) = wake_capture(&mut game);
    let mut r = replica(epoch);
    transfer(&mut r, &first, &page, true);
    game.tick_co_op(id(100)).unwrap();
    let (newer, page) = wake_capture(&mut game);
    stage(&mut r, &newer, false);
    let mut wrong_order = page.clone();
    wrong_order.points.reverse();
    assert_eq!(
        r.wake(newer.header.acquisition, wrong_order),
        Err("invalid_wake_order")
    );
    let mut future = page.clone();
    future.points[0].born_tick = newer.header.snapshot.tick + 1;
    assert_eq!(
        r.wake(newer.header.acquisition, future),
        Err("invalid_wake_point")
    );
    let mut changed = page.clone();
    changed.points[0].position[0] += 10.0;
    r.wake(newer.header.acquisition, changed).unwrap();
    let before = dots(r.scene().unwrap(), true);
    assert_eq!(
        r.commit(newer.header.acquisition, newer.header.snapshot.revision),
        Err("wake_position_changed")
    );
    assert_eq!(dots(r.scene().unwrap(), true), before);
}

#[test]
fn member_point_cap_and_native_invalid_capture_preserve_prior_publication() {
    let (mut game, epoch) = wake_fixture(2);
    for _ in 0..30 {
        game.tick_co_op(id(100)).unwrap();
    }
    let (data, _) = wake_capture(&mut game);
    let mut r = replica(epoch);
    let mut over = game
        .published_co_op_wake_page(
            id(100),
            data.header.acquisition,
            data.header.snapshot.revision,
        )
        .unwrap();
    let member = over.points.iter().find(|p| p.slot == 0).unwrap().clone();
    over.points = (0..25)
        .map(|born_tick| {
            let mut point = member.clone();
            point.born_tick = born_tick;
            point
        })
        .collect();
    let mut header = data.clone();
    header.header.snapshot.wake_fx = Some(crate::co_op_wake_fx::WakeMeta { points: 25 });
    stage(&mut r, &header, true);
    assert_eq!(
        r.wake(header.header.acquisition, over),
        Err("wake_member_bound")
    );
    let prior = serde_json::to_value(
        game.published_co_op_wake_page(
            id(100),
            data.header.acquisition,
            data.header.snapshot.revision,
        )
        .unwrap(),
    )
    .unwrap();
    let e = game
        .world
        .query::<(hecs::Entity, &co_op::Pilot)>()
        .iter()
        .find(|(_, p)| p.admission.slot == 0)
        .unwrap()
        .0;
    game.world.get::<&mut co_op::Pilot>(e).unwrap().wake[0].life = f32::NAN;
    assert!(matches!(
        game.acquire_co_op_publication(id(100)),
        Err(co_op::Error::PublicationBound)
    ));
    assert_eq!(
        serde_json::to_value(
            game.published_co_op_wake_page(
                id(100),
                data.header.acquisition,
                data.header.snapshot.revision
            )
            .unwrap()
        )
        .unwrap(),
        prior
    );
    assert!(r.scene().is_none());
}

#[test]
#[ignore = "explicit bounded serializer export; requires GALAXY_WAKE_FIXTURES"]
fn export_actual_native_wake_contract_fixtures() {
    let directory = std::path::PathBuf::from(
        std::env::var("GALAXY_WAKE_FIXTURES").expect("explicit directory"),
    );
    std::fs::create_dir_all(&directory).unwrap();
    let (mut game, epoch) = wake_fixture(8);
    let (initial, initial_page) = wake_capture(&mut game);
    let mut r = replica(epoch);
    transfer(&mut r, &initial, &initial_page, true);
    for _ in 0..24 {
        game.tick_co_op(id(100)).unwrap();
    }
    // Actual ECS capacity stress, explicitly arranged rather than earned or
    // naturally emitted at this density. Every point is a native Sample.
    for pilot in game.world.query_mut::<&mut co_op::Pilot>() {
        pilot.wake = (2..=25)
            .map(|born_tick| Sample {
                position: Vec2::new(POSITION_LIMIT - born_tick as f32 * 0.125, -POSITION_LIMIT),
                life: 3.4 - born_tick as f32 * 0.016666668,
                born_tick,
            })
            .collect();
    }
    let pilots: Vec<_> = game
        .world
        .query::<&co_op::Pilot>()
        .iter()
        .map(|p| (p.admission.id, p.admission.epoch))
        .collect();
    for i in 0..180 {
        let (owner, owner_epoch) = pilots[i % pilots.len()];
        game.world.spawn((
            Motion::new(
                Vec2::new(900_000.0 + i as f32, -900_000.0),
                Vec2::new(-500.0, 600.0),
                0.0,
            ),
            Bullet {
                player: true,
                damage: 12.0,
                life: 1.75,
                remaining_hits: 1,
                previous_hit: None,
            },
            co_op::ShotOwner {
                id: owner,
                epoch: owner_epoch,
            },
        ));
    }
    for i in 0..32 {
        let (owner, owner_epoch) = pilots[i % pilots.len()];
        game.co_op.as_mut().unwrap().nova_fx.record(
            25,
            owner,
            owner_epoch,
            Vec2::new(850.0, i as f32),
            36,
        );
    }
    let (maximum, maximum_page) = wake_capture(&mut game);
    assert_eq!(maximum.header.snapshot.participants.len(), 8);
    assert_eq!(maximum.header.snapshot.projectiles.len(), 180);
    assert_eq!(
        maximum
            .header
            .snapshot
            .nova_fx
            .as_ref()
            .unwrap()
            .records
            .len(),
        32
    );
    assert_eq!(maximum_page.points.len(), 192);
    transfer(&mut r, &maximum, &maximum_page, true);
    let write = |name: &str, value: Value| {
        std::fs::write(
            directory.join(format!("{name}.json")),
            serde_json::to_string(&value).unwrap() + "\n",
        )
        .unwrap();
        value
    };
    let maximum_header = write(
        "maximum-header",
        serde_json::to_value(&maximum.header).unwrap(),
    );
    let maximum_page_value = write("maximum-page", serde_json::to_value(&maximum_page).unwrap());
    write(
        "initial-header",
        serde_json::to_value(&initial.header).unwrap(),
    );
    write("initial-page", serde_json::to_value(&initial_page).unwrap());
    let mut legacy = serde_json::to_value(&initial.header).unwrap();
    legacy["snapshot"]
        .as_object_mut()
        .unwrap()
        .remove("wake_fx");
    write("legacy-header", legacy);
    let transport_header = write(
        "maximum-transport-publication",
        json!({"v":1,"session":id(100),"seq":u32::MAX,"kind":"publication","payload":maximum_header}),
    );
    let stage_request = write(
        "maximum-native-stage",
        json!({"v":1,"op":"replica_stage","header":maximum_header}),
    );
    let transport_page = write(
        "maximum-transport-body",
        json!({"v":1,"session":id(100),"seq":u32::MAX,"kind":"body","payload":maximum_page_value}),
    );
    let native_page = write(
        "maximum-native-wake",
        json!({"v":1,"op":"replica_wake","acquisition":maximum.header.acquisition,"page":maximum_page_value}),
    );
    let page_bytes = serde_json::to_vec(&maximum_page_value).unwrap().len();
    assert!(page_bytes <= MAX_PAGE_BYTES);
    let lengths: Vec<_> = [transport_header, stage_request, transport_page, native_page]
        .iter()
        .map(|v| serde_json::to_vec(v).unwrap().len())
        .collect();
    assert!(lengths.iter().all(|&n| n <= 65_536));
    println!(
        "actual_value_maximum_page_bytes={page_bytes}; exact_wrappers_publication_stage_body_wake_bytes={lengths:?}; pilots=8 owner_projectiles=180 nova_records=32 wake_points=192"
    );
    // Declared scalar stress exercises permitted token maxima and f64 decimal
    // expansions separately; it is not an executed tick/earned-loadout claim.
    let mut scalar_page = maximum_page_value;
    scalar_page["tick"] = json!(u32::MAX);
    scalar_page["revision"] = json!(u32::MAX);
    for point in scalar_page["points"].as_array_mut().unwrap() {
        point["epoch"] = json!(u32::MAX);
        point["born_tick"] = json!(u32::MAX - 25 + point["born_tick"].as_u64().unwrap() as u32);
    }
    let scalar_page = write("scalar-stress-page", scalar_page);
    let scalar_transport = write(
        "scalar-stress-transport-body",
        json!({"v":1,"session":id(100),"seq":u32::MAX,"kind":"body","payload":scalar_page}),
    );
    let scalar_request = write(
        "scalar-stress-native-wake",
        json!({"v":1,"op":"replica_wake","acquisition":u32::MAX,"page":scalar_page}),
    );
    let scalar_lengths: Vec<_> = [&scalar_page, &scalar_transport, &scalar_request]
        .iter()
        .map(|v| serde_json::to_vec(v).unwrap().len())
        .collect();
    assert!(scalar_lengths[0] <= MAX_PAGE_BYTES);
    assert!(scalar_lengths[1..].iter().all(|&n| n < 65_536));
    println!(
        "declared_max_u32_scalar_page_transport_native_bytes={scalar_lengths:?}; separate_from_actual_tick_execution=true"
    );
    for (name, life) in [
        ("raw-life-underflow", 1e-46),
        ("raw-life-round-down", 3.4000001),
        ("raw-life-short-valid", 3.4),
    ] {
        let mut value = serde_json::to_value(&initial_page).unwrap();
        value["points"][0]["life"] = json!(life);
        write(name, value);
    }
    // Real same-epoch death/respawn clears native wakes at the next fixed tick.
    let entities: Vec<_> = game
        .world
        .query::<(hecs::Entity, &co_op::Pilot)>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    for e in entities {
        game.world.get::<&mut Actor>(e).unwrap().health = 0.0;
    }
    game.tick_co_op(id(100)).unwrap();
    let (empty, empty_page) = wake_capture(&mut game);
    assert_eq!(empty.header.snapshot.wake_fx.unwrap().points, 0);
    assert!(empty_page.points.is_empty());
    transfer(&mut r, &empty, &empty_page, true);
    assert!(dots(r.scene().unwrap(), true).is_empty());
    write("empty-header", serde_json::to_value(&empty.header).unwrap());
    write("empty-page", serde_json::to_value(&empty_page).unwrap());
    for _ in 0..210 {
        game.tick_co_op(id(100)).unwrap();
    }
    let (expired, expired_page) = wake_capture(&mut game);
    assert!(expired_page.points.is_empty());
    write(
        "expired-header",
        serde_json::to_value(&expired.header).unwrap(),
    );
    write("expired-page", serde_json::to_value(&expired_page).unwrap());
}
