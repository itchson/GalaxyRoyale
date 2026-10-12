//! Ordinary Nova presentation only. These birth records never invoke gameplay.
use crate::{LightSource, co_op::ParticipantId};
use engine_core::{FrameBuilder, PixelMaterial, Transform, Vec2};
use serde::{Deserialize, Serialize};

pub const MAX_RECORDS: usize = 32;
pub const RAYS: u8 = 36;
pub const RAY_LIFE: f64 = 0.4;
pub const LIFE: f64 = 0.45;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Nova {
    pub sequence: u32,
    pub tick: u32,
    pub emitter: ParticipantId,
    pub epoch: u32,
    pub position: [f32; 2],
    /// The actually admitted prefix of deterministic host rays, not a request.
    pub rays: u8,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Journal {
    pub sequence: u32,
    pub dropped: u32,
    pub records: Vec<Nova>,
}
impl Journal {
    pub(crate) fn record(
        &mut self,
        tick: u32,
        emitter: ParticipantId,
        epoch: u32,
        position: Vec2,
        rays: u8,
    ) {
        let Some(sequence) = self.sequence.checked_add(1) else {
            self.dropped = self.dropped.saturating_add(1);
            return;
        };
        self.sequence = sequence;
        self.records
            .retain(|r| f64::from(tick.saturating_sub(r.tick)) / 60.0 < LIFE);
        if self.records.len() == MAX_RECORDS {
            self.records.remove(0);
            self.dropped = self.dropped.saturating_add(1);
        }
        self.records.push(Nova {
            sequence,
            tick,
            emitter,
            epoch,
            position: position.to_array(),
            rays,
        });
    }
}

/// Exact birth parameters shared by real host particles and guest hard quads.
pub(crate) fn ray(index: u8) -> (Vec2, Vec2, u32, f32) {
    let direction = crate::forward(f32::from(index) * std::f32::consts::TAU / f32::from(RAYS));
    (
        direction * 8.0,
        direction * 150.0,
        crate::rgba(147, 193, 255, 255),
        2.0,
    )
}
pub(crate) fn lights(position: Vec2) -> [(LightSource, f32); 2] {
    [
        (
            LightSource {
                position: position.to_array(),
                radius: 165.0,
                color: [0.16, 0.65, 1.0],
                intensity: 1.5,
            },
            0.45,
        ),
        (
            LightSource {
                position: position.to_array(),
                radius: 95.0,
                color: [0.22, 0.56, 1.0],
                intensity: 1.1,
            },
            0.32,
        ),
    ]
}

/// Session-local visual cache. Records are world anchored, never attached to a
/// replacement pilot. Retired emitters are cleared by a subsequent commit.
#[derive(Default)]
pub(crate) struct Cache {
    now: f64,
    active: Vec<Active>,
    pub(crate) dropped_rays: u32,
}
struct Active {
    nova: Nova,
    received_at: f64,
    received_age: f64,
    lights: [bool; 2],
}
impl Active {
    fn age(&self, now: f64) -> f64 {
        self.received_age + (now - self.received_at)
    }
}
impl Cache {
    pub(crate) fn advance(&mut self, now: f64) {
        self.now = now;
        self.active.retain(|a| a.age(now) < LIFE);
    }
    pub(crate) fn retain_members(&mut self, mut member: impl FnMut(ParticipantId, u32) -> bool) {
        self.active.retain(|a| member(a.nova.emitter, a.nova.epoch));
    }
    pub(crate) fn rays(&self) -> usize {
        self.active
            .iter()
            .filter(|a| a.age(self.now) < RAY_LIFE)
            .map(|a| usize::from(a.nova.rays))
            .sum()
    }
    pub(crate) fn light_count(&self) -> usize {
        self.active
            .iter()
            .map(|a| {
                let age = a.age(self.now);
                a.lights
                    .iter()
                    .zip([LIFE, 0.32])
                    .filter(|(on, life)| **on && age < *life)
                    .count()
            })
            .sum()
    }
    pub(crate) fn admit(&mut self, mut nova: Nova, age: f64) {
        if age >= LIFE {
            return;
        }
        if self.active.len() == MAX_RECORDS {
            let old = self.active.remove(0);
            if old.age(self.now) < RAY_LIFE {
                self.dropped_rays = self.dropped_rays.saturating_add(u32::from(old.nova.rays));
            }
        }
        let available = crate::MAX_PARTICLES.saturating_sub(self.rays());
        let admitted = usize::from(nova.rays).min(available) as u8;
        self.dropped_rays = self
            .dropped_rays
            .saturating_add(u32::from(nova.rays - admitted));
        nova.rays = admitted;
        let new_lights = if age < 0.32 { 2 } else { 1 };
        let mut excess = (self.light_count() + new_lights).saturating_sub(32);
        for old in &mut self.active {
            let elapsed = old.age(self.now);
            for (on, life) in old.lights.iter_mut().zip([LIFE, 0.32]) {
                if excess > 0 && *on && elapsed < life {
                    *on = false;
                    excess -= 1;
                }
            }
        }
        self.active.push(Active {
            nova,
            received_at: self.now,
            received_age: age,
            lights: [true, age < 0.32],
        });
    }
    pub(crate) fn sources(&self, mut visible: impl FnMut(Vec2, f32) -> bool) -> Vec<LightSource> {
        let mut sources = Vec::with_capacity(4);
        for a in self.active.iter().rev() {
            for (index, (mut source, life)) in lights(Vec2::from_array(a.nova.position))
                .into_iter()
                .enumerate()
                .rev()
            {
                let age = a.age(self.now) as f32;
                if a.lights[index]
                    && age < life
                    && visible(Vec2::from_array(source.position), source.radius)
                {
                    source.intensity *= (1.0 - age / life).clamp(0.0, 1.0);
                    sources.push(source);
                    if sources.len() == 4 {
                        return sources;
                    }
                }
            }
        }
        sources
    }
    pub(crate) fn render(
        &self,
        frame: &mut FrameBuilder,
        reduced: bool,
        mut visible: impl FnMut(Vec2, f32) -> bool,
    ) {
        let mut ordinal = 0;
        for a in &self.active {
            let elapsed = a.age(self.now);
            if elapsed >= RAY_LIFE {
                continue;
            }
            let age = elapsed as f32;
            for i in 0..a.nova.rays {
                let skip = reduced && ordinal % 2 == 1;
                ordinal += 1;
                if skip {
                    continue;
                }
                let (offset, velocity, color, size) = ray(i);
                // Same exact exponential drag integration as host debris.
                let local =
                    velocity * ((1.0 - (-crate::DEBRIS_DRAG * age).exp()) / crate::DEBRIS_DRAG);
                let origin = Vec2::from_array(a.nova.position) + offset;
                if !visible(origin + local, size + 4.0) {
                    continue;
                }
                let mut color = color.to_le_bytes();
                color[3] =
                    (f32::from(color[3]) * (1.0 - age / RAY_LIFE as f32).clamp(0.0, 1.0)) as u8;
                frame.push_cells_affine_material(
                    Transform {
                        position: origin,
                        scale: Vec2::splat(size),
                        ..Transform::default()
                    }
                    .affine(),
                    PixelMaterial::default(),
                    [(local / size, u32::from_le_bytes(color))],
                );
            }
        }
    }
}
