//! Dock-owned ability inventory, ship-local hardware and explicit cargo lifecycle.
use crate::{
    abilities,
    campaign::{AbilityState, ShipLoadout},
    *,
};
use std::sync::LazyLock;

pub(super) fn material_color(tier: u8) -> u32 {
    let [r, g, b] = [
        [217, 165, 119],
        [113, 178, 204],
        [181, 145, 209],
        [231, 208, 122],
        [183, 229, 197],
    ][tier.clamp(1, 5) as usize - 1];
    rgba(r, g, b, 255)
}
static MATERIAL_PICKUPS: LazyLock<[PixelBody; 5]> = LazyLock::new(|| {
    std::array::from_fn(|tier| {
        let shapes = [
            [
                "    x    ",
                "   xxx   ",
                "  xoxxx  ",
                "   xxx   ",
                "    x    ",
            ],
            [
                "  x   x  ",
                " xxx xxx ",
                " xox xox ",
                " xxx xxx ",
                "  x   x  ",
            ],
            [
                "    x    ",
                "   xxx   ",
                " xxoxxxx ",
                " xxxxxxx ",
                "  x   x  ",
            ],
            [
                "  xxxxx  ",
                " xx   xx ",
                " xo x xx ",
                " xx   xx ",
                "  xxxxx  ",
            ],
            [
                "x x x x x",
                " xxxxxxx ",
                "  xoxxx  ",
                " xxxxxxx ",
                "  xxxxx  ",
            ],
        ];
        let mut rows: Vec<String> = shapes[tier].iter().map(|s| (*s).into()).collect();
        rows.push("         ".into());
        rows.push(
            (0..9)
                .map(|x| {
                    if x % 2 == 0 && x / 2 <= tier {
                        'p'
                    } else {
                        ' '
                    }
                })
                .collect(),
        );
        let borrowed: Vec<_> = rows.iter().map(String::as_str).collect();
        PixelBody::from_ascii(
            &borrowed,
            &[
                ('x', material_color(tier as u8 + 1)),
                ('o', rgba(238, 242, 225, 255)),
                ('p', rgba(210, 215, 203, 255)),
            ],
        )
    })
});
pub(super) fn draw_material(frame: &mut FrameBuilder, position: Vec2, tier: u8) {
    frame.push_body(
        &MATERIAL_PICKUPS[tier.clamp(1, 5) as usize - 1],
        Transform {
            position,
            rotation: 0.0,
            scale: Vec2::splat(1.15),
        },
    );
}

#[derive(Clone, Debug, Serialize)]
pub struct AbilityHud {
    pub id: String,
    pub index: u8,
    pub name: String,
    pub category: usize,
    pub description: String,
    pub blueprint: bool,
    pub owned: bool,
    pub rank: u8,
    pub effective_rank: u8,
    pub max_rank: u8,
    pub compatible: bool,
    pub equipped: bool,
    pub required_level: u32,
    pub material_tier: u8,
    pub material_cost: u32,
    pub point_cost: u32,
    pub can_craft: bool,
    pub can_upgrade: bool,
    pub can_equip: bool,
    pub craft_reason: String,
    pub upgrade_reason: String,
    pub equip_reason: String,
    pub training_recipe: bool,
    pub required_training_material: Option<TrainingMaterialHud>,
}
#[derive(Clone, Debug, Serialize)]
pub struct TrainingMaterialHud {
    pub id: String,
    pub name: String,
    pub amount: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct EquippedAbilityHud {
    pub id: String,
    pub index: u8,
    pub name: String,
    pub category: usize,
    pub rank: u8,
    pub effective_rank: u8,
}

pub(super) fn total(materials: [u32; 5]) -> u32 {
    materials.into_iter().fold(0, u32::saturating_add)
}

/// Owned hardware survives migration; only its active strength follows this hull's XP.
pub(super) fn hardware_ranks(p: &Profile) -> [u8; 6] {
    let ship = p.campaign.selected_ship as usize;
    if ship == 9 {
        return [5; 6]; // Explicit fully upgraded Sovereign chassis reward.
    }
    let cap = campaign::rank_cap(campaign::level(p.campaign.ship_xp[ship]));
    p.campaign.loadouts[ship].hardware.map(|rank| rank.min(cap))
}

pub(super) fn equipped_rank(p: &Profile, id: u8) -> u8 {
    let Some(def) = abilities::get(id) else {
        return 0;
    };
    let ship = p.campaign.selected_ship;
    if ship > 9 || p.campaign.loadouts[ship as usize].slots[def.category] != Some(id) {
        return 0;
    }
    effective_inventory_rank(p, id)
}
pub(super) fn effective_inventory_rank(p: &Profile, id: u8) -> u8 {
    let ship = p.campaign.selected_ship;
    let cap = campaign::rank_cap(campaign::level(p.campaign.ship_xp[ship as usize]));
    abilities::effective_rank(
        id,
        p.campaign.abilities[id as usize].rank,
        ship,
        if training::trained_starter(p, id) {
            cap.max(2)
        } else {
            cap
        },
    )
}

pub(super) fn family_rank(p: &Profile, family: usize) -> u8 {
    if family >= 3 {
        return 0;
    }
    equipped_rank(p, (family + 3) as u8).max(equipped_rank(p, (family + 14) as u8))
}

pub(super) fn mirrors(p: &mut Profile) {
    let ship = p.campaign.selected_ship.min(9) as usize;
    p.ranks[..6].copy_from_slice(&p.campaign.loadouts[ship].hardware);
    for family in 0..3 {
        p.ranks[family + 6] = p.campaign.abilities[family + 3].rank;
        p.unlocked[family] = p.campaign.abilities[family + 3].blueprint;
    }
}

/// Constructor boundary only. Save validation must retain live flight cargo.
pub(super) fn initialize(p: &mut Profile, legacy: bool) -> ([u32; 5], bool) {
    let before = serde_json::to_value(&*p).expect("profile is serializable");
    let mut loss = [0; 5];
    if p.schema_version < 6 {
        // No previous schema recorded a material's named birth variety.
        p.campaign.cargo_training_materials = [0; 3];
        p.campaign.stored_training_materials = [0; 3];
    }
    if legacy {
        p.campaign.stored_materials[0] =
            total(p.campaign.stored_materials).saturating_add(total(p.campaign.cargo_materials));
        p.campaign.stored_materials[1..].fill(0);
        p.campaign.cargo_materials = [0; 5];
        p.campaign.cargo_training_materials = [0; 3];
        p.campaign.loadouts = [ShipLoadout::default(); 10];
        p.campaign.loadouts[p.campaign.selected_ship as usize]
            .hardware
            .copy_from_slice(&p.ranks[..6]);
        p.campaign.abilities = [AbilityState::default(); abilities::COUNT];
        for family in 0..3 {
            p.campaign.abilities[family + 3] = AbilityState {
                blueprint: p.unlocked[family],
                rank: if p.unlocked[family] {
                    p.ranks[family + 6].clamp(1, 5)
                } else {
                    0
                },
            };
        }
        for champion in 0..8 {
            p.campaign.abilities[champion + 6] = AbilityState {
                blueprint: p.campaign.patches[champion],
                rank: u8::from(p.campaign.crafted[champion]),
            };
        }
        for ship in 0..9u8 {
            if !p.campaign.owned(ship) {
                continue;
            }
            for family in 0..3 {
                if p.campaign.abilities[family + 3].rank > 0 {
                    p.campaign.loadouts[ship as usize].slots[family] = Some((family + 3) as u8);
                }
            }
            if ship > 0 {
                let id = ship + 5;
                let category = abilities::get(id).expect("native ability").category;
                p.campaign.loadouts[ship as usize].slots[category] = Some(id);
            }
        }
        if p.campaign.victory_claimed {
            grant_royal(p);
        }
    } else {
        loss = p.campaign.cargo_materials;
        p.campaign.cargo_materials = [0; 5];
        p.campaign.cargo_training_materials = [0; 3];
    }
    for (index, def) in abilities::ABILITIES.iter().enumerate() {
        let state = &mut p.campaign.abilities[index];
        state.rank = state.rank.min(5);
        match def.blueprint {
            abilities::Blueprint::Starter => {
                // Anonymous legacy pilots retain the three existing defaults.
                // A registered Kikai discovers other basics through real pickups.
                state.blueprint |= p.identity.is_none() || state.rank > 0;
            }
            abilities::Blueprint::Champion(i) => state.blueprint = p.campaign.patches[i],
            abilities::Blueprint::RoyalVictory => {
                state.blueprint = p.campaign.victory_claimed;
                state.rank = if state.blueprint { 5 } else { 0 };
            }
            abilities::Blueprint::BossFamily(_) => {}
        }
        if !state.blueprint {
            state.rank = 0;
        }
    }
    for ship in 0..10u8 {
        if !p.campaign.owned(ship) {
            p.campaign.loadouts[ship as usize] = ShipLoadout::default();
            continue;
        }
        for rank in &mut p.campaign.loadouts[ship as usize].hardware {
            *rank = (*rank).min(5);
        }
        for category in 0..3 {
            let slot = p.campaign.loadouts[ship as usize].slots[category];
            if slot.is_some_and(|id| {
                abilities::get(id).is_none_or(|def| {
                    def.category != category
                        || !abilities::compatible(id, ship)
                        || p.campaign.abilities[id as usize].rank == 0
                })
            }) {
                p.campaign.loadouts[ship as usize].slots[category] = None;
            }
        }
    }
    mirrors(p);
    let changed = serde_json::to_value(&*p).expect("profile is serializable") != before;
    (loss, changed)
}

pub(super) fn grant_royal(p: &mut Profile) {
    for id in 14..17 {
        p.campaign.abilities[id] = AbilityState {
            blueprint: true,
            rank: 5,
        };
    }
    p.campaign.loadouts[9].slots = [Some(14), Some(15), Some(16)];
}

impl Game {
    pub(super) fn ship_craft_check(&self, index: usize) -> Result<(), String> {
        self.require_home()?;
        let champion = campaign::CHAMPIONS
            .get(index)
            .ok_or("Unknown ship blueprint.")?;
        if !self.profile.campaign.blueprints[index] {
            return Err("Defeat this champion to earn its ship blueprint.".into());
        }
        if self.profile.campaign.crafted[index] {
            return Err("This ship is already in your hangar.".into());
        }
        self.check_recipe(champion.category, 10, [6, 0, 0, 0, 0])
    }
    pub(super) fn ship_select_check(&self, id: u8) -> Result<(), String> {
        self.require_home()?;
        if !self.profile.campaign.owned(id) {
            return Err("This ship is not in your hangar.".into());
        }
        Ok(())
    }
    pub(super) fn ship_hud_details(&self, ship: &mut campaign::ShipHud) {
        ship.family = if ship.id == 9 {
            3
        } else {
            abilities::SHIP_FAMILIES[ship.id as usize]
        };
        ship.xp = self.profile.campaign.ship_xp[ship.id as usize];
        ship.level = campaign::level(ship.xp);
        ship.next_xp = campaign::next_xp(ship.level);
        ship.rank_cap = campaign::rank_cap(ship.level);
        ship.point_cost = if (1..=8).contains(&ship.id) { 10 } else { 0 };
        ship.material_cost = if (1..=8).contains(&ship.id) { 6 } else { 0 };
        ship.material_tier = 1;
        let craft = if (1..=8).contains(&ship.id) {
            self.ship_craft_check(ship.id as usize - 1)
        } else if ship.id == 9 {
            Err("Win the Galaxy Royale tournament to unlock the Sovereign.".into())
        } else {
            Err("The Haven Scout is already provided by the dock.".into())
        };
        let select = self.ship_select_check(ship.id);
        ship.can_craft = craft.is_ok();
        ship.can_select = select.is_ok();
        ship.craft_reason = craft.err().unwrap_or_default();
        ship.select_reason = select.err().unwrap_or_default();
    }
    pub(super) fn require_home(&self) -> Result<(), String> {
        if self.position().length() > SAFE_RADIUS || self.arena.is_some() {
            Err("Return physically to Haven to use dock inventory.".into())
        } else {
            Ok(())
        }
    }
    pub(super) fn discard_cargo(&mut self, reason: &str) {
        let lost = self.profile.campaign.cargo_materials;
        if total(lost) == 0 {
            return;
        }
        self.profile.campaign.cargo_materials = [0; 5];
        self.profile.campaign.cargo_training_materials = [0; 3];
        self.material_loss = lost;
        self.material_loss_reason = reason.into();
        if let Some(run) = &mut self.run {
            for (i, amount) in lost.into_iter().enumerate() {
                run.materials_lost[i] = run.materials_lost[i].saturating_add(amount);
            }
        }
        self.changed();
    }
    fn check_recipe(
        &self,
        category: usize,
        points: u32,
        materials: [u32; 5],
    ) -> Result<(), String> {
        self.require_home()?;
        if self.profile.currency[category] < points {
            return Err(format!("Requires {points} matching category points."));
        }
        for (tier, cost) in materials.into_iter().enumerate() {
            if self.profile.campaign.stored_materials[tier] < cost {
                return Err(format!(
                    "Requires {cost} banked tier-{} materials.",
                    tier + 1
                ));
            }
        }
        Ok(())
    }
    fn spend_recipe(&mut self, category: usize, points: u32, materials: [u32; 5]) {
        self.profile.currency[category] -= points;
        self.consume_stored_materials(materials);
    }
    fn ability_check(&self, id: u8, upgrade: bool) -> Result<abilities::Recipe, String> {
        self.require_home()?;
        let def = abilities::get(id).ok_or("Unknown ability.")?;
        if !abilities::compatible(id, self.profile.campaign.selected_ship) {
            return Err(
                "This ability is incompatible with the active ship's chassis family.".into(),
            );
        }
        let state = self.profile.campaign.abilities[id as usize];
        if !state.blueprint {
            return Err("Discover this ability's blueprint first.".into());
        }
        if upgrade && state.rank == 0 {
            return Err("Craft this ability before upgrading it.".into());
        }
        if !upgrade && state.rank > 0 {
            return Err("This shared inventory ability is already crafted.".into());
        }
        let next = if upgrade {
            state.rank.saturating_add(1)
        } else {
            1
        };
        if upgrade && self.training_recipe(id) {
            self.training_upgrade_check(id)?;
            return Ok(abilities::Recipe {
                points: 0,
                materials: [10, 0, 0, 0, 0],
            });
        }
        let recipe = abilities::recipe(id, next)
            .ok_or("This ability is at maximum rank or is a Royale reward.")?;
        let level = campaign::required_level(next);
        if self.ship_level() < level {
            return Err(format!("Reach active ship level {level} for this rank."));
        }
        self.check_recipe(def.category, recipe.points, recipe.materials)?;
        Ok(recipe)
    }
    fn equip_check(&self, category: usize, id: Option<u8>) -> Result<(), String> {
        self.require_home()?;
        if category >= 3 {
            return Err("Ability slot must be offense, defense or passive.".into());
        }
        if let Some(id) = id {
            let def = abilities::get(id).ok_or("Unknown ability.")?;
            if def.category != category {
                return Err("This ability belongs to a different slot category.".into());
            }
            if self.profile.campaign.abilities[id as usize].rank == 0 {
                return Err("Craft this ability before equipping it.".into());
            }
            if !abilities::compatible(id, self.profile.campaign.selected_ship) {
                return Err(
                    "This ability is incompatible with the active ship's chassis family.".into(),
                );
            }
        }
        Ok(())
    }
    pub fn craft_ability(&mut self, key: &str) -> Result<(), String> {
        let id = abilities::by_key(key).ok_or("Unknown ability.")?;
        let recipe = self.ability_check(id, false)?;
        self.spend_recipe(
            abilities::get(id).unwrap().category,
            recipe.points,
            recipe.materials,
        );
        self.profile.campaign.abilities[id as usize].rank = 1;
        self.progression_changed();
        if self.additional_basic(id) {
            self.onboarding_fact(identity::ADDITIONAL_CRAFTED);
        }
        Ok(())
    }
    pub fn upgrade_ability(&mut self, key: &str) -> Result<(), String> {
        let id = abilities::by_key(key).ok_or("Unknown ability.")?;
        let recipe = self.ability_check(id, true)?;
        if self.training_recipe(id) {
            let material = training::material_for_corporation(
                self.profile.identity.as_ref().unwrap().corporation,
            );
            self.profile.campaign.stored_training_materials[material.index] -= 10;
            self.profile.campaign.stored_materials[0] -= 10;
        } else {
            self.spend_recipe(
                abilities::get(id).unwrap().category,
                recipe.points,
                recipe.materials,
            );
        }
        self.profile.campaign.abilities[id as usize].rank += 1;
        self.training_recognize_upgrade();
        self.progression_changed();
        Ok(())
    }
    pub fn equip_ability(&mut self, category: usize, key: Option<&str>) -> Result<(), String> {
        let id = key
            .map(|key| abilities::by_key(key).ok_or("Unknown ability."))
            .transpose()?;
        self.equip_check(category, id)?;
        let ship = self.profile.campaign.selected_ship as usize;
        if self.profile.campaign.loadouts[ship].slots[category] != id {
            self.profile.campaign.loadouts[ship].slots[category] = id;
            self.wake.clear();
            self.wake_timer = 0.0;
            self.repulsor_timer = 0.0;
            self.shot_count = 0;
            self.progression_changed();
        }
        if id.is_some_and(|id| self.additional_basic(id)) {
            self.onboarding_fact(identity::ADDITIONAL_EQUIPPED);
        }
        self.training_recognize_upgrade();
        Ok(())
    }
    pub(super) fn progression_changed(&mut self) {
        mirrors(&mut self.profile);
        self.changed();
        let stats = Stats::from_profile(&self.profile);
        if let Ok(mut actor) = self.world.get::<&mut Actor>(self.player) {
            actor.max_health = stats.hull;
            actor.health = stats.hull;
        }
        self.shield = stats.shield;
        self.event("upgrade", self.position(), 0.7);
    }
    pub(super) fn hardware_check(&self, index: usize) -> Result<(), String> {
        self.require_home()?;
        if self.profile.campaign.selected_ship == 9 {
            return Err("Sovereign chassis hardware is already fully upgraded.".into());
        }
        if index >= 6 {
            return Err("Automatic abilities are crafted in the dock inventory.".into());
        }
        let rank = self.profile.campaign.loadouts[self.profile.campaign.selected_ship as usize]
            .hardware[index];
        if rank >= 5 {
            return Err("This hardware is already at maximum rank.".into());
        }
        let next = rank + 1;
        let level = campaign::required_level(next);
        if self.ship_level() < level {
            return Err(format!(
                "Reach active ship level {level} before installing this rank."
            ));
        }
        let mut materials = [0; 5];
        materials[next as usize - 1] = 1;
        self.check_recipe(UPGRADES[index].2, 10, materials)
    }
    pub(super) fn ability_hud(&self) -> Vec<AbilityHud> {
        abilities::ABILITIES
            .iter()
            .enumerate()
            .filter(|(i, _)| {
                let state = self.profile.campaign.abilities[*i];
                state.blueprint || state.rank > 0
            })
            .map(|(i, def)| {
                let id = i as u8;
                let state = self.profile.campaign.abilities[i];
                let next = if state.rank == 0 {
                    1
                } else {
                    state.rank.saturating_add(1).min(5)
                };
                let training_recipe = self.training_recipe(id);
                let recipe = if training_recipe {
                    Some(abilities::Recipe {
                        points: 0,
                        materials: [10, 0, 0, 0, 0],
                    })
                } else {
                    abilities::recipe(id, next)
                };
                let material = training_recipe.then(|| {
                    training::material_for_corporation(
                        self.profile.identity.as_ref().unwrap().corporation,
                    )
                });
                let craft = self
                    .ability_check(id, false)
                    .map(|_| ())
                    .err()
                    .unwrap_or_default();
                let upgrade = self
                    .ability_check(id, true)
                    .map(|_| ())
                    .err()
                    .unwrap_or_default();
                let equip = self
                    .equip_check(def.category, Some(id))
                    .err()
                    .unwrap_or_default();
                AbilityHud {
                    id: def.key.into(),
                    index: id,
                    name: def.name.into(),
                    category: def.category,
                    description: def.description.into(),
                    blueprint: state.blueprint,
                    owned: state.rank > 0,
                    rank: state.rank,
                    effective_rank: effective_inventory_rank(&self.profile, id),
                    max_rank: 5,
                    compatible: abilities::compatible(id, self.profile.campaign.selected_ship),
                    equipped: self.profile.campaign.loadouts
                        [self.profile.campaign.selected_ship as usize]
                        .slots[def.category]
                        == Some(id),
                    required_level: if training_recipe {
                        1
                    } else {
                        campaign::required_level(next)
                    },
                    material_tier: if training_recipe { 1 } else { next },
                    material_cost: recipe.map_or(0, |r| total(r.materials)),
                    point_cost: recipe.map_or(0, |r| r.points),
                    can_craft: craft.is_empty(),
                    can_upgrade: upgrade.is_empty(),
                    can_equip: equip.is_empty(),
                    craft_reason: craft,
                    upgrade_reason: upgrade,
                    equip_reason: equip,
                    training_recipe,
                    required_training_material: material.map(|m| TrainingMaterialHud {
                        id: m.id.into(),
                        name: m.name.into(),
                        amount: 10,
                    }),
                }
            })
            .collect()
    }
    pub(super) fn loadout_hud(&self) -> [Option<EquippedAbilityHud>; 3] {
        self.profile.campaign.loadouts[self.profile.campaign.selected_ship as usize]
            .slots
            .map(|slot| {
                slot.and_then(|id| {
                    let def = abilities::get(id)?;
                    Some(EquippedAbilityHud {
                        id: def.key.into(),
                        index: id,
                        name: def.name.into(),
                        category: def.category,
                        rank: self.profile.campaign.abilities[id as usize].rank,
                        effective_rank: equipped_rank(&self.profile, id),
                    })
                })
            })
    }
}
