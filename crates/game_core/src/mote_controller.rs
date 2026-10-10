//! Camera-aware approach and pulse controller for the Radial Wastes Mote.
//! The caller supplies live world geometry and dispatches returned shots through
//! the ordinary projectile path; this controller never integrates physics twice.
#[path = "mote_art.rs"]
mod art;
#[cfg(test)]
pub use art::emitter_masks;
pub use art::{concept_body, draw};
use engine_core::{PixelBody, Transform, Vec2};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

pub const BIOME_KIND: u8 = 4; // Actual regions.rs name: Radial Wastes.
pub const FAMILY: usize = 0; // Existing ordinary Dart counters and rewards.
pub const BODY_SCALE: f32 = 0.6;
pub const CHARGE_SECONDS: f64 = 1.1;
pub const LOCK_SECONDS: f64 = 0.35;
pub const PULSE_INTERVAL: f64 = 0.35;
pub const ATTACK_SECONDS: f64 = 0.5;
pub const RECOVERY_SECONDS: f64 = 4.4;
pub const MAX_SHOTS_PER_TICK: usize = 6;
pub const MAX_SANCTUARIES: usize = 10; // Haven plus the current 3×3 site cache.
pub const WARNING_REACH: f32 = 35.0;
pub const SAFE_SECTOR_HALF_ANGLE: f32 = PI / 3.0;
const WORLD_LIMIT: f32 = 1_000_000.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Birth {
    pub seed: u32,
    pub grid: [i32; 2],
    /// Existing immutable spawn-roll bits, e.g. Brain.phase.to_bits().
    /// Does not require a global counter or any new saved field.
    pub ticket: u32,
    pub biome: u8,
    pub family: usize,
    pub ordinary: bool,
}

fn mix(mut value: u32) -> u32 {
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^ (value >> 16)
}
impl Birth {
    fn hash(self) -> u32 {
        mix(self.seed
            ^ (self.grid[0] as u32).wrapping_mul(0x9e37_79b9)
            ^ (self.grid[1] as u32).wrapping_mul(0x85eb_ca6b)
            ^ self.ticket.rotate_left(11)
            ^ 0x4953_4f54)
    }
    pub fn eligible(self) -> bool {
        self.ordinary
            && self.family == FAMILY
            && self.biome == BIOME_KIND
            && self.hash().is_multiple_of(4)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    Waiting,
    Charge,
    Attack,
    Recovery,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    pub position: Vec2,
    pub velocity: Vec2,
    pub heading: f32,
}
impl Pose {
    pub fn transform(self, scale: f32) -> Transform {
        Transform {
            position: self.position,
            rotation: self.heading + FRAC_PI_2,
            scale: Vec2::splat(scale),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sanctuary {
    pub center: Vec2,
    pub radius: f32,
}
impl Sanctuary {
    pub fn contains(self, position: Vec2) -> bool {
        self.center.distance_squared(position) <= self.radius * self.radius
    }
    pub fn intersects_segment(self, from: Vec2, to: Vec2) -> bool {
        let delta = to - from;
        let denominator = delta.length_squared();
        let t = if denominator > 0.0 {
            ((self.center - from).dot(delta) / denominator).clamp(0.0, 1.0)
        } else {
            0.0
        };
        (from + delta * t).distance_squared(self.center) <= self.radius * self.radius
    }
}

/// World-space geometry supplied by the host. Scenery/parallax never appears
/// here. Sanctuary list must include Haven even during an arena override.
pub struct Input<'a> {
    pub dt: f32,
    pub pose: Pose,
    pub player: Vec2,
    pub player_radius: f32,
    pub camera: Vec2,
    /// Host predicts the unchanged end-of-tick follow from player velocity.
    /// Current and predicted rectangles both admit the whole cue.
    pub camera_next: Vec2,
    /// Conservative world-space renderer shake allowance, never a camera edit.
    pub presentation_margin: f32,
    pub viewport: Vec2,
    pub radius: f32,
    pub scale: f32,
    pub contact_damage: f32,
    pub tier: u32,
    pub sanctuaries: &'a [Sanctuary],
    /// Host's global safe_zone_at(player) result, including distant sites.
    /// Stops fire even outside the enemy's local site cache.
    pub player_refuge: Option<Sanctuary>,
    /// Remaining slots from the existing global projectile cap, not a new cap.
    pub bullet_slots: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shot {
    pub position: Vec2,
    pub velocity: Vec2,
    pub damage: f32,
    pub emitter: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Output {
    /// Apply this pose to Motion before shared integrate(dt), exactly once.
    pub pose: Pose,
    pub shots: [Option<Shot>; MAX_SHOTS_PER_TICK],
    pub retire: bool,
}
impl Output {
    pub fn shots(&self) -> impl Iterator<Item = Shot> + '_ {
        self.shots.iter().flatten().copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error {
    InvalidInput,
    SanctuaryOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Controller {
    birth: Birth,
    phase: Phase,
    phase_age: f64,
    pulse_age: f64,
    gap_angle: f32,
    pulse: u8,
    first_delay: f32,
    orbit_sign: f32,
    target: Option<Vec2>,
}
impl Controller {
    pub fn new(birth: Birth) -> Option<Self> {
        birth.eligible().then(|| Self {
            birth,
            phase: Phase::Waiting,
            phase_age: 0.0,
            pulse_age: 0.0,
            gap_angle: 0.0,
            pulse: 0,
            first_delay: ((birth.hash() >> 8) & 255) as f32 / 255.0 * 1.3,
            orbit_sign: if birth.hash() & 0x10000 == 0 {
                -1.0
            } else {
                1.0
            },
            target: None,
        })
    }
    #[cfg(test)]
    pub fn birth(self) -> Birth {
        self.birth
    }
    pub fn phase(self) -> Phase {
        self.phase
    }
    #[cfg(test)]
    pub fn phase_age(self) -> f32 {
        self.phase_age as f32
    }
    #[cfg(test)]
    pub fn target(self) -> Option<Vec2> {
        self.target
    }
    pub fn gap(self) -> Vec2 {
        direction(self.gap_angle)
    }
    pub fn charge(self) -> f32 {
        match self.phase {
            Phase::Charge => (self.phase_age / CHARGE_SECONDS).clamp(0.0, 1.0) as f32,
            Phase::Attack => 1.0,
            _ => 0.0,
        }
    }
    fn reset_warning(&mut self) {
        self.phase = Phase::Waiting;
        self.phase_age = 0.0;
        self.pulse = 0;
        self.pulse_age = 0.0;
    }
    #[cfg(test)]
    pub fn force_charge_for_test(&mut self) {
        self.phase = Phase::Charge;
        self.phase_age = 0.0;
        self.pulse = 0;
        self.pulse_age = 0.0;
    }

    pub fn step(&mut self, input: &Input<'_>, damaged_body: &PixelBody) -> Result<Output, Error> {
        debug_assert!(self.birth.eligible());
        validate(input)?;
        let mut output = Output {
            pose: input.pose,
            shots: [None; MAX_SHOTS_PER_TICK],
            retire: false,
        };
        if input.dt == 0.0 {
            return Ok(output);
        }
        let displaced = enforce_sanctuaries(&mut output.pose, input.radius, input.sanctuaries);
        let delta = input.player - output.pose.position;
        let distance = delta.length();
        let towards = if distance > 0.01 {
            delta / distance
        } else {
            Vec2::X
        };
        if let Some(refuge) = input.player_refuge.or_else(|| {
            input
                .sanctuaries
                .iter()
                .find(|s| s.contains(input.player))
                .copied()
        }) {
            self.reset_warning();
            let outward = (output.pose.position - refuge.center).normalize_or_zero();
            output.pose.velocity += (outward * 95.0 - output.pose.velocity) * (input.dt * 3.0);
            output.retire = distance > 550.0;
            return Ok(output);
        }
        if distance > input.viewport.length().max(1050.0) {
            self.reset_warning();
            output.retire = true;
            return Ok(output);
        }
        // Preserve a legal firing position as the camera follows. Candidate
        // selection is bounded and tests real collision/sanctuary clearances;
        // it never moves the camera or teleports the actor to fit a warning.
        if self
            .target
            .is_none_or(|target| !legal_target(target, output.pose.position, input))
        {
            self.target = choose_target(output.pose.position, input, self.orbit_sign);
        }
        if matches!(self.phase, Phase::Charge | Phase::Attack)
            && (displaced
                || self.target.is_none()
                || !warning_visible(output.pose.position, input)
                || !warning_visible(
                    output.pose.position + output.pose.velocity * input.dt,
                    input,
                ))
        {
            self.reset_warning();
        }
        let desired = self.target.map_or(-towards * 55.0, |target| {
            let error = target - output.pose.position;
            error.normalize_or_zero() * (error.length() * 3.0).min(85.0)
        }) * (1.0 + 0.035 * input.tier.saturating_sub(1) as f32).min(1.3);
        output.pose.velocity += (desired - output.pose.velocity) * (1.0 - (-8.0 * input.dt).exp());
        // Steering inertia may otherwise overshoot a legal target's clearance.
        // Remove only its inward radial component before the shared integration;
        // this changes velocity, never position or collision ownership.
        let clearance = input.player_radius + input.radius + 8.0;
        if (output.pose.position + output.pose.velocity * input.dt).distance(input.player)
            < clearance
        {
            let outward = (output.pose.position - input.player).normalize_or_zero();
            output.pose.velocity -= outward * output.pose.velocity.dot(outward).min(0.0);
        }
        let settled = self
            .target
            .is_some_and(|target| target.distance(output.pose.position) < 3.0)
            && output.pose.velocity.length() < 8.0;
        let full_warning_visible = !displaced
            && distance < 480.0
            && warning_visible(output.pose.position, input)
            && warning_visible(
                output.pose.position + output.pose.velocity * input.dt,
                input,
            );
        // Duration clocks use f64 so a fixed-step f32 boundary does not shorten
        // a warning or defer it inconsistently. A pulse's clock starts on its
        // actual scheduled tick, not the theoretical charge threshold.
        self.phase_age += f64::from(input.dt);
        if self.phase == Phase::Attack && self.pulse > 0 {
            self.pulse_age += f64::from(input.dt);
        }
        match self.phase {
            Phase::Waiting => {
                self.gap_angle = towards.y.atan2(towards.x);
                if settled && full_warning_visible && self.phase_age >= f64::from(self.first_delay)
                {
                    self.phase = Phase::Charge;
                    self.phase_age = 0.0;
                    self.pulse = 0;
                }
            }
            Phase::Charge => {
                if self.phase_age < CHARGE_SECONDS - LOCK_SECONDS {
                    self.gap_angle = towards.y.atan2(towards.x);
                }
                if self.phase_age >= CHARGE_SECONDS {
                    self.phase = Phase::Attack;
                    self.phase_age -= CHARGE_SECONDS;
                }
            }
            Phase::Attack => {}
            Phase::Recovery => {
                if self.phase_age >= RECOVERY_SECONDS {
                    self.reset_warning();
                    self.first_delay = 0.0;
                }
            }
        }
        let heading = if matches!(self.phase, Phase::Charge | Phase::Attack) {
            // Rear-facing triad spreads all three emitter lobes around the
            // blocked arc; the player-facing gap remains between the aft pads.
            self.gap_angle + PI
        } else {
            towards.y.atan2(towards.x)
        };
        output.pose.heading = turn(output.pose.heading, heading, 3.0 * PI * input.dt);
        if self.phase == Phase::Attack {
            output.pose.heading = self.gap_angle + PI;
            if self.pulse == 0 || (self.pulse == 1 && self.pulse_age >= PULSE_INTERVAL) {
                self.pulse += 1; // Drops consume this pulse; never replay a saturated burst.
                self.pulse_age = 0.0;
                if full_warning_visible {
                    output.shots = emit_pulse(output.pose, self.gap_angle, damaged_body, input);
                }
            }
            if self.phase_age >= ATTACK_SECONDS {
                self.phase = Phase::Recovery;
                self.phase_age -= ATTACK_SECONDS;
            }
        }
        Ok(output)
    }
}

fn firing_half_extent(input: &Input<'_>) -> Vec2 {
    input.viewport * 0.5
        - Vec2::splat(input.radius + WARNING_REACH + 2.0 + input.presentation_margin)
        - (input.camera_next - input.camera).abs() * 0.5
}

fn warning_visible(position: Vec2, input: &Input<'_>) -> bool {
    [input.camera, input.camera_next].into_iter().all(|camera| {
        let delta = (position - camera).abs();
        delta.x + input.radius + WARNING_REACH + input.presentation_margin < input.viewport.x * 0.5
            && delta.y + input.radius + WARNING_REACH + input.presentation_margin
                < input.viewport.y * 0.5
    })
}

fn legal_target(target: Vec2, from: Vec2, input: &Input<'_>) -> bool {
    let half = firing_half_extent(input);
    if half.min_element() <= 0.0 || !target.is_finite() || target.abs().max_element() > WORLD_LIMIT
    {
        return false;
    }
    let center = input.camera.lerp(input.camera_next, 0.5);
    let delta = (target - center).abs();
    if delta.x > half.x || delta.y > half.y {
        return false;
    }
    let player_clearance = Sanctuary {
        center: input.player,
        radius: input.player_radius + input.radius + 8.0,
    };
    if player_clearance.intersects_segment(from, target) {
        return false;
    }
    !input.sanctuaries.iter().any(|s| {
        Sanctuary {
            center: s.center,
            radius: s.radius + input.radius + 6.0,
        }
        .intersects_segment(from, target)
    })
}

fn choose_target(from: Vec2, input: &Input<'_>, orbit_sign: f32) -> Option<Vec2> {
    // Inset candidate corners by two maximum-world f32 ulps. Constructing a
    // world target must not round a mathematically legal border point outside
    // the rectangle when subtracting the camera again.
    let half = firing_half_extent(input) - Vec2::splat(0.125);
    if half.min_element() <= 0.0 {
        return None;
    }
    let outward = (from - input.player).normalize_or_zero();
    let outward = if outward == Vec2::ZERO {
        Vec2::X * orbit_sign
    } else {
        outward
    };
    let center = input.camera.lerp(input.camera_next, 0.5);
    let preferred = (input.player + outward * 180.0 - center).clamp(-half, half) + center;
    let corners = [
        Vec2::new(half.x * orbit_sign, -half.y),
        Vec2::new(-half.x * orbit_sign, -half.y),
        Vec2::new(half.x * orbit_sign, half.y),
        Vec2::new(-half.x * orbit_sign, half.y),
    ];
    std::iter::once(preferred)
        // Legal interior candidates leave headroom for camera follow. Keep
        // outer corners available when player/path clearance rejects them.
        .chain(corners.map(|corner| center + corner * 0.7))
        .chain(corners.map(|corner| center + corner))
        .enumerate()
        .filter(|(_, target)| legal_target(*target, from, input))
        .min_by(|(ai, a), (bi, b)| {
            from.distance_squared(*a)
                .total_cmp(&from.distance_squared(*b))
                .then_with(|| ai.cmp(bi))
        })
        .map(|(_, target)| target)
}

fn validate(input: &Input<'_>) -> Result<(), Error> {
    if input.sanctuaries.len() > MAX_SANCTUARIES {
        return Err(Error::SanctuaryOverflow);
    }
    let world = |point: Vec2| point.is_finite() && point.abs().max_element() <= WORLD_LIMIT;
    if !input.dt.is_finite()
        || !(0.0..=0.05).contains(&input.dt)
        || !world(input.pose.position)
        || !world(input.player)
        || !input.player_radius.is_finite()
        || !(0.0..=96.0).contains(&input.player_radius)
        || !world(input.camera)
        || !world(input.camera_next)
        || !input.presentation_margin.is_finite()
        || !(0.0..=64.0).contains(&input.presentation_margin)
        || !input.pose.velocity.is_finite()
        || input.pose.velocity.length() > 500.0
        || !input.pose.heading.is_finite()
        || !input.viewport.is_finite()
        || input.viewport.min_element() <= 0.0
        || input.viewport.max_element() > 8192.0
        || !input.radius.is_finite()
        || !(0.0..=48.0).contains(&input.radius)
        || !input.scale.is_finite()
        || !(0.1..=4.0).contains(&input.scale)
        || !input.contact_damage.is_finite()
        || !(0.0..=1000.0).contains(&input.contact_damage)
        || input.sanctuaries.iter().any(|s| {
            !world(s.center) || !s.radius.is_finite() || !(0.0..=1024.0).contains(&s.radius)
        })
        || input.player_refuge.is_some_and(|s| {
            !world(s.center)
                || !s.radius.is_finite()
                || !(0.0..=1024.0).contains(&s.radius)
                || !s.contains(input.player)
        })
    {
        return Err(Error::InvalidInput);
    }
    Ok(())
}

/// Shared-geometry preguard. Its displacement cancels the warning before this
/// tick's commands; Game's unchanged post-integration guard catches later drift.
/// Keep its circle order and radius+6 clearance aligned with that postguard.
pub fn enforce_sanctuaries(pose: &mut Pose, radius: f32, sanctuaries: &[Sanctuary]) -> bool {
    let mut displaced = false;
    for sanctuary in sanctuaries {
        let delta = pose.position - sanctuary.center;
        let distance = delta.length();
        let boundary = sanctuary.radius + radius + 6.0;
        if distance < boundary {
            let outward = if distance > 0.001 {
                delta / distance
            } else {
                Vec2::X
            };
            pose.position = sanctuary.center + outward * boundary;
            pose.velocity -= outward * pose.velocity.dot(outward).min(0.0);
            displaced = true;
        }
    }
    displaced
}

fn emit_pulse(
    pose: Pose,
    gap: f32,
    body: &PixelBody,
    input: &Input<'_>,
) -> [Option<Shot>; MAX_SHOTS_PER_TICK] {
    let mut shots = [None; MAX_SHOTS_PER_TICK];
    let mut used = 0;
    // Six centers at80/120/160/200/240/280 degrees: all outside the120-degree
    // indicated escape wedge, with a20-degree buffer from its boundary.
    for degrees in [80.0_f32, 120.0, 160.0, 200.0, 240.0, 280.0] {
        if used >= input.bullet_slots.min(MAX_SHOTS_PER_TICK) {
            break;
        }
        let velocity = direction(gap + degrees.to_radians()) * 110.0;
        let Some((position, emitter)) =
            art::shot_origin(body, pose.transform(input.scale), velocity)
        else {
            continue;
        };
        if input
            .sanctuaries
            .iter()
            .any(|s| s.intersects_segment(position, position + velocity * input.dt))
        {
            continue;
        }
        shots[used] = Some(Shot {
            position,
            velocity,
            damage: input.contact_damage * 0.6,
            emitter,
        });
        used += 1;
    }
    shots
}

fn direction(angle: f32) -> Vec2 {
    Vec2::new(angle.cos(), angle.sin())
}
fn turn(current: f32, target: f32, maximum: f32) -> f32 {
    current + ((target - current + PI).rem_euclid(TAU) - PI).clamp(-maximum, maximum)
}
