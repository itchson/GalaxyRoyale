//! Discovery-only navigation snapshots. Queried on demand, not every frame.
use crate::{Game, Vec2, regions};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct UniverseMap {
    pub position: [f32; 2],
    pub heading: f32,
    pub biome: regions::Biome,
    pub home_radius: f32,
    pub belt_inner: f32,
    pub belt_outer: f32,
    pub sites: Vec<MapSite>,
}

#[derive(Clone, Debug, Serialize)]
pub struct MapSite {
    pub id: String,
    pub name: String,
    pub position: [f32; 2],
    pub radius: f32,
    pub event: String,
    pub status: String,
    pub biome: regions::Biome,
    pub progress: Option<u32>,
    pub target: Option<u32>,
    pub belt_inner: Option<f32>,
    pub belt_outer: Option<f32>,
    pub material_tier: Option<u8>,
    pub recommended_level: Option<u32>,
}

impl Game {
    /// A beacon enters this map only after authoritative exploration discovers
    /// it. Coordinates and resources are regenerated from this world's seed.
    pub fn universe_map(&self) -> UniverseMap {
        let position = self.position();
        let heading = self
            .world
            .get::<&crate::Motion>(self.player)
            .map_or(0.0, |m| m.heading);
        let sites = self
            .profile
            .discoveries
            .iter()
            .take(crate::MAX_ARCHIVE)
            .filter_map(|id| regions::site_by_id(self.profile.seed, id))
            .map(|site| {
                let hud = self.site_hud(&site);
                let quest = self.profile.quests.iter().find(|q| q.site_id == site.id);
                let belt = regions::site_belt(self.profile.seed, &site);
                MapSite {
                    biome: regions::at(self.profile.seed, Vec2::from_array(site.position)),
                    position: site.position,
                    radius: site.radius,
                    id: hud.id,
                    name: hud.name,
                    event: hud.event,
                    status: hud.status,
                    progress: quest.map(|q| q.progress),
                    target: quest.map(|q| q.target),
                    belt_inner: belt.map(|b| b.inner),
                    belt_outer: belt.map(|b| b.outer),
                    material_tier: belt.map(|b| b.material_tier),
                    recommended_level: belt.map(|b| b.recommended_level),
                }
            })
            .collect();
        UniverseMap {
            position: position.to_array(),
            heading,
            biome: regions::at(self.profile.seed, position),
            home_radius: crate::SAFE_RADIUS,
            belt_inner: crate::BELT_INNER,
            belt_outer: crate::COMBAT_RADIUS,
            sites,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InputFrame, Profile};

    #[test]
    fn fresh_map_reveals_home_and_live_position_without_undiscovered_beacons() {
        let mut game = Game::new(Profile::default());
        let first = game.universe_map();
        assert!(first.sites.is_empty());
        assert_eq!(
            (first.home_radius, first.belt_inner, first.belt_outer),
            (240.0, 700.0, 1400.0)
        );
        game.tick(
            &InputFrame {
                joystick: Some([1.0, 0.0]),
                ..Default::default()
            },
            1.0 / 60.0,
        );
        assert_eq!(game.universe_map().position, game.hud().position);
        assert!(game.universe_map().heading.is_finite());
    }

    #[test]
    fn saved_discoveries_regenerate_exact_seeded_geometry_and_quest_status() {
        let mut profile = Profile::default();
        let site = regions::sites_near(profile.seed, Vec2::ZERO)
            .into_iter()
            .find(|site| regions::site_belt(profile.seed, site).is_some())
            .unwrap();
        let progress = if site.event == "courier" { 1 } else { 3 };
        profile.discoveries.push(site.id.clone());
        profile.quests.push(crate::QuestProgress {
            site_id: site.id.clone(),
            event: site.event.into(),
            family: site.family,
            target: if site.event == "courier" {
                1
            } else {
                site.target_kills
            },
            progress,
            completed: false,
        });
        let game = Game::new(profile.clone());
        let snapshot = game.universe_map();
        assert_eq!(snapshot.sites.len(), 1);
        let mapped = &snapshot.sites[0];
        assert_eq!(mapped.position, site.position);
        assert_eq!(mapped.radius, site.radius);
        assert_eq!(mapped.progress, Some(progress));
        assert_eq!(
            mapped.biome.kind,
            regions::at(profile.seed, Vec2::from_array(site.position)).kind
        );
        let belt = regions::site_belt(profile.seed, &site).unwrap();
        assert_eq!(mapped.belt_inner, Some(belt.inner));
        assert_eq!(mapped.belt_outer, Some(belt.outer));
        assert_eq!(mapped.material_tier, Some(belt.material_tier));
        assert_eq!(mapped.recommended_level, Some(belt.recommended_level));
        assert_eq!(mapped.status, game.site_hud(&site).status);
        assert_eq!(
            Game::new(profile).universe_map().sites[0].position,
            mapped.position
        );
    }

    #[test]
    fn invalid_save_discoveries_cannot_inject_map_positions() {
        let profile = Profile {
            discoveries: vec!["999999:0".into(), "0:0:extra".into(), "NaN:0".into()],
            ..Default::default()
        };
        assert!(Game::new(profile).universe_map().sites.is_empty());
    }
}
