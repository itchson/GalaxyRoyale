//! Finite dock-crafted parts. These are owned inventory, never unbanked cargo.
use crate::{Game, Profile, abilities};
use serde::{Deserialize, Deserializer, Serialize, de::Error};

pub const COUNT: usize = 3;
pub const MAX_COUNT: u32 = 99;
pub const SCHEMA_VERSION: u32 = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComponentDef {
    pub id: &'static str,
    pub name: &'static str,
    pub category: usize,
    pub description: &'static str,
    pub required_level: u32,
    pub materials: [u32; 5],
}
pub const COMPONENTS: [ComponentDef; COUNT] = [
    ComponentDef {
        id: "pulse-conductor",
        name: "Pulse Conductor",
        category: 0,
        description: "A crafted firing circuit for advanced offensive ability ranks.",
        required_level: 3,
        materials: [2, 1, 0, 0, 0],
    },
    ComponentDef {
        id: "aegis-lattice",
        name: "Aegis Lattice",
        category: 1,
        description: "A crafted protective matrix for advanced defensive ability ranks.",
        required_level: 3,
        materials: [2, 1, 0, 0, 0],
    },
    ComponentDef {
        id: "flux-bearing",
        name: "Flux Bearing",
        category: 2,
        description: "A crafted drive coupling for advanced passive ability ranks.",
        required_level: 3,
        materials: [2, 1, 0, 0, 0],
    },
];
pub fn by_key(key: &str) -> Option<usize> {
    COMPONENTS.iter().position(|entry| entry.id == key)
}
pub fn discovered(profile: &Profile, index: usize) -> bool {
    let Some(def) = COMPONENTS.get(index) else {
        return false;
    };
    profile.campaign.components[index] > 0
        || abilities::ABILITIES
            .iter()
            .enumerate()
            .any(|(id, ability)| {
                let state = profile.campaign.abilities[id];
                ability.category == def.category
                    && !matches!(ability.blueprint, abilities::Blueprint::RoyalVictory)
                    && (state.rank >= 2
                        || (state.blueprint
                            && !matches!(ability.blueprint, abilities::Blueprint::Starter)))
            })
}
/// Only ordinary target ranks 3..5 consume components. Royal rewards and first
/// starter training are outside this recipe; existing ranks are never rewritten.
pub fn requirement(id: u8, next_rank: u8) -> Option<(usize, u32)> {
    let def = abilities::get(id)?;
    if !(3..=5).contains(&next_rank) || matches!(def.blueprint, abilities::Blueprint::RoyalVictory)
    {
        return None;
    }
    Some((def.category, u32::from(next_rank - 2)))
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ComponentHud {
    pub id: String,
    pub index: u8,
    pub name: String,
    pub category: usize,
    pub description: String,
    pub count: u32,
    pub max_count: u32,
    pub required_level: u32,
    pub materials: [u32; 5],
    pub can_craft: bool,
    pub craft_reason: String,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ComponentRequirementHud {
    pub id: String,
    pub name: String,
    pub amount: u32,
    pub owned: u32,
}

// Validate the version before CampaignProgress's legacy/default deserialization.
// This explicit field list preserves old missing-field defaults while refusing
// schema7 missing/invalid component stacks and ignoring forged older stacks.
impl<'de> Deserialize<'de> for Profile {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut value = serde_json::Value::deserialize(deserializer)?;
        let fields = value
            .as_object_mut()
            .ok_or_else(|| D::Error::custom("Profile must be an object."))?;
        let version: u32 = fields
            .get("schema_version")
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()
            .map_err(D::Error::custom)?
            .unwrap_or(0);
        if version > SCHEMA_VERSION {
            return Err(D::Error::custom("Unsupported profile schema."));
        }
        if version == SCHEMA_VERSION {
            let raw = fields
                .get("campaign")
                .and_then(|v| v.get("components"))
                .ok_or_else(|| D::Error::custom("Schema 7 requires campaign.components."))?;
            let counts: [u32; COUNT] =
                serde_json::from_value(raw.clone()).map_err(D::Error::custom)?;
            if counts.iter().any(|&count| count > MAX_COUNT) {
                return Err(D::Error::custom("Component count exceeds 99."));
            }
        } else if let Some(campaign) = fields
            .get_mut("campaign")
            .and_then(serde_json::Value::as_object_mut)
        {
            campaign.remove("components");
        }
        let mut profile = Profile {
            schema_version: version,
            ..Profile::default()
        };
        macro_rules! field {
            ($($name:ident),* $(,)?) => { $(if let Some(value) = fields.remove(stringify!($name)) { profile.$name = serde_json::from_value(value).map_err(D::Error::custom)?; })* };
        }
        field!(
            schema_version,
            revision,
            seed,
            currency,
            ranks,
            unlocked,
            kills,
            boss_kills,
            boss_generations,
            pity,
            pending,
            discoveries,
            quests,
            relics,
            campaign,
            identity,
            onboarding
        );
        Ok(profile)
    }
}

impl Game {
    fn component_check(&self, index: usize) -> Result<(), String> {
        self.require_home()?;
        if !self.progression_enabled() {
            return Err("Return to normal flight before crafting components.".into());
        }
        let def = COMPONENTS.get(index).ok_or("Unknown component.")?;
        if !discovered(&self.profile, index) {
            return Err(
                "Discover a matching advanced blueprint or own a rank 2 ability first.".into(),
            );
        }
        if self.ship_level() < def.required_level {
            return Err(format!(
                "Reach active RAICHO level {} to craft this component.",
                def.required_level
            ));
        }
        if self.profile.campaign.components[index] >= MAX_COUNT {
            return Err("This component stack is full (99).".into());
        }
        for (tier, cost) in def.materials.into_iter().enumerate() {
            if self.profile.campaign.stored_materials[tier] < cost {
                return Err(format!(
                    "Requires {cost} banked tier-{} materials.",
                    tier + 1
                ));
            }
        }
        Ok(())
    }
    pub fn craft_component(&mut self, key: &str) -> Result<(), String> {
        let index = by_key(key).ok_or("Unknown component.")?;
        self.component_check(index)?;
        self.consume_stored_materials(COMPONENTS[index].materials);
        self.profile.campaign.components[index] += 1;
        self.changed();
        self.event("upgrade", self.position(), 0.5);
        Ok(())
    }
    pub(super) fn component_hud(&self) -> Vec<ComponentHud> {
        COMPONENTS
            .iter()
            .enumerate()
            .filter(|(index, _)| discovered(&self.profile, *index))
            .map(|(index, def)| {
                let check = self.component_check(index);
                ComponentHud {
                    id: def.id.into(),
                    index: index as u8,
                    name: def.name.into(),
                    category: def.category,
                    description: def.description.into(),
                    count: self.profile.campaign.components[index],
                    max_count: MAX_COUNT,
                    required_level: def.required_level,
                    materials: def.materials,
                    can_craft: check.is_ok(),
                    craft_reason: check.err().unwrap_or_default(),
                }
            })
            .collect()
    }
    pub(super) fn component_requirement(
        &self,
        id: u8,
        next: u8,
    ) -> Option<ComponentRequirementHud> {
        requirement(id, next).map(|(index, amount)| ComponentRequirementHud {
            id: COMPONENTS[index].id.into(),
            name: COMPONENTS[index].name.into(),
            amount,
            owned: self.profile.campaign.components[index],
        })
    }
    pub(super) fn check_ability_components(&self, id: u8, next: u8) -> Result<(), String> {
        if let Some((index, amount)) = requirement(id, next) {
            if !self.progression_enabled() {
                return Err("Return to normal flight before spending components.".into());
            }
            if self.profile.campaign.components[index] < amount {
                return Err(format!(
                    "Requires {amount} crafted {} at the dock.",
                    COMPONENTS[index].name
                ));
            }
        }
        Ok(())
    }
    pub(super) fn spend_ability_components(&mut self, id: u8, next: u8) {
        if let Some((index, amount)) = requirement(id, next) {
            self.profile.campaign.components[index] -= amount;
        }
    }
}
