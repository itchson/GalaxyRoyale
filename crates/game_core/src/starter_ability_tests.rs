//! Real tick/projectile probes in declared quiet QA fixtures; no play-balance claim.
use super::*;

const DT: f32 = 1.0 / 60.0;

fn licensed(main: &str) -> Game {
    Game::new(identity::create_profile("dora", main, "hugo", "0123456789abcdef").unwrap())
}

fn outside(game: &mut Game) {
    game.diagnostic("warp 300 0");
    assert_eq!(game.position(), Vec2::new(300.0, 0.0));
    game.take_events();
    assert!(!game.safe());
}

fn close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.0002, "{actual} != {expected}");
}

fn checkpoint(game: &Game) -> String {
    serde_json::to_string(game.profile()).unwrap()
}

fn step(game: &mut Game, fire: bool, dt: f32) -> (usize, Vec<(f32, f32)>) {
    let previous: HashSet<_> = game
        .world
        .query::<(Entity, &Bullet)>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    let before = game.shot_count;
    game.tick(
        &InputFrame {
            fire,
            aim: Some([700.0, 0.0]),
            ..Default::default()
        },
        dt,
    );
    let events = game.take_events();
    assert!(
        !events
            .iter()
            .any(|e| e.kind == "death" || e.kind == "ability-echo")
    );
    let shots = events.iter().filter(|e| e.kind == "shot").count();
    assert!(shots <= 1);
    assert_eq!(game.shot_count - before, shots as u32);
    let added = game
        .world
        .query::<(Entity, (&Motion, &Bullet))>()
        .iter()
        .filter(|(e, (_, b))| b.player && !previous.contains(e))
        .map(|(_, (m, b))| (b.damage, m.velocity.length()))
        .collect();
    (shots, added)
}

fn next_shot(game: &mut Game) -> Vec<(f32, f32)> {
    for _ in 0..60 {
        let (shots, added) = step(game, true, DT);
        if shots == 1 {
            return added;
        }
        assert!(added.is_empty());
    }
    panic!("No actual admitted primary within one second");
}

fn clear_bullets(game: &mut Game) {
    let bullets: Vec<_> = game
        .world
        .query::<(Entity, &Bullet)>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    for e in bullets {
        game.world.despawn(e).unwrap();
    }
}

#[test]
fn every_fifth_actual_primary_adds_one_stronger_bolt_at_all_accepted_step_sizes() {
    for dt in [1.0 / 120.0, DT, 0.05] {
        let mut game = licensed("renko");
        outside(&mut game);
        let profile = checkpoint(&game);
        let primary = game.hud().ship_stats.primary_damage;
        let mut shots = 0;
        let mut bonuses = 0;
        for _ in 0..1000 {
            let (fired, added) = step(&mut game, true, dt);
            shots += fired;
            let bonus = fired == 1 && shots % 5 == 0;
            assert_eq!(added.len(), fired + usize::from(bonus));
            if fired == 1 {
                assert!(
                    added
                        .iter()
                        .any(|(damage, speed)| (*damage - primary).abs() < 0.0001
                            && (*speed - 430.0).abs() < 0.001)
                );
            }
            if bonus {
                bonuses += 1;
                assert!(
                    added
                        .iter()
                        .any(|(damage, speed)| (*damage - primary * 1.5).abs() < 0.0001
                            && (*speed - 500.0).abs() < 0.001)
                );
            }
            if shots == 20 {
                break;
            }
        }
        assert_eq!((shots, bonuses), (20, 4));
        assert_eq!(game.shot_count, 20);
        assert_eq!(checkpoint(&game), profile); // QA shots never award progress.
    }
}

#[test]
fn capacity_refusal_does_not_count_a_primary_and_dropped_fifth_bonus_never_replays() {
    let mut game = licensed("renko");
    outside(&mut game);
    for _ in 0..4 {
        assert_eq!(next_shot(&mut game).len(), 1);
    }
    while game.world.query::<&Bullet>().iter().count() < MAX_BULLETS {
        game.spawn_bullet(Vec2::new(10000.0, 5000.0), Vec2::ZERO, 1.0, false, 1);
    }
    for _ in 0..12 {
        let (shots, added) = step(&mut game, true, DT);
        assert_eq!(shots, 0);
        assert!(added.is_empty());
        assert_eq!(game.shot_count, 4);
    }
    assert_eq!(game.world.query::<&Bullet>().iter().count(), MAX_BULLETS);
    let filler = game
        .world
        .query::<(Entity, &Bullet)>()
        .iter()
        .find(|(_, b)| !b.player)
        .unwrap()
        .0;
    game.world.despawn(filler).unwrap(); // Exactly one admitted slot.
    let added = next_shot(&mut game);
    assert_eq!(game.shot_count, 5);
    assert_eq!(added.len(), 1); // Primary takes the final slot; stronger bolt is refused.
    close(added[0].0, game.hud().ship_stats.primary_damage);
    assert_eq!(game.world.query::<&Bullet>().iter().count(), MAX_BULLETS);
    clear_bullets(&mut game);
    assert_eq!(next_shot(&mut game).len(), 1); // Shot 6 does not replay shot 5's dropped bolt.
    for _ in 7..10 {
        assert_eq!(next_shot(&mut game).len(), 1);
    }
    assert_eq!(next_shot(&mut game).len(), 2);
    assert_eq!(game.shot_count, 10);
}

#[test]
fn safe_hold_pause_short_release_unequip_and_reload_do_not_fabricate_bonus_shots() {
    let mut game = licensed("renko");
    for _ in 0..600 {
        assert_eq!(step(&mut game, true, DT).0, 0);
    }
    assert_eq!(game.shot_count, 0);
    assert_eq!(game.world.query::<&Bullet>().iter().count(), 0);
    outside(&mut game);
    for _ in 0..4 {
        assert_eq!(next_shot(&mut game).len(), 1);
    }
    for _ in 0..3 {
        assert_eq!(step(&mut game, false, DT).0, 0);
    }
    let remaining = game.fire_timer;
    assert!(remaining > 0.0);
    game.set_paused(true);
    for _ in 0..300 {
        assert_eq!(step(&mut game, true, DT).0, 0);
    }
    close(game.fire_timer, remaining);
    game.set_paused(false);
    assert_eq!(next_shot(&mut game).len(), 2);
    let saved = checkpoint(&game);
    let mut reload = Game::new(serde_json::from_str(&saved).unwrap());
    assert_eq!(checkpoint(&reload), saved);
    outside(&mut reload);
    assert_eq!(next_shot(&mut reload).len(), 1);
    assert_eq!(reload.shot_count, 1); // No saved combat counter.
    reload.reset_to_dock();
    reload.equip_ability(0, None).unwrap();
    outside(&mut reload);
    for _ in 0..10 {
        assert_eq!(next_shot(&mut reload).len(), 1);
    }
    reload.reset_to_dock();
    reload.equip_ability(0, Some("pulse-lens")).unwrap();
    outside(&mut reload);
    for _ in 0..4 {
        assert_eq!(next_shot(&mut reload).len(), 1);
    }
    assert_eq!(next_shot(&mut reload).len(), 2);
    assert_eq!(reload.profile.campaign.ship_xp, [0; 10]);
    assert_eq!(reload.profile.currency, [0; 3]);
}

#[test]
fn stronger_fifth_bolt_hits_real_surviving_cells_and_deals_its_authored_damage() {
    let mut game = licensed("renko");
    outside(&mut game);
    for _ in 0..4 {
        next_shot(&mut game);
    }
    clear_bullets(&mut game); // Earlier actual shots have passed this declared target.
    let target = game.world.spawn((
        Motion::new(game.position() + Vec2::X * 60.0, Vec2::ZERO, -FRAC_PI_2),
        Actor {
            kind: Kind::Asteroid,
            health: 200.0,
            max_health: 200.0,
            radius: 15.0,
            scale: 1.0,
            damage: 0.0,
        },
        PixelBody::from_ascii(
            &["####################"; 20],
            &[('#', rgba(150, 170, 180, 255))],
        ),
    ));
    let primary = game.hud().ship_stats.primary_damage;
    assert_eq!(next_shot(&mut game).len(), 2);
    for _ in 0..12 {
        step(&mut game, false, DT);
    }
    let actor = game.world.get::<&Actor>(target).unwrap();
    close(actor.health, 200.0 - primary * 2.5);
    assert!(
        game.world
            .get::<&PixelBody>(target)
            .unwrap()
            .occupied_count()
            < 400
    );
    assert_eq!(game.profile.campaign.ship_xp, [0; 10]);
}

#[test]
fn ordinary_boss_echo_stays_a_separate_eighth_shot_ability() {
    let mut profile = identity::create_profile("dora", "renko", "pan", "0123456789abcdef").unwrap();
    profile.unlocked[0] = true;
    profile.kills[0] = 24;
    profile.boss_kills[0] = 1;
    profile.boss_generations[0][0] = 1;
    profile.campaign.abilities[3] = campaign::AbilityState {
        blueprint: true,
        rank: 1,
    };
    profile.campaign.loadouts[0].slots[0] = Some(3);
    let mut game = Game::new(profile);
    outside(&mut game);
    let mut fired = 0;
    let mut echo = 0;
    for _ in 0..120 {
        game.tick(
            &InputFrame {
                fire: true,
                ..Default::default()
            },
            DT,
        );
        let events = game.take_events();
        fired += events.iter().filter(|e| e.kind == "shot").count();
        let cue = events.iter().filter(|e| e.kind == "ability-echo").count();
        if cue > 0 {
            assert_eq!(fired, 8);
            echo += cue;
        }
        assert!(
            game.world
                .query::<(&Motion, &Bullet)>()
                .iter()
                .all(|(m, b)| !b.player || (m.velocity.length() - 500.0).abs() > 0.01)
        );
        if fired == 8 {
            break;
        }
    }
    assert_eq!((fired, echo), (8, 1));
}
