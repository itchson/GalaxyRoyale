//! Actual fixed-tick controller/projectile checks. Quiet-space fixtures isolate
//! desktop steering from naturally spawned combat; no reward or save mutation.
use super::*;

const DT: f32 = 1.0 / 60.0;

fn fixture() -> Game {
    let mut game = Game::new(Profile::default());
    game.move_player(Vec2::new(450.0, 0.0));
    game.spawn_timer = 10_000.0;
    game.boss_timer = 10_000.0;
    {
        let mut motion = game.world.get::<&mut Motion>(game.player).unwrap();
        motion.heading = 0.0;
        motion.previous_heading = 0.0;
    }
    assert!(!game.safe());
    game
}

fn motion(game: &Game) -> Motion {
    *game.world.get::<&Motion>(game.player).unwrap()
}

fn aiming(point: Vec2) -> InputFrame {
    InputFrame {
        aim: Some(point.to_array()),
        fire: true,
        ..Default::default()
    }
}

#[test]
fn stationary_right_fire_turns_smoothly_and_shoots_from_the_actual_nose() {
    let mut game = fixture();
    let origin = game.position();
    let input = aiming(origin + Vec2::Y * 180.0);
    game.tick(&input, DT);
    let first = motion(&game);
    assert!((first.heading - 4.0 * PI * DT).abs() < 0.00001);
    assert_eq!(first.position, origin);
    assert_eq!(first.velocity, Vec2::ZERO);
    let shots: Vec<_> = game
        .world
        .query::<(&Bullet, &Motion)>()
        .iter()
        .filter_map(|(b, m)| b.player.then_some(*m))
        .collect();
    assert_eq!(shots.len(), 1);
    assert!((shots[0].velocity - forward(first.heading) * 430.0).length() < 0.001);
    // A short click fires during the turn; it does not teleport ship facing.
    assert!(shots[0].velocity.x > 400.0);
    for _ in 0..30 {
        game.tick(&input, DT);
    }
    assert!((motion(&game).heading - PI * 0.5).abs() < 0.00001);
    assert_eq!(game.position(), origin);
    assert!(
        game.world
            .query::<(&Bullet, &Motion)>()
            .iter()
            .any(|(b, m)| b.player && m.velocity.x.abs() < 0.001 && m.velocity.y > 429.0)
    );
}

#[test]
fn aiming_preserves_left_course_and_arrival_while_release_resumes_course_facing() {
    let mut aimed = fixture();
    let mut navigation = fixture();
    let target = Vec2::new(630.0, 100.0);
    let cursor = Vec2::new(450.0, -600.0);
    for step in 0..360 {
        let course = (step == 0).then_some(target.to_array());
        navigation.tick(
            &InputFrame {
                destination: course,
                ..Default::default()
            },
            DT,
        );
        aimed.tick(
            &InputFrame {
                destination: course,
                ..aiming(cursor)
            },
            DT,
        );
        assert!(aimed.position().distance(navigation.position()) < 0.001);
        assert!((motion(&aimed).velocity - motion(&navigation).velocity).length() < 0.001);
        if step == 10 {
            assert_eq!(aimed.destination, Some(target));
            assert!(motion(&aimed).heading < -1.0);
            assert!(motion(&navigation).heading > 0.0);
        }
    }
    assert!(aimed.position().distance(target) < 4.0);
    assert!(motion(&aimed).velocity.length() < 1.0);
    assert_eq!(aimed.destination, None);

    let mut released = fixture();
    released.tick(
        &InputFrame {
            destination: Some(target.to_array()),
            ..aiming(cursor)
        },
        DT,
    );
    let before = motion(&released).heading;
    released.tick(&InputFrame::default(), DT);
    assert_eq!(released.destination, Some(target));
    assert!(motion(&released).heading > before);
}

#[test]
fn keyboard_thrust_and_release_drift_do_not_curve_toward_the_aim_cursor() {
    let mut game = fixture();
    let cursor = Vec2::new(450.0, -800.0);
    for _ in 0..60 {
        game.tick(
            &InputFrame {
                joystick: Some([1.0, 0.0]),
                ..aiming(cursor)
            },
            DT,
        );
    }
    let moving = motion(&game);
    assert!(moving.velocity.x > 99.0);
    assert_eq!(moving.velocity.y, 0.0);
    assert!(moving.heading < -1.0);
    for _ in 0..15 {
        game.tick(&aiming(cursor), DT);
        assert_eq!(motion(&game).velocity.y, 0.0);
    }
    assert_eq!(game.throttle, 0.0);
    assert!(motion(&game).velocity.x < moving.velocity.x);
    // Primary bullets preserve the existing quarter-velocity inheritance.
    game.fire_timer = 0.0;
    game.tick(&aiming(cursor), DT);
    let ship = motion(&game);
    assert!(
        game.world
            .query::<(&Bullet, &Motion)>()
            .iter()
            .any(|(b, m)| {
                b.player
                    && (m.velocity - ship.velocity * 0.25 - forward(ship.heading) * 430.0).length()
                        < 0.001
            })
    );
}

#[test]
fn invalid_close_or_inactive_aim_keeps_existing_controller_behavior() {
    for point in [
        [f32::NAN, 0.0],
        [f32::INFINITY, 0.0],
        [0.0, -f32::INFINITY],
        [WORLD_LIMIT + 1.0, 0.0],
        [450.0, 0.0],
        [454.0, 0.0],
    ] {
        let mut base = fixture();
        let mut candidate = fixture();
        let input = InputFrame {
            joystick: Some([0.0, 1.0]),
            fire: true,
            ..Default::default()
        };
        base.tick(&input, DT);
        candidate.tick(
            &InputFrame {
                aim: Some(point),
                ..input
            },
            DT,
        );
        assert_eq!(motion(&candidate).heading, motion(&base).heading);
        assert_eq!(motion(&candidate).velocity, motion(&base).velocity);
        assert_eq!(candidate.position(), base.position());
        assert!(candidate.position().is_finite());
    }
    let mut game = fixture();
    game.tick(
        &InputFrame {
            aim: Some([450.0, -800.0]),
            destination: Some([630.0, 100.0]),
            fire: false,
            ..Default::default()
        },
        DT,
    );
    assert!(motion(&game).heading > 0.0);
}

#[test]
fn cursor_aim_does_not_bypass_sanctuary_or_pause() {
    let mut game = Game::new(Profile::default());
    let input = aiming(Vec2::new(0.0, 180.0));
    for _ in 0..30 {
        game.tick(&input, DT);
    }
    assert!(game.safe());
    assert_eq!(game.shot_count, 0);
    assert_eq!(game.world.query::<&Bullet>().iter().count(), 0);
    game.set_paused(true);
    let before = motion(&game);
    let time = game.time;
    game.tick(&aiming(Vec2::new(-180.0, 0.0)), DT);
    assert_eq!(motion(&game).heading, before.heading);
    assert_eq!(game.position(), before.position);
    assert_eq!(game.time, time);
    game.set_paused(false);
    game.tick(&InputFrame::default(), DT);
    assert_eq!(motion(&game).heading, before.heading);
    assert_eq!(game.shot_count, 0);
}

#[test]
fn legacy_inputs_default_to_heading_fire_without_a_save_schema_change() {
    let legacy: InputFrame = serde_json::from_str(r#"{"fire":true}"#).unwrap();
    assert!(legacy.aim.is_none());
    let current: InputFrame = serde_json::from_str(r#"{"fire":true,"aim":[450,-180]}"#).unwrap();
    assert_eq!(current.aim, Some([450.0, -180.0]));
    let profile = serde_json::to_value(Profile::default()).unwrap();
    assert!(profile.get("aim").is_none());
    let mut game = fixture();
    game.tick(&legacy, DT);
    assert_eq!(motion(&game).heading, 0.0);
    assert!(
        game.world
            .query::<(&Bullet, &Motion)>()
            .iter()
            .any(|(b, m)| { b.player && m.velocity.x > 429.0 && m.velocity.y == 0.0 })
    );
}
