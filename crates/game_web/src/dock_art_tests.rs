use super::*;
use engine_core::{CameraView, RenderFrame};
use game_core::{Game, Profile};

fn input() -> DockInput {
    DockInput {
        parent: Transform::default(),
        corporation: None,
        time: 0.0,
        reduced_motion: false,
    }
}
fn draw(input: DockInput) -> (RenderFrame, DockStats) {
    let mut frame = FrameBuilder::new();
    let mut budget = DockBudget::default();
    let stats = append(&mut frame, &input, &mut budget).unwrap();
    assert_eq!(
        budget,
        DockBudget {
            cells: 32000 - CELL_COST,
            transforms: 0
        }
    );
    (frame.finish(), stats)
}
type CellPacket = ([f32; 2], u32, [f32; 4], [f32; 4]);
fn packet(frame: &RenderFrame) -> Vec<CellPacket> {
    frame
        .batches
        .iter()
        .flat_map(|batch| {
            batch.pixels.iter().map(|pixel| {
                let t = batch.transforms[pixel.transform_slot()];
                (pixel.local, pixel.color, t.a, t.b)
            })
        })
        .collect()
}
#[test]
fn dock_art_emits_exact_28549_cells_16_transforms_for_all_main_corps_and_poses() {
    let mut minimum = Vec2::splat(f32::INFINITY);
    let mut maximum = Vec2::splat(f32::NEG_INFINITY);
    for corporation in [
        None,
        Some(Corporation::Pan),
        Some(Corporation::Renko),
        Some(Corporation::Hugo),
    ] {
        for (state, time) in [0.0, 0.7, 1.4].into_iter().enumerate() {
            let (frame, stats) = draw(DockInput {
                corporation,
                time,
                ..input()
            });
            assert_eq!(
                stats,
                DockStats {
                    cells: CELL_COST,
                    transforms: 16,
                    state
                }
            );
            assert_eq!(frame.pixel_count, CELL_COST);
            assert_eq!(
                frame
                    .batches
                    .iter()
                    .map(|b| b.transforms.len())
                    .sum::<usize>(),
                TRANSFORM_COST
            );
            let mut unique = std::collections::HashSet::new();
            for batch in &frame.batches {
                for pixel in &batch.pixels {
                    let t = batch.transforms[pixel.transform_slot()];
                    let center = t.point(Vec2::from_array(pixel.local));
                    assert!(
                        unique.insert((
                            (center.x * 4.0).round() as i32,
                            (center.y * 4.0).round() as i32
                        )),
                        "tile/role emitted twice"
                    );
                    assert_eq!(t.a[2], 0.5);
                    assert_eq!(t.b[1], 0.5);
                    assert!(pixel.color >> 24 == 255);
                    let half = Vec2::splat(0.25);
                    minimum = minimum.min(center - half);
                    maximum = maximum.max(center + half);
                }
            }
        }
    }
    println!(
        "Dock source bounds {:?}..{:?}; host1.5 bounds {:?}..{:?}; cost28549/16",
        minimum,
        maximum,
        minimum * 1.5,
        maximum * 1.5
    );
    assert!(minimum.min_element() >= -64.0 && maximum.max_element() <= 64.0);
}

#[test]
fn dock_art_only_signals_recolor_and_only_external_lamps_animate() {
    let pack: Pack = serde_json::from_str(include_str!(
        "../../../assets/pipeline/pan-coop-dock/native256/pack.json"
    ))
    .unwrap();
    let mut changed = [0, 0];
    let mut recolored = [0, 0];
    for (index, tile) in pack.pixel_rig_tiles.tiles.iter().enumerate() {
        for state in 0..3 {
            let sprite = &pack.sprites[&tile.frame_sprites[state]];
            for (cell, _, color) in ART.tiles[index].bodies[0][state].cells_indexed() {
                assert_eq!(color, pack.palette[sprite.indices[cell] as usize]);
                let baseline = pack.sprites[&tile.frame_sprites[0]].indices[cell];
                if sprite.indices[cell] != baseline {
                    let global =
                        (cell / SIDE + (index / 2) * SIDE) * 256 + cell % SIDE + (index % 2) * SIDE;
                    assert!(pack.pixel_rig_tiles.lamp_cells_source.contains(&global));
                    if state > 0 {
                        changed[state - 1] += 1;
                    }
                }
            }
            for corporation in 1..3 {
                for (cell, _, color) in ART.tiles[index].bodies[corporation][state].cells_indexed()
                {
                    let original = pack.palette[sprite.indices[cell] as usize];
                    if tile.roles[cell] != 3 {
                        assert_eq!(
                            color, original,
                            "neutral alloy/deck/cyan instrument recolored"
                        );
                    } else if state == 0 && color != original {
                        recolored[corporation - 1] += 1;
                    }
                }
            }
        }
    }
    assert!(changed.into_iter().all(|count| count > 1000));
    assert_eq!(recolored, [1925, 1925]);
}

#[test]
fn dock_art_reduced_motion_freezes_complete_geometry_color_and_state() {
    for corporation in [
        Some(Corporation::Pan),
        Some(Corporation::Renko),
        Some(Corporation::Hugo),
    ] {
        let baseline = packet(
            &draw(DockInput {
                corporation,
                reduced_motion: true,
                ..input()
            })
            .0,
        );
        for time in [0.7, 1.4, 29.6, f32::NAN, f32::INFINITY] {
            let (frame, stats) = draw(DockInput {
                corporation,
                time,
                reduced_motion: true,
                ..input()
            });
            assert_eq!(stats.state, 0);
            assert_eq!(packet(&frame), baseline);
        }
    }
}

#[test]
fn dock_art_preserves_fractional_reflected_nonuniform_cells_and_zoom_projection() {
    for camera in [Vec2::ZERO, Vec2::new(999000.0, -999000.0)] {
        let parent = Transform {
            position: camera + Vec2::new(3.5, -1.25),
            scale: Vec2::new(-0.31, 0.73),
            rotation: 0.37,
        };
        let (frame, _) = draw(DockInput { parent, ..input() });
        for zoom in [0.6, 1.6] {
            let view = CameraView::new(camera, Vec2::new(533.0, 300.0), zoom).unwrap();
            for batch in &frame.batches {
                for pixel in &batch.pixels {
                    let t = view.relative_transform(batch.transforms[pixel.transform_slot()]);
                    // Relative camera extraction preserves the fine basis even far
                    // from origin, without a hard-cell raster minimum.
                    assert!((Vec2::new(t.a[2], t.a[3]).length() - 0.155).abs() < 0.00001);
                    assert!((Vec2::new(t.b[0], t.b[1]).length() - 0.365).abs() < 0.00001);
                    for corner in [
                        Vec2::splat(-0.5),
                        Vec2::new(0.5, -0.5),
                        Vec2::new(-0.5, 0.5),
                        Vec2::splat(0.5),
                    ] {
                        let projected = t.point(Vec2::from_array(pixel.local) + corner) * zoom
                            + Vec2::new(533.0, 300.0) * 0.5;
                        assert!(projected.is_finite());
                    }
                }
            }
        }
    }
}

#[test]
fn dock_art_short_quota_and_invalid_parent_never_append_or_debit() {
    for variant in 0..7 {
        let mut i = input();
        let mut budget = DockBudget::default();
        match variant {
            0 => budget.cells = CELL_COST - 1,
            1 => budget.transforms = 15,
            2 => i.parent.position.x = f32::NAN,
            3 => i.parent.position.x = 1_100_001.0,
            4 => i.parent.rotation = f32::INFINITY,
            5 => i.parent.scale.y = 0.0,
            _ => i.parent.scale.x = 65.0,
        }
        let before = budget;
        let mut frame = FrameBuilder::new();
        frame.push_pixel(Vec2::ZERO, 1.0, 0xffaabbcc);
        let previous = packet(frame.frame());
        assert_eq!(append(&mut frame, &i, &mut budget), None);
        assert_eq!(packet(frame.frame()), previous);
        assert_eq!(budget, before);
    }
}

#[test]
fn dock_art_actual_game_hook_preserves_fallback_order_facilities_and_profile() {
    let mut game = Game::new(Profile::default());
    game.set_viewport(533.0, 300.0);
    let profile = serde_json::to_string(game.profile()).unwrap();
    let mut baseline = FrameBuilder::new();
    game.render_into(1.0, &mut baseline);
    let mut rejected = FrameBuilder::new();
    game.render_into_with_home(1.0, &mut rejected, |frame, parent, time| {
        let mut budget = DockBudget {
            cells: CELL_COST - 1,
            ..Default::default()
        };
        append(
            frame,
            &DockInput {
                parent,
                time,
                ..input()
            },
            &mut budget,
        )
        .is_some()
    });
    assert_eq!(packet(baseline.frame()), packet(rejected.frame()));
    let mut accepted = FrameBuilder::new();
    let mut start = 0;
    let mut expected = Vec::new();
    game.render_into_with_home(1.0, &mut accepted, |frame, parent, time| {
        assert_eq!(parent.position, Vec2::ZERO);
        assert_eq!(parent.scale, Vec2::splat(1.5));
        start = frame.frame().pixel_count;
        expected = packet(
            &draw(DockInput {
                parent,
                time,
                ..input()
            })
            .0,
        );
        append(
            frame,
            &DockInput {
                parent,
                time,
                ..input()
            },
            &mut DockBudget::default(),
        )
        .is_some()
    });
    let actual = packet(accepted.frame());
    assert_eq!(actual[start..start + CELL_COST], expected);
    assert!(
        actual.len() > start + CELL_COST,
        "gameplay objects remain above home artwork"
    );
    for (id, center) in [
        [-120.0, -80.0],
        [120.0, -80.0],
        [-120.0, 95.0],
        [120.0, 95.0],
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(game.dock_facility_at(center), Some(id as u8));
        // Current dock art ends at96 on each axis; the actual facility cells
        // lie within13 of center. All four semantic structures remain clear.
        assert!(center[0].abs() - 13.0 > 96.0);
    }
    assert_eq!(serde_json::to_string(game.profile()).unwrap(), profile);
    game.diagnostic("warp 10000 10000");
    let mut calls = 0;
    game.render_into_with_home(1.0, &mut accepted, |_, _, _| {
        calls += 1;
        true
    });
    assert_eq!(calls, 0, "invisible home artwork should not extract");
}
