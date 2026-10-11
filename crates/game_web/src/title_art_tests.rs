use super::*;
use crate::ui_scene::{UiGroup, UiNode, UiScene};
use engine_core::RenderFrame;

fn input(rect: [f32; 4], time: f32, reduced: bool) -> Input {
    Input {
        parent: AffineTransform::default(),
        rect,
        clip: None,
        time,
        reduced,
        disabled: false,
    }
}
fn extract(input: &Input) -> RenderFrame {
    let mut frame = FrameBuilder::new();
    let (mut pixels, mut work) = (60_000, 120_000);
    assert_eq!(
        append(&mut frame, input, &mut pixels, &mut work),
        OCCUPIED_CELLS
    );
    assert_eq!(
        (pixels, work),
        (60_000 - OCCUPIED_CELLS, 120_000 - OCCUPIED_CELLS)
    );
    frame.finish()
}
fn signature(frame: &RenderFrame) -> Vec<([f32; 2], u32, [f32; 4])> {
    frame
        .batches
        .iter()
        .flat_map(|b| {
            b.pixels.iter().map(|p| {
                let t = b.transforms[p.transform as usize];
                (
                    t.point(Vec2::from_array(p.local)).to_array(),
                    p.color,
                    [t.a[2], t.a[3], t.b[0], t.b[1]],
                )
            })
        })
        .collect()
}

#[test]
fn title_native_source_roles_counts_and_materials_match_exact_pack() {
    assert_eq!(
        ART.parts
            .iter()
            .map(|p| p.body.occupied_count())
            .sum::<usize>(),
        41662
    );
    assert_eq!(
        ART.parts
            .iter()
            .map(|p| p.body.occupied_count())
            .sum::<usize>()
            + ART.star.occupied_count() * 3,
        OCCUPIED_CELLS
    );
    let frame = extract(&input([0.0, 0.0, 512.0, 192.0], 0.0, true));
    assert_eq!(frame.pixel_count, OCCUPIED_CELLS);
    assert_eq!(
        frame
            .batches
            .iter()
            .map(|b| b.transforms.len())
            .sum::<usize>(),
        12
    );
    let mut cells = signature(&frame).into_iter();
    for part in &ART.parts {
        for (index, cell, color) in part.body.cells_indexed() {
            let (world, actual, basis) = cells.next().unwrap();
            assert_eq!(actual, color);
            assert_eq!(basis, [1.0, 0.0, 0.0, 1.0]);
            assert!(
                (Vec2::from_array(world) - (cell + part.offset + Vec2::new(256.0, 96.0))).length()
                    < 0.00001
            );
            assert!(part.roles[index] <= 2);
        }
    }
    assert!(
        frame
            .batches
            .iter()
            .all(|b| b.transforms.iter().all(|t| t.b[2..] == [0.0, 0.0]))
    );
}

#[test]
fn title_relative_crystal_never_moves_letter_corners_or_duplicates_them() {
    let still = signature(&extract(&input([0.0, 0.0, 512.0, 192.0], 0.0, false)));
    let moving = signature(&extract(&input([0.0, 0.0, 512.0, 192.0], 2.0, false)));
    // Actual L and second A corners border the crystal mount. The initial
    // broad crown crop included these; their geometry must stay in static tiles.
    for (x, y) in [
        (215.0, 48.0),
        (294.0, 48.0),
        (216.0, 55.0),
        (285.0, 55.0),
        (211.0, 75.0),
    ] {
        let indices: Vec<_> = still
            .iter()
            .enumerate()
            .filter(|(_, cell)| cell.0 == [x + 0.5, y + 0.5])
            .map(|(i, _)| i)
            .collect();
        assert_eq!(indices.len(), 1);
        assert_eq!(still[indices[0]].0, moving[indices[0]].0);
    }
    let crystal = still
        .iter()
        .position(|cell| cell.0 == [256.5, 20.5])
        .unwrap();
    assert_eq!(moving[crystal].0[0], still[crystal].0[0]);
    assert!((moving[crystal].0[1] - still[crystal].0[1] - 0.6).abs() < 0.00001);
}

#[test]
fn title_tiny_and_fractional_fits_keep_every_source_cell_without_lod() {
    for rect in [
        [0.0, 0.0, 512.0, 192.0],
        [3.125, 7.375, 217.25, 81.46875],
        [0.0, 0.0, 51.2, 19.2],
        [1.125, 2.375, 5.12, 1.92],
        [0.0, 0.0, 1024.0, 400.0],
    ] {
        let frame = extract(&input(rect, 1.3, false));
        assert_eq!(frame.pixel_count, OCCUPIED_CELLS);
        assert_eq!(
            signature(&frame)
                .iter()
                .filter(|(_, c, _)| c >> 24 != 0)
                .count(),
            OCCUPIED_CELLS
        );
        let scale = (rect[2] / 512.0).min(rect[3] / 192.0);
        for (_, _, basis) in signature(&frame) {
            assert_eq!(basis, [scale, 0.0, 0.0, scale]);
        }
    }
}

#[test]
fn title_real_partial_clip_checks_all_four_corners_under_reflection_and_shear() {
    let parent = AffineTransform {
        position: Vec2::new(91.0, -27.0),
        basis_x: Vec2::new(-1.3, 0.4),
        basis_y: Vec2::new(0.25, 0.8),
    };
    let clip = [83.125, 21.25, 105.75, 59.375];
    let mut settings = input([11.0, 7.0, 256.0, 96.0], 2.25, false);
    settings.parent = parent;
    settings.clip = Some(clip);
    let frame = extract(&settings);
    let mut visible = 0;
    for batch in &frame.batches {
        for p in &batch.pixels {
            if p.color >> 24 == 0 {
                continue;
            }
            visible += 1;
            let t = batch.transforms[p.transform as usize];
            for corner in [
                Vec2::splat(-0.5),
                Vec2::new(0.5, -0.5),
                Vec2::new(-0.5, 0.5),
                Vec2::splat(0.5),
            ] {
                let point = parent.inverse_point(t.point(Vec2::from_array(p.local) + corner));
                assert!(
                    point.x >= clip[0] - 0.00004
                        && point.y >= clip[1] - 0.00004
                        && point.x <= clip[0] + clip[2] + 0.00004
                        && point.y <= clip[1] + clip[3] + 0.00004
                );
            }
        }
    }
    assert!(visible > 1000 && visible < OCCUPIED_CELLS);
}

#[test]
fn title_rejection_leaves_frame_and_both_quotas_untouched() {
    for (pixels, work) in [(OCCUPIED_CELLS - 1, 120000), (60000, OCCUPIED_CELLS - 1)] {
        let mut frame = FrameBuilder::new();
        frame.push_pixel(Vec2::new(4.0, 9.0), 1.0, 0xffff_ffff);
        let before = signature(frame.frame());
        let (mut p, mut w) = (pixels, work);
        assert_eq!(
            append(
                &mut frame,
                &input([0.0, 0.0, 512.0, 192.0], 0.0, true),
                &mut p,
                &mut w
            ),
            0
        );
        assert_eq!(signature(frame.frame()), before);
        assert_eq!((p, w), (pixels, work));
    }
    for mut invalid in [
        input([0.0, 0.0, 0.0, 192.0], 0.0, true),
        input([0.0, 0.0, f32::NAN, 192.0], 0.0, true),
        input([0.0, 0.0, 512.0, 192.0], 0.0, true),
    ] {
        if invalid.rect[2] == 512.0 {
            invalid.parent.position = Vec2::new(1_100_000.0, 1_100_000.0);
            invalid.parent.basis_x = Vec2::splat(64.0);
        }
        let mut frame = FrameBuilder::new();
        let (mut p, mut w) = (60000, 120000);
        assert_eq!(append(&mut frame, &invalid, &mut p, &mut w), 0);
        assert_eq!((p, w, frame.frame().pixel_count), (60000, 120000, 0));
    }
}

#[test]
fn title_animation_is_relative_masked_bounded_and_reduced_motion_is_static() {
    let baseline = extract(&input([0.0, 0.0, 512.0, 192.0], 0.0, false));
    let later = extract(&input([0.0, 0.0, 512.0, 192.0], 3.2, false));
    assert_ne!(signature(&baseline), signature(&later));
    let before = signature(&baseline);
    let after = signature(&later);
    let mut cursor = 0;
    let mut changed = 0;
    for (n, part) in ART.parts.iter().enumerate() {
        for (i, _, _) in part.body.cells_indexed() {
            if part.roles[i] == 0 {
                assert_eq!(before[cursor].1, after[cursor].1);
            }
            if n < 8 {
                assert_eq!(before[cursor].0, after[cursor].0);
            }
            if before[cursor].1 != after[cursor].1 {
                changed += 1;
            }
            cursor += 1;
        }
    }
    println!("TITLE changed mask cells={changed} of41662; neutral mask remains exact");
    assert!(changed > 100 && changed < OCCUPIED_CELLS / 6);
    let still = signature(&extract(&input([0.0, 0.0, 512.0, 192.0], 0.0, true)));
    for t in [3.2, 120.0, f32::MAX, f32::NAN] {
        assert_eq!(
            signature(&extract(&input([0.0, 0.0, 512.0, 192.0], t, true))),
            still
        );
    }
    assert_eq!(
        signature(&extract(&input([0.0, 0.0, 512.0, 192.0], f32::NAN, false))),
        before
    );
}

#[test]
fn title_complete_group_crosses_transform_batch_boundary_without_loss() {
    let mut frame = FrameBuilder::new();
    for i in 0..125 {
        frame.push_cells_affine_material(
            Transform {
                position: Vec2::new(i as f32, 0.0),
                ..Transform::default()
            }
            .affine(),
            Default::default(),
            [(Vec2::ZERO, 0xff_ffffff)],
        );
    }
    let (mut p, mut w) = (60000, 120000);
    assert_eq!(
        append(
            &mut frame,
            &input([0.0, 0.0, 512.0, 192.0], 1.0, false),
            &mut p,
            &mut w
        ),
        OCCUPIED_CELLS
    );
    assert_eq!(frame.frame().pixel_count, OCCUPIED_CELLS + 125);
    assert_eq!(frame.frame().batches.len(), 2);
    assert!(
        frame
            .frame()
            .batches
            .iter()
            .all(|b| b.transforms.len() <= 128)
    );
}

fn title_scene() -> UiScene {
    let make = |kind: &str, rect, text: &str| UiNode {
        kind: kind.into(),
        rect,
        text: text.into(),
        font: 1.0,
        ..UiNode::default()
    };
    UiScene {
        groups: vec![UiGroup {
            position: [0.0; 2],
            nodes: vec![
                make("title-backdrop", [0.0, 0.0, 640.0, 360.0], ""),
                make("logo", [32.0, 25.0, 576.0, 216.0], "GALAXY ROYALE"),
                make("text", [32.0, 8.0, 576.0, 9.0], "AN ENDLESS PIXEL FRONTIER"),
                make(
                    "text",
                    [32.0, 246.0, 576.0, 9.0],
                    "LEAVE THE SANCTUARY. BREAK THE BELT.",
                ),
                make(
                    "primary-button",
                    [160.0, 267.0, 320.0, 24.0],
                    "LAUNCH EXPEDITION",
                ),
                make("button", [160.0, 299.0, 150.0, 20.0], "SETTINGS"),
                make("quit-icon", [330.0, 299.0, 20.0, 20.0], "QUIT"),
                make(
                    "text",
                    [32.0, 329.0, 576.0, 9.0],
                    "LEFT CLICK / SET COURSE    RIGHT HOLD / AIM + FIRE",
                ),
                make(
                    "text",
                    [32.0, 344.0, 576.0, 9.0],
                    "PLAYABLE SLICE / LOCAL BUILD",
                ),
            ],
        }],
    }
}

#[test]
fn title_real_ui_route_retains_all_copy_controls_and_every_mark_cell() {
    let scene = title_scene();
    let mut joined = FrameBuilder::new();
    crate::ui_scene::draw(&mut joined, &scene, [0.0; 2], [640.0, 360.0], 2.25, false);
    let actual = signature(joined.frame());
    let mut expected = vec![];
    for node in scene.groups.into_iter().next().unwrap().nodes {
        let isolated = UiScene {
            groups: vec![UiGroup {
                position: [0.0; 2],
                nodes: vec![node],
            }],
        };
        let mut frame = FrameBuilder::new();
        crate::ui_scene::draw(&mut frame, &isolated, [0.0; 2], [640.0, 360.0], 2.25, false);
        assert!(frame.frame().pixel_count > 0);
        expected.extend(signature(frame.frame()));
    }
    assert_eq!(actual, expected);
    assert!(joined.frame().pixel_count < 60000);
    println!(
        "TITLE joined native cells={} transforms={} logo={}",
        joined.frame().pixel_count,
        joined
            .frame()
            .batches
            .iter()
            .map(|b| b.transforms.len())
            .sum::<usize>(),
        OCCUPIED_CELLS
    );
}

#[test]
fn title_native_packet_export_is_optional_work_only_and_keeps_source_cells() {
    for (label, time, reduced) in [
        ("still", 0.0, true),
        ("scan", 3.2, false),
        ("crown", 6.0, false),
    ] {
        let frame = extract(&input([0.0, 0.0, 512.0, 192.0], time, reduced));
        assert_eq!(frame.pixel_count, OCCUPIED_CELLS);
        if let Some(folder) = std::env::var_os("GALAXY_TITLE_REVIEW_OUTPUT") {
            let folder = std::path::PathBuf::from(folder);
            std::fs::create_dir_all(&folder).unwrap();
            let value = serde_json::json!({"scope":"actual native FrameBuilder extraction; software review only, no GPU observation","time":time,"reduced":reduced,"source_size":SOURCE_SIZE,"cells":frame.pixel_count,"batches":frame.batches.iter().map(|b|serde_json::json!({"transforms":b.transforms.iter().map(|t|serde_json::json!({"a":t.a,"b":t.b})).collect::<Vec<_>>(),"pixels":b.pixels.iter().map(|p|serde_json::json!({"local":p.local,"color":p.color,"transform":p.transform})).collect::<Vec<_>>()})).collect::<Vec<_>>()});
            std::fs::write(
                folder.join(format!("{label}.json")),
                serde_json::to_vec(&value).unwrap(),
            )
            .unwrap();
        }
    }
}
