//! Shared solo/session steering; caller supplies one actual ECS Motion.
use super::*;

pub(super) fn control_motion(
    mut motion: Motion,
    throttle: &mut f32,
    destination: &mut Option<Vec2>,
    input: &InputFrame,
    stats: Stats,
    dt: f32,
) -> Motion {
    if let Some(point) = input.destination
        && point
            .iter()
            .all(|v| v.is_finite() && v.abs() <= WORLD_LIMIT)
    {
        (*destination) = Some(Vec2::from_array(point));
    }
    let aim = input
        .aim
        .filter(|point| {
            input.fire
                && point
                    .iter()
                    .all(|v| v.is_finite() && v.abs() <= WORLD_LIMIT)
        })
        .map(|point| Vec2::from_array(point) - motion.position)
        .filter(|delta| delta.length_squared() > 16.0);
    let mut desired = Vec2::ZERO;
    let joystick = input
        .joystick
        .map(Vec2::from_array)
        .filter(|stick| stick.is_finite() && stick.abs().max_element() <= 4.0);
    let held_thrust = input
        .thrust
        .filter(|point| {
            point
                .iter()
                .all(|v| v.is_finite() && v.abs() <= WORLD_LIMIT)
        })
        .map(Vec2::from_array);
    if let Some(vector) = joystick {
        let stick = vector.clamp_length_max(1.0);
        (*destination) = None;
        let power = ((stick.length() - 0.08) / 0.92).clamp(0.0, 1.0);
        (*throttle) += (power - (*throttle)) * (1.0 - (-20.0 * dt).exp());
        if power > 0.0 && aim.is_none() {
            motion.heading = turn_toward(motion.heading, stick.y.atan2(stick.x), 4.0 * PI * dt);
        }
        let direction = if aim.is_some() {
            // Desktop steering and cursor facing are independent while
            // aiming. Mobile heading-fire retains the original controller.
            if power > 0.0 {
                stick.normalize_or_zero()
            } else {
                motion.velocity.normalize_or_zero()
            }
        } else {
            forward(motion.heading)
        };
        desired = direction * stats.speed * (*throttle);
    } else if let Some(target) = held_thrust {
        (*destination) = Some(target);
        let delta = target - motion.position;
        (*throttle) = 1.0;
        if aim.is_none() && delta.length_squared() > 16.0 {
            motion.heading = turn_toward(motion.heading, delta.y.atan2(delta.x), 4.0 * PI * dt);
        }
        desired = forward(motion.heading) * stats.speed;
    } else if let Some(target) = *destination {
        let delta = target - motion.position;
        let distance = delta.length();
        if distance < 4.0 && motion.velocity.length() < 15.0 {
            (*destination) = None;
            (*throttle) = 0.0;
        } else if distance > 0.01 {
            if aim.is_none() {
                motion.heading = turn_toward(motion.heading, delta.y.atan2(delta.x), 4.0 * PI * dt);
            }
            let speed = (2.0 * stats.acceleration * (distance - 2.0).max(0.0))
                .sqrt()
                .min(stats.speed);
            desired = delta / distance * speed;
            (*throttle) = speed / stats.speed;
        }
    } else {
        (*throttle) = ((*throttle) - dt / 0.15).max(0.0);
        let direction = if aim.is_some() {
            motion.velocity.normalize_or_zero()
        } else {
            forward(motion.heading)
        };
        desired = direction * stats.speed * (*throttle);
    }
    if let Some(delta) = aim {
        motion.heading = turn_toward(motion.heading, delta.y.atan2(delta.x), 4.0 * PI * dt);
        if joystick.is_none() && held_thrust.is_some() {
            desired = forward(motion.heading) * stats.speed;
        }
    }
    motion.velocity += (desired - motion.velocity).clamp_length_max(stats.acceleration * dt);
    motion.velocity = motion.velocity.clamp_length_max(stats.speed);
    if desired.length_squared() < 0.01 {
        motion.velocity *= (-3.0 * dt).exp();
    }
    motion
}
