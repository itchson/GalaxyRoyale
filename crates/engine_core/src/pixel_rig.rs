//! Rigid local pixel subobjects with per-part shading and cell animation.
//! A rig partitions the live body's cells; damage and collision remain exact.
use crate::{AffineTransform, FrameBuilder, PixelBody, PixelMaterial, Transform, Vec2};
use serde::Deserialize;

pub const MAX_PIXEL_RIG_PARTS: usize = 16;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PixelRigRole {
    Armor,
    Core,
    Emitter,
    Trim,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PixelRigPart {
    pub name: String,
    pub parent: Option<usize>,
    /// Translation relative to the earlier parent part, in body-local units.
    pub position: [f32; 2],
    pub role: PixelRigRole,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PixelRigState {
    pub time: f32,
    /// Telegraph/weapon charge, normalized 0..1.
    pub charge: f32,
    pub core_open: bool,
    pub hit_flash: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelRigError {
    InvalidDimensions,
    InvalidRoles,
    InvalidHierarchy,
}

impl std::fmt::Display for PixelRigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidDimensions => "pixel rig dimensions must be within 1..128",
            Self::InvalidRoles => "pixel rig roles must match its dimensions and part indices",
            Self::InvalidHierarchy => {
                "pixel rig parts require finite placements and earlier parents"
            }
        })
    }
}
impl std::error::Error for PixelRigError {}

#[derive(Debug, Clone)]
pub struct PixelRig {
    width: usize,
    height: usize,
    roles: Box<[u8]>,
    parts: Box<[PixelRigPart]>,
    placements: Box<[AffineTransform]>,
}

impl PixelRig {
    pub fn new(
        width: usize,
        height: usize,
        roles: Vec<u8>,
        parts: Vec<PixelRigPart>,
    ) -> Result<Self, PixelRigError> {
        if !(1..=128).contains(&width) || !(1..=128).contains(&height) {
            return Err(PixelRigError::InvalidDimensions);
        }
        if parts.is_empty()
            || parts.len() > MAX_PIXEL_RIG_PARTS
            || roles.len() != width * height
            || roles.iter().any(|&part| part as usize >= parts.len())
        {
            return Err(PixelRigError::InvalidRoles);
        }
        let mut placements: Vec<AffineTransform> = Vec::with_capacity(parts.len());
        for (index, part) in parts.iter().enumerate() {
            let position = Vec2::from_array(part.position);
            if !position.is_finite()
                || position.abs().max_element() > 128.0
                || part.parent.is_some_and(|parent| parent >= index)
            {
                return Err(PixelRigError::InvalidHierarchy);
            }
            let local = Transform {
                position,
                ..Transform::default()
            }
            .affine();
            placements.push(
                part.parent
                    .map_or(local, |parent| placements[parent].compose(local)),
            );
        }
        Ok(Self {
            width,
            height,
            roles: roles.into_boxed_slice(),
            parts: parts.into_boxed_slice(),
            placements: placements.into_boxed_slice(),
        })
    }

    pub fn parts(&self) -> &[PixelRigPart] {
        &self.parts
    }

    pub fn draw(
        &self,
        frame: &mut FrameBuilder,
        damaged_body: &PixelBody,
        parent: Transform,
        state: PixelRigState,
    ) -> usize {
        self.draw_affine(frame, damaged_body, parent.affine(), state)
    }

    /// Every surviving cell is emitted exactly once. Child-local coordinates
    /// are derived from the damaged body and compose back to its exact position.
    /// No pristine role template can restore a collision hole or missing limb.
    pub fn draw_affine(
        &self,
        frame: &mut FrameBuilder,
        damaged_body: &PixelBody,
        parent: AffineTransform,
        state: PixelRigState,
    ) -> usize {
        if damaged_body.width != self.width || damaged_body.height != self.height {
            return 0;
        }
        let finite_unit = |value: f32| {
            if value.is_finite() {
                value.clamp(0.0, 1.0)
            } else {
                0.0
            }
        };
        let charge = finite_unit(state.charge);
        let flash = finite_unit(state.hit_flash);
        let time = if state.time.is_finite() {
            state.time.clamp(-1_000_000.0, 1_000_000.0)
        } else {
            0.0
        };
        let pulse = (time * 5.0).sin() * 0.5 + 0.5;
        let mut emitted = 0;
        for (index, part) in self.parts.iter().enumerate() {
            let placement = self.placements[index];
            let material = match part.role {
                PixelRigRole::Armor => PixelMaterial {
                    light_response: 0.8,
                    emission: 0.0,
                },
                PixelRigRole::Trim => PixelMaterial {
                    light_response: 0.65,
                    emission: 0.025 + charge * 0.03,
                },
                PixelRigRole::Emitter => PixelMaterial {
                    light_response: 0.5,
                    emission: 0.04 + charge * 0.46,
                },
                PixelRigRole::Core => PixelMaterial {
                    light_response: 0.4,
                    emission: if state.core_open {
                        0.45 + pulse * 0.15
                    } else {
                        0.06 + charge * 0.06
                    },
                },
            };
            let cells = damaged_body
                .cells_indexed()
                .filter(|&(cell_index, _, _)| self.roles[cell_index] as usize == index)
                .map(|(cell_index, local, color)| {
                    let tint = match part.role {
                        PixelRigRole::Core if !state.core_open => 0.56 + charge * 0.1,
                        PixelRigRole::Core => 1.0 + pulse * 0.1,
                        PixelRigRole::Emitter => 0.8 + charge * 0.2,
                        _ => 1.0,
                    };
                    let [r, g, b, alpha] = color.to_le_bytes();
                    let animate = |channel: u8| {
                        let lit = (channel as f32 * tint).min(255.0);
                        (lit + (255.0 - lit) * flash * 0.65).round() as u8
                    };
                    (
                        local - placement.position,
                        u32::from_le_bytes([animate(r), animate(g), animate(b), alpha]),
                        damaged_body.surface_index(cell_index),
                    )
                });
            emitted += frame.push_cells_affine_surface_material(
                parent.compose(placement),
                material,
                cells,
            );
        }
        emitted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rig() -> PixelRig {
        PixelRig::new(
            4,
            3,
            vec![0, 0, 1, 1, 0, 0, 2, 2, 0, 0, 1, 1],
            vec![
                PixelRigPart {
                    name: "hull".into(),
                    parent: None,
                    position: [0.0; 2],
                    role: PixelRigRole::Armor,
                },
                PixelRigPart {
                    name: "reactor".into(),
                    parent: Some(0),
                    position: [1.0, 0.0],
                    role: PixelRigRole::Core,
                },
                PixelRigPart {
                    name: "lens".into(),
                    parent: Some(1),
                    position: [0.5, 0.0],
                    role: PixelRigRole::Emitter,
                },
            ],
        )
        .unwrap()
    }

    #[test]
    fn damaged_rig_emits_survivors_once_with_exact_nested_world_positions() {
        let mut body = PixelBody::from_ascii(&["####", "# ##", "####"], &[('#', 0xff_998877)]);
        body.damage(Vec2::new(0.5, 0.0), 0.2);
        let parent = Transform {
            position: Vec2::new(210.0, -30.0),
            rotation: 0.7,
            scale: Vec2::new(1.3, 0.8),
        }
        .affine();
        let expected: Vec<_> = body.cells().map(|(p, _)| parent.point(p)).collect();
        let mut frame = FrameBuilder::new();
        assert_eq!(
            rig().draw_affine(&mut frame, &body, parent, PixelRigState::default()),
            expected.len()
        );
        let positions: Vec<_> = frame
            .frame()
            .batches
            .iter()
            .flat_map(|b| {
                b.pixels
                    .iter()
                    .map(|p| b.transforms[p.transform_slot()].point(Vec2::from_array(p.local)))
            })
            .collect();
        assert_eq!(positions.len(), expected.len());
        for p in expected {
            assert_eq!(
                positions
                    .iter()
                    .filter(|&&q| p.distance(q) < 0.0001)
                    .count(),
                1
            );
        }
    }

    #[test]
    fn phase_animation_changes_core_emission_without_changing_geometry_or_occupancy() {
        let body = PixelBody::from_ascii(&["####", "####", "####"], &[('#', 0xff_aabbcc)]);
        let rig = rig();
        let extract = |state| {
            let mut frame = FrameBuilder::new();
            rig.draw(&mut frame, &body, Transform::default(), state);
            frame.finish()
        };
        let closed = extract(PixelRigState::default());
        let open = extract(PixelRigState {
            time: 0.2,
            core_open: true,
            charge: 1.0,
            ..PixelRigState::default()
        });
        assert_eq!(closed.pixel_count, open.pixel_count);
        for (a, b) in closed.batches[0].pixels.iter().zip(&open.batches[0].pixels) {
            assert_eq!(a.local, b.local);
            assert_eq!(a.transform, b.transform);
        }
        assert!(open.batches[0].transforms[1].b[3] > closed.batches[0].transforms[1].b[3]);
        assert!(
            open.batches[0]
                .transforms
                .iter()
                .all(|t| t.b[3] >= 0.0 && t.b[3] <= 0.6)
        );
        assert!(
            closed.batches[0]
                .pixels
                .iter()
                .zip(&open.batches[0].pixels)
                .any(|(a, b)| a.color != b.color)
        );
        let bad = extract(PixelRigState {
            time: f32::INFINITY,
            charge: f32::NAN,
            hit_flash: f32::INFINITY,
            core_open: false,
        });
        assert_eq!(bad.batches[0].transforms, closed.batches[0].transforms);
        assert_eq!(bad.batches[0].pixels, closed.batches[0].pixels);
    }

    #[test]
    fn invalid_rig_layouts_and_mismatched_bodies_fail_closed() {
        assert!(matches!(
            PixelRig::new(129, 1, vec![], vec![]),
            Err(PixelRigError::InvalidDimensions)
        ));
        assert!(matches!(
            PixelRig::new(1, 1, vec![0], vec![]),
            Err(PixelRigError::InvalidRoles)
        ));
        let mut parts = rig().parts().to_vec();
        parts[1].parent = Some(1);
        assert!(matches!(
            PixelRig::new(4, 3, vec![0; 12], parts),
            Err(PixelRigError::InvalidHierarchy)
        ));
        let mut frame = FrameBuilder::new();
        let body = PixelBody::from_ascii(&["#"], &[('#', 0xff_998877)]);
        assert_eq!(
            rig().draw(
                &mut frame,
                &body,
                Transform::default(),
                PixelRigState::default()
            ),
            0
        );
        assert_eq!(frame.frame().pixel_count, 0);
    }

    #[test]
    fn minimum_and_maximum_dimensions_preserve_holes_with_sixteen_nested_parts() {
        for size in [1, 128] {
            let rows: Vec<_> = (0..size)
                .map(|y| {
                    (0..size)
                        .map(|x| if (x + y) % 2 == 0 { '#' } else { ' ' })
                        .collect::<String>()
                })
                .collect();
            let mut body = PixelBody::from_ascii(
                &rows.iter().map(String::as_str).collect::<Vec<_>>(),
                &[('#', 0xff_aabbcc)],
            );
            let parts: Vec<_> = (0..MAX_PIXEL_RIG_PARTS)
                .map(|index| PixelRigPart {
                    name: format!("part{index}"),
                    parent: index.checked_sub(1),
                    position: [0.5, -0.5],
                    role: PixelRigRole::Armor,
                })
                .collect();
            let rig = PixelRig::new(
                size,
                size,
                (0..size * size)
                    .map(|i| (i % MAX_PIXEL_RIG_PARTS) as u8)
                    .collect(),
                parts.clone(),
            )
            .unwrap();
            assert!(
                PixelRig::new(
                    size,
                    size,
                    vec![0; size * size],
                    [parts, vec![rig.parts()[0].clone()]].concat()
                )
                .is_err()
            );
            if size == 128 {
                assert_eq!(body.damage(Vec2::splat(0.5), 0.2).len(), 1);
                assert!(
                    body.hit_segment(Vec2::splat(0.5), Vec2::splat(0.5))
                        .is_none()
                );
            }
            let mut frame = FrameBuilder::new();
            assert_eq!(
                rig.draw(
                    &mut frame,
                    &body,
                    Transform::default(),
                    PixelRigState::default()
                ),
                body.occupied_count()
            );
            for batch in &frame.frame().batches {
                for pixel in &batch.pixels {
                    let local = batch.transforms[pixel.transform_slot()]
                        .point(Vec2::from_array(pixel.local));
                    let grid = local + Vec2::splat(size as f32 * 0.5);
                    assert!(body.is_occupied(grid.x.floor() as usize, grid.y.floor() as usize));
                }
            }
        }
    }
}
