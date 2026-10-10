//! Actual Game event probes; declared QA/capacity/damage-mask fixtures.
use super::*;
use crate::{GameEvent, InputFrame, Profile};
const DT: f32 = 1.0 / 60.0;
fn fixture() -> (Game, Entity) {
    let mut game = Game::new(Profile::default());
    game.set_viewport(640.0, 360.0);
    game.diagnostic("biome-mote");
    game.spawn_timer = 10000.0;
    game.boss_timer = 10000.0;
    game.take_events();
    let entity = game
        .world
        .query::<(Entity, &Mote)>()
        .iter()
        .next()
        .unwrap()
        .0;
    (game, entity)
}
fn tick(game: &mut Game) -> Vec<GameEvent> {
    game.tick(&InputFrame::default(), DT);
    game.take_events()
}
fn count(events: &[GameEvent], kind: &str) -> usize {
    events.iter().filter(|event| event.kind == kind).count()
}
fn charge(game: &mut Game, entity: Entity) -> GameEvent {
    for _ in 0..600 {
        let events = tick(game);
        assert_eq!(count(&events, "mote-pulse"), 0);
        if game.world.get::<&Mote>(entity).unwrap().controller.phase() == Phase::Charge {
            assert_eq!(count(&events, "mote-charge"), 1);
            return events
                .into_iter()
                .find(|event| event.kind == "mote-charge")
                .unwrap();
        }
        assert_eq!(count(&events, "mote-charge"), 0);
    }
    panic!("real Mote fixture did not enter Charge")
}
fn hostile(game: &Game) -> Vec<Entity> {
    game.world
        .query::<(Entity, &Bullet)>()
        .iter()
        .filter_map(|(entity, bullet)| (!bullet.player).then_some(entity))
        .collect()
}

#[test]
fn real_charge_and_two_real_pulses_emit_one_cue_each_without_changing_reward_state() {
    let (mut game, entity) = fixture();
    let before = serde_json::to_value(&game.profile).unwrap();
    let cue = charge(&mut game, entity);
    let origin = game.world.get::<&Motion>(entity).unwrap().previous;
    assert_eq!([cue.x, cue.y], origin.to_array());
    let mut emissions = Vec::new();
    for step in 1..=100 {
        let before_count = hostile(&game).len();
        let events = tick(&mut game);
        assert_eq!(
            count(&events, "mote-charge"),
            0,
            "charge must not repeat on every warning tick"
        );
        let created = hostile(&game).len().saturating_sub(before_count);
        assert!(created <= 6);
        assert_eq!(count(&events, "mote-pulse"), usize::from(created > 0));
        if created > 0 {
            assert_eq!(created, 6);
            let cue = events.iter().find(|e| e.kind == "mote-pulse").unwrap();
            assert_eq!(cue.intensity, 1.0);
            assert!(cue.x.is_finite() && cue.y.is_finite());
            emissions.push(step);
        }
    }
    assert_eq!(emissions, [66, 87]);
    assert_eq!(serde_json::to_value(&game.profile).unwrap(), before);
    assert!(
        game.transient_lights.is_empty(),
        "audio kinds do not create gameplay or lighting side effects"
    );
}

#[test]
fn cancelled_warning_has_no_pulse_or_repeated_charge_and_requires_a_new_entry() {
    let (mut game, entity) = fixture();
    charge(&mut game, entity);
    game.set_viewport(120.0, 80.0);
    for _ in 0..120 {
        let events = tick(&mut game);
        assert_eq!(count(&events, "mote-charge"), 0);
        assert_eq!(count(&events, "mote-pulse"), 0);
        assert!(hostile(&game).is_empty());
    }
    game.set_viewport(640.0, 360.0);
    charge(&mut game, entity);
    game.move_player(Vec2::ZERO);
    for _ in 0..30 {
        let events = tick(&mut game);
        assert_eq!(count(&events, "mote-charge"), 0);
        assert_eq!(count(&events, "mote-pulse"), 0);
    }
}

#[test]
fn actual_primary_taking_last_slot_suppresses_mote_audio_but_later_real_pulse_sounds_once() {
    let (mut game, entity) = fixture();
    charge(&mut game, entity);
    let p = game.position();
    for _ in 0..MAX_BULLETS - 1 {
        game.spawn_bullet(p + Vec2::X * 1000.0, Vec2::ZERO, 1.0, false, 1);
    }
    for _ in 0..65 {
        let events = tick(&mut game);
        assert_eq!(count(&events, "mote-pulse"), 0);
    }
    game.tick(
        &InputFrame {
            fire: true,
            ..Default::default()
        },
        DT,
    );
    let events = game.take_events();
    assert_eq!(count(&events, "shot"), 1);
    assert_eq!(
        count(&events, "mote-pulse"),
        0,
        "scheduled but rejected pulse must stay silent"
    );
    assert_eq!(game.world.query::<&Bullet>().iter().count(), MAX_BULLETS);
    let bullets: Vec<_> = game
        .world
        .query::<(Entity, &Bullet)>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    for e in bullets {
        let _ = game.world.despawn(e);
    }
    for _ in 0..20 {
        let events = tick(&mut game);
        assert_eq!(
            count(&events, "mote-pulse"),
            0,
            "suppressed first pulse must not replay audio or shots"
        );
    }
    let events = tick(&mut game);
    assert_eq!(hostile(&game).len(), 6);
    assert_eq!(count(&events, "mote-pulse"), 1);
}

#[test]
fn destroyed_emitters_and_full_cap_do_not_produce_phantom_discharge_events() {
    for erase in [false, true] {
        let (mut game, entity) = fixture();
        charge(&mut game, entity);
        if erase {
            let mut body = game.world.get::<&mut PixelBody>(entity).unwrap();
            for mask in controller::emitter_masks() {
                for &[x, y] in mask {
                    body.damage(
                        Vec2::new(x as f32 + 0.5 - 20.0, y as f32 + 0.5 - 20.0),
                        0.01,
                    );
                }
            }
            assert!(body.occupied_count() > 0);
        } else {
            for _ in 0..MAX_BULLETS {
                game.spawn_bullet(
                    game.position() + Vec2::X * 1000.0,
                    Vec2::ZERO,
                    1.0,
                    false,
                    1,
                );
            }
        }
        for _ in 0..110 {
            let events = tick(&mut game);
            assert_eq!(count(&events, "mote-pulse"), 0);
        }
    }
}

#[test]
fn one_remaining_slot_emits_one_real_ray_and_one_lower_intensity_actor_cue() {
    let (mut game, entity) = fixture();
    charge(&mut game, entity);
    for _ in 0..MAX_BULLETS - 1 {
        game.spawn_bullet(
            game.position() + Vec2::X * 1000.0,
            Vec2::ZERO,
            1.0,
            false,
            1,
        );
    }
    for _ in 0..65 {
        tick(&mut game);
    }
    let events = tick(&mut game);
    assert_eq!(hostile(&game).len(), MAX_BULLETS);
    assert_eq!(count(&events, "mote-pulse"), 1);
    assert!(
        (events
            .iter()
            .find(|e| e.kind == "mote-pulse")
            .unwrap()
            .intensity
            - 1.0 / 6.0)
            .abs()
            < 0.000001
    );
}

#[test]
fn pause_and_invalid_time_do_not_emit_events_or_advance_gameplay() {
    let (mut game, entity) = fixture();
    charge(&mut game, entity);
    let time = game.time;
    let age = game
        .world
        .get::<&Mote>(entity)
        .unwrap()
        .controller
        .phase_age();
    game.set_paused(true);
    for _ in 0..120 {
        assert!(tick(&mut game).is_empty());
    }
    game.set_paused(false);
    for dt in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        game.tick(&InputFrame::default(), dt);
        assert!(game.take_events().is_empty());
    }
    assert_eq!(game.time, time);
    assert_eq!(
        game.world
            .get::<&Mote>(entity)
            .unwrap()
            .controller
            .phase_age(),
        age
    );
}
