//! Actual native soft-material packets; no browser/GPU or visual acceptance claim.
use super::*;
use engine_core::{CameraView, GpuTransform, RenderFrame};

fn scene(reduced: bool) -> SceneInput {
    SceneInput {
        seed: 17,
        camera: Vec2::ZERO,
        viewport: Vec2::new(533.0, 300.0),
        time: 18.0,
        reduced,
        reduced_motion: false,
    }
}
fn nebula() -> Motif {
    Motif {
        kind: MotifKind::Nebula,
        origin: Vec2::ZERO,
        hash: 93,
        scale: 1.1,
    }
}
fn packets(frame: &RenderFrame) -> Vec<([f32; 2], u32, GpuTransform)> {
    frame
        .batches
        .iter()
        .flat_map(|b| {
            b.pixels
                .iter()
                .map(|p| (p.local, p.color, b.transforms[p.transform_slot()]))
        })
        .collect()
}

#[test]
fn deep_cloud_cache_has_real_coherent_volume_and_bounded_complete_cost() {
    for reduced in [false, true] {
        let required = motif_cost(MotifKind::Nebula, reduced);
        let soft = GAS_ART.veil.body(reduced).occupied_count()
            + GAS_ART.billows.body(reduced).occupied_count();
        eprintln!(
            "Deep nebula reduced={reduced}: veil{}, billows{}, hard{}, whole{required}/4 transforms",
            GAS_ART.veil.body(reduced).occupied_count(),
            GAS_ART.billows.body(reduced).occupied_count(),
            required - soft
        );
        assert!(soft > if reduced { 100 } else { 400 });
        assert!(required <= if reduced { 1_100 } else { 2_200 });
        for (pair, spacing, threshold) in [
            (&GAS_ART.veil, VEIL_SPACING, VEIL_THRESHOLD),
            (&GAS_ART.billows, BILLOW_SPACING, BILLOW_THRESHOLD),
        ] {
            let s = spacing * if reduced { 2.0 } else { 1.0 };
            assert_eq!(pair.full.width, pair.low.width * 2);
            assert_eq!(pair.full.height, pair.low.height * 2);
            assert!(
                pair.body(reduced)
                    .cells()
                    .all(|(p, _)| gas_density(p * s) > threshold)
            );
            assert!(!pair.body(reduced).is_occupied(0, 0));
        }
    }
}

#[test]
fn cloud_has_diffuse_outskirts_dark_sky_and_unequal_connected_billow_outline() {
    // Retain a connected central mass without requiring closed dark pockets
    // along one arbitrary axis. The entire sampled field must have both a
    // diffuse transition and ample surrounding negative space.
    assert!(gas_density(Vec2::ZERO) > 0.27);
    let mut bridges = 0;
    let mut asymmetric = 0;
    for x in -45..=45 {
        let p = Vec2::new(x as f32, 0.0);
        bridges += usize::from(gas_density(p) > 0.27);
        asymmetric += usize::from(
            (gas_density(p + Vec2::Y * 12.0) - gas_density(p - Vec2::Y * 12.0)).abs() > 0.1,
        );
    }
    assert!(bridges >= 20, "actual central bridges={bridges}");
    assert!(asymmetric >= 15, "actual asymmetric outline={asymmetric}");
    let mut diffuse = 0;
    let mut dense = 0;
    let mut dark = 0;
    for y in -40..=40 {
        for x in -60..=60 {
            let value = gas_density(Vec2::new(x as f32, y as f32));
            assert!(value.is_finite() && (0.0..=1.0).contains(&value));
            diffuse += usize::from((VEIL_THRESHOLD..BILLOW_THRESHOLD).contains(&value));
            dense += usize::from(value >= BILLOW_THRESHOLD);
            dark += usize::from(value < VEIL_THRESHOLD);
        }
    }
    eprintln!("Nebula source field diffuse={diffuse}, dense={dense}, sky={dark}");
    assert!(
        diffuse > dense,
        "diffuse transition should dominate bright source knots"
    );
    assert!(
        dense >= 300 && dark > 121 * 81 / 2,
        "substantial knots inside predominantly dark space"
    );
    for reduced in [false, true] {
        let spacing = VEIL_SPACING * if reduced { 2.0 } else { 1.0 };
        assert!(GAS_ART.veil.body(reduced).cells().any(|(p, _)| {
            let physical = p * spacing;
            physical.y.abs() < 4.0 && physical.x.abs() < 6.0
        }));
    }
}

#[test]
fn dust_negative_space_is_open_not_large_enclosed_oval_cavities() {
    // A visual failure oracle independent of the authored path coordinates:
    // flood dark source samples from outside. Isolated fine dust flecks are
    // allowed, but a conspicuous enclosed void must open into surrounding sky.
    const WIDTH: usize = 121;
    const HEIGHT: usize = 81;
    let dark: Vec<_> = (0..WIDTH * HEIGHT)
        .map(|i| {
            gas_density(Vec2::new(
                (i % WIDTH) as f32 - 60.0,
                (i / WIDTH) as f32 - 40.0,
            )) < 0.12
        })
        .collect();
    let mut visited = vec![false; dark.len()];
    let mut enclosed = Vec::new();
    for start in 0..dark.len() {
        if !dark[start] || visited[start] {
            continue;
        }
        let mut pending = vec![start];
        visited[start] = true;
        let mut area = 0;
        let mut border = false;
        while let Some(i) = pending.pop() {
            area += 1;
            let (x, y) = (i % WIDTH, i / WIDTH);
            border |= x == 0 || y == 0 || x == WIDTH - 1 || y == HEIGHT - 1;
            for next in [
                x.checked_sub(1).map(|x| y * WIDTH + x),
                (x + 1 < WIDTH).then_some(i + 1),
                y.checked_sub(1).map(|y| y * WIDTH + x),
                (y + 1 < HEIGHT).then_some(i + WIDTH),
            ]
            .into_iter()
            .flatten()
            {
                if dark[next] && !visited[next] {
                    visited[next] = true;
                    pending.push(next);
                }
            }
        }
        if !border {
            enclosed.push(area);
        }
    }
    eprintln!("Nebula enclosed dark source regions (one-world-unit samples): {enclosed:?}");
    assert!(
        enclosed.iter().all(|&area| area <= 48),
        "large closed cavity {enclosed:?}"
    );
}

#[test]
fn deep_cloud_whole_group_rejects_one_short_and_crosses_transform_batch_safely() {
    for reduced in [false, true] {
        let required = motif_cost(MotifKind::Nebula, reduced);
        let mut frame = FrameBuilder::new();
        for _ in 0..127 {
            frame.push_cells_affine_material(
                AffineTransform::default(),
                PixelMaterial::default(),
                [(Vec2::ZERO, 0xffffffff)],
            );
        }
        let before = packets(frame.frame());
        let mut short = required - 1;
        assert_eq!(
            draw_motif(&mut frame, nebula(), &scene(reduced), &mut short),
            0
        );
        assert_eq!(short, required - 1);
        assert_eq!(before, packets(frame.frame()));
        let mut exact = required;
        assert_eq!(
            draw_motif(&mut frame, nebula(), &scene(reduced), &mut exact),
            required
        );
        assert_eq!(exact, 0);
        assert_eq!(frame.frame().pixel_count, required + 127);
        assert_eq!(frame.frame().batches.len(), 2);
        assert_eq!(frame.frame().batches[0].transforms.len(), 128);
        assert_eq!(frame.frame().batches[1].transforms.len(), 3);
        assert!(frame.frame().batches.iter().all(|b| {
            b.pixels
                .iter()
                .all(|p| (p.transform_slot()) < b.transforms.len())
        }));
    }
}

#[test]
fn gas_volume_local_cells_compose_reflection_and_shear_without_minimum_size() {
    let parent = AffineTransform {
        position: Vec2::new(12.3, -7.4),
        basis_x: Vec2::new(-0.021, 0.003),
        basis_y: Vec2::new(0.009, 0.013),
    };
    let style = CellStyle {
        time: 18.0,
        phase: 0.7,
        cell_scale: 0.5,
        tint: TINTS[1],
        alpha: 0.9,
        curl: 0.14,
    };
    let mut frame = FrameBuilder::new();
    let body = GAS_ART.billows.body(false);
    draw_gas_part(&mut frame, body, parent, BILLOW_SPACING, style, 1);
    let batch = &frame.frame().batches[0];
    assert_eq!(frame.frame().pixel_count, body.occupied_count());
    let t = batch.transforms[0];
    assert_eq!(t.b[2], 0.0);
    assert!((t.b[3] + 1.09).abs() < 0.00001);
    let pose = Transform {
        position: Vec2::new(1.5, -1.0),
        rotation: (style.time * 0.025 + style.phase).sin() * -0.012,
        scale: Vec2::splat(BILLOW_SPACING * 2.4),
    }
    .affine();
    for (p, (source, _)) in batch.pixels.iter().zip(body.cells()) {
        let physical = source * BILLOW_SPACING;
        let local =
            (physical + gas_flow(physical, style.time, style.phase, 1)) / (BILLOW_SPACING * 2.4);
        assert_eq!(p.local, local.to_array());
        for corner in [
            Vec2::new(-0.5, -0.5),
            Vec2::new(0.5, -0.5),
            Vec2::new(0.5, 0.5),
            Vec2::new(-0.5, 0.5),
        ] {
            assert!(
                t.point(local + corner)
                    .distance(parent.point(pose.point(local + corner)))
                    < 0.000003
            );
        }
    }
    for zoom in [0.6, 1.0, 1.6] {
        let view = CameraView::new(Vec2::ZERO, Vec2::new(848.0, 390.0), zoom).unwrap();
        let local = Vec2::from_array(batch.pixels[0].local);
        let width = view
            .world_to_screen(t.point(local + Vec2::X * 0.5))
            .distance(view.world_to_screen(t.point(local - Vec2::X * 0.5)));
        assert!((width - parent.vector(pose.vector(Vec2::X)).length() * zoom).abs() < 0.00005);
        assert!(width < 0.25);
    }
}

#[test]
fn cloud_layers_breathe_continuously_and_reduced_motion_is_exact() {
    for reduced in [false, true] {
        let extract = |time, reduced_motion| {
            let mut f = FrameBuilder::new();
            draw_motif(
                &mut f,
                nebula(),
                &SceneInput {
                    time,
                    reduced_motion,
                    ..scene(reduced)
                },
                &mut 8_000,
            );
            f.finish()
        };
        let a = extract(20.0, false);
        assert_eq!(packets(&a), packets(&extract(20.0, false)));
        let b = extract(20.001, false);
        for (x, y) in packets(&a).iter().zip(packets(&b)) {
            assert!(
                x.2.point(Vec2::from_array(x.0))
                    .distance(y.2.point(Vec2::from_array(y.0)))
                    < 0.002
            );
            assert!(
                x.1.to_le_bytes()
                    .iter()
                    .zip(y.1.to_le_bytes())
                    .all(|(a, b)| a.abs_diff(b) <= 1)
            );
        }
        assert_ne!(packets(&a), packets(&extract(35.0, false)));
        assert_eq!(
            packets(&extract(20.0, true)),
            packets(&extract(901.0, true))
        );
        let soft: Vec<_> = a
            .batches
            .iter()
            .flat_map(|b| &b.transforms)
            .filter(|t| t.b[3] < 0.0)
            .collect();
        assert_eq!(soft.len(), 2);
        assert!(soft.iter().all(|t| t.b[2] == 0.0));
    }
}

#[test]
fn deep_cloud_streamed_camera_motion_keeps_birth_field_and_single_parallax_subtraction() {
    let seed = crate::Profile::default().seed;
    let m = (-16..=16)
        .flat_map(|y| (-16..=16).filter_map(move |x| motif(seed, [x, y])))
        .filter(|m| m.kind == MotifKind::Nebula)
        .min_by(|a, b| {
            a.origin
                .length_squared()
                .total_cmp(&b.origin.length_squared())
        })
        .unwrap();
    for reduced in [false, true] {
        let camera = m.origin / MOTIF_PARALLAX;
        let delta = Vec2::new(3.25, -1.75);
        let extract = |camera| {
            let mut f = FrameBuilder::new();
            draw_motif(
                &mut f,
                m,
                &SceneInput {
                    seed,
                    camera,
                    ..scene(reduced)
                },
                &mut 8_000,
            );
            f.finish()
        };
        let a = extract(camera);
        let b = extract(camera + delta);
        for ((local, color, t), (next_local, next_color, next_t)) in
            packets(&a).iter().zip(packets(&b))
        {
            assert_eq!(*local, next_local);
            assert_eq!(*color, next_color);
            assert_eq!(t.a[2..], next_t.a[2..]);
            assert_eq!(t.b, next_t.b);
            let before = t.point(Vec2::from_array(*local)) - camera;
            let after = next_t.point(Vec2::from_array(next_local)) - (camera + delta);
            assert!(
                (after - before + delta * MOTIF_PARALLAX)
                    .abs()
                    .max_element()
                    < 0.008
            );
        }
        for zoom in [0.6, 1.0, 1.6] {
            let mut f = FrameBuilder::new();
            let s = SceneInput {
                seed,
                camera,
                viewport: Vec2::new(533.0, 300.0) / zoom,
                ..scene(reduced)
            };
            let stats = render(&mut f, s);
            assert_eq!(
                stats.transforms,
                f.frame()
                    .batches
                    .iter()
                    .map(|b| b.transforms.len())
                    .sum::<usize>()
            );
            assert!(f.frame().pixel_count <= if reduced { 4_000 } else { 8_000 });
            let needle = packets(&a);
            assert!(
                packets(f.frame())
                    .windows(needle.len())
                    .any(|p| p == needle)
            );
        }
    }
}

#[test]
#[ignore = "explicit native packet export; requires owned output directory"]
fn export_actual_deep_nebula_packets() {
    let output = std::path::PathBuf::from(
        std::env::var_os("DEEP_NEBULA_PROOF_DIR").expect("explicit owned output path"),
    );
    assert!(output.is_absolute());
    std::fs::create_dir_all(&output).unwrap();
    let seed = crate::Profile::default().seed;
    let m = (-16..=16)
        .flat_map(|y| (-16..=16).filter_map(move |x| motif(seed, [x, y])))
        .filter(|m| m.kind == MotifKind::Nebula)
        .min_by(|a, b| {
            a.origin
                .length_squared()
                .total_cmp(&b.origin.length_squared())
        })
        .unwrap();
    let camera = m.origin / MOTIF_PARALLAX;
    for reduced in [false, true] {
        for time in [0.0, 18.0, 36.0] {
            let mut f = FrameBuilder::new();
            let count = draw_motif(
                &mut f,
                m,
                &SceneInput {
                    seed,
                    camera,
                    time,
                    ..scene(reduced)
                },
                &mut 8_000,
            );
            let views:Vec<_>=[0.6,1.0,1.6].into_iter().map(|zoom|{
                let view=CameraView::new(camera,Vec2::new(256.0,180.0),zoom).unwrap();
                let cells:Vec<_>=f.frame().batches.iter().flat_map(|b|b.pixels.iter().map(|p|{
                    let t=b.transforms[p.transform_slot()];
                    let center=view.world_to_screen(t.point(Vec2::from_array(p.local)));
                    let basis_x=Vec2::new(t.a[2],t.a[3])*zoom;
                    let basis_y=Vec2::new(t.b[0],t.b[1])*zoom;
                    serde_json::json!({"center":center.to_array(),"basis_x":basis_x.to_array(),"basis_y":basis_y.to_array(),"material":[t.b[2],t.b[3]],"rgba":p.color.to_le_bytes()})
                })).collect();
                serde_json::json!({"zoom":zoom,"cells":cells})
            }).collect();
            let data = serde_json::json!({"scope":"actual native FrameBuilder/CameraView cells; software radial material model, not WGSL/GPU acceptance","seed":seed,"camera":camera.to_array(),"hash":m.hash,"reduced":reduced,"time":time,"quads":count,"transforms":4,"logical":[256,180],"views":views});
            std::fs::write(
                output.join(format!("nebula-low{reduced}-time{time}.json")),
                serde_json::to_vec(&data).unwrap(),
            )
            .unwrap();
        }
    }
}
