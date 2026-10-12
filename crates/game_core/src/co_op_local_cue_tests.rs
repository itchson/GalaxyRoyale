//! Real authority collisions exercise the cues consumed by the host shell.
use super::*;

fn hit_pilot(game: &mut Game, target: ParticipantId, other: ParticipantId, damage: f32) {
    place(game, target, Vec2::new(550.0, 0.0), 0.0);
    place(game, other, Vec2::new(450.0, 150.0), 0.0);
    game.take_events();
    game.spawn_bullet(
        Vec2::new(450.0, 0.0),
        Vec2::new(12_000.0, 0.0),
        damage,
        false,
        1,
    );
    game.tick_co_op(session()).unwrap();
}

#[test]
fn actual_guest_death_respawns_guest_without_host_local_death_or_hit_cues() {
    let mut game = start();
    let admission = guest(&mut game, 2);
    let save = serde_json::to_string(game.profile()).unwrap();
    let host_health = hp(&game, host());
    hit_pilot(&mut game, admission.id, host(), 1000.0);
    let snapshot = game.co_op_snapshot().unwrap();
    let pilot = snapshot
        .participants
        .iter()
        .find(|p| p.id == admission.id)
        .unwrap();
    assert_eq!(
        (pilot.hull, pilot.shield),
        (pilot.max_hull, pilot.max_shield)
    );
    assert!(Vec2::from_array(pilot.position).length() < SAFE_RADIUS);
    assert_eq!(hp(&game, host()), host_health);
    assert!(Vec2::from_array(game.hud().position).length() > SAFE_RADIUS);
    assert_eq!(serde_json::to_string(game.profile()).unwrap(), save);
    let events = game.take_events();
    assert!(
        !events.iter().any(|e| e.kind == "death"),
        "guest cue enters host showDeath()"
    );
    assert!(
        !events.iter().any(|e| e.kind == "hit" && e.intensity >= 0.8),
        "guest impact enters host player-hit audio"
    );
    assert_eq!(events.iter().filter(|e| e.kind == "explosion").count(), 1);
    assert!(!game.paused);
}

#[test]
fn actual_guest_shield_hit_is_world_impact_and_does_not_change_host_vitals() {
    let mut game = start();
    let admission = guest(&mut game, 2);
    let host_health = hp(&game, host());
    let guest_health = hp(&game, admission.id);
    hit_pilot(&mut game, admission.id, host(), 5.0);
    assert_eq!(hp(&game, host()), host_health);
    assert!(hp(&game, admission.id).1 < guest_health.1);
    let events = game.take_events();
    let impacts: Vec<_> = events.iter().filter(|e| e.kind == "hit").collect();
    assert_eq!(impacts.len(), 1);
    assert_eq!(impacts[0].intensity, 0.25);
    assert!(events.iter().any(|e| e.kind == "shield"));
    assert!(!events.iter().any(|e| e.kind == "death"));
    assert!(
        game.transient_lights.iter().any(|light| {
            light.source.radius == 48.0
                && light.source.color == [1.0, 0.55, 0.19]
                && light.source.intensity == 1.0
        }),
        "remote cue classification must preserve the admitted impact light"
    );
}

#[test]
fn actual_host_collision_keeps_local_death_and_player_hit_routing() {
    let mut game = start();
    let admission = guest(&mut game, 2);
    let guest_health = hp(&game, admission.id);
    hit_pilot(&mut game, host(), admission.id, 1000.0);
    assert_eq!(hp(&game, admission.id), guest_health);
    let events = game.take_events();
    assert_eq!(events.iter().filter(|e| e.kind == "death").count(), 1);
    assert!(events.iter().any(|e| e.kind == "hit" && e.intensity == 0.9));
    assert!(Vec2::from_array(game.hud().position).length() < SAFE_RADIUS);
}

#[test]
fn eight_simultaneous_respawns_emit_one_local_death_with_existing_effect_caps() {
    let mut game = start();
    for i in 2..=8 {
        guest(&mut game, i);
    }
    let save = serde_json::to_string(game.profile()).unwrap();
    for _ in 0..MAX_PARTICLES - 30 {
        game.particle(Vec2::ZERO, Vec2::ZERO, 1.0, rgba(1, 2, 3, 255), 1.0);
    }
    for _ in 0..30 {
        game.transient_light(Vec2::ZERO, 48.0, [1.0; 3], 0.5, 1.0);
    }
    for (_, actor) in game.world.query_mut::<(Entity, &mut Actor)>() {
        if actor.kind == Kind::Player {
            actor.health = 0.0;
        }
    }
    game.take_events();
    game.tick_co_op(session()).unwrap();
    let events = game.take_events();
    assert_eq!(events.iter().filter(|e| e.kind == "death").count(), 1);
    assert_eq!(events.iter().filter(|e| e.kind == "explosion").count(), 7);
    assert_eq!(game.particles.len(), MAX_PARTICLES);
    assert_eq!(game.transient_lights.len(), 32);
    let snapshot = game.co_op_snapshot().unwrap();
    assert_eq!(snapshot.participants.len(), 8);
    assert!(
        snapshot
            .participants
            .iter()
            .all(|p| p.hull == p.max_hull && p.shield == p.max_shield)
    );
    assert_eq!(serde_json::to_string(game.profile()).unwrap(), save);
    game.tick_co_op(session()).unwrap();
    assert!(
        !game
            .take_events()
            .iter()
            .any(|e| matches!(e.kind.as_str(), "death" | "explosion"))
    );
}

#[test]
fn full_cue_queue_does_not_reclassify_an_earlier_local_hit() {
    let mut game = start();
    let admission = guest(&mut game, 2);
    place(&mut game, admission.id, Vec2::new(550.0, 0.0), 0.0);
    game.take_events();
    for i in 0..128 {
        game.event("hit", Vec2::new(i as f32, 0.0), 0.9);
    }
    let before = serde_json::to_string(&game.events).unwrap();
    game.hurt_co_op_pilot(entity(&game, admission.id), 5.0, None);
    assert_eq!(serde_json::to_string(&game.events).unwrap(), before);
    assert_eq!(game.transient_lights.len(), 32);
    assert!(hp(&game, admission.id).1 < 30.0);
}
