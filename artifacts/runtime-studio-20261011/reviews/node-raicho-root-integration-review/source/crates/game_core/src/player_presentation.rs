//! Player-only color feedback; no physics, health, occupancy or save mutation.
use std::f32::consts::TAU;

/// Keep every surviving cell and its authored shade/alpha. Recent real hits
/// give a short red tint modulation; critical living hull pulses gently once
/// per second. Reduced Motion uses a steady critical tint and a fading hit tint.
pub(crate) fn tint(
    color: u32,
    hull: f32,
    maximum: f32,
    since_damage: f32,
    time: f32,
    reduced_motion: bool,
) -> u32 {
    if !hull.is_finite() || hull <= 0.0 || !maximum.is_finite() || maximum <= 0.0 {
        return color;
    }
    let critical = if hull / maximum <= 0.25 {
        if reduced_motion {
            0.30
        } else {
            let time = if time.is_finite() { time } else { 0.0 };
            0.20 + 0.20 * (0.5 + 0.5 * (time.rem_euclid(1.0) * TAU).sin())
        }
    } else {
        0.0
    };
    let hit = if since_damage.is_finite() && (0.0..0.45).contains(&since_damage) {
        let fade = 1.0 - since_damage / 0.45;
        let modulation = if reduced_motion {
            1.0
        } else {
            0.55 + 0.45 * (0.5 + 0.5 * (since_damage * TAU * 4.0).cos())
        };
        0.72 * fade * modulation
    } else {
        0.0
    };
    let amount = critical.max(hit).clamp(0.0, 0.72);
    if amount <= 0.0 {
        return color;
    }
    let [r, g, b, alpha] = color.to_le_bytes();
    let mix =
        |source: u8, target: f32| (source as f32 + (target - source as f32) * amount).round() as u8;
    u32::from_le_bytes([
        mix(r, r.max(230) as f32),
        mix(g, g as f32 * 0.28),
        mix(b, b as f32 * 0.34),
        alpha,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Game, InputFrame, Kind, PixelBody, Profile};
    use engine_core::FrameBuilder;

    const BASE: u32 = u32::from_le_bytes([70, 160, 210, 153]);

    #[test]
    fn real_hit_red_tint_preserves_alpha_and_returns_to_the_authored_palette() {
        let hit = tint(BASE, 100.0, 100.0, 0.0, 0.0, false).to_le_bytes();
        assert!(hit[0] > 70 && hit[1] < 160 && hit[2] < 210);
        assert_eq!(hit[3], 153);
        assert_ne!(
            tint(BASE, 100.0, 100.0, 0.05, 0.0, false),
            tint(BASE, 100.0, 100.0, 0.15, 0.0, false)
        );
        for age in [0.45, 1.0, 10.0, f32::NAN, f32::INFINITY, -1.0] {
            assert_eq!(tint(BASE, 100.0, 100.0, age, 0.0, false), BASE);
        }
    }

    #[test]
    fn critical_hull_pulses_only_while_alive_and_reduced_motion_is_static() {
        assert_ne!(
            tint(BASE, 25.0, 100.0, 10.0, 0.25, false),
            tint(BASE, 25.0, 100.0, 10.0, 0.75, false)
        );
        for time in [0.0, 0.25, 0.75, 999.0, f32::NAN] {
            assert_eq!(
                tint(BASE, 25.0, 100.0, 10.0, time, true),
                tint(BASE, 25.0, 100.0, 10.0, 0.0, true)
            );
            assert_eq!(tint(BASE, 25.01, 100.0, 10.0, time, false), BASE);
        }
        for (health, max) in [(0.0, 100.0), (-1.0, 100.0), (1.0, 0.0), (f32::NAN, 100.0)] {
            assert_eq!(tint(BASE, health, max, 0.0, 0.25, false), BASE);
        }
        // Reduced Motion still communicates a real hit without periodic flicker.
        assert_eq!(
            tint(BASE, 100.0, 100.0, 0.1, 0.0, true),
            tint(BASE, 100.0, 100.0, 0.1, 99.0, true)
        );
    }

    fn player_packet(game: &Game) -> Vec<([f32; 2], u32)> {
        let mut frame = FrameBuilder::new();
        game.render_into(1.0, &mut frame);
        let motion = *game.world.get::<&crate::Motion>(game.player).unwrap();
        let actor = *game.world.get::<&crate::Actor>(game.player).unwrap();
        let transform = engine_core::GpuTransform::from(crate::actor_transform(motion, actor))
            .with_material(crate::ACTOR_MATERIAL);
        frame
            .frame()
            .batches
            .iter()
            .flat_map(|b| {
                b.pixels.iter().filter_map(|p| {
                    let t = b.transforms[p.transform as usize];
                    (t == transform).then_some((p.local, p.color))
                })
            })
            .collect()
    }

    #[test]
    fn actual_damage_draws_every_surviving_player_cell_and_never_blinks_the_ship_away() {
        let mut game = Game::new(Profile::default());
        game.diagnostic("warp 900 0");
        game.invulnerability = 0.0;
        game.shield = 0.0; // Declared depleted shield isolates real hull/pixel damage.
        game.hurt_player(32.0, Some(engine_core::Vec2::new(-3.0, 2.0)));
        assert_eq!(game.hud().hull, game.hud().max_hull - 32.0);
        let occupied = game
            .world
            .get::<&PixelBody>(game.player)
            .unwrap()
            .occupied_count();
        assert!(occupied > 0);
        let actor = *game.world.get::<&crate::Actor>(game.player).unwrap();
        assert_eq!(actor.kind, Kind::Player);
        assert!(game.invulnerability > 0.0);
        let checkpoint = serde_json::to_string(game.profile()).unwrap();
        let before = player_packet(&game);
        assert_eq!(before.len(), occupied);
        for age in [0.02, 0.08, 0.16, 0.24, 0.40] {
            game.since_damage = age;
            game.time = age;
            let packet = player_packet(&game);
            assert_eq!(packet.len(), occupied);
            assert!(packet.iter().all(|(_, color)| color >> 24 != 0));
            assert_eq!(
                game.world
                    .get::<&PixelBody>(game.player)
                    .unwrap()
                    .occupied_count(),
                occupied
            );
            assert_eq!(serde_json::to_string(game.profile()).unwrap(), checkpoint);
        }
        game.set_reduced_motion(true);
        game.since_damage = 0.1;
        game.time = 1.0;
        let first = player_packet(&game);
        game.time = 9.0;
        assert_eq!(first, player_packet(&game));
        game.set_reduced_motion(false);
        game.since_damage = 10.0;
        let normal = player_packet(&game);
        assert_eq!(normal.len(), occupied);
        assert_ne!(before, normal);
        // Reset uses existing longer spawn grace, not a health/regen timer edit.
        game.since_damage = 0.0;
        game.reset_to_dock();
        let at_dock = player_packet(&game);
        game.since_damage = 10.0;
        assert_eq!(at_dock, player_packet(&game));
        // Normal simulation remains available; presentation introduces no state.
        game.tick(&InputFrame::default(), 1.0 / 60.0);
    }

    #[test]
    fn actual_critical_player_palette_changes_without_changing_the_damaged_body_or_state() {
        let mut game = Game::new(Profile::default());
        game.diagnostic("damage");
        for _ in 0..12 {
            game.tick(&InputFrame::default(), 0.05);
        }
        let max = crate::Stats::from_profile(game.profile()).hull;
        game.hurt_player(game.hud().hull - max * 0.25, None);
        game.since_damage = 10.0;
        game.time = 0.25;
        let peak = player_packet(&game);
        game.time = 0.75;
        let trough = player_packet(&game);
        assert_eq!(peak.len(), trough.len());
        assert!(!peak.is_empty());
        assert_ne!(peak, trough);
        let count = game
            .world
            .get::<&PixelBody>(game.player)
            .unwrap()
            .occupied_count();
        assert_eq!(peak.len(), count);
        assert_eq!(game.hud().hull, max * 0.25);
        let checkpoint = serde_json::to_string(game.profile()).unwrap();
        game.set_reduced_motion(true);
        game.time = 0.0;
        let still = player_packet(&game);
        game.time = 500.0;
        assert_eq!(still, player_packet(&game));
        assert_eq!(serde_json::to_string(game.profile()).unwrap(), checkpoint);
        assert_eq!(game.hud().hull, max * 0.25);
    }
}
