//! Observational sound state and actual phase-entry cues; no saved audio state.
use crate::*;

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct AudioHud {
    pub area: &'static str,
    pub biome: u8,
    /// In-range living hostile heads following the same sanctuary/pursuit rule
    /// as the controllers. Radial danger and asteroids never create aggro.
    pub combat: f32,
    pub hostiles: u32,
    /// Ambient identity at the canonical 320-unit champion contact region, or
    /// the actual active opponent. This is not an invented combat flag.
    pub champion: Option<u8>,
    pub boss: bool,
    pub hull: f32,
    pub speed: f32,
    pub throttle: f32,
    pub engines: u8,
}
fn unit(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}
impl Game {
    pub(super) fn audio_hud(&self) -> AudioHud {
        let position = self.position();
        let safe = self.safe();
        let pursuit = self.viewport.length().max(1050.0);
        let hostiles = if safe {
            0
        } else {
            self.world
                .query::<(&Motion, &Actor, &Brain)>()
                .iter()
                .filter(|(motion, actor, _)| {
                    matches!(actor.kind, Kind::Enemy(_) | Kind::Boss(_))
                        && actor.health > 0.0
                        && motion.position.distance_squared(position) <= pursuit * pursuit
                        && self.safe_zone_at(motion.position).is_none()
                })
                .count() as u32
        };
        let active_champion = self
            .arena
            .as_ref()
            .filter(|a| {
                self.world
                    .get::<&Actor>(a.opponent)
                    .is_ok_and(|actor| actor.health > 0.0)
            })
            .map(|a| a.index as u8);
        let champion = active_champion.or_else(|| {
            self.nearby_sites
                .iter()
                .filter_map(|site| {
                    let distance = Vec2::from_array(site.position).distance_squared(position);
                    if distance > 320.0 * 320.0 {
                        return None;
                    }
                    campaign::CHAMPIONS
                        .iter()
                        .position(|c| c.grid == site.grid)
                        .map(|index| (distance, index as u8))
                })
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, index)| index)
        });
        let boss = !safe
            && self.active_boss.is_some_and(|entity| {
                self.world
                    .get::<&Actor>(entity)
                    .is_ok_and(|actor| actor.health > 0.0)
                    && self.world.get::<&Motion>(entity).is_ok_and(|motion| {
                        motion.position.distance_squared(position) <= pursuit * pursuit
                    })
            });
        let stats = Stats::from_profile(&self.profile);
        let hull = self
            .world
            .get::<&Actor>(self.player)
            .map_or(0.0, |a| unit(a.health / stats.hull));
        let speed = self
            .world
            .get::<&Motion>(self.player)
            .map_or(0.0, |m| unit(m.velocity.length() / stats.speed));
        let area = if champion.is_some() {
            "arena"
        } else if position.length_squared() <= SAFE_RADIUS * SAFE_RADIUS {
            "haven"
        } else if safe {
            "site"
        } else if position.length_squared() <= COMBAT_RADIUS * COMBAT_RADIUS {
            "belt"
        } else {
            "reach"
        };
        AudioHud {
            area,
            biome: regions::at(self.profile.seed, position).kind,
            combat: if hostiles == 0 {
                0.0
            } else {
                unit(
                    0.18 + hostiles.min(8) as f32 * 0.075
                        + if boss || active_champion.is_some() {
                            0.3
                        } else {
                            0.0
                        },
                )
            },
            hostiles,
            champion,
            boss,
            hull,
            speed,
            throttle: unit(self.throttle),
            engines: if hull > 0.0 {
                self.exhaust_ports().iter().filter(|p| p.is_some()).count() as u8
            } else {
                0
            },
        }
    }

    pub(super) fn boss_audio_transition(&mut self, previous: Option<(Entity, bosses::Phase)>) {
        let cue = self.active_boss.and_then(|entity| {
            let boss = self.world.get::<&BossEncounter>(entity).ok()?;
            let actor = self.world.get::<&Actor>(entity).ok()?;
            if boss.phase != bosses::Phase::Tell
                || actor.health <= 0.0
                || previous == Some((entity, bosses::Phase::Tell))
                || self.safe()
            {
                return None;
            }
            self.world.get::<&Motion>(entity).ok().map(|m| m.position)
        });
        if let Some(position) = cue {
            self.event("boss-charge", position, 0.7);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const DT: f32 = 1.0 / 60.0;
    fn fixture() -> Game {
        let mut game = Game::new(Profile::default());
        game.move_player(Vec2::new(1800.0, 0.0));
        game.spawn_timer = 1000.0;
        game.boss_timer = 1000.0;
        game.take_events();
        game
    }
    fn clear_bullets(game: &mut Game) {
        let bullets: Vec<_> = game
            .world
            .query::<(Entity, &Bullet)>()
            .iter()
            .map(|(e, _)| e)
            .collect();
        for entity in bullets {
            game.world.despawn(entity).unwrap();
        }
    }
    fn fill_bullets(game: &mut Game) {
        for _ in 0..MAX_BULLETS {
            game.spawn_bullet(Vec2::new(1900.0, 100.0), Vec2::ZERO, 1.0, true, 1);
        }
    }
    fn learned(id: u8, ship: u8) -> Game {
        let mut profile = Profile::default();
        if ship > 0 && ship < 9 {
            profile.campaign.patches[ship as usize - 1] = true;
            profile.campaign.blueprints[ship as usize - 1] = true;
            profile.campaign.crafted[ship as usize - 1] = true;
        }
        profile.campaign.selected_ship = ship;
        profile.campaign.abilities[id as usize] = campaign::AbilityState {
            blueprint: true,
            rank: 1,
        };
        if (3..=5).contains(&id) {
            profile.unlocked[id as usize - 3] = true;
        }
        if (6..=13).contains(&id) {
            profile.campaign.patches[id as usize - 6] = true;
            profile.campaign.blueprints[id as usize - 6] = true;
        }
        profile.campaign.loadouts[ship as usize].slots[abilities::get(id).unwrap().category] =
            Some(id);
        let mut game = Game::new(profile);
        game.move_player(Vec2::new(1800.0, 0.0));
        game.spawn_timer = 1000.0;
        game.take_events();
        game
    }
    #[test]
    fn audio_aggro_is_live_pursuit_not_distance_or_asteroids_and_refuges_quiet_it() {
        let mut game = fixture();
        assert_eq!(game.hud().audio.combat, 0.0);
        assert_eq!(game.hud().audio.area, "reach");
        let near = game.spawn_enemy(0, false, game.position() + Vec2::X * 250.0, 1);
        let far = game.spawn_enemy(1, false, game.position() + Vec2::X * 1400.0, 1);
        assert_eq!(game.hud().audio.hostiles, 1);
        assert!(game.hud().audio.combat > 0.0);
        game.world.get::<&mut Actor>(near).unwrap().health = 0.0;
        assert_eq!(game.hud().audio.combat, 0.0);
        game.world.get::<&mut Actor>(near).unwrap().health = 20.0;
        game.move_player(Vec2::ZERO);
        assert_eq!(game.hud().audio.area, "haven");
        assert_eq!(game.hud().audio.combat, 0.0);
        assert_eq!(game.hud().audio.hostiles, 0);
        assert!(game.world.contains(far));
        // Snapshot extraction has no RNG, progression or simulation effects.
        let profile = serde_json::to_value(game.profile()).unwrap();
        let rng = game.rng;
        for _ in 0..20 {
            let _ = game.hud().audio;
        }
        assert_eq!(game.rng, rng);
        assert_eq!(serde_json::to_value(game.profile()).unwrap(), profile);
    }
    #[test]
    fn real_motion_hull_and_destroyed_ports_drive_finite_audio_state() {
        let mut game = fixture();
        game.tick(
            &InputFrame {
                joystick: Some([1.0, 0.0]),
                ..Default::default()
            },
            DT,
        );
        let audio = game.hud().audio;
        assert!(audio.throttle > 0.0 && audio.speed > 0.0 && audio.engines > 0);
        game.world.get::<&mut Actor>(game.player).unwrap().health = 20.0;
        assert!(game.hud().audio.hull <= 0.25);
        let ports = game.exhaust_ports();
        for port in ports.into_iter().flatten() {
            game.world
                .get::<&mut PixelBody>(game.player)
                .unwrap()
                .damage(port, 8.0);
        }
        assert_eq!(game.hud().audio.engines, 0);
        for value in [audio.hull, audio.speed, audio.throttle, audio.combat] {
            assert!(value.is_finite() && (0.0..=1.0).contains(&value));
        }
    }
    #[test]
    fn actual_primary_and_echo_cues_require_new_projectiles_without_cadence_changes() {
        let mut game = learned(3, 0);
        let stats = Stats::from_profile(game.profile());
        game.shot_count = 7;
        game.tick(
            &InputFrame {
                fire: true,
                ..Default::default()
            },
            DT,
        );
        let events = game.take_events();
        assert_eq!(events.iter().filter(|e| e.kind == "shot").count(), 1);
        assert_eq!(
            events.iter().filter(|e| e.kind == "ability-echo").count(),
            1
        );
        assert_eq!(game.shot_count, 8);
        assert_eq!(game.world.query::<&Bullet>().iter().count(), 4);
        clear_bullets(&mut game);
        fill_bullets(&mut game);
        game.take_events();
        let shots = game.shot_count;
        game.fire(stats);
        let events = game.take_events();
        assert!(
            !events
                .iter()
                .any(|e| e.kind == "shot" || e.kind == "ability-echo")
        );
        assert_eq!(game.shot_count, shots + 1);
        assert_eq!(game.world.query::<&Bullet>().iter().count(), MAX_BULLETS);
        game.move_player(Vec2::ZERO);
        clear_bullets(&mut game);
        game.fire_timer = 0.0;
        game.tick(
            &InputFrame {
                fire: true,
                ..Default::default()
            },
            DT,
        );
        assert!(
            !game
                .take_events()
                .iter()
                .any(|e| e.kind == "shot" || e.kind.starts_with("ability-"))
        );
    }
    #[test]
    fn crafted_signature_cues_follow_actual_volley_and_never_cap_blocked_shots() {
        for (id, ship, shot, count) in [
            (6, 1, 4, 2),
            (7, 2, 6, 2),
            (8, 3, 5, 1),
            (9, 4, 3, 2),
            (12, 7, 4, 1),
            (13, 8, 6, 1),
        ] {
            let mut game = learned(id, ship);
            game.shot_count = shot;
            let motion = *game.world.get::<&Motion>(game.player).unwrap();
            let stats = Stats::from_profile(game.profile());
            game.ship_signature(motion, stats);
            assert_eq!(
                game.world.query::<&Bullet>().iter().count(),
                count,
                "ability{id}"
            );
            assert_eq!(
                game.take_events()
                    .iter()
                    .filter(|e| e.kind == format!("ability-signature-{id}"))
                    .count(),
                1
            );
            clear_bullets(&mut game);
            fill_bullets(&mut game);
            game.ship_signature(motion, stats);
            assert!(
                !game
                    .take_events()
                    .iter()
                    .any(|e| e.kind.starts_with("ability-signature"))
            );
            clear_bullets(&mut game);
            game.profile.campaign.loadouts[ship as usize].slots[0] = None;
            game.ship_signature(motion, stats);
            assert_eq!(game.world.query::<&Bullet>().iter().count(), 0);
            assert!(game.take_events().is_empty());
        }
    }
    #[test]
    fn shield_absorption_hull_impacts_and_real_nova_have_distinct_cues() {
        let mut game = learned(4, 0);
        game.shield = 20.0;
        game.hurt_player(5.0, None);
        let events = game.take_events();
        assert_eq!(events.iter().filter(|e| e.kind == "shield").count(), 1);
        assert_eq!(
            events.iter().filter(|e| e.kind == "ability-nova").count(),
            1
        );
        assert!(!events.iter().any(|e| e.kind == "player-hit"));
        assert_eq!(events.iter().filter(|e| e.kind == "hit").count(), 1);
        game.invulnerability = 0.0;
        game.shield = 0.0;
        game.hurt_player(2.0, None);
        let events = game.take_events();
        assert!(events.iter().any(|e| e.kind == "hit" && e.intensity == 0.9));
        assert!(
            !events
                .iter()
                .any(|e| e.kind == "shield" || e.kind == "ability-nova")
        );
        game.move_player(Vec2::ZERO);
        game.invulnerability = 0.0;
        game.hurt_player(99.0, None);
        assert!(game.take_events().is_empty());
    }
    #[test]
    fn wake_and_boss_warning_cues_are_one_per_real_entry_or_emission() {
        let mut game = learned(5, 0);
        game.world.get::<&mut Motion>(game.player).unwrap().velocity = Vec2::X * 100.0;
        game.update_wake(game.position(), DT);
        assert_eq!(game.wake.len(), 1);
        assert_eq!(
            game.take_events()
                .iter()
                .filter(|e| e.kind == "ability-wake")
                .count(),
            1
        );
        game.update_wake(game.position(), DT);
        assert!(game.take_events().is_empty());
        game.diagnostic("boss-dart");
        game.take_events();
        let entity = game.active_boss.unwrap();
        game.boss_audio_transition(Some((entity, bosses::Phase::Recover)));
        assert_eq!(
            game.take_events()
                .iter()
                .filter(|e| e.kind == "boss-charge")
                .count(),
            1
        );
        game.boss_audio_transition(Some((entity, bosses::Phase::Tell)));
        assert!(game.take_events().is_empty());
        // Actual tick phase transitions must produce one cue, never per cell.
        let mut cues = 0;
        for _ in 0..360 {
            game.invulnerability = 10.0;
            game.tick(&InputFrame::default(), DT);
            cues += game
                .take_events()
                .iter()
                .filter(|e| e.kind == "boss-charge")
                .count();
        }
        assert!(cues > 0 && cues <= 3);
    }
    #[test]
    fn eight_real_champion_encounters_expose_their_exact_theme_identity() {
        for index in 0..8 {
            let mut game = Game::new(Profile::default());
            game.diagnostic(&format!("champion-{index}"));
            let audio = game.hud().audio;
            assert_eq!(audio.champion, Some(index));
            assert_eq!(audio.area, "arena");
            assert_eq!(audio.hostiles, 1);
            assert!(audio.combat > 0.5 && !audio.boss);
            game.reset_to_dock();
            assert_eq!(game.hud().audio.champion, None);
            assert_eq!(game.hud().audio.combat, 0.0);
        }
    }
    #[test]
    fn eight_canonical_champion_visits_and_retreats_have_theme_without_fake_aggro() {
        for index in 0..8 {
            let mut game = Game::new(Profile::default());
            let site = campaign::arena(game.profile.seed, index).unwrap();
            let center = Vec2::from_array(site.position);
            game.move_player(center);
            let before = serde_json::to_value(game.profile()).unwrap();
            let rng = game.rng;
            let audio = game.hud().audio;
            assert_eq!(audio.champion, Some(index as u8));
            assert_eq!(audio.area, "arena");
            assert!(game.safe());
            assert_eq!(audio.combat, 0.0);
            assert_eq!(audio.hostiles, 0);
            assert!(game.arena.is_none());
            assert_eq!(serde_json::to_value(game.profile()).unwrap(), before);
            assert_eq!(game.rng, rng);
            game.move_player(center + Vec2::X * 321.0);
            assert_eq!(game.hud().audio.champion, None);
            // An actual ordinary pursuer in the contact region must retain
            // its normal one-head intensity, without a fictitious duel bonus.
            game.move_player(center + Vec2::X * 300.0);
            game.spawn_enemy(0, false, center + Vec2::X * 550.0, 1);
            let audio = game.hud().audio;
            assert_eq!(audio.champion, Some(index as u8));
            assert_eq!(audio.hostiles, 1);
            assert!((audio.combat - 0.255).abs() < 0.0001);
            game.move_player(center);
            assert_eq!(game.hud().audio.combat, 0.0);
        }
    }
    #[test]
    fn royal_volley_and_repair_cues_require_real_effects_not_full_or_dead_hull() {
        let mut profile = Profile::default();
        profile.campaign.patches = [true; 8];
        profile.campaign.blueprints = [true; 8];
        profile.campaign.royale_results = [1; 8];
        profile.campaign.victory_claimed = true;
        profile.campaign.selected_ship = 9;
        ship_progression::grant_royal(&mut profile);
        let mut game = Game::new(profile);
        game.move_player(Vec2::new(1800.0, 0.0));
        game.take_events();
        game.shot_count = 8;
        let motion = *game.world.get::<&Motion>(game.player).unwrap();
        let stats = Stats::from_profile(game.profile());
        game.world.get::<&mut Actor>(game.player).unwrap().health -= 10.0;
        game.ship_signature(motion, stats);
        let events = game.take_events();
        assert_eq!(
            events.iter().filter(|e| e.kind == "ability-royal").count(),
            1
        );
        assert_eq!(
            events.iter().filter(|e| e.kind == "ability-repair").count(),
            1
        );
        assert_eq!(game.world.query::<&Bullet>().iter().count(), 8);
        game.world.get::<&mut Actor>(game.player).unwrap().health = stats.hull;
        fill_bullets(&mut game);
        game.ship_signature(motion, stats);
        assert!(
            !game
                .take_events()
                .iter()
                .any(|e| e.kind == "ability-repair" || e.kind == "ability-royal")
        );
        game.world.get::<&mut Actor>(game.player).unwrap().health = 0.0;
        game.ship_signature(motion, stats);
        assert!(
            !game
                .take_events()
                .iter()
                .any(|e| e.kind == "ability-repair")
        );
        assert_eq!(game.world.get::<&Actor>(game.player).unwrap().health, 0.0);
    }
}
