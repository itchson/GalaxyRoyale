//! Explicit QA-only native relighting scene. No saved state or world simulation.
use super::*;

impl Game {
    pub(super) fn begin_material_scene(&mut self) -> bool {
        if self.profile.campaign.selected_ship != 0
            || !raicho_fine::pan_material_ready()
            || self
                .profile
                .identity
                .as_ref()
                .is_none_or(|i| i.corporation != identity::Corporation::Pan)
            || self.profile.campaign.cargo_materials.iter().any(|&n| n > 0)
            || self
                .profile
                .campaign
                .cargo_training_materials
                .iter()
                .any(|&n| n > 0)
            || self.arena.is_some()
            || self.active_boss.is_some()
            || self.run.is_some()
            || !self
                .world
                .get::<&Actor>(self.player)
                .is_ok_and(|a| a.health > 0.0)
        {
            return false;
        }
        // Caller established diagnostic isolation before reset. No cargo/real
        // active run is discarded or finalized by this deliberately quiet scene.
        self.reset_to_dock();
        let others: Vec<_> = self
            .world
            .query::<Entity>()
            .iter()
            .filter(|e| *e != self.player)
            .collect();
        for entity in others {
            let _ = self.world.despawn(entity);
        }
        self.chunks.clear();
        self.particles.clear();
        self.transient_lights.clear();
        self.wake.clear();
        self.events.clear();
        self.move_player(Vec2::new(450.0, 0.0));
        self.material_scene = true;
        self.diagnostic_scene = true;
        true
    }
    pub(super) fn tick_material_scene(&mut self, input: &InputFrame, dt: f32) {
        if let Ok(mut m) = self.world.get::<&mut Motion>(self.player) {
            m.previous = m.position;
            m.previous_heading = m.heading;
        }
        self.previous_camera = self.camera;
        let motion = co_op_control::control_motion(
            *self.world.get::<&Motion>(self.player).unwrap(),
            &mut self.throttle,
            &mut self.destination,
            input,
            Stats::from_profile(&self.profile),
            dt,
        );
        *self.world.get::<&mut Motion>(self.player).unwrap() = motion;
        self.integrate(dt); // Only the actual player exists in this scene.
        self.confine_player();
        let position = self.position();
        if position.length_squared() <= DOCK_RADIUS * DOCK_RADIUS {
            self.reset_to_dock();
            return;
        }
        self.camera += (position - self.camera) * (1.0 - (-5.0 * dt).exp());
    }
    pub(super) fn material_scene_lights(&self, alpha: f32) -> Vec<LightSource> {
        let alpha = if alpha.is_finite() {
            alpha.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let Some(position) = self
            .world
            .get::<&Motion>(self.player)
            .ok()
            .map(|m| m.previous.lerp(m.position, alpha))
        else {
            return Vec::new();
        };
        let time = self.cosmetic_time();
        vec![
            LightSource {
                position: (position
                    + Vec2::new((time * 0.8).cos() * 44.0, (time * 0.8).sin() * 28.0))
                .to_array(),
                radius: 110.0,
                color: [1.0, 0.38, 0.12],
                intensity: 1.0,
            },
            LightSource {
                position: (position
                    + Vec2::new(
                        -(time * 0.57 + 0.8).cos() * 38.0,
                        (time * 0.57 + 0.8).sin() * 32.0,
                    ))
                .to_array(),
                radius: 105.0,
                color: [0.12, 0.48, 1.0],
                intensity: 0.9,
            },
        ]
    }
}

#[cfg(test)]
#[path = "material_scene_tests.rs"]
mod tests;
