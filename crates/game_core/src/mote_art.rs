use super::{Controller, Phase, Pose, SAFE_SECTOR_HALF_ANGLE, WARNING_REACH, direction};
use engine_core::{FrameBuilder, PixelBody, PixelMaterial, Transform, Vec2};
use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Deserialize)]
struct Atlas {
    version: u32,
    width: usize,
    height: usize,
    palette: Vec<u32>,
    indices: Vec<u8>,
    emitters: [Vec<[usize; 2]>; 3],
}
struct Art {
    body: PixelBody,
    emitters: [Vec<[usize; 2]>; 3],
    palette: Vec<u32>,
}
static ART: LazyLock<Art> = LazyLock::new(|| {
    let atlas: Atlas =
        serde_json::from_str(include_str!("../../../assets/pixels/mote-atlas.json")).unwrap();
    assert_eq!(atlas.version, 1);
    assert_eq!((atlas.width, atlas.height), (40, 40));
    assert_eq!(atlas.indices.len(), 1600);
    assert_eq!(atlas.palette.len(), 64);
    assert_eq!(atlas.palette[0], 0);
    assert!(atlas.palette[1..].iter().all(|c| c >> 24 == 255));
    let key = |i: usize| char::from_u32(0xe000 + i as u32).unwrap();
    let palette: Vec<_> = atlas
        .palette
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, &c)| (key(i), c))
        .collect();
    let rows: Vec<String> = atlas
        .indices
        .chunks_exact(40)
        .map(|r| {
            r.iter()
                .map(|&i| {
                    assert!(i < 64);
                    if i == 0 { ' ' } else { key(i as usize) }
                })
                .collect()
        })
        .collect();
    let borrowed: Vec<_> = rows.iter().map(String::as_str).collect();
    let body = PixelBody::from_ascii(&borrowed, &palette);
    for emitter in &atlas.emitters {
        assert!(!emitter.is_empty());
        assert!(
            emitter
                .iter()
                .all(|&[x, y]| x < 40 && y < 40 && body.is_occupied(x, y))
        );
    }
    Art {
        body,
        emitters: atlas.emitters,
        palette: atlas.palette,
    }
});
pub fn concept_body() -> PixelBody {
    ART.body.clone()
}
#[cfg(test)]
pub fn emitter_masks() -> &'static [Vec<[usize; 2]>; 3] {
    &ART.emitters
}
fn local(x: usize, y: usize) -> Vec2 {
    Vec2::new(x as f32 + 0.5 - 20.0, y as f32 + 0.5 - 20.0)
}

pub(crate) fn shot_origin(
    body: &PixelBody,
    transform: Transform,
    velocity: Vec2,
) -> Option<(Vec2, usize)> {
    let direction = velocity.normalize_or_zero();
    // All host scales are uniform. Keep ranking in local space: one inverse,
    // no per-cell trig or far-world translation cancellation.
    let local_direction = Transform {
        position: Vec2::ZERO,
        ..transform
    }
    .inverse_point(direction);
    let emitter = ART
        .emitters
        .iter()
        .enumerate()
        .map(|(index, mask)| {
            let score = mask
                .iter()
                .map(|&[x, y]| local(x, y).dot(local_direction))
                .fold(f32::NEG_INFINITY, f32::max);
            (index, score)
        })
        .max_by(|a, b| a.1.total_cmp(&b.1).then_with(|| b.0.cmp(&a.0)))?
        .0;
    // The nominal emitter owns this ray. Its destruction disables that sector;
    // another pristine or surviving emitter does not replace it invisibly.
    let point = ART.emitters[emitter]
        .iter()
        .filter(|&&[x, y]| body.is_occupied(x, y))
        .map(|&[x, y]| (local(x, y), y * 40 + x))
        .max_by(|a, b| {
            a.0.dot(local_direction)
                .total_cmp(&b.0.dot(local_direction))
                .then_with(|| b.1.cmp(&a.1))
        })?
        .0;
    Some((transform.point(point), emitter))
}

/// Every cell is extracted from the damaged body. Warning rays are world-space
/// presentation only; no raster texture, parallax hazard or collision overlay.
pub fn draw(
    frame: &mut FrameBuilder,
    body: &PixelBody,
    pose: Pose,
    scale: f32,
    controller: Controller,
) {
    let charge = controller.charge();
    frame.push_body_colored_material(
        body,
        pose.transform(scale),
        PixelMaterial {
            light_response: 0.75,
            emission: 0.0,
        },
        |_, color| {
            if color == ART.palette[9] || color == ART.palette[10] {
                let [r, g, b, a] = color.to_le_bytes();
                u32::from_le_bytes([
                    (r as f32 + (255.0 - r as f32) * charge) as u8,
                    (g as f32 + (255.0 - g as f32) * charge) as u8,
                    (b as f32 + (220.0 - b as f32) * charge) as u8,
                    a,
                ])
            } else {
                color
            }
        },
    );
    if controller.phase() != Phase::Charge {
        return;
    }
    for sign in [-1.0, 1.0] {
        let angle = controller.gap().y.atan2(controller.gap().x) + sign * SAFE_SECTOR_HALF_ANGLE;
        for step in 1..=7 {
            frame.push_pixel(
                pose.position + direction(angle) * (step as f32 * WARNING_REACH / 7.0),
                1.0,
                u32::from_le_bytes([179, 244, 147, (85.0 + charge * 150.0) as u8]),
            );
        }
    }
    for degrees in [80.0_f32, 120.0, 160.0, 200.0, 240.0, 280.0] {
        let velocity =
            direction(controller.gap().y.atan2(controller.gap().x) + degrees.to_radians());
        if shot_origin(body, pose.transform(scale), velocity).is_some() {
            frame.push_pixel(
                pose.position + velocity * 26.0,
                1.1,
                u32::from_le_bytes([239, 163, 107, (80.0 + charge * 160.0) as u8]),
            );
        }
    }
}
