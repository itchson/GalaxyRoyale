use super::*;

fn chain(game: &Game, owner: Entity) -> Vec<(Entity, Motion, Segment)> {
    let mut parts: Vec<_> = game
        .world
        .query::<(Entity, (&Motion, &Segment))>()
        .iter()
        .filter(|(_, (_, s))| s.owner == owner)
        .map(|(e, (m, s))| (e, *m, *s))
        .collect();
    parts.sort_by_key(|(_, _, s)| s.index);
    parts
}

fn assert_joints(game: &Game, owner: Entity, alpha: f32, witness_cells: bool) {
    let poses = game.segment_render_poses(alpha);
    let head = *game.world.get::<&Motion>(owner).unwrap();
    let mut parent_position = head.previous.lerp(head.position, alpha);
    let mut parent_heading = interpolate_heading(head.previous_heading, head.heading, alpha);
    let mut rear = game.world.get::<&ChainHead>(owner).unwrap().rear;
    let mut parent_entity = owner;
    let mut parent_scale = game.world.get::<&Actor>(owner).unwrap().scale;
    for (entity, motion, part) in chain(game, owner) {
        assert!(part.attached, "segment {} separated", part.index);
        let (position, heading) = poses[&entity];
        let a = parent_position + rotated(rear, parent_heading);
        let b = position + rotated(part.front, heading);
        // At the practical world edge a whole-object f32 translation has a
        // 0.0625-unit ULP. The authored attachment cells are at least 0.85 wide.
        let tolerance = if parent_position.abs().max_element() > 100_000.0 {
            0.125
        } else {
            0.003
        };
        assert!(
            a.distance(b) <= tolerance,
            "joint {} gap {}",
            part.index,
            a.distance(b)
        );
        if alpha == 1.0 {
            assert!(position.distance(motion.position) < 0.003);
            assert!(wrap(heading - motion.heading).abs() < 0.00001);
        }
        if witness_cells {
            for (body_entity, p, h, scale) in [
                (parent_entity, parent_position, parent_heading, parent_scale),
                (entity, position, heading, part.scale),
            ] {
                let body = game.world.get::<&PixelBody>(body_entity).unwrap();
                let transform = Transform {
                    position: p,
                    rotation: h + FRAC_PI_2,
                    scale: Vec2::splat(scale),
                };
                let local = transform.inverse_point(a);
                assert!(
                    body.hit_segment(local, local).is_some(),
                    "joint must overlap actual occupied cells"
                );
            }
        }
        parent_position = position;
        parent_heading = heading;
        rear = part.rear;
        parent_entity = entity;
        parent_scale = part.scale;
    }
}

#[test]
fn all_authored_chains_keep_occupied_attachment_overlap_near_both_world_edges() {
    for center in [
        Vec2::new(999_000.0, 998_000.0),
        Vec2::new(-999_000.0, -998_000.0),
    ] {
        for generation in 0..=5 {
            for angle in 0..8 {
                let mut game = Game::new(Profile::default());
                game.move_player(center);
                let direction = forward(angle as f32 * TAU / 8.0);
                let owner = game.spawn_enemy(2, generation > 0, center + direction * 360.0, 1);
                if generation > 0 {
                    game.configure_boss(owner, 2, generation, true, 1);
                }
                assert_joints(&game, owner, 1.0, true);
                for _ in 0..12 {
                    for (_, motion) in game.world.query_mut::<(Entity, &mut Motion)>() {
                        motion.previous = motion.position;
                        motion.previous_heading = motion.heading;
                    }
                    {
                        let mut head = game.world.get::<&mut Motion>(owner).unwrap();
                        head.heading += 0.0125;
                        let movement = forward(head.heading) * 0.5;
                        head.position += movement;
                    }
                    game.update_segments(1.0 / 60.0);
                    for alpha in [0.0, 0.5, 1.0] {
                        assert_joints(&game, owner, alpha, true);
                    }
                }
            }
        }
    }
}

#[test]
fn ordinary_and_all_five_boss_chains_overlap_real_occupied_cells_at_birth() {
    for generation in 0..=5 {
        for angle in 0..16 {
            let mut game = Game::new(Profile::default());
            game.move_player(Vec2::new(1800.0, 0.0));
            let owner = game.spawn_enemy(
                2,
                generation > 0,
                game.position() + forward(angle as f32 * TAU / 16.0) * 360.0,
                1,
            );
            if generation > 0 {
                game.configure_boss(owner, 2, generation, true, 1);
            }
            assert_eq!(
                chain(&game, owner).len(),
                if generation == 0 {
                    4
                } else {
                    (9 + 2 * generation) as usize
                }
            );
            assert_joints(&game, owner, 1.0, true);
            for (_, _, s) in chain(&game, owner) {
                let body = game
                    .world
                    .get::<&PixelBody>(chain(&game, owner)[s.index as usize - 1].0)
                    .unwrap();
                assert!((s.radius - pixel_assets::radius(&body, s.scale)).abs() < 0.00001);
            }
        }
    }
}

#[test]
fn actual_tick_turns_and_every_interpolated_pose_keep_connected_boss_cells() {
    for generation in 1..=5 {
        let mut game = Game::new(Profile::default());
        game.diagnostic(&format!("boss-coil-{generation}"));
        game.invulnerability = 1000.0;
        let owner = game.active_boss.unwrap();
        let mut turn = 0.0;
        let mut last_heading = game.world.get::<&Motion>(owner).unwrap().heading;
        for tick in 0..180 {
            game.tick(
                &InputFrame {
                    joystick: Some([
                        ((tick as f32 * 0.055).cos()) * 0.55,
                        ((tick as f32 * 0.055).sin()) * 0.55,
                    ]),
                    ..Default::default()
                },
                1.0 / 60.0,
            );
            let heading = game.world.get::<&Motion>(owner).unwrap().heading;
            turn += wrap(heading - last_heading).abs();
            last_heading = heading;
            for alpha in [0.0, 0.25, 0.5, 0.75, 1.0] {
                assert_joints(&game, owner, alpha, true);
            }
        }
        assert!(turn > 0.1, "actual boss must move through a turn");
    }
}

#[test]
fn erased_modules_reconnect_without_restoring_surviving_body_holes() {
    let mut game = Game::new(Profile::default());
    game.diagnostic("boss-coil-5");
    let owner = game.active_boss.unwrap();
    let parts = chain(&game, owner);
    let removed = parts[4].0;
    let damaged = parts[5].0;
    game.damage_segment(damaged, 1.0, Vec2::ZERO);
    let occupied = game
        .world
        .get::<&PixelBody>(damaged)
        .unwrap()
        .occupied_count();
    assert!(occupied < boss_assets::segment(5).occupied_count());
    game.damage_segment(removed, 100.0, Vec2::ZERO);
    assert!(!game.world.contains(removed));
    game.update_segments(1.0 / 60.0);
    assert_eq!(chain(&game, owner).len(), 18);
    assert_joints(&game, owner, 1.0, false);
    let poses = game.segment_render_poses(0.5);
    let (_, _, part) = chain(&game, owner)
        .into_iter()
        .find(|(e, _, _)| *e == damaged)
        .unwrap();
    let (position, heading) = poses[&damaged];
    let mut frame = FrameBuilder::new();
    let emitted = boss_assets::segment_art(5).rig.draw(
        &mut frame,
        &game.world.get::<&PixelBody>(damaged).unwrap(),
        Transform {
            position,
            rotation: heading + FRAC_PI_2,
            scale: Vec2::splat(part.scale),
        },
        game.world
            .get::<&BossEncounter>(owner)
            .unwrap()
            .rig_state(game.cosmetic_time(), 2),
    );
    assert_eq!(emitted, occupied);
    assert_eq!(frame.frame().pixel_count, occupied);
}

#[test]
fn sanctuary_boundary_articulates_before_separating_and_remains_protected() {
    let mut game = Game::new(Profile::default());
    let site = regions::sites_near(game.profile.seed, Vec2::new(5000.0, 0.0)).remove(0);
    let center = Vec2::from_array(site.position);
    game.nearby_sites = regions::sites_near(game.profile.seed, center);
    game.move_player(center + Vec2::X * 800.0);
    for angle in 0..16 {
        let direction = forward(angle as f32 * TAU / 16.0);
        let owner = game.spawn_enemy(2, true, center + direction * (site.radius + 100.0), 1);
        game.configure_boss(owner, 2, 5, true, 1);
        game.enforce_actor_sanctuaries();
        game.update_segments(0.05);
        assert_joints(&game, owner, 1.0, true);
        for (_, m, s) in chain(&game, owner) {
            assert!(m.position.distance(center) >= site.radius + s.radius + 5.99);
        }
        game.remove_enemy(owner, false);
    }
}

#[test]
fn coil_family_champions_rebind_the_actual_chassis_and_keep_every_module_inside_arena() {
    for index in 0..8 {
        if campaign::CHAMPIONS[index].family != 2 {
            continue;
        }
        let mut game = Game::new(Profile::default());
        game.diagnostic(&format!("champion-{index}"));
        game.invulnerability = 1000.0;
        let arena = game.arena.unwrap();
        assert_joints(&game, arena.opponent, 1.0, true);
        for _ in 0..180 {
            game.tick(&InputFrame::default(), 1.0 / 60.0);
            for alpha in [0.0, 0.5, 1.0] {
                assert_joints(&game, arena.opponent, alpha, true);
            }
            for (_, m, s) in chain(&game, arena.opponent) {
                assert!(m.position.distance(arena.center) + s.radius + 3.99 <= arena.radius);
            }
        }
    }
}
