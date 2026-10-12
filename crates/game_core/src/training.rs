//! Real counted starter training and labeled subsets of the existing T1 stock.
//! Labels never add capacity, rewards, XP, or a second copy of a material.
use crate::{identity::*, *};
use std::sync::LazyLock;

pub(super) struct TrainingMaterial {
    pub index: usize,
    pub id: &'static str,
    pub name: &'static str,
    pub asteroid: &'static str,
}
pub(super) const MATERIALS: [TrainingMaterial; 3] = [
    TrainingMaterial {
        index: 0,
        id: "phase-silicate",
        name: "Phase Silicate",
        asteroid: "Porous asteroid",
    },
    TrainingMaterial {
        index: 1,
        id: "iron-filament",
        name: "Iron Filament",
        asteroid: "Metal asteroid",
    },
    TrainingMaterial {
        index: 2,
        id: "coolant-crystal",
        name: "Coolant KESSHO",
        asteroid: "Ice asteroid",
    },
];
pub(super) fn material_for_corporation(corp: Corporation) -> &'static TrainingMaterial {
    &MATERIALS[match corp {
        Corporation::Pan => 0,
        Corporation::Renko => 1,
        Corporation::Hugo => 2,
    }]
}
pub(super) fn material_for_variety(variety: u8) -> Option<u8> {
    match variety {
        3 => Some(0),
        1 => Some(1),
        2 => Some(2),
        _ => None,
    }
}
pub(super) fn clamp_labels(total: u32, labels: &mut [u32; 3]) {
    let mut remaining = total;
    for label in labels {
        *label = (*label).min(remaining);
        remaining -= *label;
    }
}
pub(super) fn label_total(labels: [u32; 3]) -> u32 {
    labels.into_iter().fold(0, u32::saturating_add)
}
/// Generic T1 recipes spend unlabeled stock first, then labels in stable order.
pub(super) fn consume_labels(total_before: u32, amount: u32, labels: &mut [u32; 3]) {
    let unlabeled = total_before.saturating_sub(label_total(*labels));
    let mut debit = amount.saturating_sub(unlabeled);
    for label in labels {
        let taken = (*label).min(debit);
        *label -= taken;
        debit -= taken;
    }
}
/// Banking moves named units first, retaining all untransferred labels in cargo.
pub(super) fn transfer_labels(amount: u32, cargo: &mut [u32; 3], stash: &mut [u32; 3]) {
    let mut available = amount;
    for i in 0..3 {
        let banked = cargo[i].min(available);
        cargo[i] -= banked;
        stash[i] += banked; // Existing T1 total bounds the sum and this addition.
        available -= banked;
    }
}

pub(super) fn sanitize_progress(profile: &mut Profile) {
    let Some(p) = &mut profile.onboarding else {
        return;
    };
    if p.version != 2 {
        let completed = p.version == 1 && p.flags & 127 == 127;
        *p = OnboardingProgress {
            flags: if completed {
                127
            } else {
                p.flags & (LICENSE | DEPARTED)
            },
            legacy_complete: completed,
            ..Default::default()
        };
    }
    if p.legacy_complete
        && p.flags == 127
        && p.asteroids_destroyed == 0
        && p.materials_collected == 0
    {
        p.stage = 6;
        return;
    }
    p.legacy_complete = false;
    p.asteroids_destroyed = p.asteroids_destroyed.min(10);
    p.materials_collected = if p.asteroids_destroyed == 10 {
        p.materials_collected.min(10)
    } else {
        0
    };
    p.flags &= 127;
    p.flags &= !(BELT_DESTROYED | T1_COLLECTED);
    if p.asteroids_destroyed == 10 {
        p.flags |= BELT_DESTROYED;
    }
    if p.materials_collected == 10 {
        p.flags |= T1_COLLECTED;
    }
    if p.flags & T1_COLLECTED == 0 {
        p.flags &= !T1_BANKED;
    }
    let upgraded = profile
        .identity
        .as_ref()
        .is_some_and(|id| profile.campaign.abilities[id.corporation.ability() as usize].rank >= 2);
    if p.flags & T1_BANKED == 0 || p.flags & 96 != 96 || !upgraded {
        p.flags &= !96;
    }
    p.stage = stage(p.flags);
}
pub(super) fn trained_starter(profile: &Profile, id: u8) -> bool {
    profile.campaign.selected_ship == 0
        && profile
            .onboarding
            .as_ref()
            .is_some_and(|p| p.version == 2 && !p.legacy_complete && p.stage == 6)
        && profile
            .identity
            .as_ref()
            .is_some_and(|identity| identity.corporation.ability() == id)
        && abilities::get(id)
            .is_some_and(|def| profile.campaign.loadouts[0].slots[def.category] == Some(id))
}

static PICKUPS: LazyLock<[PixelBody; 3]> = LazyLock::new(|| {
    let rows = [
        [
            "    g    ",
            "  ggggg  ",
            " ggogggg ",
            "  ggggg  ",
            "   g g   ",
        ],
        [
            " gggggg  ",
            " gg   gg ",
            " goggggg ",
            " gg   gg ",
            "  gggggg ",
        ],
        [
            "    g    ",
            "   ggg   ",
            "  ggogg  ",
            " ggggggg ",
            "   ggg   ",
        ],
    ];
    std::array::from_fn(|i| {
        PixelBody::from_ascii(
            &rows[i],
            &[
                (
                    'g',
                    rgba(
                        [109, 225, 133][i],
                        [225, 145, 180][i],
                        [133, 119, 244][i],
                        255,
                    ),
                ),
                ('o', rgba(245, 243, 222, 255)),
            ],
        )
    })
});
pub(super) fn draw_material(frame: &mut FrameBuilder, position: Vec2, index: u8) {
    if let Some(body) = PICKUPS.get(index as usize) {
        frame.push_body(
            body,
            Transform {
                position,
                rotation: 0.0,
                scale: Vec2::splat(1.15),
            },
        );
    }
}

impl Game {
    pub(super) fn training_recipe(&self, id: u8) -> bool {
        self.profile.campaign.selected_ship == 0
            && self
                .profile
                .identity
                .as_ref()
                .is_some_and(|i| i.corporation.ability() == id)
            && self
                .profile
                .onboarding
                .as_ref()
                .is_some_and(|p| p.version == 2 && !p.legacy_complete && p.stage < 6)
            && self.profile.campaign.abilities[id as usize].rank == 1
    }
    pub(super) fn training_upgrade_check(&self, id: u8) -> Result<(), String> {
        self.require_home()?;
        if !self.progression_enabled() {
            return Err("Return to normal flight before completing training.".into());
        }
        if !self.training_recipe(id) {
            return Err("The first starter training upgrade is already completed or unavailable on this ship.".into());
        }
        let p = self.profile.onboarding.as_ref().unwrap();
        if p.asteroids_destroyed < 10 {
            return Err("Destroy 10 asteroids of any variety first.".into());
        }
        if p.materials_collected < 10 {
            return Err("Collect 10 matching named materials after the asteroid mission.".into());
        }
        if p.flags & T1_BANKED == 0 {
            return Err("Physically bank 10 matching materials at Haven.".into());
        }
        let corp = self.profile.identity.as_ref().unwrap().corporation;
        let material = material_for_corporation(corp);
        if self.profile.campaign.stored_training_materials[material.index] < 10 {
            return Err(format!(
                "Requires 10 banked {}. Recollect any materials lost or spent.",
                material.name
            ));
        }
        let category = abilities::get(id).unwrap().category;
        if self.profile.campaign.loadouts[0].slots[category] != Some(id) {
            return Err(
                "Equip your corporation's starter in its matching slot on the starter ship.".into(),
            );
        }
        Ok(())
    }
    pub(super) fn training_asteroid_defeated(&mut self) {
        if !self.progression_enabled() {
            return;
        }
        if let Some(p) = &mut self.profile.onboarding
            && !p.legacy_complete
            && p.asteroids_destroyed < 10
        {
            p.asteroids_destroyed += 1;
            if p.asteroids_destroyed == 10 {
                p.flags |= BELT_DESTROYED;
            }
            p.stage = stage(p.flags);
            self.changed();
        }
    }
    pub(super) fn training_collected(&mut self, index: u8, amount: u32) {
        if !self.progression_enabled() {
            return;
        }
        let matching = self
            .profile
            .identity
            .as_ref()
            .is_some_and(|id| material_for_corporation(id.corporation).index == index as usize);
        if matching
            && let Some(p) = &mut self.profile.onboarding
            && !p.legacy_complete
            && p.asteroids_destroyed == 10
            && p.materials_collected < 10
        {
            p.materials_collected = (p.materials_collected as u32 + amount.min(10)).min(10) as u8;
            if p.materials_collected == 10 {
                p.flags |= T1_COLLECTED;
            }
            p.stage = stage(p.flags);
            self.changed();
        }
    }
    pub(super) fn training_banked(&mut self, matching_transferred: u32) {
        if !self.progression_enabled()
            || matching_transferred == 0
            || self.position().length() > SAFE_RADIUS
        {
            return;
        }
        let Some(id) = &self.profile.identity else {
            return;
        };
        let index = material_for_corporation(id.corporation).index;
        if self.profile.campaign.stored_training_materials[index] >= 10
            && let Some(p) = &mut self.profile.onboarding
            && !p.legacy_complete
            && p.materials_collected == 10
            && p.flags & T1_BANKED == 0
        {
            p.flags |= T1_BANKED;
            p.stage = stage(p.flags);
            self.changed();
        }
        self.training_recognize_upgrade();
    }
    pub(super) fn training_recognize_upgrade(&mut self) {
        if !self.progression_enabled()
            || self.profile.campaign.selected_ship != 0
            || self.position().length() > SAFE_RADIUS
        {
            return;
        }
        let Some(identity) = &self.profile.identity else {
            return;
        };
        let id = identity.corporation.ability();
        let category = abilities::get(id).unwrap().category;
        if self.profile.campaign.abilities[id as usize].rank < 2
            || self.profile.campaign.loadouts[0].slots[category] != Some(id)
        {
            return;
        }
        if let Some(p) = &mut self.profile.onboarding
            && !p.legacy_complete
            && p.flags & T1_BANKED != 0
            && p.flags & 96 != 96
        {
            p.flags |= 96;
            p.stage = stage(p.flags);
            self.changed();
        }
    }
    pub(super) fn consume_stored_materials(&mut self, materials: [u32; 5]) {
        consume_labels(
            self.profile.campaign.stored_materials[0],
            materials[0],
            &mut self.profile.campaign.stored_training_materials,
        );
        for (tier, amount) in materials.into_iter().enumerate() {
            self.profile.campaign.stored_materials[tier] -= amount;
        }
    }
}

#[cfg(test)]
#[path = "training_tests.rs"]
mod tests;
