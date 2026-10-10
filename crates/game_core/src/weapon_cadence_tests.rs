//! Cadence regressions exercise actual Game::tick and real primary/Echo shots.
use super::*;

const DT: f32 = 1.0 / 60.0;
fn fixture(rank: u8, echo: bool) -> Game {
    let mut p = Profile::default();
    p.ranks[..6].fill(5);
    p.ranks[1] = rank;
    p.campaign.ship_xp[0] = 1320;
    if echo {
        p.unlocked[0] = true;
        p.ranks[6] = 5;
    }
    let mut game = Game::new(p);
    game.set_viewport(800.0, 360.0);
    game.diagnostic("boss-dart");
    game.take_events();
    game
}
fn tick(game: &mut Game, fire: bool, dt: f32) -> usize {
    let hud = game.hud();
    let target = if let Some(boss) = hud.boss {
        Vec2::from_array(if boss.phase == "Core exposed" {
            boss.core_position
        } else {
            boss.position
        })
    } else {
        Vec2::from_array(hud.position) + Vec2::X * 100.0
    };
    let aim = (target - Vec2::from_array(hud.position)).normalize_or_zero() * 0.081;
    let before = game.shot_count;
    game.tick(
        &InputFrame {
            joystick: Some(aim.to_array()),
            fire,
            ..Default::default()
        },
        dt,
    );
    let events = game.take_events();
    assert!(
        !events.iter().any(|e| e.kind == "death"),
        "Declared diagnostic fixture died unexpectedly"
    );
    let shots = events.iter().filter(|e| e.kind == "shot").count();
    assert!(shots <= 1, "Multiple primary shots in one tick");
    assert_eq!(game.shot_count - before, shots as u32);
    assert!(game.fire_timer.is_finite());
    shots
}
fn clear_area(rank: u8, echo: bool) -> Game {
    let mut game = fixture(rank, echo);
    for _ in 0..1200 {
        tick(&mut game, true, DT);
        if game.hud().boss.is_none() {
            break;
        }
    }
    assert!(
        game.hud().boss.is_none(),
        "Actual projectile fixture did not clear the diagnostic boss"
    );
    for _ in 0..120 {
        assert_eq!(tick(&mut game, false, DT), 0);
    }
    assert_eq!(game.fire_timer, 0.0);
    assert!(!game.safe());
    game
}

#[test]
fn all_six_actual_steady_cadences_are_authored_and_strictly_increasing() {
    let mut previous = 0;
    for rank in 0..=5 {
        let mut game = fixture(rank, false);
        let before = game.profile.campaign.ship_xp;
        let mut steps = Vec::new();
        for step in 0..1800 {
            if tick(&mut game, true, DT) == 1 {
                steps.push(step + 1);
            }
        }
        let count = steps.len();
        let rate = 5.0 * (1.0 + 0.1 * rank as f32);
        assert!((count as f32 - rate * 30.0).abs() <= 1.0);
        assert!(
            count > previous,
            "Purchased rank produced no cadence improvement"
        );
        previous = count;
        let intervals: Vec<_> = steps.windows(2).map(|w| w[1] - w[0]).collect();
        let min = (60.0 / rate).floor() as usize;
        let max = (60.0 / rate).ceil() as usize;
        assert!(
            intervals
                .iter()
                .all(|n| (*n as usize) >= min && (*n as usize) <= max),
            "rank{rank}: expected intervals {min}..{max}; actual {:?}",
            intervals
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
        );
        assert_eq!(game.profile.campaign.ship_xp, before);
    }
}

#[test]
fn long_idle_has_immediate_ready_shot_then_a_full_new_period() {
    for rank in 0..=5 {
        let mut game = clear_area(rank, false);
        let count = game.shot_count;
        for _ in 0..1800 {
            assert_eq!(tick(&mut game, false, DT), 0);
        }
        assert_eq!(game.shot_count, count);
        assert_eq!(game.fire_timer, 0.0);
        assert_eq!(tick(&mut game, true, DT), 1);
        let period = 1.0 / (5.0 * (1.0 + 0.1 * rank as f32));
        assert!((game.fire_timer - period).abs() < 0.00001);
        let fired_at = game.time;
        let mut next = None;
        for _ in 0..20 {
            if tick(&mut game, true, DT) == 1 {
                next = Some(game.time);
                break;
            }
        }
        assert!(next.unwrap() - fired_at >= period - 0.00001);
    }
}

#[test]
fn short_release_and_pause_keep_a_positive_remaining_cooldown() {
    for rank in 0..=5 {
        let mut game = clear_area(rank, false);
        assert_eq!(tick(&mut game, true, DT), 1);
        let fired_at = game.time;
        for _ in 0..3 {
            assert_eq!(tick(&mut game, false, DT), 0);
        }
        assert!(game.fire_timer > 0.0);
        let remaining = game.fire_timer;
        let time = game.time;
        game.set_paused(true);
        for _ in 0..600 {
            assert_eq!(tick(&mut game, true, DT), 0);
        }
        assert_eq!(game.fire_timer, remaining);
        assert_eq!(game.time, time);
        game.set_paused(false);
        assert_eq!(
            tick(&mut game, true, DT),
            0,
            "Short release reset readiness"
        );
        let period = 1.0 / (5.0 * (1.0 + 0.1 * rank as f32));
        let mut next = None;
        for _ in 0..20 {
            if tick(&mut game, true, DT) == 1 {
                next = Some(game.time);
                break;
            }
        }
        assert!(next.unwrap() - fired_at >= period - 0.00001);
        assert!(next.unwrap() - fired_at <= period + DT + 0.00001);
    }
}

#[test]
fn held_fire_in_haven_then_retry_has_no_sanctuary_debt() {
    let mut game = clear_area(5, false);
    assert_eq!(tick(&mut game, true, DT), 1);
    game.reset_to_dock();
    assert!(game.safe());
    let before = game.shot_count;
    for _ in 0..1800 {
        assert_eq!(tick(&mut game, true, DT), 0);
    }
    assert_eq!(game.fire_timer, 0.0);
    assert_eq!(game.shot_count, before);
    assert!(game.world.query::<&Bullet>().iter().next().is_none());
    game.diagnostic("boss-dart");
    game.take_events();
    assert_eq!(tick(&mut game, true, DT), 1);
    let period = 1.0 / 7.5;
    assert!((game.fire_timer - period).abs() < 0.00001);
    assert_eq!(tick(&mut game, true, DT), 0);
}

#[test]
fn diagnostic_death_haven_hold_and_fresh_retry_do_not_bank_a_burst() {
    // The existing die command sets zero hull without resetting since_damage;
    // active regeneration can revive that diagnostic before death processing.
    // This declared regen-zero fixture exercises the actual death-return path.
    let cleared = clear_area(2, false);
    let mut p = cleared.profile.clone();
    p.ranks[4] = 0;
    let mut game = Game::new(p);
    game.diagnostic("boss-dart");
    game.take_events();
    let before_xp = game.profile.campaign.ship_xp;
    game.diagnostic("die");
    game.tick(&InputFrame::default(), DT);
    assert!(game.take_events().iter().any(|e| e.kind == "death"));
    assert!(game.safe());
    let before = game.shot_count;
    for _ in 0..600 {
        assert_eq!(tick(&mut game, true, DT), 0);
    }
    assert_eq!(game.shot_count, before);
    assert_eq!(game.fire_timer, 0.0);
    game.diagnostic("boss-dart");
    game.take_events();
    assert_eq!(tick(&mut game, true, DT), 1);
    assert_eq!(tick(&mut game, true, DT), 0);
    assert_eq!(game.profile.campaign.ship_xp, before_xp);
}

#[test]
fn automatic_echo_tracks_actual_primaries_after_cadence_idle_and_pause() {
    for rank in 0..=5 {
        let mut game = clear_area(rank, true);
        let start = game.shot_count;
        let mut counted = 0;
        for step in 0..900 {
            if step == 10 {
                game.set_paused(true);
                assert_eq!(tick(&mut game, true, DT), 0);
                game.set_paused(false);
            }
            let prior: HashSet<_> = game
                .world
                .query::<(Entity, &Bullet)>()
                .iter()
                .filter(|(_, b)| b.player && b.remaining_hits > 1)
                .map(|(e, _)| e)
                .collect();
            let fired = tick(&mut game, step % 37 != 0, DT);
            counted += fired;
            let added = game
                .world
                .query::<(Entity, &Bullet)>()
                .iter()
                .filter(|(e, b)| b.player && b.remaining_hits > 1 && !prior.contains(e))
                .count();
            let expected = if fired == 1 && game.shot_count.is_multiple_of(8) {
                3
            } else {
                0
            };
            assert_eq!(
                added, expected,
                "Real Echo projectile creation does not match primary count"
            );
            assert_eq!(game.shot_count, start + counted as u32);
            if counted >= 40 {
                break;
            }
        }
        assert_eq!(counted, 40);
    }
}

#[test]
fn accepted_variable_steps_and_invalid_time_never_create_two_primaries() {
    let mut game = clear_area(5, false);
    let steps = [1.0 / 120.0, DT, 0.05, 0.025];
    let mut total = 0;
    for step in 0..1200 {
        total += tick(&mut game, true, steps[step % steps.len()]);
    }
    assert!(total > 100);
    let before = game.shot_count;
    let timer = game.fire_timer;
    let time = game.time;
    for dt in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        assert_eq!(tick(&mut game, true, dt), 0);
    }
    assert_eq!(game.shot_count, before);
    assert_eq!(game.fire_timer, timer);
    assert_eq!(game.time, time);
}
