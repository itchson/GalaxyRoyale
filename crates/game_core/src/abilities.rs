//! Finite dock ability inventory. Save IDs are the positions in `ABILITIES`.
//! Do not reorder existing entries. Chassis construction currencies are independent
//! of these effect categories. Every normal-hull effect requires an equipped,
//! owned ability and uses its selected ship's effective rank.
//!
//! Sovereign's three royalty entries are the explicit rank-five reward exception.
//! They still require equipped slots. Its level and XP use ordinary per-ship XP.

pub const COUNT: usize = 17;
pub const MAX_RANK: u8 = 5;
pub const SHIP_COUNT: u8 = 10;
pub const ALL_SHIPS: u16 = (1 << SHIP_COUNT) - 1;
pub const ROYAL_SHIP: u8 = 9;
pub const ROYAL_SHIPS: u16 = 1 << ROYAL_SHIP;
/// Ship 0 is Scout. Ships 1..8 follow champion order. Sovereign is a wildcard
/// for ordinary abilities, but only its three royal entries ignore its XP cap.
pub const SHIP_FAMILIES: [usize; 9] = [0, 0, 1, 2, 0, 0, 1, 2, 1];

pub const fn family_ships(family: usize) -> u16 {
    if family > 2 {
        return 0;
    }
    let mut ships = ROYAL_SHIPS;
    let mut i = 0;
    while i < SHIP_FAMILIES.len() {
        if SHIP_FAMILIES[i] == family {
            ships |= 1 << i;
        }
        i += 1;
    }
    ships
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blueprint {
    Starter,
    BossFamily(usize),
    Champion(usize),
    RoyalVictory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    PulseLens,
    GuardCoil,
    FluxDrive,
    PulseEcho,
    Repulsor,
    IonWake,
    NeedleBurst,
    Broadside,
    GuidedMissile,
    ForkVolley,
    WindrunnerWake,
    ImpactNova,
    PiercingSeeker,
    SniperSpear,
    RoyalPrism,
    RoyalAegis,
    RoyalWake,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AbilityDef {
    pub key: &'static str,
    pub name: &'static str,
    /// Slot and point currency: 0 offense, 1 defense, 2 passive.
    pub category: usize,
    pub blueprint: Blueprint,
    pub effect: Effect,
    /// Bits 0..9 identify compatible owned hulls; ownership is checked by Game.
    pub compatible_ships: u16,
    pub description: &'static str,
}

const fn def(
    key: &'static str,
    name: &'static str,
    category: usize,
    blueprint: Blueprint,
    effect: Effect,
    description: &'static str,
) -> AbilityDef {
    AbilityDef {
        key,
        name,
        category,
        blueprint,
        effect,
        compatible_ships: match blueprint {
            Blueprint::Champion(index) => family_ships(SHIP_FAMILIES[index + 1]),
            Blueprint::RoyalVictory => ROYAL_SHIPS,
            _ => ALL_SHIPS,
        },
        description,
    }
}

/// Stable IDs: starters 0..2, ordinary boss rewards 3..5, champions 6..13,
/// Sovereign reward bundles 14..16.
/// Champions retain their authored index order; their construction category
/// must never be used to determine their ability slot.
pub const ABILITIES: [AbilityDef; COUNT] = [
    def(
        "pulse-lens",
        "Pulse Lens",
        0,
        Blueprint::Starter,
        Effect::PulseLens,
        "Primary shots deal +10% damage per effective rank; every fifth fired shot adds a bolt dealing 150% of primary damage.",
    ),
    def(
        "guard-coil",
        "Guard Coil",
        1,
        Blueprint::Starter,
        Effect::GuardCoil,
        "Grants 30 shield at rank 1, then +8 per effective rank; rank 2 unlocks recharge after 4s without damage.",
    ),
    def(
        "flux-drive",
        "Flux Drive",
        2,
        Blueprint::Starter,
        Effect::FluxDrive,
        "Adds +4% speed and acceleration per effective rank.",
    ),
    def(
        "pulse-echo",
        "Pulse Echo",
        0,
        Blueprint::BossFamily(0),
        Effect::PulseEcho,
        "Periodically fires an additional piercing fan; rank increases its damage.",
    ),
    def(
        "repulsor",
        "Repulsor",
        1,
        Blueprint::BossFamily(1),
        Effect::Repulsor,
        "Taking damage releases a repulsor nova; rank shortens its cooldown.",
    ),
    def(
        "ion-wake",
        "Ion Wake",
        2,
        Blueprint::BossFamily(2),
        Effect::IonWake,
        "Fast flight leaves a damaging ion trail; rank increases its damage.",
    ),
    def(
        "needle-burst",
        "Needle Burst",
        0,
        Blueprint::Champion(0),
        Effect::NeedleBurst,
        "Every fourth primary shot releases a narrow two-shot needle burst.",
    ),
    def(
        "broadside",
        "Broadside",
        0,
        Blueprint::Champion(1),
        Effect::Broadside,
        "Every sixth primary shot fires a pair of side-facing projectiles.",
    ),
    def(
        "guided-missile",
        "Guided Missile",
        0,
        Blueprint::Champion(2),
        Effect::GuidedMissile,
        "Every fifth primary shot launches a guided missile.",
    ),
    def(
        "fork-volley",
        "Fork Volley",
        0,
        Blueprint::Champion(3),
        Effect::ForkVolley,
        "Every third primary shot releases a split-wing fork volley.",
    ),
    def(
        "windrunner-wake",
        "Windrunner Wake",
        2,
        Blueprint::Champion(4),
        Effect::WindrunnerWake,
        "Fast flight leaves the champion's damaging ion trail.",
    ),
    def(
        "impact-nova",
        "Impact Nova",
        1,
        Blueprint::Champion(5),
        Effect::ImpactNova,
        "Taking damage releases a nova with the champion's short cooldown.",
    ),
    def(
        "piercing-seeker",
        "Piercing Seeker",
        0,
        Blueprint::Champion(6),
        Effect::PiercingSeeker,
        "Every fourth primary shot launches a guided piercing seeker.",
    ),
    def(
        "sniper-spear",
        "Sniper Spear",
        0,
        Blueprint::Champion(7),
        Effect::SniperSpear,
        "Every sixth primary shot fires a fast projectile that pierces multiple targets.",
    ),
    def(
        "royal-prism",
        "Royal Prism",
        0,
        Blueprint::RoyalVictory,
        Effect::RoyalPrism,
        "Sovereign reward: an eight-shot prism burst and fully evolved Pulse Echo.",
    ),
    def(
        "royal-aegis",
        "Royal Aegis",
        1,
        Blueprint::RoyalVictory,
        Effect::RoyalAegis,
        "Sovereign reward: every eighth primary repairs 4 hull; damage triggers a fully evolved nova.",
    ),
    def(
        "royal-wake",
        "Royal Wake",
        2,
        Blueprint::RoyalVictory,
        Effect::RoyalWake,
        "Sovereign reward: fast flight leaves the fully evolved rank-five ion trail.",
    ),
];

pub fn get(index: u8) -> Option<&'static AbilityDef> {
    ABILITIES.get(index as usize)
}

pub fn by_key(key: &str) -> Option<u8> {
    ABILITIES
        .iter()
        .position(|entry| entry.key == key)
        .map(|i| i as u8)
}

pub fn compatible(index: u8, ship: u8) -> bool {
    ship < SHIP_COUNT && get(index).is_some_and(|entry| entry.compatible_ships & (1 << ship) != 0)
}

/// The caller supplies `campaign::rank_cap(campaign::level(ship_xp[ship]))`.
/// A library rank is preserved when a lower-level ship equips it. The three
/// royal reward entries alone bypass that cap, and only on Sovereign. This
/// helper does not establish ship ownership, blueprint validity or equipment;
/// Game must verify them before querying an effect.
pub fn effective_rank(index: u8, owned_rank: u8, ship: u8, ship_rank_cap: u8) -> u8 {
    if !compatible(index, ship) {
        return 0;
    }
    let rank = owned_rank.min(MAX_RANK);
    if matches!(get(index).unwrap().blueprint, Blueprint::RoyalVictory) {
        rank
    } else {
        rank.min(ship_rank_cap.min(MAX_RANK))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recipe {
    pub points: u32,
    pub materials: [u32; 5],
}

/// Crafting uses next rank 1; upgrades use the actual next library rank 2..5.
/// The selected ship's level gate and available inventory are Game transactions.
/// Royal rewards are granted by a verified victory and cannot be crafted.
pub fn recipe(index: u8, next_rank: u8) -> Option<Recipe> {
    let entry = get(index)?;
    if !(1..=MAX_RANK).contains(&next_rank) || matches!(entry.blueprint, Blueprint::RoyalVictory) {
        return None;
    }
    let mut materials = [0; 5];
    materials[next_rank as usize - 1] = 2;
    Some(Recipe {
        points: if next_rank == 1 && matches!(entry.blueprint, Blueprint::Starter) {
            3
        } else {
            10
        },
        materials,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_keys_roundtrip_and_are_unique() {
        let expected = [
            "pulse-lens",
            "guard-coil",
            "flux-drive",
            "pulse-echo",
            "repulsor",
            "ion-wake",
            "needle-burst",
            "broadside",
            "guided-missile",
            "fork-volley",
            "windrunner-wake",
            "impact-nova",
            "piercing-seeker",
            "sniper-spear",
            "royal-prism",
            "royal-aegis",
            "royal-wake",
        ];
        for (index, key) in expected.into_iter().enumerate() {
            assert_eq!(by_key(key), Some(index as u8));
            assert_eq!(get(index as u8).unwrap().key, key);
            assert_eq!(ABILITIES.iter().filter(|entry| entry.key == key).count(), 1);
        }
        assert_eq!(by_key(""), None);
        assert_eq!(by_key("Pulse Lens"), None);
        assert_eq!(get(COUNT as u8), None);
        assert_eq!(get(u8::MAX), None);
    }

    #[test]
    fn starter_and_boss_categories_fill_exactly_three_slots() {
        for category in 0..3 {
            assert_eq!(ABILITIES[category].category, category);
            assert_eq!(ABILITIES[category].blueprint, Blueprint::Starter);
            assert_eq!(ABILITIES[category + 3].category, category);
            assert_eq!(
                ABILITIES[category + 3].blueprint,
                Blueprint::BossFamily(category)
            );
        }
    }

    #[test]
    fn champion_effect_categories_are_independent_of_construction_currency() {
        let categories = [0, 0, 0, 0, 2, 1, 0, 0];
        for (champion, category) in categories.into_iter().enumerate() {
            let entry = &ABILITIES[champion + 6];
            assert_eq!(entry.category, category);
            assert_eq!(entry.blueprint, Blueprint::Champion(champion));
        }
        assert_eq!(ABILITIES[7].effect, Effect::Broadside);
        assert_eq!(ABILITIES[8].effect, Effect::GuidedMissile);
        assert_eq!(ABILITIES[12].effect, Effect::PiercingSeeker);
    }

    #[test]
    fn masks_follow_the_actual_champion_chassis_families() {
        assert_eq!(SHIP_FAMILIES[0], 0);
        for (index, champion) in crate::campaign::CHAMPIONS.iter().enumerate() {
            assert_eq!(SHIP_FAMILIES[index + 1], champion.family);
            assert_eq!(
                ABILITIES[index + 6].compatible_ships,
                family_ships(champion.family)
            );
        }
    }

    #[test]
    fn universal_abilities_and_family_compatibility_are_bounded_to_real_hulls() {
        assert_eq!(family_ships(3), 0);
        assert_eq!(family_ships(usize::MAX), 0);
        for ability in 0..6 {
            for ship in 0..SHIP_COUNT {
                assert!(compatible(ability, ship));
            }
            assert!(!compatible(ability, SHIP_COUNT));
            assert!(!compatible(ability, u8::MAX));
            assert_eq!(ABILITIES[ability as usize].compatible_ships, ALL_SHIPS);
            assert!(ABILITIES[ability as usize].category < 3);
        }
        let families = [0, 1, 2, 0, 0, 1, 2, 1];
        for (champion, family) in families.into_iter().enumerate() {
            let ability = champion as u8 + 6;
            for (ship, &ship_family) in SHIP_FAMILIES.iter().enumerate() {
                assert_eq!(compatible(ability, ship as u8), family == ship_family);
            }
            assert!(compatible(ability, ROYAL_SHIP));
            assert!(!compatible(ability, SHIP_COUNT));
            assert!(!compatible(ability, u8::MAX));
        }
        assert!(!compatible(COUNT as u8, 0));
        assert!(!compatible(u8::MAX, 9));
    }

    #[test]
    fn lower_ship_cap_never_rewrites_or_inherits_the_library_rank() {
        let library = 5;
        assert_eq!(effective_rank(0, library, 0, 1), 1);
        assert_eq!(effective_rank(0, library, 0, 3), 3);
        assert_eq!(effective_rank(0, library, 0, 5), 5);
        assert_eq!(library, 5);
        assert_eq!(effective_rank(0, 0, 0, 5), 0);
        assert_eq!(effective_rank(0, 3, 0, 0), 0);
        assert_eq!(effective_rank(0, u8::MAX, 0, u8::MAX), 5);
        assert_eq!(effective_rank(0, 5, ROYAL_SHIP, 1), 1);
        assert_eq!(effective_rank(7, 5, 0, 5), 0);
        assert_eq!(effective_rank(u8::MAX, 5, 0, 5), 0);
    }

    #[test]
    fn royalty_is_ship_nine_only_with_three_explicit_slots() {
        for category in 0..3 {
            let id = 14 + category as u8;
            let entry = get(id).unwrap();
            assert_eq!(entry.blueprint, Blueprint::RoyalVictory);
            assert_eq!(entry.category, category);
            assert_eq!(entry.compatible_ships, ROYAL_SHIPS);
            assert_eq!(effective_rank(id, 5, ROYAL_SHIP, 1), 5);
            assert_eq!(effective_rank(id, 0, ROYAL_SHIP, 1), 0);
            for ship in 0..ROYAL_SHIP {
                assert!(!compatible(id, ship));
                assert_eq!(effective_rank(id, 5, ship, 5), 0);
            }
        }
    }

    #[test]
    fn recipes_charge_only_the_exact_rank_tier_and_roots_onboarding_cost() {
        for id in 0..14 {
            for rank in 1..=MAX_RANK {
                let cost = recipe(id, rank).unwrap();
                assert_eq!(cost.points, if id < 3 && rank == 1 { 3 } else { 10 });
                for tier in 0..5 {
                    assert_eq!(
                        cost.materials[tier],
                        if tier + 1 == rank as usize { 2 } else { 0 }
                    );
                }
                assert_eq!(cost.materials.iter().sum::<u32>(), 2);
            }
            assert_eq!(recipe(id, 0), None);
            assert_eq!(recipe(id, 6), None);
            assert_eq!(recipe(id, u8::MAX), None);
        }
        for id in [14, 15, 16, 17, u8::MAX] {
            assert_eq!(recipe(id, 1), None);
            assert_eq!(recipe(id, 5), None);
        }
    }

    #[test]
    fn every_registered_effect_is_distinct_and_described() {
        for (i, entry) in ABILITIES.iter().enumerate() {
            assert!(!entry.name.is_empty());
            assert!(!entry.description.is_empty());
            assert!(
                ABILITIES[..i]
                    .iter()
                    .all(|other| other.effect != entry.effect)
            );
        }
    }
}
