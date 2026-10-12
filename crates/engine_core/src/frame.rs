use crate::{AffineTransform, PIXEL_TRANSFORM_SLOT_MASK, PixelBody, Surface, Transform, Vec2};
use bytemuck::{Pod, Zeroable};

pub const MAX_BATCH_TRANSFORMS: usize = 128;

/// Object-level shading controls. Zero is unlit, preserving palette-authored UI
/// and scenery. Values are sanitized when packed into the GPU transform table.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct PixelMaterial {
    pub light_response: f32,
    pub emission: f32,
}

/// 32-byte affine record matching the shared WGSL uniform contract.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct GpuTransform {
    /// Translation.xy, basis_x.xy.
    pub a: [f32; 4],
    /// basis_y.xy, light response, emission. Only explicit soft extraction uses
    /// negative emission, encoded as -(1 + emission); hard materials stay >=0.
    pub b: [f32; 4],
}

impl From<Transform> for GpuTransform {
    fn from(value: Transform) -> Self {
        value.affine().into()
    }
}

impl From<AffineTransform> for GpuTransform {
    fn from(value: AffineTransform) -> Self {
        Self {
            a: [
                value.position.x,
                value.position.y,
                value.basis_x.x,
                value.basis_x.y,
            ],
            b: [value.basis_y.x, value.basis_y.y, 0.0, 0.0],
        }
    }
}

impl GpuTransform {
    pub fn with_material(mut self, material: PixelMaterial) -> Self {
        let finite_unit = |value: f32| {
            if value.is_finite() {
                value.clamp(0.0, 1.0)
            } else {
                0.0
            }
        };
        self.b[2] = finite_unit(material.light_response);
        self.b[3] = finite_unit(material.emission);
        self
    }

    pub fn point(self, local: Vec2) -> Vec2 {
        Vec2::new(self.a[0], self.a[1])
            + Vec2::new(self.a[2], self.a[3]) * local.x
            + Vec2::new(self.b[0], self.b[1]) * local.y
    }
}

/// 16-byte instanced vertex record. Quads use local unit-square centers.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct PixelInstance {
    pub local: [f32; 2],
    pub color: u32,
    /// Low7 bits select the transform; optional surface occupies the upper25.
    pub transform: u32,
}
impl PixelInstance {
    pub fn transform_slot(self) -> usize {
        (self.transform & PIXEL_TRANSFORM_SLOT_MASK) as usize
    }
    pub fn surface(self) -> Option<Surface> {
        Surface::from_cell_word(self.transform)
    }
}

#[derive(Debug, Clone, Default)]
pub struct RenderBatch {
    pub transforms: Vec<GpuTransform>,
    pub pixels: Vec<PixelInstance>,
}

#[derive(Debug, Clone, Default)]
pub struct RenderFrame {
    pub batches: Vec<RenderBatch>,
    pub pixel_count: usize,
}

/// Order-preserving extraction. Splits before the portable transform-table
/// ceiling; many particles with the same size share one transform per batch.
#[derive(Debug, Default)]
pub struct FrameBuilder {
    frame: RenderFrame,
    spare_batches: Vec<RenderBatch>,
    pixel_transforms: Vec<(u32, u32)>,
}

impl FrameBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reuses batch allocations while resetting their contents.
    pub fn clear(&mut self) {
        // Spare batches are a stack. Reverse the drain so a stable scene reuses
        // each batch's existing capacity in the same extraction position.
        for mut batch in self.frame.batches.drain(..).rev() {
            batch.transforms.clear();
            batch.pixels.clear();
            self.spare_batches.push(batch);
        }
        self.frame.pixel_count = 0;
        self.pixel_transforms.clear();
    }

    /// Borrows the current extraction without moving or copying its buffers.
    /// Clear and rebuild this builder after the renderer has consumed the borrow.
    pub fn frame(&self) -> &RenderFrame {
        &self.frame
    }

    pub fn push_body(&mut self, body: &PixelBody, transform: Transform) {
        self.push_body_affine(body, transform.affine());
    }

    /// Animates each surviving cell's color without cloning the body or changing
    /// collision occupancy. The callback receives centered local coordinates and
    /// the original RGBA color, so lights can pulse independently of their group.
    pub fn push_body_colored(
        &mut self,
        body: &PixelBody,
        transform: Transform,
        color: impl FnMut(Vec2, u32) -> u32,
    ) {
        self.push_body_affine_colored(body, transform.affine(), color);
    }

    pub fn push_body_affine(&mut self, body: &PixelBody, transform: AffineTransform) {
        self.push_body_affine_colored(body, transform, |_, color| color);
    }

    pub fn push_body_affine_colored(
        &mut self,
        body: &PixelBody,
        transform: AffineTransform,
        color: impl FnMut(Vec2, u32) -> u32,
    ) {
        self.push_body_affine_colored_material(body, transform, PixelMaterial::default(), color);
    }

    /// Extracts a color-animated body with an object-level lighting material.
    pub fn push_body_colored_material(
        &mut self,
        body: &PixelBody,
        transform: Transform,
        material: PixelMaterial,
        color: impl FnMut(Vec2, u32) -> u32,
    ) {
        self.push_body_affine_colored_material(body, transform.affine(), material, color);
    }

    pub fn push_body_affine_colored_material(
        &mut self,
        body: &PixelBody,
        transform: AffineTransform,
        material: PixelMaterial,
        mut color: impl FnMut(Vec2, u32) -> u32,
    ) {
        if body.occupied_count() == 0
            || !transform.position.is_finite()
            || !transform.basis_x.is_finite()
            || !transform.basis_y.is_finite()
        {
            return;
        }
        let slot = self.add_transform(GpuTransform::from(transform).with_material(material));
        let batch = self
            .frame
            .batches
            .last_mut()
            .expect("transform creates batch");
        batch.pixels.reserve(body.occupied_count());
        batch.pixels.extend(
            body.cells_indexed()
                .map(|(index, local, original)| PixelInstance {
                    local: local.to_array(),
                    color: color(local, original),
                    transform: slot | body.surface_index(index).map_or(0, Surface::packed),
                }),
        );
        self.frame.pixel_count += body.occupied_count();
    }

    pub fn push_pixel(&mut self, position: Vec2, size: f32, color: u32) {
        if !position.is_finite() || !size.is_finite() || size <= 0.0 {
            return;
        }
        let local = position / size;
        if !local.is_finite() {
            return;
        }
        let bits = size.to_bits();
        let slot =
            if let Some(&(_, slot)) = self.pixel_transforms.iter().find(|&&(key, _)| key == bits) {
                slot
            } else {
                // Zero translation permits positions to differ while sharing a scale.
                let slot = self.add_transform(
                    Transform {
                        scale: Vec2::splat(size),
                        ..Transform::default()
                    }
                    .into(),
                );
                self.pixel_transforms.push((bits, slot));
                slot
            };
        self.frame
            .batches
            .last_mut()
            .expect("transform creates batch")
            .pixels
            .push(PixelInstance {
                local: local.to_array(),
                color,
                transform: slot,
            });
        self.frame.pixel_count += 1;
    }

    /// Extracts a streamed occupied subobject without allocating a PixelBody or
    /// cloning a pristine template. Empty subobjects consume no transform slot.
    pub fn push_cells_affine_material(
        &mut self,
        transform: AffineTransform,
        material: PixelMaterial,
        cells: impl IntoIterator<Item = (Vec2, u32)>,
    ) -> usize {
        if !transform.position.is_finite()
            || !transform.basis_x.is_finite()
            || !transform.basis_y.is_finite()
        {
            return 0;
        }
        self.push_cells_packed(GpuTransform::from(transform).with_material(material), cells)
    }
    /// Occupied subobject extraction retaining independent local surface fields.
    /// Invalid positions are skipped; no PixelBody clone or extra GPU stream.
    pub fn push_cells_affine_surface_material(
        &mut self,
        transform: AffineTransform,
        material: PixelMaterial,
        cells: impl IntoIterator<Item = (Vec2, u32, Option<Surface>)>,
    ) -> usize {
        if !transform.position.is_finite()
            || !transform.basis_x.is_finite()
            || !transform.basis_y.is_finite()
        {
            return 0;
        }
        self.push_surface_cells_packed(GpuTransform::from(transform).with_material(material), cells)
    }

    /// Radially filtered cosmetic cells with local fractional geometry. The
    /// shader conserves projected area while filtering a subpixel footprint;
    /// hard quads and collision occupancy are never enlarged by this API.
    /// All submitted cells, including alpha zero, consume the caller's budget.
    pub fn push_soft_cells_affine(
        &mut self,
        transform: AffineTransform,
        cells: impl IntoIterator<Item = (Vec2, u32)>,
    ) -> usize {
        self.push_soft_cells_affine_material(transform, PixelMaterial::default(), cells)
    }

    /// Soft gas/fire can receive bounded lighting and emission independently.
    /// Emission changes this material's brightness; real light sources must
    /// still be supplied by the simulation. Invalid or unrepresentable soft
    /// bases add neither pixels nor transform slots.
    pub fn push_soft_cells_affine_material(
        &mut self,
        transform: AffineTransform,
        material: PixelMaterial,
        cells: impl IntoIterator<Item = (Vec2, u32)>,
    ) -> usize {
        let determinant = transform.basis_x.perp_dot(transform.basis_y);
        let area_scale = transform.basis_x.length() * transform.basis_y.length();
        if !transform.position.is_finite()
            || !transform.basis_x.is_finite()
            || !transform.basis_y.is_finite()
            || transform.basis_x.abs().max_element() > 4096.0
            || transform.basis_y.abs().max_element() > 4096.0
            || !determinant.is_finite()
            || determinant == 0.0
            || determinant.abs() < area_scale * 0.0001
        {
            return 0;
        }
        let mut packed = GpuTransform::from(transform).with_material(material);
        packed.b[3] = -(1.0 + packed.b[3]);
        self.push_cells_packed(
            packed,
            cells
                .into_iter()
                .filter(|(local, _)| local.is_finite() && transform.point(*local).is_finite()),
        )
    }

    fn push_cells_packed(
        &mut self,
        transform: GpuTransform,
        cells: impl IntoIterator<Item = (Vec2, u32)>,
    ) -> usize {
        self.push_surface_cells_packed(
            transform,
            cells.into_iter().map(|(local, color)| (local, color, None)),
        )
    }
    fn push_surface_cells_packed(
        &mut self,
        transform: GpuTransform,
        cells: impl IntoIterator<Item = (Vec2, u32, Option<Surface>)>,
    ) -> usize {
        let mut cells = cells.into_iter().filter(|(local, _, _)| local.is_finite());
        let Some(first) = cells.next() else {
            return 0;
        };
        let slot = self.add_transform(transform);
        let batch = self
            .frame
            .batches
            .last_mut()
            .expect("transform creates batch");
        let previous = batch.pixels.len();
        batch.pixels.extend(
            std::iter::once(first)
                .chain(cells)
                .map(|(local, color, surface)| PixelInstance {
                    local: local.to_array(),
                    color,
                    transform: slot | surface.map_or(0, Surface::packed),
                }),
        );
        let count = batch.pixels.len() - previous;
        self.frame.pixel_count += count;
        count
    }

    pub fn finish(self) -> RenderFrame {
        self.frame
    }

    fn add_transform(&mut self, transform: GpuTransform) -> u32 {
        if self
            .frame
            .batches
            .last()
            .is_none_or(|batch| batch.transforms.len() == MAX_BATCH_TRANSFORMS)
        {
            let batch = self.spare_batches.pop().unwrap_or_else(|| RenderBatch {
                transforms: Vec::with_capacity(MAX_BATCH_TRANSFORMS),
                pixels: Vec::with_capacity(1024),
            });
            self.frame.batches.push(batch);
            self.pixel_transforms.clear();
        }
        let batch = self.frame.batches.last_mut().expect("nonempty batch pool");
        let slot = batch.transforms.len() as u32;
        batch.transforms.push(transform);
        slot
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pixel_body() -> PixelBody {
        PixelBody::from_ascii(&["#"], &[('#', 0xff_abcdef)])
    }
    fn near(a: Vec2, b: Vec2) {
        assert!((a - b).length() < 0.0001, "{a:?} != {b:?}");
    }

    #[test]
    fn gpu_layouts_have_the_fixed_byte_contract() {
        assert_eq!(std::mem::size_of::<PixelInstance>(), 16);
        assert_eq!(std::mem::size_of::<GpuTransform>(), 32);
        assert_eq!(std::mem::offset_of!(PixelInstance, local), 0);
        assert_eq!(std::mem::offset_of!(PixelInstance, color), 8);
        assert_eq!(std::mem::offset_of!(PixelInstance, transform), 12);
        assert_eq!(std::mem::offset_of!(GpuTransform, b), 16);
        let pixel = PixelInstance {
            local: [1.0, 2.0],
            color: 0xff_332211,
            transform: 7,
        };
        assert_eq!(
            &bytemuck::bytes_of(&pixel)[8..12],
            &[0x11, 0x22, 0x33, 0xff]
        );
    }

    #[test]
    fn material_packing_preserves_geometry_and_sanitizes_nonfinite_fields() {
        let transform = Transform {
            position: Vec2::new(11.0, -8.0),
            rotation: 0.6,
            scale: Vec2::new(3.0, 0.5),
        };
        let plain = GpuTransform::from(transform);
        assert_eq!(&plain.b[2..], &[0.0, 0.0]);
        assert_eq!(plain.with_material(PixelMaterial::default()), plain);
        let material = plain.with_material(PixelMaterial {
            light_response: 0.75,
            emission: 1.4,
        });
        assert_eq!(&material.b[2..], &[0.75, 1.0]);
        near(
            material.point(Vec2::new(2.0, -1.0)),
            plain.point(Vec2::new(2.0, -1.0)),
        );
        let invalid = plain.with_material(PixelMaterial {
            light_response: f32::NAN,
            emission: f32::INFINITY,
        });
        assert_eq!(invalid, plain);
        assert_eq!(
            plain.with_material(PixelMaterial {
                light_response: -1.0,
                emission: -0.1
            }),
            plain
        );

        let mut frame = FrameBuilder::new();
        frame.push_body_colored_material(
            &pixel_body(),
            transform,
            PixelMaterial {
                light_response: 0.25,
                emission: 0.5,
            },
            |_, color| color,
        );
        frame.push_body_affine_colored_material(
            &pixel_body(),
            transform.affine(),
            PixelMaterial {
                light_response: f32::NEG_INFINITY,
                emission: f32::NAN,
            },
            |_, color| color,
        );
        frame.push_body(&pixel_body(), transform);
        let batch = &frame.frame().batches[0];
        assert_eq!(&batch.transforms[0].b[2..], &[0.25, 0.5]);
        assert_eq!(batch.transforms[1], plain);
        assert_eq!(batch.transforms[2], plain);
        assert_eq!(std::mem::size_of::<GpuTransform>(), 32);
    }

    #[test]
    fn body_batches_split_at_128_and_remap_indices() {
        let body = pixel_body();
        let mut builder = FrameBuilder::new();
        for x in 0..257 {
            builder.push_body(
                &body,
                Transform {
                    position: Vec2::new(x as f32, 0.0),
                    ..Transform::default()
                },
            );
        }
        let frame = builder.finish();
        assert_eq!(frame.pixel_count, 257);
        assert_eq!(
            frame
                .batches
                .iter()
                .map(|b| b.transforms.len())
                .collect::<Vec<_>>(),
            [128, 128, 1]
        );
        for batch in &frame.batches {
            assert!(
                batch
                    .pixels
                    .iter()
                    .all(|pixel| pixel.transform_slot() < batch.transforms.len())
            );
        }
        assert_eq!(frame.batches[1].pixels[0].transform, 0);
        assert_eq!(frame.batches[2].transforms[0].a[0], 256.0);
    }

    #[test]
    fn particle_sizes_share_transforms_without_changing_world_positions() {
        let mut builder = FrameBuilder::new();
        for x in 0..500 {
            builder.push_pixel(
                Vec2::new(x as f32, -3.0),
                if x % 2 == 0 { 2.0 } else { 3.0 },
                0xff_ffffff,
            );
        }
        let frame = builder.finish();
        assert_eq!(frame.batches.len(), 1);
        assert_eq!(frame.batches[0].transforms.len(), 2);
        for (x, pixel) in frame.batches[0].pixels.iter().enumerate() {
            let t = frame.batches[0].transforms[pixel.transform_slot()];
            near(
                t.point(Vec2::from_array(pixel.local)),
                Vec2::new(x as f32, -3.0),
            );
        }
    }

    #[test]
    fn particles_rebind_after_body_batch_split() {
        let mut builder = FrameBuilder::new();
        builder.push_pixel(Vec2::new(8.0, 2.0), 2.0, 1);
        for _ in 0..128 {
            builder.push_body(&pixel_body(), Transform::default());
        }
        builder.push_pixel(Vec2::new(-3.0, 9.0), 2.0, 2);
        let frame = builder.finish();
        assert_eq!(frame.batches.len(), 2);
        let batch = &frame.batches[1];
        let p = batch.pixels.last().unwrap();
        near(
            batch.transforms[p.transform_slot()].point(Vec2::from_array(p.local)),
            Vec2::new(-3.0, 9.0),
        );
    }

    #[test]
    fn exact_affine_hierarchy_survives_gpu_extraction() {
        let parent = Transform {
            position: Vec2::new(10.0, -4.0),
            rotation: 0.7,
            scale: Vec2::new(3.0, 0.5),
        };
        let child = Transform {
            position: Vec2::new(2.0, 1.0),
            rotation: -0.9,
            scale: Vec2::new(0.5, 2.0),
        };
        let t = parent.compose_affine(child);
        let mut builder = FrameBuilder::new();
        builder.push_body_affine(&pixel_body(), t);
        let gpu = builder.finish().batches[0].transforms[0];
        for corner in [Vec2::new(-0.5, -0.5), Vec2::new(0.5, 0.5)] {
            near(gpu.point(corner), parent.point(child.point(corner)));
        }
    }

    #[test]
    fn clear_removes_previous_frame_and_ignores_invalid_geometry() {
        let mut builder = FrameBuilder::new();
        for _ in 0..130 {
            builder.push_body(&pixel_body(), Transform::default());
        }
        builder.clear();
        builder.push_pixel(Vec2::ZERO, 0.0, 1);
        builder.push_pixel(Vec2::splat(f32::NAN), 1.0, 1);
        builder.push_body(&PixelBody::from_ascii(&[], &[]), Transform::default());
        builder.push_pixel(Vec2::new(3.0, 4.0), 1.0, 9);
        let frame = builder.finish();
        assert_eq!(frame.pixel_count, 1);
        assert_eq!(frame.batches.len(), 1);
        assert_eq!(frame.batches[0].pixels[0].color, 9);
    }

    #[test]
    fn borrowed_frames_reuse_pixel_and_transform_allocations() {
        let mut builder = FrameBuilder::new();
        for _ in 0..130 {
            builder.push_body(&pixel_body(), Transform::default());
        }
        let allocations: Vec<_> = builder
            .frame()
            .batches
            .iter()
            .map(|batch| {
                (
                    batch.pixels.as_ptr() as usize,
                    batch.transforms.as_ptr() as usize,
                )
            })
            .collect();
        for _ in 0..4 {
            builder.clear();
            assert_eq!(builder.frame().pixel_count, 0);
            assert!(builder.frame().batches.is_empty());
            for _ in 0..130 {
                builder.push_body(&pixel_body(), Transform::default());
            }
            let next: Vec<_> = builder
                .frame()
                .batches
                .iter()
                .map(|batch| {
                    (
                        batch.pixels.as_ptr() as usize,
                        batch.transforms.as_ptr() as usize,
                    )
                })
                .collect();
            assert_eq!(next, allocations);
            assert_eq!(builder.frame().pixel_count, 130);
        }
    }

    #[test]
    fn per_cell_animation_preserves_damage_and_base_palette() {
        let mut body = PixelBody::from_ascii(&["###"], &[('#', 0xff_abcdef)]);
        body.damage(Vec2::ZERO, 0.1);
        let mut builder = FrameBuilder::new();
        builder.push_body_colored(&body, Transform::default(), |local, color| {
            if local.x < 0.0 {
                color ^ 0x0000_00ff
            } else {
                color
            }
        });
        let first = builder.frame();
        assert_eq!(first.pixel_count, 2);
        assert_ne!(
            first.batches[0].pixels[0].color,
            first.batches[0].pixels[1].color
        );
        assert!(
            first.batches[0]
                .pixels
                .iter()
                .all(|pixel| pixel.local[0] != 0.0)
        );
        assert!(body.cells().all(|(_, color)| color == 0xff_abcdef));
        builder.clear();
        builder.push_body_colored(&body, Transform::default(), |_, color| color ^ 0x0000_ff00);
        assert_eq!(builder.frame().pixel_count, 2);
        assert!(
            builder.frame().batches[0]
                .pixels
                .iter()
                .all(|pixel| pixel.color == 0xff_ab32ef)
        );
    }

    #[test]
    fn soft_material_keeps_local_fractional_geometry_and_fixed_abi() {
        let transform = Transform {
            position: Vec2::new(999_000.0, -999_000.0),
            rotation: 0.7,
            scale: Vec2::new(0.2, -0.4),
        }
        .affine();
        let local = Vec2::new(0.05, -0.075);
        let mut builder = FrameBuilder::new();
        assert_eq!(
            builder.push_soft_cells_affine_material(
                transform,
                PixelMaterial {
                    light_response: 0.6,
                    emission: 0.75
                },
                [(local, 0xff_abcdef), (Vec2::ZERO, 0)],
            ),
            2
        );
        let batch = &builder.frame().batches[0];
        let original = GpuTransform::from(transform);
        assert_eq!(batch.transforms[0].a, original.a);
        assert_eq!(&batch.transforms[0].b[..2], &original.b[..2]);
        assert_eq!(&batch.transforms[0].b[2..], &[0.6, -1.75]);
        assert_eq!(batch.pixels[0].local, local.to_array());
        assert_eq!(batch.pixels[1].color, 0);
        assert_eq!(std::mem::size_of::<PixelInstance>(), 16);
        assert_eq!(std::mem::size_of::<GpuTransform>(), 32);
        let packed = crate::CameraView::new(transform.position, Vec2::new(533.0, 300.0), 0.6)
            .unwrap()
            .relative_transform(batch.transforms[0]);
        near(
            packed.point(local),
            transform.basis_x * local.x + transform.basis_y * local.y,
        );
    }

    #[test]
    fn soft_invalid_groups_and_empty_iterators_do_not_spend_slots() {
        let mut builder = FrameBuilder::new();
        for transform in [
            Transform {
                scale: Vec2::ZERO,
                ..Default::default()
            }
            .affine(),
            Transform {
                position: Vec2::splat(f32::NAN),
                ..Default::default()
            }
            .affine(),
            AffineTransform {
                position: Vec2::ZERO,
                basis_x: Vec2::X,
                basis_y: Vec2::X,
            },
            AffineTransform {
                position: Vec2::ZERO,
                basis_x: Vec2::new(4096.0, 4095.0),
                basis_y: Vec2::new(4095.0, 4094.0),
            },
        ] {
            assert_eq!(
                builder.push_soft_cells_affine(transform, [(Vec2::ZERO, 1)]),
                0
            );
        }
        assert_eq!(
            builder.push_soft_cells_affine(AffineTransform::default(), []),
            0
        );
        assert!(builder.frame().batches.is_empty());
        let tiny = Transform {
            scale: Vec2::splat(0.00005),
            ..Default::default()
        }
        .affine();
        assert_eq!(
            builder.push_soft_cells_affine_material(
                tiny,
                PixelMaterial {
                    light_response: f32::NAN,
                    emission: f32::INFINITY
                },
                [(Vec2::ZERO, 1), (Vec2::splat(f32::NAN), 2)]
            ),
            1
        );
        assert_eq!(
            &builder.frame().batches[0].transforms[0].b[2..],
            &[0.0, -1.0]
        );
        near(
            builder.frame().batches[0].transforms[0].point(Vec2::X),
            Vec2::new(0.00005, 0.0),
        );
    }

    #[test]
    fn soft_groups_split_and_reuse_without_changing_hard_materials() {
        let mut builder = FrameBuilder::new();
        for _ in 0..129 {
            builder.push_soft_cells_affine(AffineTransform::default(), [(Vec2::ZERO, 1)]);
        }
        builder.push_body(&pixel_body(), Transform::default());
        assert_eq!(builder.frame().pixel_count, 130);
        assert_eq!(builder.frame().batches[0].transforms.len(), 128);
        let batch = &builder.frame().batches[1];
        assert_eq!(batch.pixels[0].transform, 0);
        assert_eq!(batch.pixels[1].transform, 1);
        assert_eq!(&batch.transforms[1].b[2..], &[0.0, 0.0]);
        let allocation = batch.pixels.as_ptr();
        builder.clear();
        for _ in 0..129 {
            builder.push_soft_cells_affine(AffineTransform::default(), [(Vec2::ZERO, 1)]);
        }
        builder.push_body(&pixel_body(), Transform::default());
        assert_eq!(builder.frame().batches[1].pixels.as_ptr(), allocation);
    }
}
