//! Actual bounded cosmetic lifecycle and extraction regressions.
use super::*;
use engine_core::CameraView;

fn state(g: &Game) -> Vec<(Vec2, Vec2, f32)> {
    g.particles
        .iter()
        .map(|p| (p.local, p.velocity, p.life))
        .collect()
}

#[test]
fn bounded_pool_rejects_invalid_births_and_reuses_its_lifetime_storage() {
    let mut g = Game::new(Profile::default());
    let pointer = g.particles.as_ptr();
    let rng = g.rng;
    for (position, velocity, life, size) in [
        (Vec2::splat(f32::NAN), Vec2::ZERO, 1.0, 1.0),
        (Vec2::ZERO, Vec2::splat(f32::INFINITY), 1.0, 1.0),
        (Vec2::ZERO, Vec2::ZERO, f32::NAN, 1.0),
        (Vec2::ZERO, Vec2::ZERO, 1.0, 0.0),
    ] {
        g.particle(position, velocity, life, 0xffff_ffff, size);
    }
    assert!(g.particles.is_empty());
    for _ in 0..MAX_PARTICLES + 20 {
        g.styled_particle(
            Vec2::ZERO,
            Vec2::X * 30.0,
            0.1,
            0xffff_ffff,
            0.5,
            ParticleStyle::Fire,
        );
    }
    assert_eq!(g.particles.len(), MAX_PARTICLES);
    assert_eq!(g.particles.as_ptr(), pointer);
    // Exact size costs one bounded transform per visible particle. Repeated
    // extraction reuses both transform and cell allocations across 128-slot
    // batch boundaries, without silently reducing size to preserve batching.
    let mut frame = FrameBuilder::new();
    assert_eq!(g.render_particles(&mut frame, 1.0), (1100, 1100));
    assert_eq!(frame.frame().batches.len(), 9);
    let pointers: Vec<_> = frame
        .frame()
        .batches
        .iter()
        .map(|b| (b.transforms.as_ptr(), b.pixels.as_ptr()))
        .collect();
    frame.clear();
    assert_eq!(g.render_particles(&mut frame, 1.0), (1100, 1100));
    assert_eq!(
        pointers,
        frame
            .frame()
            .batches
            .iter()
            .map(|b| (b.transforms.as_ptr(), b.pixels.as_ptr()))
            .collect::<Vec<_>>()
    );
    for _ in 0..8 {
        g.update_particles(1.0 / 60.0);
    }
    assert!(g.particles.is_empty());
    g.particle(Vec2::ZERO, Vec2::ZERO, 1.0, 0xffff_ffff, 1.0);
    assert_eq!(g.particles.as_ptr(), pointer);
    assert_eq!(g.rng, rng);
    g.particles.clear();
    g.stress_mode = true;
    for _ in 0..2010 {
        g.particle(Vec2::ZERO, Vec2::ZERO, 1.0, 0xffff_ffff, 0.1);
    }
    assert_eq!(g.particles.len(), 2000);
    frame.clear();
    assert_eq!(g.render_particles(&mut frame, 1.0), (2000, 2000));
    assert_eq!(frame.frame().batches.len(), 16);
    assert!(
        frame
            .frame()
            .batches
            .iter()
            .all(|b| b.transforms.len() <= 128 && b.pixels.len() == b.transforms.len())
    );
}

#[test]
fn actual_drag_curl_and_acceleration_are_deterministic_finite_and_time_bounded() {
    let make = || {
        let mut g = Game::new(Profile::default());
        g.styled_particle(
            Vec2::new(100.0, -50.0),
            Vec2::new(40.0, 10.0),
            1.0,
            0xffff_ffff,
            1.2,
            ParticleStyle::Fire,
        );
        g
    };
    let mut a = make();
    let mut b = make();
    let profile = serde_json::to_value(a.profile()).unwrap();
    let rng = a.rng;
    let events = a.events.len();
    for _ in 0..24 {
        a.update_particles(1.0 / 60.0);
        b.update_particles(1.0 / 60.0);
    }
    assert_eq!(state(&a), state(&b));
    assert!(a.particles[0].local.length() > 5.0);
    assert_ne!(a.particles[0].velocity, Vec2::new(40.0, 10.0));
    let before = state(&a);
    for dt in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        a.update_particles(dt);
    }
    assert_eq!(state(&a), before);
    a.update_particles(100.0);
    b.update_particles(0.05);
    assert_eq!(state(&a), state(&b));
    assert_eq!(serde_json::to_value(a.profile()).unwrap(), profile);
    assert_eq!(a.rng, rng);
    assert_eq!(a.events.len(), events);
    let mut slow = make();
    let mut fast = make();
    slow.particles[0].style = ParticleStyle::Debris;
    slow.particles[0].acceleration = Vec2::ZERO;
    fast.particles[0].style = ParticleStyle::Debris;
    fast.particles[0].acceleration = Vec2::ZERO;
    for _ in 0..10 {
        slow.update_particles(0.05);
    }
    for _ in 0..30 {
        fast.update_particles(1.0 / 60.0);
    }
    assert!(slow.particles[0].local.distance(fast.particles[0].local) < 0.001);
    for _ in 0..50 {
        a.update_particles(1.0 / 60.0);
    }
    assert!(a.particles.is_empty());
}

#[test]
fn live_authored_engine_cells_emit_real_exhaust_and_destroyed_ports_emit_neither_gas_nor_light() {
    for ship in 0..=9 {
        let mut p = Profile::default();
        p.campaign.patches = [true; 8];
        p.campaign.blueprints = [true; 8];
        p.campaign.crafted = [true; 8];
        p.campaign.royale_results = [1; 8];
        p.campaign.victory_claimed = true;
        p.campaign.selected_ship = ship;
        let mut g = Game::new(p);
        let ports = g.exhaust_ports();
        assert!(
            ports.into_iter().all(|port| port.is_some()),
            "ship{ship} has no authored rear port: {ports:?}"
        );
        g.tick(
            &InputFrame {
                joystick: Some([0.0, -1.0]),
                ..Default::default()
            },
            1.0 / 60.0,
        );
        assert!(
            g.particles
                .iter()
                .any(|p| p.style == ParticleStyle::Exhaust)
        );
        assert!(
            g.light_sources(1.0)
                .iter()
                .any(|l| l.color == [0.2, 0.78, 1.0])
        );
        for port in ports.into_iter().flatten() {
            g.world
                .get::<&mut PixelBody>(g.player)
                .unwrap()
                .damage(port, 6.2);
        }
        assert!(
            g.world
                .get::<&PixelBody>(g.player)
                .unwrap()
                .occupied_count()
                > 0
        );
        assert!(g.exhaust_ports().iter().all(Option::is_none));
        g.particles.clear();
        g.exhaust_timer = 0.0;
        g.tick(
            &InputFrame {
                joystick: Some([0.0, -1.0]),
                ..Default::default()
            },
            1.0 / 60.0,
        );
        assert!(
            g.particles
                .iter()
                .all(|p| p.style != ParticleStyle::Exhaust)
        );
        assert!(
            g.light_sources(1.0)
                .iter()
                .all(|l| l.color != [0.2, 0.78, 1.0])
        );
    }
}

#[test]
fn real_projectiles_leave_bounded_short_trails_without_phantom_bullets_lights_or_rewards() {
    let mut g = Game::new(Profile::default());
    g.move_player(Vec2::new(500.0, 0.0));
    for i in 0..20 {
        g.spawn_bullet(
            g.position() + Vec2::new(i as f32 * 2.0, 80.0),
            Vec2::X * 100.0,
            1.0,
            i % 2 == 0,
            1,
        );
    }
    let before = serde_json::to_value(g.profile()).unwrap();
    let rng = g.rng;
    let lights = g.light_sources(1.0);
    g.emit_projectile_trails(1.0 / 60.0);
    assert_eq!(g.particles.len(), 8);
    assert!(
        g.particles
            .iter()
            .all(|p| p.style == ParticleStyle::Trail && p.initial_life == 0.18)
    );
    assert_eq!(g.world.query::<&Bullet>().iter().count(), 20);
    assert_eq!(g.light_sources(1.0), lights);
    assert_eq!(serde_json::to_value(g.profile()).unwrap(), before);
    assert_eq!(g.rng, rng);
    let mut frame = FrameBuilder::new();
    let (quads, groups) = g.render_particles(&mut frame, 1.0);
    assert_eq!(quads, 8);
    assert_eq!(groups, 8);
    assert!(
        frame
            .frame()
            .batches
            .iter()
            .flat_map(|b| &b.transforms)
            .all(|t| t.b[3] < -1.0)
    );
    for _ in 0..12 {
        g.update_particles(1.0 / 60.0);
    }
    assert!(g.particles.is_empty());
}

#[test]
fn fractional_particle_groups_keep_far_local_motion_and_zoom_changes_projection_only() {
    for camera in [
        Vec2::new(999_000.0, -999_000.0),
        Vec2::new(-999_000.0, 999_000.0),
    ] {
        let mut g = Game::new(Profile::default());
        g.camera = camera;
        g.previous_camera = camera;
        for style in [
            ParticleStyle::Debris,
            ParticleStyle::Fire,
            ParticleStyle::Exhaust,
            ParticleStyle::Trail,
        ] {
            for size in [0.01, 0.1, 0.599, 0.601, 4.2] {
                g.styled_particle(camera, Vec2::new(9.0, 2.0), 1.0, 0xffbc_e8fa, size, style);
            }
        }
        g.update_particles(1.0 / 60.0);
        let mut full = FrameBuilder::new();
        let (quads, groups) = g.render_particles(&mut full, 0.5);
        assert_eq!(quads, 20);
        assert_eq!(groups, 20);
        for zoom in [0.6, 1.0, 1.6] {
            let view = CameraView::new(camera, Vec2::new(533.0, 300.0), zoom).unwrap();
            let mut particles = g.particles.iter();
            for batch in &full.frame().batches {
                for cell in &batch.pixels {
                    let particle = particles.next().unwrap();
                    let age = 1.0 - particle.life / particle.initial_life;
                    let expected_size = particle.size
                        * if particle.style == ParticleStyle::Debris {
                            1.0
                        } else {
                            1.0 + age * 0.9
                        };
                    let packed = batch.transforms[cell.transform as usize];
                    assert_eq!(packed.a[2], expected_size);
                    assert_eq!(packed.b[1], expected_size);
                    let relative = view.relative_transform(packed);
                    let center = relative.point(Vec2::from_array(cell.local));
                    assert!(center.is_finite() && center.length() > 0.03 && center.length() < 0.2);
                    assert!(
                        center.distance(particle.previous_local.lerp(particle.local, 0.5))
                            < 0.00001
                    );
                    let width =
                        (relative.point(Vec2::from_array(cell.local) + Vec2::X) - center).length();
                    assert!((width - expected_size).abs() < 0.00001);
                    let screen_delta =
                        Vec2::new(width, 0.0) / view.world_extent() * Vec2::new(533.0, 300.0);
                    assert!((screen_delta.x - expected_size * zoom).abs() < 0.00001);
                    assert_eq!(packed.b[3] >= 0.0, particle.style == ParticleStyle::Debris);
                }
            }
            assert!(particles.next().is_none());
        }
        // Cross the former .6 size bucket continuously. Real lifecycle age
        // changes the exact soft scale while hard fragments retain their size.
        let initial_widths: Vec<_> = full.frame().batches[0]
            .transforms
            .iter()
            .map(|t| t.a[2])
            .collect();
        g.update_particles(1.0 / 120.0);
        full.clear();
        assert_eq!(g.render_particles(&mut full, 0.5), (20, 20));
        for (index, (before, after)) in initial_widths
            .iter()
            .zip(&full.frame().batches[0].transforms)
            .enumerate()
        {
            if index < 5 {
                assert_eq!(*before, after.a[2]);
            } else {
                let expected_delta = g.particles[index].size * 0.9 / 120.0;
                assert!((after.a[2] - before - expected_delta).abs() < 0.000001);
            }
        }
        g.set_cosmetic_density(true);
        let mut low = FrameBuilder::new();
        assert_eq!(g.render_particles(&mut low, 0.5), (10, 10));
    }
}

#[test]
fn presentation_flags_pause_and_invalid_ticks_do_not_change_combat_rng_or_ship_progression() {
    let mut a = Game::new(Profile::default());
    let mut b = Game::new(Profile::default());
    b.set_cosmetic_density(true);
    b.set_reduced_motion(true);
    for g in [&mut a, &mut b] {
        g.move_player(Vec2::new(500.0, 0.0));
    }
    let input = InputFrame {
        joystick: Some([1.0, 0.0]),
        fire: true,
        ..Default::default()
    };
    for _ in 0..120 {
        a.tick(&input, 1.0 / 60.0);
        b.tick(&input, 1.0 / 60.0);
    }
    assert_eq!(
        serde_json::to_value(a.profile()).unwrap(),
        serde_json::to_value(b.profile()).unwrap()
    );
    assert_eq!(a.rng, b.rng);
    assert_eq!(a.position(), b.position());
    assert_eq!(
        serde_json::to_value(a.take_events()).unwrap(),
        serde_json::to_value(b.take_events()).unwrap()
    );
    let before = state(&a);
    let sequence = a.cosmetic_sequence;
    a.set_paused(true);
    a.tick(&input, 0.05);
    a.set_paused(false);
    for dt in [f32::NAN, 0.0, -1.0] {
        a.tick(&input, dt);
    }
    assert_eq!(state(&a), before);
    assert_eq!(a.cosmetic_sequence, sequence);
}
