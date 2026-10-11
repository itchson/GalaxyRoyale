//! Chassis shields, purchased recharge and equipped main-manufacturer bonuses.
//! All strength derives from existing XP/rank/loadout records; no save fields.
use crate::{Profile, abilities, campaign, identity::Corporation, ship_progression};

pub(super) const SHIELD_CHUNK_HP: f32 = 10.0;
pub(super) const BASE_SHIELD: f32 = 3.0 * SHIELD_CHUNK_HP;

pub(super) fn capacity(profile: &Profile) -> f32 {
    let hardware = ship_progression::hardware_ranks(profile)[3];
    let guard = ship_progression::equipped_rank(profile, 1);
    BASE_SHIELD
        + SHIELD_CHUNK_HP * hardware as f32
        + if guard > 0 {
            20.0 + 8.0 * guard.saturating_sub(1) as f32
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

/// The main manufacturer's bonus requires a real, compatible equipped item in
/// its canonical category. Secondary manufacture remains visual only.
fn matching_slot(profile: &Profile, category: usize) -> bool {
    let Some(identity) = &profile.identity else {
        return false;
    };
    let matching_category = match identity.corporation {
        Corporation::Pan => 2,
        Corporation::Renko => 0,
        Corporation::Hugo => 1,
    };
    if category != matching_category {
        return false;
    }
    let slot = profile.campaign.loadouts[profile.campaign.selected_ship as usize].slots[category];
    slot.is_some_and(|id| {
        abilities::get(id).is_some_and(|def| def.category == category)
            && ship_progression::equipped_rank(profile, id) > 0
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
