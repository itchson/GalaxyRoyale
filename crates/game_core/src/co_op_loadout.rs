//! Bounded offline build declarations, never remote saves or ownership proofs.
use crate::{
    Profile, Stats, abilities, bosses, campaign, co_op, identity, ship_progression, training,
};
use co_op::{Error, ParticipantSpec};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedAbility {
    pub index: u8,
    pub rank: u8,
}

/// Only combat-relevant facts from the guest's actual native Profile. The host
/// validates plausible registry/ownership claims; offline peers can lie about
/// durable ownership. This does not authorize any grants or save mutations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedLoadout {
    pub version: u8,
    pub selected_ship: u8,
    pub owned_ships: u16,
    pub patches: u8,
    pub royal_victory: bool,
    pub ship_xp: u32,
    pub hardware: [u8; 6],
    pub slots: [Option<OwnedAbility>; 3],
    pub trained_starter: bool,
    pub relic_count: u8,
    /// Effect-only offline claim; missing historical builds keep evolution one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub echo_evolution: Option<u8>,
    /// Ordinary Repulsor effect only; no remote boss defeat history is imported.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_evolution"
    )]
    pub nova_evolution: Option<u8>,
}

fn present_evolution<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<u8>, D::Error> {
    // Missing is the historical default. A supplied null is not a numeric claim.
    u8::deserialize(deserializer).map(Some)
}

impl OwnedLoadout {
    pub(super) fn from_profile(profile: &Profile) -> Result<Self, Error> {
        let ship = profile.campaign.selected_ship;
        let loadout = profile
            .campaign
            .loadouts
            .get(ship as usize)
            .ok_or(Error::InvalidLoadout)?;
        let result = Self {
            version: 1,
            selected_ship: ship,
            owned_ships: (0..10)
                .filter(|&id| profile.campaign.owned(id))
                .fold(0, |mask, id| mask | (1 << id)),
            patches: profile
                .campaign
                .patches
                .iter()
                .enumerate()
                .fold(0, |mask, (i, &earned)| mask | (u8::from(earned) << i)),
            royal_victory: profile.campaign.victory_claimed,
            ship_xp: profile.campaign.ship_xp[ship as usize],
            hardware: loadout.hardware,
            slots: loadout.slots.map(|index| {
                index.map(|index| OwnedAbility {
                    index,
                    rank: profile
                        .campaign
                        .abilities
                        .get(index as usize)
                        .map_or(0, |state| if state.blueprint { state.rank } else { 0 }),
                })
            }),
            trained_starter: loadout
                .slots
                .iter()
                .flatten()
                .any(|&id| training::trained_starter(profile, id)),
            relic_count: profile.relics.len().min(20) as u8,
            echo_evolution: (ship_progression::equipped_rank(profile, 3) > 0)
                .then(|| bosses::evolution(profile, 0))
                .filter(|&evolution| evolution > 1),
            nova_evolution: (ship_progression::equipped_rank(profile, 4) > 0)
                .then(|| bosses::evolution(profile, 1))
                .filter(|&evolution| evolution > 1),
        };
        result.profile(&ParticipantSpec {
            id: co_op::ParticipantId::parse("00000000000000000000000000000000")?,
            local_seed: profile.seed,
            identity: profile.identity.clone(),
        })?;
        Ok(result)
    }

    pub(super) fn profile(&self, spec: &ParticipantSpec) -> Result<Profile, Error> {
        if self.version != 1
            || self.selected_ship > 9
            || self.owned_ships & !abilities::ALL_SHIPS != 0
            || self.owned_ships & 1 == 0
            || self.owned_ships & (1 << self.selected_ship) == 0
            || self.hardware.iter().any(|&rank| rank > 5)
            || self.relic_count > 20
            || self.echo_evolution.is_some_and(|evolution| {
                !(1..=5).contains(&evolution) || !self.slots[0].is_some_and(|slot| slot.index == 3)
            })
            || self.nova_evolution.is_some_and(|evolution| {
                !(1..=5).contains(&evolution) || !self.slots[1].is_some_and(|slot| slot.index == 4)
            })
            || ((self.owned_ships >> 9) != 0) != self.royal_victory
            || (self.royal_victory && self.patches != u8::MAX)
            || ((self.owned_ships >> 1) as u8 & !self.patches) != 0
        {
            return Err(Error::InvalidLoadout);
        }
        let mut p = co_op::guest_profile(spec)?;
        p.campaign.selected_ship = self.selected_ship;
        p.campaign.patches = std::array::from_fn(|i| self.patches & (1 << i) != 0);
        p.campaign.blueprints = p.campaign.patches;
        p.campaign.crafted = std::array::from_fn(|i| self.owned_ships & (1 << (i + 1)) != 0);
        p.campaign.victory_claimed = self.royal_victory;
        p.campaign.ship_xp[self.selected_ship as usize] = self.ship_xp;
        p.campaign.abilities.fill(campaign::AbilityState::default());
        p.campaign.loadouts.fill(campaign::ShipLoadout::default());
        p.campaign.loadouts[self.selected_ship as usize].hardware = self.hardware;
        for (category, slot) in self.slots.iter().enumerate() {
            let Some(slot) = slot else { continue };
            let def = abilities::get(slot.index).ok_or(Error::InvalidLoadout)?;
            if slot.rank == 0
                || slot.rank > 5
                || def.category != category
                || !abilities::compatible(slot.index, self.selected_ship)
                || match def.blueprint {
                    abilities::Blueprint::Champion(i) => self.patches & (1 << i) == 0,
                    abilities::Blueprint::RoyalVictory => !self.royal_victory || slot.rank != 5,
                    _ => false,
                }
            {
                return Err(Error::InvalidLoadout);
            }
            p.campaign.abilities[slot.index as usize] = campaign::AbilityState {
                blueprint: true,
                rank: slot.rank,
            };
            p.campaign.loadouts[self.selected_ship as usize].slots[category] = Some(slot.index);
        }
        if self.trained_starter {
            let starter = spec
                .identity
                .as_ref()
                .ok_or(Error::InvalidLoadout)?
                .corporation
                .ability();
            let category = abilities::get(starter).unwrap().category;
            if self.selected_ship != 0
                || !self.slots[category].is_some_and(|s| s.index == starter && s.rank >= 2)
            {
                return Err(Error::InvalidLoadout);
            }
            // An isolated combat prerequisite only. The real onboarding record,
            // counters, cargo and paid recipe never travel to this temporary build.
            p.onboarding = Some(identity::OnboardingProgress {
                stage: 6,
                flags: 127,
                asteroids_destroyed: 10,
                materials_collected: 10,
                ..Default::default()
            });
        } else {
            p.onboarding = None;
        }
        ship_progression::mirrors(&mut p);
        Ok(p)
    }
}

pub(super) fn stats(profile: &Profile, owned: Option<&OwnedLoadout>) -> Stats {
    let mut stats = Stats::from_profile(profile);
    if let Some(owned) = owned {
        // Temporary profiles contain no named relic inventory. Apply only the
        // same bounded hull contribution as Stats::from_profile's real relics.
        stats.hull += f32::from(owned.relic_count);
    }
    stats
}

impl crate::Game {
    /// Read the retained solo native save even while a host session is active.
    pub fn co_op_owned_loadout(&self) -> Result<OwnedLoadout, Error> {
        OwnedLoadout::from_profile(self.profile())
    }
}

#[cfg(test)]
#[path = "co_op_loadout_tests.rs"]
mod tests;
