use super::*;
use crate::{
    FrameBuilder, GpuTransform, PixelBody, PixelInstance, PixelMaterial, PixelRig, PixelRigPart,
    PixelRigRole, PixelRigState, Transform, Vec2,
};

fn normal(v: [f32; 3]) -> [f32; 3] {
    let length = v.iter().map(|n| n * n).sum::<f32>().sqrt();
    v.map(|n| n / length)
}
fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f32>()
        .sqrt()
}
fn body() -> PixelBody {
    PixelBody::from_indexed_rgba(3, 2, &[[0, 0xffaabbcc], [2, 0], [4, 0xff112233]], 6).unwrap()
}

#[test]
fn octahedral_axes_both_hemispheres_and_dense_directions_round_trip() {
    for axis in [
        [1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, -1.0],
    ] {
        assert!(distance(Surface::new(axis, 0).unwrap().normal(), axis) < 0.016);
    }
    for x in -10..=10 {
        for y in -10..=10 {
            for z in -10..=10 {
                let v = [x as f32, y as f32, z as f32];
                if v == [0.0; 3] {
                    continue;
                }
                let surface = Surface::new(v, 37).unwrap();
                assert!(distance(surface.normal(), normal(v)) < 0.017, "{v:?}");
                assert_eq!(surface.emission_mask(), 37);
                assert_eq!(surface.packed() & PIXEL_TRANSFORM_SLOT_MASK, 0);
            }
        }
    }
}
#[test]
fn invalid_normals_fail_and_finite_extreme_magnitudes_normalize_safely() {
    for v in [
        [0.0; 3],
        [f32::NAN, 0.0, 1.0],
        [0.0, f32::INFINITY, 1.0],
        [0.0, 1.0, f32::NEG_INFINITY],
    ] {
        assert!(Surface::new(v, 255).is_err());
    }
    for magnitude in [f32::MAX, f32::MIN_POSITIVE, f32::from_bits(1), 1e-20] {
        let surface = Surface::new([magnitude, -magnitude, magnitude], 255).unwrap();
        assert!(distance(surface.normal(), normal([1.0, -1.0, 1.0])) < 0.017);
    }
}
#[test]
fn surface_and_slot_127_share_the_exact_existing_sixteen_byte_cell_layout() {
    let surface = Surface::new([0.0, 0.0, -1.0], 255).unwrap();
    assert_eq!(surface.packed(), 0xffff_ff80);
    let cell = PixelInstance {
        local: [1.25, -2.5],
        color: 0x12345678,
        transform: 127 | surface.packed(),
    };
    assert_eq!(cell.transform_slot(), 127);
    assert_eq!(cell.surface(), Some(surface));
    assert_eq!(std::mem::size_of::<Surface>(), 4);
    assert_eq!(std::mem::size_of::<PixelInstance>(), 16);
    assert_eq!(std::mem::size_of::<GpuTransform>(), 32);
    let words: &[u32] = bytemuck::cast_slice(std::slice::from_ref(&cell));
    assert_eq!(
        words,
        &[
            1.25_f32.to_bits(),
            (-2.5_f32).to_bits(),
            0x12345678,
            u32::MAX
        ]
    );
    let legacy = PixelInstance {
        transform: 127,
        ..cell
    };
    assert_eq!(legacy.surface(), None);
    assert_eq!(legacy.transform_slot(), 127);
}
#[test]
fn absent_maps_allocate_no_surface_and_import_rejections_keep_the_old_map() {
    let mut body = body();
    assert!(body.surfaces().is_none());
    let surface = Surface::new([0.2, -0.3, 1.0], 91).unwrap();
    let map = [surface; 6];
    assert!(body.set_surfaces(&map[..5], 6).is_err());
    assert!(body.surfaces().is_none());
    assert!(body.set_surfaces(&map, 5).is_err());
    assert!(body.surfaces().is_none());
    body.set_surfaces(&map, 6).unwrap();
    let pointer = body.surfaces().unwrap().as_ptr();
    assert!(body.set_surfaces(&map[..5], 6).is_err());
    assert_eq!(body.surfaces().unwrap().as_ptr(), pointer);
    let clone = body.clone();
    assert_eq!(clone.surfaces(), body.surfaces());
    assert_ne!(clone.surfaces().unwrap().as_ptr(), pointer);
    body.width = usize::MAX;
    assert!(body.set_surfaces(&map, usize::MAX).is_err());
    assert_eq!(body.surfaces().unwrap().as_ptr(), pointer);
    body.clear_surfaces();
    assert!(body.surfaces().is_none());
}
#[test]
fn mapped_body_destruction_and_alpha_zero_keep_occupancy_as_only_collision_source() {
    let mut mapped = body();
    let mut plain = mapped.clone();
    let surfaces: Vec<_> = (0..6)
        .map(|i| Surface::new([i as f32 / 8.0, 0.0, 1.0], i * 35).unwrap())
        .collect();
    mapped.set_surfaces(&surfaces, 6).unwrap();
    assert_eq!(
        mapped.cells_indexed().collect::<Vec<_>>(),
        plain.cells_indexed().collect::<Vec<_>>()
    );
    assert_eq!(mapped.surface_index(1), None);
    assert_eq!(mapped.surface_index(2), Some(surfaces[2]));
    assert_eq!(mapped.color_at(2, 0), Some(0));
    assert_eq!(
        mapped.hit_segment(Vec2::new(1.0, -2.0), Vec2::new(1.0, 2.0)),
        plain.hit_segment(Vec2::new(1.0, -2.0), Vec2::new(1.0, 2.0))
    );
    assert_eq!(
        mapped.damage(Vec2::new(1.0, -0.5), 0.1),
        plain.damage(Vec2::new(1.0, -0.5), 0.1)
    );
    assert_eq!(mapped.surface_index(2), None);
    assert_eq!(mapped.connected_components(), plain.connected_components());
    let mut a = FrameBuilder::new();
    let mut b = FrameBuilder::new();
    a.push_body(&mapped, Transform::default());
    b.push_body(&plain, Transform::default());
    let a = a.finish();
    let b = b.finish();
    assert_eq!(a.pixel_count, 2);
    for (a, b) in a.batches[0].pixels.iter().zip(&b.batches[0].pixels) {
        assert_eq!(a.local, b.local);
        assert_eq!(a.color, b.color);
        assert_eq!(a.transform_slot(), b.transform_slot());
        assert!(a.surface().is_some());
        assert!(b.surface().is_none());
    }
}
#[test]
fn surface_body_and_streamed_subobjects_keep_color_callback_and_batch_slots_independent() {
    let mut body = body();
    let map = [Surface::new([0.0, 0.0, 1.0], 255).unwrap(); 6];
    body.set_surfaces(&map, 6).unwrap();
    let mut frame = FrameBuilder::new();
    for _ in 0..129 {
        frame.push_body_colored_material(
            &body,
            Transform::default(),
            PixelMaterial {
                light_response: 0.7,
                emission: 0.1,
            },
            |_, c| c ^ 0x00ff0000,
        );
    }
    assert_eq!(frame.frame().batches.len(), 2);
    for batch in &frame.frame().batches {
        for p in &batch.pixels {
            assert!(p.transform_slot() < batch.transforms.len());
            assert_eq!(p.surface(), Some(map[0]));
            assert!(
                body.cells()
                    .any(|(local, c)| local.to_array() == p.local && (c ^ 0x00ff0000) == p.color)
            );
            assert_eq!(batch.transforms[p.transform_slot()].b[2..], [0.7, 0.1]);
        }
    }
    assert!(
        frame.frame().batches[0]
            .pixels
            .iter()
            .any(|p| p.transform_slot() == 127)
    );
    let emitted = frame.push_cells_affine_surface_material(
        AffineTransform::default(),
        PixelMaterial::default(),
        [
            (Vec2::ZERO, 12, Some(map[0])),
            (Vec2::ONE, 34, None),
            (Vec2::splat(f32::NAN), 56, Some(map[0])),
        ],
    );
    assert_eq!(emitted, 2);
    let pixels = &frame.frame().batches.last().unwrap().pixels;
    assert_eq!(pixels[pixels.len() - 2].surface(), Some(map[0]));
    assert!(pixels.last().unwrap().surface().is_none());
}
#[test]
fn uniform_scaling_rotation_and_reflection_preserve_expected_three_dimensional_normals() {
    let surface = Surface::new([0.5, -0.25, 1.0], 0).unwrap();
    let n = surface.normal();
    for scale in [0.000001, 0.6, 1.0, 1.6, 1000.0, f32::MAX] {
        let result = surface
            .world_normal(
                Transform {
                    scale: Vec2::splat(scale),
                    ..Transform::default()
                }
                .affine(),
            )
            .unwrap();
        assert!(distance(result, n) < 0.000001);
    }
    let rotated = surface
        .world_normal(
            Transform {
                rotation: std::f32::consts::FRAC_PI_2,
                ..Transform::default()
            }
            .affine(),
        )
        .unwrap();
    assert!(distance(rotated, [-n[1], n[0], n[2]]) < 0.000001);
    let reflected = surface
        .world_normal(
            Transform {
                scale: Vec2::new(-1.0, 1.0),
                ..Transform::default()
            }
            .affine(),
        )
        .unwrap();
    assert!(distance(reflected, [-n[0], n[1], n[2]]) < 0.000001);
}
#[test]
fn inverse_transpose_nonuniform_nested_shear_is_orthogonal_to_transformed_tangents() {
    let surface = Surface::new([0.4, -0.7, 1.0], 0).unwrap();
    let parent = Transform {
        rotation: 0.63,
        scale: Vec2::new(3.0, 0.4),
        ..Transform::default()
    };
    let child = Transform {
        rotation: -0.82,
        scale: Vec2::new(0.7, 2.3),
        ..Transform::default()
    };
    let matrix = parent.compose_affine(child);
    let determinant = matrix.basis_x.perp_dot(matrix.basis_y);
    let n = surface.normal();
    let world = surface.world_normal(matrix).unwrap();
    for tangent in [[n[2], 0.0, -n[0]], [0.0, n[2], -n[1]]] {
        let xy = matrix.vector(Vec2::new(tangent[0], tangent[1]));
        let transformed = [xy.x, xy.y, tangent[2] * determinant.abs().sqrt()];
        let dot = world
            .iter()
            .zip(transformed)
            .map(|(a, b)| a * b)
            .sum::<f32>();
        assert!(dot.abs() < 0.000001);
    }
    let composed = surface.world_normal(matrix).unwrap();
    let sequential = Surface::new(surface.world_normal(child.affine()).unwrap(), 0)
        .unwrap()
        .world_normal(parent.affine())
        .unwrap();
    // Intermediate re-encoding quantizes; full composition is the exact path.
    assert!(distance(composed, sequential) < 0.03);
}
#[test]
fn singular_nonfinite_and_extreme_bases_are_explicit_without_a_pixel_size_floor() {
    let surface = Surface::new([0.3, 0.2, 1.0], 1).unwrap();
    for matrix in [
        Transform {
            scale: Vec2::new(0.0, 1.0),
            ..Transform::default()
        }
        .affine(),
        AffineTransform {
            basis_y: Vec2::splat(f32::NAN),
            ..AffineTransform::default()
        },
        AffineTransform {
            basis_x: Vec2::X,
            basis_y: Vec2::X,
            ..AffineTransform::default()
        },
    ] {
        assert!(surface.world_normal(matrix).is_none());
    }
    for scale in [f32::from_bits(1), f32::MIN_POSITIVE] {
        assert!(
            distance(
                surface
                    .world_normal(
                        Transform {
                            scale: Vec2::splat(scale),
                            ..Transform::default()
                        }
                        .affine()
                    )
                    .unwrap(),
                surface.normal()
            ) < 0.000001
        );
    }
    let extreme = surface
        .world_normal(
            Transform {
                scale: Vec2::new(f32::MAX, f32::MIN_POSITIVE),
                ..Transform::default()
            }
            .affine(),
        )
        .unwrap();
    assert!(extreme.iter().all(|n| n.is_finite()));
}
#[test]
fn rig_partition_preserves_surface_fields_after_holes_and_actual_color_animation() {
    let mut body = PixelBody::from_ascii(&["###"], &[('#', 0xff224466)]);
    let map = [
        Surface::new([0.5, 0.0, 1.0], 0).unwrap(),
        Surface::new([0.0, 0.0, 1.0], 80).unwrap(),
        Surface::new([-0.5, 0.0, 1.0], 255).unwrap(),
    ];
    body.set_surfaces(&map, 3).unwrap();
    let rig = PixelRig::new(
        3,
        1,
        vec![0, 1, 1],
        vec![
            PixelRigPart {
                name: "hull".into(),
                parent: None,
                position: [0.0; 2],
                role: PixelRigRole::Armor,
            },
            PixelRigPart {
                name: "core".into(),
                parent: Some(0),
                position: [1.0, 0.0],
                role: PixelRigRole::Core,
            },
        ],
    )
    .unwrap();
    body.damage(Vec2::ZERO, 0.1);
    let parent = Transform {
        rotation: 0.6,
        scale: Vec2::new(2.0, 0.5),
        ..Transform::default()
    }
    .affine();
    let mut frame = FrameBuilder::new();
    assert_eq!(
        rig.draw_affine(
            &mut frame,
            &body,
            parent,
            PixelRigState {
                time: 1.0,
                charge: 0.7,
                core_open: true,
                hit_flash: 0.2
            }
        ),
        2
    );
    for batch in &frame.frame().batches {
        for cell in &batch.pixels {
            let world = batch.transforms[cell.transform_slot()].point(Vec2::from_array(cell.local));
            let original = parent.try_inverse_point(world).unwrap();
            let index = if original.x < 0.0 { 0 } else { 2 };
            assert_eq!(cell.surface(), Some(map[index]));
            assert_ne!(cell.color, 0xff224466);
        }
    }
}

#[test]
#[ignore = "explicit native material reference exporter; requires GALAXY_SURFACE_FIXTURES"]
fn export_actual_native_surface_reference() {
    let directory = std::path::PathBuf::from(
        std::env::var("GALAXY_SURFACE_FIXTURES").expect("explicit directory"),
    );
    std::fs::create_dir_all(&directory).unwrap();
    let parent = Transform {
        rotation: 0.63,
        scale: Vec2::new(3.0, 0.4),
        ..Transform::default()
    };
    let child = Transform {
        rotation: -0.82,
        scale: Vec2::new(0.7, 2.3),
        ..Transform::default()
    };
    let transforms = [
        AffineTransform::default(),
        Transform {
            scale: Vec2::splat(0.6),
            ..Transform::default()
        }
        .affine(),
        Transform {
            scale: Vec2::splat(1.6),
            ..Transform::default()
        }
        .affine(),
        Transform {
            rotation: 0.75,
            ..Transform::default()
        }
        .affine(),
        Transform {
            scale: Vec2::new(-1.0, 1.0),
            ..Transform::default()
        }
        .affine(),
        parent.compose_affine(child),
    ];
    let cases:Vec<_> = [([0.0,0.0,1.0],255),([0.4,-0.7,1.0],0),([-0.2,0.5,1.0],64),([0.0,0.0,-1.0],255),([1.0,0.0,0.0],128),([0.0,-1.0,0.0],7)].into_iter().map(|(normal,mask)|{
        let surface = Surface::new(normal,mask).unwrap();
        serde_json::json!({"normal":normal,"linearEmissionMask":mask,"packed":surface.packed(),"octX":(surface.packed()>>7)&255,"octY":(surface.packed()>>15)&255,"decodedNormal":surface.normal(),"slot127Word":surface.packed()|127,"transforms":transforms.iter().map(|t|serde_json::json!({"basisX":t.basis_x.to_array(),"basisY":t.basis_y.to_array(),"worldNormal":surface.world_normal(*t).unwrap()})).collect::<Vec<_>>()})
    }).collect();
    let fixture = serde_json::json!({"version":1,"scope":"Actual native packed Surface and CPU normal oracle; no shader execution, artwork or browser proof.","axes":"+X right/+Y down/+Z toward viewer","normalEmbedding":"diag(A,sqrt(abs(detA))) inverse transpose","mask":"Linear multiplier of existing material emission;255 preserve,0 block; no automatic light","cellBytes":std::mem::size_of::<PixelInstance>(),"transformBytes":std::mem::size_of::<GpuTransform>(),"cases":cases});
    std::fs::write(
        directory.join("native-surface-reference.json"),
        serde_json::to_string_pretty(&fixture).unwrap() + "\n",
    )
    .unwrap();
    println!("actual native surface cases=6 transform_oracles=36 cell_bytes=16 affine_bytes=32");
}
