//! Configured ordinary-Nova builds; triggers use actual native accepted damage.
//! These are native capture/replica tests, not browser or earned-unlock evidence.
use super::*;

fn nova_fixture(players: u32) -> (Game, u32) {
    let mut profile =
        crate::identity::create_profile("talo", "hugo", "pan", "0123456789abcdef").unwrap();
    profile.seed = 812345;
    profile.campaign.abilities[4] = crate::campaign::AbilityState {
        blueprint: true,
        rank: 1,
    };
    profile.campaign.loadouts[0].slots[1] = Some(4);
    profile.campaign.loadouts[0].hardware[3] = 1; // Configured ten-point chassis shield.
    let mut game = Game::new(profile.clone());
    game.begin_co_op(PROTOCOL, id(100), id(1)).unwrap();
    let mut epoch = 0;
    for n in 2..=players {
        let solo = Game::new(profile.clone());
        let a = game
            .join_co_op_owned(
                id(100),
                ParticipantSpec {
                    id: id(n),
                    local_seed: profile.seed,
                    identity: profile.identity.clone(),
                },
                solo.co_op_owned_loadout().unwrap(),
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
        .filter(|(_, pilot)| pilot.is_none())
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
        .map(|(e, p)| (e, p.admission.slot))
        .collect();
    for (e, slot) in pilots {
        *game.world.get::<&mut Motion>(e).unwrap() =
            Motion::new(Vec2::new(450.0, slot as f32 * 150.0), Vec2::ZERO, 0.0);
        let mut p = game.world.get::<&mut co_op::Pilot>(e).unwrap();
        p.invulnerability = 0.0;
        p.was_home = false;
    }
    game.refresh_sites();
    game.take_events();
    (game, epoch)
}
fn hit(game: &mut Game, peer: ParticipantId) {
    let e = game
        .world
        .query::<(hecs::Entity, &co_op::Pilot)>()
        .iter()
        .find(|(_, p)| p.admission.id == peer)
        .unwrap()
        .0;
    game.hurt_co_op_pilot(e, 1.0, None);
}
fn baseline(game: &mut Game, epoch: u32) -> Replica {
    let mut r = replica(epoch);
    let empty = capture(game);
    assert!(empty.header.snapshot.nova_fx.is_none());
    stage(&mut r, &empty, true);
    commit(&mut r, &empty);
    assert_eq!(r.fx_sequence, Some(0));
    r
}
fn ray_geometry(scene: &Scene, reduced: bool) -> Vec<CellGeometry> {
    let mut frame = FrameBuilder::new();
    scene.fx.render(&mut frame, reduced, |_, _| true);
    geometry(&frame.finish())
}
#[test]
fn real_admission_capture_is_non_draining_and_guest_rays_match_actual_host_births() {
    let (mut game, epoch) = nova_fixture(2);
    let before = serde_json::to_string(game.profile()).unwrap();
    let mut r = baseline(&mut game, epoch);
    hit(&mut game, id(1));
    assert_eq!(game.particles.len(), 36);
    assert_eq!(
        game.take_events()
            .iter()
            .filter(|e| e.kind == "ability-nova")
            .count(),
        1
    );
    let data = capture(&mut game);
    let fx = data.header.snapshot.nova_fx.as_ref().unwrap();
    assert_eq!(
        (fx.sequence, fx.records.len(), fx.records[0].rays),
        (1, 1, 36)
    );
    assert_eq!(fx.records[0].emitter, id(1));
    stage(&mut r, &data, true);
    assert_eq!(r.scene().unwrap().fx.rays(), 0);
    commit(&mut r, &data);
    let mut host = FrameBuilder::new();
    game.camera = Vec2::new(450.0, 0.0);
    game.previous_camera = game.camera;
    game.render_particles(&mut host, 1.0);
    assert_eq!(
        geometry(&host.finish()),
        ray_geometry(r.scene().unwrap(), false)
    );
    assert_eq!(ray_geometry(r.scene().unwrap(), true).len(), 18);
    let repeat = capture(&mut game);
    assert_eq!(
        serde_json::to_value(repeat.header.snapshot.nova_fx).unwrap(),
        serde_json::to_value(fx).unwrap()
    );
    assert_eq!(before, serde_json::to_string(game.profile()).unwrap());
}
#[test]
fn repeated_and_newer_acquisitions_never_restart_or_duplicate_live_nova() {
    let (mut game, epoch) = nova_fixture(2);
    let mut r = baseline(&mut game, epoch);
    hit(&mut game, id(1));
    let first = capture(&mut game);
    stage(&mut r, &first, true);
    commit(&mut r, &first);
    r.advance_presentation(0.2).unwrap();
    let second = capture(&mut game);
    stage(&mut r, &second, false);
    commit(&mut r, &second);
    assert_eq!(r.scene().unwrap().fx.rays(), 36);
    r.advance_presentation(0.451).unwrap();
    assert_eq!(r.scene().unwrap().fx.rays(), 0);
    assert!(r.scene().unwrap().fx.sources(|_, _| true).is_empty());
    let third = capture(&mut game);
    stage(&mut r, &third, false);
    commit(&mut r, &third);
    assert!(ray_geometry(r.scene().unwrap(), false).is_empty());
    assert_eq!(r.fx_sequence, Some(1));
}

#[test]
fn analytic_guest_motion_tracks_actual_fixed_step_debris_with_fractional_quads() {
    let (mut game, epoch) = nova_fixture(2);
    let mut r = baseline(&mut game, epoch);
    hit(&mut game, id(1));
    let data = capture(&mut game);
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    for _ in 0..6 {
        game.update_particles(1.0 / 60.0);
    }
    r.advance_presentation(0.1).unwrap();
    game.camera = Vec2::new(450.0, 0.0);
    game.previous_camera = game.camera;
    let mut frame = FrameBuilder::new();
    game.render_particles(&mut frame, 1.0);
    let host = geometry(&frame.finish());
    let guest = ray_geometry(r.scene().unwrap(), false);
    assert_eq!(host.len(), guest.len());
    for (a, b) in host.iter().zip(guest) {
        assert!(Vec2::from_array(a.0).distance(Vec2::from_array(b.0)) < 0.0001);
        assert_eq!(a.2, b.2);
        assert_eq!(a.3, b.3);
        assert_eq!(a.4, b.4);
        let ac = a.1.to_le_bytes();
        let bc = b.1.to_le_bytes();
        assert_eq!(ac[..3], bc[..3]);
        assert!(ac[3].abs_diff(bc[3]) <= 1);
    }
}

#[test]
fn actual_dropped_counter_and_oversized_journal_reject_without_losing_scene() {
    let (mut game, epoch) = nova_fixture(2);
    let mut r = baseline(&mut game, epoch);
    for _ in 0..40 {
        game.co_op
            .as_mut()
            .unwrap()
            .nova_fx
            .record(0, id(1), 1, Vec2::new(450.0, 0.0), 36);
    }
    let data = capture(&mut game);
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    assert_eq!(r.fx_dropped, 8);
    assert_eq!(r.scene().unwrap().nova_fx_dropped_rays(), 52);
    let mut bad = capture(&mut game);
    bad.header.snapshot.nova_fx.as_mut().unwrap().dropped = 7;
    assert_eq!(r.stage(bad.header), Err("invalid_nova_fx"));
    let mut bad = capture(&mut game);
    let fx = bad.header.snapshot.nova_fx.as_mut().unwrap();
    fx.sequence += 1;
    let mut extra = *fx.records.last().unwrap();
    extra.sequence += 1;
    fx.records.push(extra);
    assert_eq!(r.stage(bad.header), Err("invalid_nova_fx"));
    assert_eq!(r.fx_sequence, Some(40));
    assert_eq!(r.scene().unwrap().fx.rays(), MAX_PARTICLES);
}
#[test]
fn first_populated_publication_skips_join_history_but_legacy_baseline_allows_first_nova() {
    let (mut game, epoch) = nova_fixture(2);
    hit(&mut game, id(1));
    let data = capture(&mut game);
    let mut r = replica(epoch);
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    assert_eq!(r.fx_sequence, Some(1));
    assert_eq!(r.scene().unwrap().fx.rays(), 0);
    // Second participant's first actual accepted damage is a new activation.
    hit(&mut game, id(2));
    let second = capture(&mut game);
    stage(&mut r, &second, true);
    commit(&mut r, &second);
    assert_eq!(r.scene().unwrap().fx.rays(), 36);
    assert_eq!(r.fx_sequence, Some(2));
}
#[test]
fn failed_body_commit_cannot_emit_or_advance_dedup_and_transfer_time_expires_births() {
    for delay in [0.05, 0.5, 100.0] {
        let (mut game, epoch) = nova_fixture(2);
        let mut r = baseline(&mut game, epoch);
        object(&mut game); // A real new occupied body requires coherent assembly.
        hit(&mut game, id(1));
        let data = capture(&mut game);
        stage(&mut r, &data, false);
        assert_eq!(
            r.commit(data.header.acquisition, data.header.snapshot.revision),
            Err("missing_body")
        );
        assert_eq!(r.fx_sequence, Some(0));
        assert_eq!(r.scene().unwrap().fx.rays(), 0);
        r.advance_presentation(delay).unwrap();
        for page in &data.bodies {
            r.body(data.header.acquisition, page.clone()).unwrap();
        }
        commit(&mut r, &data);
        assert_eq!(
            r.scene().unwrap().fx.rays(),
            if delay < 0.4 { 36 } else { 0 }
        );
        assert_eq!(
            r.scene().unwrap().fx.sources(|_, _| true).len(),
            if delay < 0.32 { 2 } else { 0 }
        );
        assert_eq!(r.fx_sequence, Some(1));
    }
}

#[test]
fn real_shield_hit_uses_cached_bodies_and_emits_before_its_short_lifetime() {
    let (mut game, epoch) = nova_fixture(2);
    let mut r = baseline(&mut game, epoch);
    let original: Vec<_> = r
        .scene()
        .unwrap()
        .entities
        .iter()
        .map(|(_, body)| Rc::clone(body))
        .collect();
    let hull_before = game.world.get::<&Actor>(game.player).unwrap().health;
    hit(&mut game, id(1));
    assert_eq!(
        game.world.get::<&Actor>(game.player).unwrap().health,
        hull_before
    );
    let data = capture(&mut game);
    stage(&mut r, &data, false); // No occupied-body transfer or invented cache.
    r.advance_presentation(0.05).unwrap();
    commit(&mut r, &data);
    for (old, (_, current)) in original.iter().zip(&r.scene().unwrap().entities) {
        assert!(Rc::ptr_eq(old, current));
    }
    assert_eq!(ray_geometry(r.scene().unwrap(), false).len(), 36);
    assert_eq!(r.scene().unwrap().fx.sources(|_, _| true).len(), 2);
    assert_eq!(r.scene().unwrap().header.snapshot.projectiles.len(), 0);
}
#[test]
fn captured_tick_age_and_receiver_age_combine_instead_of_restart_on_commit() {
    let (mut game, epoch) = nova_fixture(2);
    let mut r = baseline(&mut game, epoch);
    hit(&mut game, id(1));
    // Real ticks age host Nova; fixture suppresses incidental spawns.
    for _ in 0..18 {
        game.tick_co_op(id(100)).unwrap();
    }
    let data = capture(&mut game);
    assert_eq!(data.header.snapshot.tick, 18);
    stage(&mut r, &data, true);
    r.advance_presentation(0.16).unwrap();
    commit(&mut r, &data);
    assert_eq!(r.scene().unwrap().fx.rays(), 0);
    assert!(r.scene().unwrap().fx.sources(|_, _| true).is_empty());
}

#[test]
fn capture_age_survives_large_finite_monotonic_receiver_clock() {
    let (mut game, epoch) = nova_fixture(2);
    let mut r = baseline(&mut game, epoch);
    hit(&mut game, id(1));
    for _ in 0..18 {
        game.tick_co_op(id(100)).unwrap();
    }
    let data = capture(&mut game);
    r.advance_presentation(1e20).unwrap();
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    let rays = ray_geometry(r.scene().unwrap(), false);
    assert_eq!(rays.len(), 36);
    // .3 seconds of authoritative age survives, even when subtracting .3
    // directly from this absolute clock would round back to the same clock.
    assert!(rays.iter().all(|r| r.1.to_le_bytes()[3] <= 64));
    r.advance_presentation(1e20 + 32768.0).unwrap();
    assert!(ray_geometry(r.scene().unwrap(), false).is_empty());
}
#[test]
fn clock_nonfinite_or_regression_is_atomic_and_end_discards_all_transients() {
    let (mut game, epoch) = nova_fixture(2);
    let mut r = baseline(&mut game, epoch);
    hit(&mut game, id(1));
    let data = capture(&mut game);
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    r.advance_presentation(0.1).unwrap();
    let before = ray_geometry(r.scene().unwrap(), false);
    for clock in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0, 0.09] {
        assert_eq!(
            r.advance_presentation(clock),
            Err("invalid_presentation_clock")
        );
        assert_eq!(r.clock, 0.1);
        assert_eq!(r.fx_sequence, Some(1));
        assert_eq!(before, ray_geometry(r.scene().unwrap(), false));
    }
    assert_eq!(r.end(id(999)), Err("stale_session"));
    assert!(r.scene().is_some());
    r.end(id(100)).unwrap();
    assert!(r.scene().is_none());
    assert!(r.stage(data.header).is_err());
}
#[test]
fn partial_real_host_pool_admission_preserves_only_actual_ray_prefix() {
    let (mut game, epoch) = nova_fixture(2);
    let mut r = baseline(&mut game, epoch);
    for _ in 0..(MAX_PARTICLES - 5) {
        game.particle(
            Vec2::new(1000.0, 0.0),
            Vec2::ZERO,
            1.0,
            rgba(1, 1, 1, 255),
            1.0,
        );
    }
    hit(&mut game, id(1));
    assert_eq!(game.particles.len(), MAX_PARTICLES);
    let data = capture(&mut game);
    assert_eq!(
        data.header.snapshot.nova_fx.as_ref().unwrap().records[0].rays,
        5
    );
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    assert_eq!(ray_geometry(r.scene().unwrap(), false).len(), 5);
}
#[test]
fn old_epoch_is_pruned_and_cannot_attach_to_same_id_rejoin() {
    let (mut game, epoch) = nova_fixture(3);
    let mut r = baseline(&mut game, epoch);
    hit(&mut game, id(3));
    let data = capture(&mut game);
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    let old_epoch = data.header.snapshot.nova_fx.as_ref().unwrap().records[0].epoch;
    game.leave_co_op(id(100), id(3), old_epoch).unwrap();
    let new = game
        .join_co_op(
            id(100),
            ParticipantSpec {
                id: id(3),
                local_seed: 9987,
                identity: None,
            },
        )
        .unwrap();
    assert!(new.epoch > old_epoch);
    let new_data = capture(&mut game);
    assert!(
        new_data
            .header
            .snapshot
            .nova_fx
            .as_ref()
            .unwrap()
            .records
            .is_empty()
    );
    stage(&mut r, &new_data, true);
    commit(&mut r, &new_data);
    assert!(ray_geometry(r.scene().unwrap(), false).is_empty());
    let mut forged = capture(&mut game);
    forged.header.snapshot.nova_fx.as_mut().unwrap().records =
        data.header.snapshot.nova_fx.unwrap().records;
    assert_eq!(r.stage(forged.header), Err("invalid_nova_fx"));
}
#[test]
fn legacy_absent_is_accepted_explicit_null_unknown_and_missing_fields_are_not() {
    let (mut game, _) = nova_fixture(2);
    let legacy = capture(&mut game);
    let value = serde_json::to_value(&legacy.header).unwrap();
    assert!(value["snapshot"].get("nova_fx").is_none());
    assert!(serde_json::from_value::<Header>(value.clone()).is_ok());
    let mut bad = value;
    bad["snapshot"]["nova_fx"] = Value::Null;
    assert!(serde_json::from_value::<Header>(bad).is_err());
    hit(&mut game, id(1));
    let data = capture(&mut game);
    let value = serde_json::to_value(&data.header).unwrap();
    for field in ["sequence", "dropped", "records"] {
        let mut bad = value.clone();
        bad["snapshot"]["nova_fx"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(serde_json::from_value::<Header>(bad).is_err());
    }
    for field in ["sequence", "tick", "emitter", "epoch", "position", "rays"] {
        let mut bad = value.clone();
        bad["snapshot"]["nova_fx"]["records"][0]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(serde_json::from_value::<Header>(bad).is_err());
    }
    let mut bad = value.clone();
    bad["snapshot"]["nova_fx"]["records"][0]["kind"] = json!("death");
    assert!(serde_json::from_value::<Header>(bad).is_err());
    let mut bad = value;
    bad["snapshot"]["nova_fx"]["profile"] = json!({"currency":[999,999,999]});
    assert!(serde_json::from_value::<Header>(bad).is_err());
}
#[test]
fn invalid_sequences_ticks_positions_ownership_or_counters_retain_scene() {
    let (mut game, epoch) = nova_fixture(2);
    let mut r = baseline(&mut game, epoch);
    hit(&mut game, id(1));
    let data = capture(&mut game);
    for change in 0..10 {
        let mut bad = data.header.clone();
        let fx = bad.snapshot.nova_fx.as_mut().unwrap();
        match change {
            0 => fx.records[0].sequence = 0,
            1 => fx.records[0].sequence = 2,
            2 => fx.records.push(fx.records[0]),
            3 => fx.records[0].tick = bad.snapshot.tick + 1,
            4 => fx.records[0].epoch = 0,
            5 => fx.records[0].epoch += 1,
            6 => fx.records[0].position[0] = f32::NAN,
            7 => fx.records[0].position[1] = 1_000_001.0,
            8 => fx.records[0].rays = 37,
            _ => fx.records[0].emitter = id(999),
        }
        assert_eq!(r.stage(bad), Err("invalid_nova_fx"));
        assert_eq!(r.fx_sequence, Some(0));
        assert_eq!(r.scene().unwrap().fx.rays(), 0);
    }
    stage(&mut r, &data, true);
    commit(&mut r, &data);
    let mut absent = capture(&mut game);
    absent.header.snapshot.nova_fx = None;
    stage(&mut r, &absent, false);
    commit(&mut r, &absent);
    assert_eq!(r.fx_sequence, Some(1));
    let mut regressed = capture(&mut game);
    regressed.header.snapshot.nova_fx.as_mut().unwrap().sequence = 0;
    assert_eq!(r.stage(regressed.header), Err("invalid_nova_fx"));
}
#[test]
fn no_safe_hit_or_foreign_owner_birth_and_sequence_exhaustion_keeps_real_host_effect() {
    let (mut game, _) = nova_fixture(2);
    let host = game.player;
    game.world.get::<&mut Motion>(host).unwrap().position = Vec2::ZERO;
    hit(&mut game, id(1));
    assert!(game.co_op_snapshot().unwrap().nova_fx.is_none());
    game.world.get::<&mut Motion>(host).unwrap().position = Vec2::new(450.0, 0.0);
    game.emit_repulsor_nova(
        Vec2::new(500.0, 0.0),
        1,
        Some(co_op::ShotOwner {
            id: id(999),
            epoch: 1,
        }),
    );
    assert!(game.co_op_snapshot().unwrap().nova_fx.is_none());
    game.particles.clear();
    game.co_op.as_mut().unwrap().nova_fx.sequence = u32::MAX;
    hit(&mut game, id(1));
    assert_eq!(game.particles.len(), 36);
    let fx = game.co_op_snapshot().unwrap().nova_fx.unwrap();
    assert_eq!(fx.sequence, u32::MAX);
    assert_eq!(fx.dropped, 1);
    assert!(fx.records.is_empty());
}

#[test]
fn actual_light_ray_cache_and_journal_bounds_are_independent_and_visible() {
    use crate::co_op_fx::{Cache, Journal, MAX_RECORDS};
    let mut journal = Journal::default();
    for _ in 0..40 {
        journal.record(1, id(1), 1, Vec2::new(450.0, 0.0), 36);
    }
    assert_eq!(journal.records.len(), MAX_RECORDS);
    assert_eq!(journal.dropped, 8);
    let mut cache = Cache::default();
    for &record in &journal.records {
        cache.admit(record, 0.0);
    }
    assert_eq!(cache.rays(), MAX_PARTICLES);
    assert_eq!(cache.dropped_rays, 52);
    assert_eq!(cache.light_count(), 32);
    // Same four selected recent sources as host; exact pool is independently32.
    assert_eq!(cache.sources(|_, _| true).len(), 4);
    let mut full = FrameBuilder::new();
    cache.render(&mut full, false, |_, _| true);
    assert_eq!(geometry(&full.finish()).len(), MAX_PARTICLES);
    cache.advance(0.4);
    assert_eq!(cache.rays(), 0);
    cache.advance(0.451);
    assert!(cache.sources(|_, _| true).is_empty());
    journal.record(100, id(1), 1, Vec2::ZERO, 0);
    assert_eq!(journal.records.len(), 1);
    assert_eq!(journal.dropped, 8);
}

#[test]
#[ignore = "explicit actual native fixture exporter; requires GALAXY_NOVA_FX_FIXTURES"]
fn export_actual_native_nova_fx_headers() {
    let directory = std::path::PathBuf::from(
        std::env::var("GALAXY_NOVA_FX_FIXTURES").expect("explicit directory"),
    );
    std::fs::create_dir_all(&directory).unwrap();
    let (mut game, _) = nova_fixture(8);
    let legacy = capture(&mut game);
    for i in 0..180 {
        assert!(
            game.spawn_bullet_entity(
                Vec2::new(900000.0 + i as f32, -900000.0),
                Vec2::new(-500.0, 600.0),
                12.0,
                true,
                1
            )
            .is_some()
        );
    }
    hit(&mut game, id(1));
    let nova = capture(&mut game);
    // Density stress here is configured directly in the retained journal, not
    // forty natural damage activations during a .45-second window.
    for _ in 0..31 {
        game.co_op
            .as_mut()
            .unwrap()
            .nova_fx
            .record(0, id(1), 1, Vec2::new(450.0, 0.0), 36);
    }
    let maximum = capture(&mut game);
    let bytes = serde_json::to_vec(&json!({"v":1,"session":id(100),"seq":u32::MAX,"kind":"publication","payload":maximum.header})).unwrap().len();
    let request_bytes =
        serde_json::to_vec(&json!({"v":1,"op":"replica_stage","header":maximum.header}))
            .unwrap()
            .len();
    assert!(bytes < 65536);
    assert!(request_bytes < 65536);
    println!(
        "actual_native_nova_fx_header_bytes={} wrapped_maxcounts_transport_bytes={} wrapped_maxcounts_stage_request_bytes={} records=32 pilots=8 projectiles=180",
        serde_json::to_vec(&nova.header).unwrap().len(),
        bytes,
        request_bytes
    );
    for _ in 0..30 {
        game.tick_co_op(id(100)).unwrap();
    }
    let expired = capture(&mut game);
    for (name, header) in [
        ("legacy", legacy.header),
        ("nova", nova.header),
        ("maximum", maximum.header),
        ("expired", expired.header),
    ] {
        std::fs::write(
            directory.join(format!("{name}-header.json")),
            serde_json::to_string_pretty(&header).unwrap() + "\n",
        )
        .unwrap();
    }
}
