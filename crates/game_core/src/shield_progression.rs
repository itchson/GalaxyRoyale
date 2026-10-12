//! Chassis shields, purchased recharge and equipped main-manufacturer bonuses.
//! All strength derives from existing XP/rank/loadout records; no save fields.
use crate::{Profile, abilities, campaign, identity::Corporation, ship_progression};
use serde::Serialize;

/// Display provenance keeps Sovereign's licensed affinity distinct from an
/// authored chassis manufacturer. This is transient HUD data, never save state.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RaichoAffinitySource {
    FactoryLicense,
    RosterManufacturer,
    SovereignLicense,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub struct RaichoAffinityHud {
    pub corporation: Corporation,
    pub slot: u8,
    pub active: bool,
    pub source: RaichoAffinitySource,
}

pub(super) const SHIELD_CHUNK_HP: f32 = 10.0;
pub(super) const BASE_SHIELD: f32 = 3.0 * SHIELD_CHUNK_HP;

pub(super) fn capacity(profile: &Profile) -> f32 {
    let hardware = ship_progression::hardware_ranks(profile)[3];
    let guard = ship_progression::equipped_rank(profile, 1);
    // Licensed factory hulls start with their single equipped corporate ability.
    // Guard Coil supplies HUGO's three chunks; speed/offense hulls have no innate
    // shield. Anonymous legacy Scout and later chassis retain their base armor.
    let base = if profile.campaign.selected_ship == 0 && profile.identity.is_some() {
        0.0
    } else {
        BASE_SHIELD
    };
    base + SHIELD_CHUNK_HP * hardware as f32
        + if guard > 0 {
            30.0 + 8.0 * guard.saturating_sub(1) as f32
        } else {
            0.0
        }
        + if profile.campaign.selected_ship == 9 {
            40.0 // Explicit enhanced Sovereign chassis; not a hidden ability.
        } else {
            0.0
        }
}

pub(super) fn recharge_rate(profile: &Profile) -> f32 {
    let hardware = ship_progression::hardware_ranks(profile)[3];
    // A factory Guard Coil R1 increases capacity. Its first real upgrade also
    // installs recharge, including the scoped level-one training exception.
    let guard = ship_progression::equipped_rank(profile, 1).saturating_sub(1);
    let rank = hardware.max(guard);
    if rank == 0 {
        return 0.0;
    }
    let level =
        campaign::level(profile.campaign.ship_xp[profile.campaign.selected_ship as usize]).min(12);
    let rate = 3.0
        * (1.0 + 0.2 * rank.saturating_sub(1) as f32)
        * (1.0 + 0.03 * level.saturating_sub(1) as f32);
    if profile.campaign.selected_ship == 9 {
        rate.max(18.0)
    } else {
        rate
    }
}

fn main_manufacturer(profile: &Profile) -> Option<(Corporation, RaichoAffinitySource)> {
    match profile.campaign.selected_ship {
        // Hull 0 is the licensed factory variant. Sovereign has no authored
        // manufacturer yet, so its explicit current policy is licensed-main
        // affinity; this does not change its enhanced chassis or royal slots.
        0 | 9 => profile.identity.as_ref().map(|identity| {
            (
                identity.corporation,
                if profile.campaign.selected_ship == 0 {
                    RaichoAffinitySource::FactoryLicense
                } else {
                    RaichoAffinitySource::SovereignLicense
                },
            )
        }),
        1..=8 => Some((
            campaign::CHAMPIONS[profile.campaign.selected_ship as usize - 1]
                .identity
                .corporation,
            RaichoAffinitySource::RosterManufacturer,
        )),
        _ => None,
    }
}

fn matching_category(manufacturer: Corporation) -> usize {
    match manufacturer {
        Corporation::Pan => 2,
        Corporation::Renko => 0,
        Corporation::Hugo => 1,
    }
}

/// The active RAICHO manufacturer's bonus requires a real, compatible equipped
/// item in its canonical category. Sponsor and secondary manufacture are visual.
fn matching_slot(profile: &Profile, category: usize) -> bool {
    let Some((manufacturer, _)) = main_manufacturer(profile) else {
        return false;
    };
    if category != matching_category(manufacturer) {
        return false;
    }
    let slot = profile.campaign.loadouts[profile.campaign.selected_ship as usize].slots[category];
    slot.is_some_and(|id| {
        abilities::get(id).is_some_and(|def| def.category == category)
            && ship_progression::equipped_rank(profile, id) > 0
    })
}

pub(super) fn hud(profile: &Profile) -> Option<RaichoAffinityHud> {
    let (corporation, source) = main_manufacturer(profile)?;
    let category = matching_category(corporation);
    Some(RaichoAffinityHud {
        corporation,
        slot: category as u8,
        active: matching_slot(profile, category),
        source,
    })
}

pub(super) fn weapon_multiplier(profile: &Profile) -> f32 {
    if matching_slot(profile, 0) { 1.1 } else { 1.0 }
}

pub(super) fn drive_multiplier(profile: &Profile) -> f32 {
    if matching_slot(profile, 2) { 1.1 } else { 1.0 }
}

pub(super) fn incoming_damage_multiplier(profile: &Profile) -> f32 {
    if matching_slot(profile, 1) { 0.9 } else { 1.0 }
}

#[cfg(test)]
#[path = "shield_progression_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "raicho_affinity_hud_tests.rs"]
mod hud_tests;
