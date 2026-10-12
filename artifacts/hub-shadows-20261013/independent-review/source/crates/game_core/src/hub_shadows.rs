//! One local RAICHO over the main hub. Elevation is presentation-only: no new
//! component, collision, RNG, progression, serialized state or network field.
use super::*;
pub(super) const RAICHO_HEIGHT: f32 = 6.0;

pub(super) fn beacon_light(position: Vec2, time: f32) -> LightSource {
    LightSource {
        position: position.to_array(),
        radius: 120.0,
        color: [0.2, 0.85, 0.92],
        intensity: 0.48 + (time * 1.2).sin() * 0.08,
    }
}
impl Game {
    pub(super) fn extract_hub_shadow(&self, alpha: f32, frame: &mut FrameBuilder) {
        if self.co_op.is_some() || self.material_scene || !self.visible(Vec2::ZERO, 100.0) {
            return;
        }
        let mut query = self
            .world
            .query_one::<(&Motion, &Actor, &PixelBody)>(self.player);
        let Ok((motion, actor, body)) = query.get() else {
            return;
        };
        let mut motion = *motion;
        let actor = *actor;
        if actor.health <= 0.0 || actor.kind != Kind::Player {
            return;
        }
        motion.position = motion.previous.lerp(motion.position, alpha);
        motion.heading = interpolate_heading(motion.previous_heading, motion.heading, alpha);
        // Main artwork corners lie within radius~135. Only admit a caster near
        // this bounded receiver footprint; projection away from the center is
        // outward, so this conservative radius cannot hide a farther shadow.
        if motion.position.length_squared() > 160.0 * 160.0 {
            return;
        }
        let key = beacon_light(Vec2::ZERO, self.cosmetic_time());
        frame.set_planar_shadow(
            body,
            actor_transform(motion, actor).affine(),
            RAICHO_HEIGHT,
            engine_core::ShadowLight {
                position: key.position,
                radius: key.radius,
                color: key.color,
                intensity: key.intensity,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn draw(game: &Game, frame: &mut FrameBuilder, accept: bool) {
        game.render_into_with_home(0.5, frame, |frame, station, _| {
            if accept {
                frame.with_object_meta(engine_core::ObjectMeta::receiver(0.0), |frame| {
                    frame.push_body(&game.dock_body, station)
                });
            }
            accept
        });
    }
    #[test]
    fn actual_game_shadow_uses_interpolated_current_body_without_changing_authority() {
        let mut game = Game::new(
            identity::create_profile("dora", "pan", "renko", "0123456789abcdef").unwrap(),
        );
        game.set_reduced_motion(true);
        {
            let mut m = game.world.get::<&mut Motion>(game.player).unwrap();
            m.previous = Vec2::new(20.0, 10.0);
            m.position = Vec2::new(24.0, 14.0);
            m.previous_heading = -0.5;
            m.heading = 0.5;
        }
        let saved = serde_json::to_string(game.profile()).unwrap();
        let before = *game.world.get::<&Motion>(game.player).unwrap();
        let occupied = game
            .world
            .get::<&PixelBody>(game.player)
            .unwrap()
            .occupied_count();
        let mut frame = FrameBuilder::new();
        draw(&game, &mut frame, true);
        let s = frame.frame().shadows.as_deref().unwrap();
        let caster = s.caster.unwrap();
        assert_eq!(caster.transform.position, Vec2::new(22.0, 12.0));
        assert_eq!(caster.height, RAICHO_HEIGHT);
        assert_eq!(
            s.masks()
                .iter()
                .map(|w| w.count_ones() as usize)
                .sum::<usize>(),
            occupied
        );
        let body = game.world.get::<&PixelBody>(game.player).unwrap();
        let (_, cell, _) = body.cells_indexed().next().unwrap();
        let receiver = caster.transform.point(cell) / (1.0 - RAICHO_HEIGHT / 42.0);
        assert!(s.blocks_ray(receiver, 0.0, Vec2::ZERO, 42.0));
        drop(body);
        let pointer = s.masks().as_ptr();
        {
            let mut body = game.world.get::<&mut PixelBody>(game.player).unwrap();
            assert!(!body.damage(cell, 0.0).is_empty());
        }
        draw(&game, &mut frame, true);
        let s = frame.frame().shadows.as_deref().unwrap();
        assert_eq!(s.masks().as_ptr(), pointer);
        assert!(!s.blocks_ray(receiver, 0.0, Vec2::ZERO, 42.0));
        assert_eq!(
            s.masks()
                .iter()
                .map(|w| w.count_ones() as usize)
                .sum::<usize>(),
            occupied - 1
        );
        assert_eq!(serde_json::to_string(game.profile()).unwrap(), saved);
        let after = *game.world.get::<&Motion>(game.player).unwrap();
        assert_eq!(before.position, after.position);
        assert_eq!(before.heading, after.heading);
        assert!(
            frame
                .frame()
                .batches
                .iter()
                .flat_map(|b| b.objects.iter().flatten())
                .any(|m| m.height == 6.0 && m.flags == 0)
        );
        draw(&game, &mut frame, false);
        assert!(frame.frame().shadows.as_ref().unwrap().caster.is_none());
        assert!(
            frame
                .frame()
                .batches
                .iter()
                .flat_map(|b| b.objects.iter().flatten())
                .all(|m| *m == engine_core::ObjectMeta::default())
        );
        {
            let mut m = game.world.get::<&mut Motion>(game.player).unwrap();
            m.previous = Vec2::splat(2000.0);
            m.position = m.previous;
        }
        draw(&game, &mut frame, true);
        assert!(frame.frame().shadows.as_ref().unwrap().caster.is_none());
        {
            let mut m = game.world.get::<&mut Motion>(game.player).unwrap();
            m.position = Vec2::ZERO;
            m.previous = Vec2::ZERO;
        }
        game.world.get::<&mut Actor>(game.player).unwrap().health = 0.0;
        draw(&game, &mut frame, true);
        assert!(frame.frame().shadows.as_ref().unwrap().caster.is_none());
    }

    #[test]
    fn co_op_and_material_diagnostics_do_not_retain_solo_casters() {
        let mut game = Game::new(
            identity::create_profile("dora", "pan", "renko", "0123456789abcdef").unwrap(),
        );
        let mut frame = FrameBuilder::new();
        draw(&game, &mut frame, true);
        assert!(frame.frame().shadows.as_ref().unwrap().caster.is_some());
        game.material_scene = true;
        draw(&game, &mut frame, true);
        assert!(frame.frame().shadows.as_ref().unwrap().caster.is_none());
        game.material_scene = false;
        let session = co_op::ParticipantId::parse("00000000000000000000000000000100").unwrap();
        let host = co_op::ParticipantId::parse("00000000000000000000000000000001").unwrap();
        game.begin_co_op(co_op::PROTOCOL, session, host).unwrap();
        draw(&game, &mut frame, true);
        assert!(frame.frame().shadows.as_ref().unwrap().caster.is_none());
    }

    #[test]
    #[ignore = "explicit actual Game healthy/damaged shadow fixture exporter; requires GALAXY_HUB_SHADOW_NATIVE"]
    fn export_actual_game_shadow_fields() {
        let path = std::env::var("GALAXY_HUB_SHADOW_NATIVE").expect("absolute native output path");
        let mut cases = Vec::new();
        for (position, heading) in [
            (Vec2::ZERO, 0.0),
            (Vec2::new(25.0, -30.0), 0.37),
            (Vec2::new(55.0, 20.0), 1.2),
        ] {
            for damaged in [false, true] {
                let mut game = Game::new(
                    identity::create_profile("dora", "pan", "renko", "0123456789abcdef").unwrap(),
                );
                game.set_reduced_motion(true);
                {
                    let mut m = game.world.get::<&mut Motion>(game.player).unwrap();
                    *m = Motion::new(position, Vec2::ZERO, heading);
                }
                if damaged {
                    let mut body = game.world.get::<&mut PixelBody>(game.player).unwrap();
                    assert!(!body.damage(Vec2::new(-3.0, 0.0), 3.0).is_empty());
                    game.world.get::<&mut Actor>(game.player).unwrap().health = 60.0;
                }
                let mut frame = FrameBuilder::new();
                draw(&game, &mut frame, true);
                let s = frame.frame().shadows.as_ref().unwrap();
                let c = s.caster.unwrap();
                let key = s.light.unwrap();
                let body = game.world.get::<&PixelBody>(game.player).unwrap();
                let cells = body
                    .cells_indexed()
                    .map(|(i, _, rgba)| [i as u32, rgba])
                    .collect::<Vec<_>>();
                assert_eq!(
                    cells.len(),
                    s.masks()
                        .iter()
                        .map(|w| w.count_ones() as usize)
                        .sum::<usize>()
                );
                cases.push(serde_json::json!({"position":position.to_array(),"heading":heading,"damaged":damaged,
                    "width":c.width,"height_cells":c.height_cells,"height":c.height,
                    "basis_x":c.transform.basis_x.to_array(),"basis_y":c.transform.basis_y.to_array(),"cells":cells,
                    "mask":s.masks().as_slice(),"light":{"position":key.position,"radius":key.radius,"color":key.color,"intensity":key.intensity}}));
            }
        }
        std::fs::write(
            path,
            serde_json::to_vec(&serde_json::json!({"cases":cases})).unwrap(),
        )
        .unwrap();
    }
}
