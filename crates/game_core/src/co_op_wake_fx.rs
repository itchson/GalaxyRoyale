//! Bounded active Ion Wake state. This is cosmetic capture data, never progression.
use crate::{co_op::ParticipantId, *};

pub const MAX_POINTS: usize = 192;
pub const MAX_MEMBER_POINTS: usize = 24;
pub const MAX_PAGE_BYTES: usize = 32_768;
pub const MAX_LIFE: f32 = 3.4;
pub const POSITION_LIMIT: f32 = WORLD_LIMIT + 14.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Sample {
    pub position: Vec2,
    pub life: f32,
    pub born_tick: u32,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WakeMeta {
    pub points: u16,
}
/// Raw JSON numbers must be validated before narrowing to native f32.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WakePoint {
    pub slot: u8,
    pub epoch: u32,
    pub born_tick: u32,
    pub position: [f64; 2],
    pub life: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WakePage {
    pub session: ParticipantId,
    pub tick: u32,
    pub revision: u32,
    pub points: Vec<WakePoint>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Captured {
    pub member: ParticipantId,
    pub epoch: u32,
    pub born_tick: u32,
    pub position: Vec2,
    pub life: f32,
}
impl WakePoint {
    pub(crate) fn canonical(
        &self,
        member: ParticipantId,
        tick: u32,
    ) -> Result<Captured, &'static str> {
        let life = self.life as f32;
        let position = Vec2::new(self.position[0] as f32, self.position[1] as f32);
        if self.epoch == 0
            || self.born_tick > tick
            || !self.life.is_finite()
            || self.life <= 0.0
            || self.life > f64::from(MAX_LIFE)
            || !life.is_finite()
            || life <= 0.0
            || life > MAX_LIFE
            || self
                .position
                .iter()
                .any(|v| !v.is_finite() || v.abs() > f64::from(POSITION_LIMIT))
            || !position.is_finite()
            || position.abs().max_element() > POSITION_LIMIT
        {
            return Err("invalid_wake_point");
        }
        Ok(Captured {
            member,
            epoch: self.epoch,
            born_tick: self.born_tick,
            position,
            life,
        })
    }
}
#[derive(Clone, Default)]
pub(crate) struct Cache {
    points: Vec<(Captured, f64)>,
    clock: f64,
}
impl Cache {
    /// Retain expired captures until the next authoritative commit, so an
    /// equivalent same-tick publication cannot start their deadline again.
    pub fn advance(&mut self, clock: f64) {
        self.clock = clock;
    }
    pub fn reconcile(
        old: Option<(&Self, u32)>,
        tick: u32,
        received_at: f64,
        clock: f64,
        points: &[Captured],
        survives: impl Fn(ParticipantId, u32) -> bool,
    ) -> Result<Self, &'static str> {
        if let Some((old, old_tick)) = old {
            if old_tick == tick {
                let mut before: Vec<_> = old
                    .points
                    .iter()
                    .map(|(p, _)| p)
                    .filter(|p| survives(p.member, p.epoch))
                    .collect();
                let mut after: Vec<_> = points
                    .iter()
                    .filter(|p| survives(p.member, p.epoch))
                    .collect();
                before.sort_by_key(|p| (p.member, p.epoch, p.born_tick));
                after.sort_by_key(|p| (p.member, p.epoch, p.born_tick));
                if before != after {
                    return Err("same_tick_wake_mutation");
                }
            }
            for point in points {
                if old
                    .points
                    .iter()
                    .any(|(p, _)| same_key(p, point) && p.position != point.position)
                {
                    return Err("wake_position_changed");
                }
            }
        }
        let points = points
            .iter()
            .map(|&point| {
                let deadline = received_at + f64::from(point.life);
                let deadline = old
                    .filter(|(_, old_tick)| *old_tick == tick)
                    .and_then(|(cache, _)| cache.points.iter().find(|(p, _)| same_key(p, &point)))
                    .map_or(deadline, |(_, before)| before.min(deadline));
                (point, deadline)
            })
            .collect();
        Ok(Self { points, clock })
    }
    pub fn visible(&self) -> impl Iterator<Item = (Vec2, f32)> + '_ {
        self.points.iter().filter_map(|(p, deadline)| {
            let remaining = (deadline - self.clock).min(f64::from(p.life));
            (remaining > 0.0).then_some((p.position, remaining as f32))
        })
    }
}
fn same_key(a: &Captured, b: &Captured) -> bool {
    (a.member, a.epoch, a.born_tick) == (b.member, b.epoch, b.born_tick)
}
/// Exact solo five-dot style, shared by the host and committed guest view.
pub(crate) fn append(
    frame: &mut FrameBuilder,
    points: impl IntoIterator<Item = (Vec2, f32)>,
    phase: f32,
    visible: impl Fn(Vec2, f32) -> bool,
) {
    for (position, life) in points {
        if life <= 0.0 || !visible(position, 12.0) {
            continue;
        }
        for n in 0..5 {
            let offset = forward(n as f32 * TAU / 5.0 + phase) * 6.0;
            frame.push_pixel(
                position + offset,
                2.0,
                rgba(132, 255, 202, (life / 2.0 * 155.0) as u8),
            );
        }
    }
}
impl Game {
    pub(crate) fn co_op_wake_points(&self) -> Vec<WakePoint> {
        let mut points = Vec::with_capacity(MAX_POINTS);
        for pilot in self.world.query::<&co_op::Pilot>().iter() {
            for p in &pilot.wake {
                points.push(WakePoint {
                    slot: pilot.admission.slot,
                    epoch: pilot.admission.epoch,
                    born_tick: p.born_tick,
                    position: p.position.to_array().map(f64::from),
                    life: f64::from(p.life),
                });
            }
        }
        points.sort_by_key(|p| (p.slot, p.born_tick));
        points
    }
}
