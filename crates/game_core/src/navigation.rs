//! Discovery-only compass targets. No new progression or world-state writes.
use crate::{Game, MAX_ARCHIVE, Vec2, regions};
use serde::Serialize;

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct NavigationHud {
    pub home: [f32; 2],
    pub quest: Option<NavigationTarget>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct NavigationTarget {
    pub position: [f32; 2],
    /// 0: deliver courier to Haven, 1: return to claim, 2: active bounty site.
    pub kind: u8,
    pub site_id: String,
    pub name: String,
}

impl Game {
    /// Completed quests and undiscovered sites never become compass spoilers.
    /// Ready claims precede courier delivery, then the nearest active bounty;
    /// canonical site ID breaks equal-distance ties, independent of save order.
    pub fn navigation_hud(&self) -> NavigationHud {
        let position = self.position();
        let mut best: Option<(u8, f32, NavigationTarget)> = None;
        for quest in self.profile.quests.iter().take(MAX_ARCHIVE) {
            if quest.completed
                || self.profile.relics.contains(&quest.site_id)
                || !self.profile.discoveries.contains(&quest.site_id)
            {
                continue;
            }
            let Some(site) = regions::site_by_id(self.profile.seed, &quest.site_id) else {
                continue;
            };
            let (priority, kind, destination) = if quest.progress >= quest.target {
                (0, 1, site.position)
            } else if quest.event == "courier" {
                (1, 0, [0.0; 2])
            } else {
                (2, 2, site.position)
            };
            let distance = position.distance_squared(Vec2::from_array(destination));
            let prefer = best.as_ref().is_none_or(|(rank, old_distance, old)| {
                priority < *rank
                    || (priority == *rank
                        && (distance < *old_distance
                            || (distance == *old_distance && site.id < old.site_id)))
            });
            if prefer {
                best = Some((
                    priority,
                    distance,
                    NavigationTarget {
                        position: destination,
                        kind,
                        site_id: site.id,
                        name: site.name.into(),
                    },
                ));
            }
        }
        NavigationHud {
            home: [0.0; 2],
            quest: best.map(|(_, _, target)| target),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Profile, QuestProgress};

    fn with_quest(event: &str, progress: u32) -> Profile {
        let mut profile = Profile::default();
        let site = regions::sites_near(profile.seed, Vec2::ZERO)
            .into_iter()
            .find(|s| s.event == event)
            .expect("actual seeded event fixture");
        profile.discoveries.push(site.id.clone());
        profile.quests.push(QuestProgress {
            site_id: site.id,
            event: site.event.into(),
            family: site.family,
            target: if event == "courier" {
                1
            } else {
                site.target_kills
            },
            progress,
            completed: false,
        });
        profile
    }

    #[test]
    fn fresh_and_undiscovered_quests_reveal_only_home() {
        assert!(
            Game::new(Profile::default())
                .navigation_hud()
                .quest
                .is_none()
        );
        let mut profile = with_quest("bounty", 0);
        profile.discoveries.clear();
        let game = Game::new(profile);
        let before = serde_json::to_value(game.profile()).unwrap();
        assert_eq!(game.navigation_hud().home, [0.0; 2]);
        assert!(game.navigation_hud().quest.is_none());
        assert_eq!(serde_json::to_value(game.profile()).unwrap(), before);
    }

    #[test]
    fn courier_delivery_and_claim_use_actual_home_and_seeded_site() {
        let mut profile = with_quest("courier", 0);
        // Game::new intentionally marks retained courier packets delivered;
        // set the actual transient state explicitly to test the live stage.
        let mut game = Game::new(profile.clone());
        game.profile.quests[0].progress = 0;
        let delivery = game.navigation_hud().quest.unwrap();
        assert_eq!((delivery.kind, delivery.position), (0, [0.0; 2]));
        profile.quests[0].progress = 1;
        let claim = Game::new(profile.clone()).navigation_hud().quest.unwrap();
        assert_eq!(claim.kind, 1);
        assert_eq!(
            claim.position,
            regions::site_by_id(profile.seed, &claim.site_id)
                .unwrap()
                .position
        );
        profile.quests[0].completed = true;
        assert!(Game::new(profile).navigation_hud().quest.is_none());
    }

    #[test]
    fn active_bounty_is_seeded_and_priority_is_save_order_independent() {
        let mut profile = with_quest("bounty", 0);
        let mut courier = with_quest("courier", 1);
        profile.discoveries.append(&mut courier.discoveries);
        profile.quests.append(&mut courier.quests);
        let first = Game::new(profile.clone()).navigation_hud();
        assert_eq!(first.quest.as_ref().unwrap().kind, 1);
        profile.quests.reverse();
        assert_eq!(Game::new(profile).navigation_hud(), first);
        let profile = with_quest("bounty", 0);
        let target = Game::new(profile.clone()).navigation_hud().quest.unwrap();
        assert_eq!(target.kind, 2);
        assert_eq!(
            target.position,
            regions::site_by_id(profile.seed, &target.site_id)
                .unwrap()
                .position
        );
    }

    #[test]
    fn nearest_active_discovered_bounty_uses_live_player_position() {
        let mut profile = Profile::default();
        let sites: Vec<_> = regions::sites_near(profile.seed, Vec2::ZERO)
            .into_iter()
            .filter(|s| s.event == "bounty")
            .take(2)
            .collect();
        assert_eq!(sites.len(), 2);
        for site in &sites {
            profile.discoveries.push(site.id.clone());
            profile.quests.push(QuestProgress {
                site_id: site.id.clone(),
                event: site.event.into(),
                family: site.family,
                target: site.target_kills,
                progress: 0,
                completed: false,
            });
        }
        let mut game = Game::new(profile);
        for site in &sites {
            // A position fixture, not a claimed naturally travelled journey.
            game.world
                .get::<&mut crate::Motion>(game.player)
                .unwrap()
                .position = Vec2::from_array(site.position);
            let first = game.navigation_hud();
            assert_eq!(first.quest.as_ref().unwrap().site_id, site.id);
            game.profile.quests.reverse();
            assert_eq!(game.navigation_hud(), first);
        }
    }
}
