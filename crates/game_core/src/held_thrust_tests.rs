use super::*;
const DT: f32 = 1.0 / 60.0;

fn fixture() -> Game {
    let mut game = Game::new(Profile::default());
    game.move_player(Vec2::new(450.0, 0.0));
    game.spawn_timer = 10000.0;
    game.boss_timer = 10000.0;
    let mut m = game.world.get::<&mut Motion>(game.player).unwrap();
    m.heading = 0.0;
    m.previous_heading = 0.0;
    drop(m);
    game
}

#[test]
fn held_left_at_arrival_has_full_power_and_release_resumes_braking() {
    let mut game = fixture();
    for _ in 0..60 {
        let target = game.position();
        game.tick(
            &InputFrame {
                thrust: Some(target.to_array()),
                ..Default::default()
            },
            DT,
        );
        assert_eq!(game.throttle, 1.0);
        assert_eq!(game.destination, Some(target));
        assert_eq!(game.world.get::<&Motion>(game.player).unwrap().heading, 0.0);
    }
    let before = game
        .world
        .get::<&Motion>(game.player)
        .unwrap()
        .velocity
        .length();
    assert!((before - Stats::from_profile(&game.profile).speed).abs() < 0.001);
    game.tick(&InputFrame::default(), DT);
    assert!(game.throttle < 1.0);
    assert!(
        game.world
            .get::<&Motion>(game.player)
            .unwrap()
            .velocity
            .length()
            < before
    );
    game.cancel_course();
    for _ in 0..120 {
        game.tick(&InputFrame::default(), DT);
    }
    assert_eq!(game.destination, None);
    assert!(
        game.world
            .get::<&Motion>(game.player)
            .unwrap()
            .velocity
            .length()
            < 0.2
    );
}

#[test]
fn held_left_turns_smoothly_and_right_aim_thrusts_along_actual_facing() {
    let mut game = fixture();
    let p = game.position();
    game.tick(
        &InputFrame {
            thrust: Some((p + Vec2::Y * 100.0).to_array()),
            ..Default::default()
        },
        DT,
    );
    let m = *game.world.get::<&Motion>(game.player).unwrap();
    assert!((m.heading - 4.0 * PI * DT).abs() < 0.00001);
    assert!(m.velocity.normalize().distance(forward(m.heading)) < 0.00001);
    let mut aimed = fixture();
    let p = aimed.position();
    aimed.tick(
        &InputFrame {
            thrust: Some((p - Vec2::Y * 100.0).to_array()),
            aim: Some((p + Vec2::Y * 100.0).to_array()),
            fire: true,
            ..Default::default()
        },
        DT,
    );
    let m = *aimed.world.get::<&Motion>(aimed.player).unwrap();
    assert!((m.heading - 4.0 * PI * DT).abs() < 0.00001);
    assert!(m.velocity.normalize().distance(forward(m.heading)) < 0.00001);
    let shot = aimed
        .world
        .query::<(&Motion, &Bullet)>()
        .iter()
        .find(|(_, b)| b.player)
        .map(|(m, _)| *m)
        .unwrap();
    assert!(shot.velocity.normalize().distance(forward(m.heading)) < 0.00001);
}

#[test]
fn joystick_priority_and_invalid_held_world_targets_preserve_existing_controls() {
    let mut game = fixture();
    game.tick(
        &InputFrame {
            joystick: Some([0.0, -1.0]),
            thrust: Some([650.0, 450.0]),
            destination: Some([900.0, 0.0]),
            ..Default::default()
        },
        DT,
    );
    assert!(game.world.get::<&Motion>(game.player).unwrap().heading < 0.0);
    assert_eq!(game.destination, None);
    for invalid in [
        [f32::NAN, 0.0],
        [f32::INFINITY, 0.0],
        [WORLD_LIMIT + 1.0, 0.0],
    ] {
        let mut baseline = fixture();
        let mut candidate = fixture();
        baseline.tick(
            &InputFrame {
                destination: Some([600.0, 0.0]),
                ..Default::default()
            },
            DT,
        );
        candidate.tick(
            &InputFrame {
                thrust: Some(invalid),
                destination: Some([600.0, 0.0]),
                ..Default::default()
            },
            DT,
        );
        let a = *baseline.world.get::<&Motion>(baseline.player).unwrap();
        let b = *candidate.world.get::<&Motion>(candidate.player).unwrap();
        assert_eq!(a.position, b.position);
        assert_eq!(a.velocity, b.velocity);
        assert_eq!(a.heading, b.heading);
        assert_eq!(baseline.destination, candidate.destination);
    }
}
