//! Actual cached licensed-type cells, semantic tokens and extraction invariants.
use super::*;
use std::collections::HashSet;

fn input() -> DoraInput {
    DoraInput {
        parent: AffineTransform::default(),
        rect: [7.3, 9.7, 48.0, 64.0],
        clip: None,
        time: 0.0,
        reduced_motion: true,
        expression: DoraExpression::Neutral,
        speaking: false,
        disabled: false,
    }
}
fn extract(i: DoraInput, appearance: ProfileAppearance) -> (FrameBuilder, DoraStats) {
    let mut frame = FrameBuilder::new();
    let (mut pixels, mut work) = (60_000, 120_000);
    let stats = append_profile_portrait(&mut frame, &i, appearance, &mut pixels, &mut work);
    assert_eq!(
        (pixels, work),
        (60_000 - stats.quads, 120_000 - stats.quads)
    );
    assert_eq!(frame.frame().pixel_count, stats.quads);
    (frame, stats)
}
fn colors(frame: &FrameBuilder) -> Vec<u32> {
    frame
        .frame()
        .batches
        .iter()
        .flat_map(|b| b.pixels.iter().map(|p| p.color))
        .collect()
}
fn cells(body: &PixelBody) -> Vec<(u32, u32)> {
    body.cells()
        .map(|(p, _)| (p.x.to_bits(), p.y.to_bits()))
        .collect()
}
fn same(a: &FrameBuilder, b: &FrameBuilder) {
    assert_eq!(a.frame().pixel_count, b.frame().pixel_count);
    assert_eq!(a.frame().batches.len(), b.frame().batches.len());
    for (a, b) in a.frame().batches.iter().zip(&b.frame().batches) {
        assert_eq!(a.pixels, b.pixels);
        assert_eq!(a.transforms, b.transforms);
    }
}

#[test]
fn tokens_are_explicit_and_invalid_type_or_corporation_never_falls_back() {
    for kind in 0..3 {
        for main in 0..3 {
            for sub in 0..3 {
                assert!(ProfileAppearance::from_tokens(kind, main, sub).is_some());
            }
        }
    }
    for bad in [3, 9, 255] {
        assert!(ProfileAppearance::from_tokens(bad, 0, 1).is_none());
        assert!(ProfileAppearance::from_tokens(0, bad, 1).is_none());
        assert!(ProfileAppearance::from_tokens(0, 0, bad).is_none());
    }
}

#[test]
fn three_types_have_cached_distinct_authored_masks_and_disjoint_constant_state_layers() {
    let mut masks = HashSet::new();
    for kind in [PortraitKind::Dora, PortraitKind::Doro, PortraitKind::Talo] {
        let baseline = portrait_layers(kind, DoraExpression::Neutral, false, 0);
        assert!(
            masks.insert(cells(&baseline[0].body)),
            "type geometry was substituted"
        );
        assert!(baseline[0].hair.iter().any(|&v| v));
        assert!(baseline[0].horns.iter().any(|&v| v));
        assert!(
            baseline[0]
                .hair
                .iter()
                .zip(&baseline[0].horns)
                .all(|(&a, &b)| !(a && b))
        );
        assert!(std::ptr::eq(
            baseline[0],
            portrait_layers(kind, DoraExpression::Neutral, false, 0)[0]
        ));
        println!(
            "{kind:?}: body={} eyes={} mouth={} total={} hair={} horns={}",
            baseline[0].body.occupied_count(),
            baseline[1].body.occupied_count(),
            baseline[2].body.occupied_count(),
            baseline
                .iter()
                .map(|layer| layer.body.occupied_count())
                .sum::<usize>(),
            baseline[0].hair.iter().filter(|&&value| value).count(),
            baseline[0].horns.iter().filter(|&&value| value).count(),
        );
        for expression in [
            DoraExpression::Neutral,
            DoraExpression::Focused,
            DoraExpression::Concerned,
            DoraExpression::Pleased,
        ] {
            for blink in [false, true] {
                for mouth in 0..3 {
                    let layers = portrait_layers(kind, expression, blink, mouth);
                    let mut positions = HashSet::new();
                    for (i, layer) in layers.iter().enumerate() {
                        assert_eq!((layer.body.width, layer.body.height), (96, 128));
                        assert_eq!(cells(&layer.body), cells(&baseline[i].body));
                        for (p, color) in layer.body.cells() {
                            assert!(
                                positions.insert((p.x.to_bits(), p.y.to_bits())),
                                "overlapping type layers"
                            );
                            assert_eq!(color >> 24, 255);
                        }
                    }
                    assert!(positions.len() <= MAX_PORTRAIT_QUADS);
                }
            }
        }
    }
    assert_eq!(masks.len(), 3);
}

#[test]
fn all_27_profiles_recolor_only_main_hair_and_sub_horns_without_changing_geometry() {
    let mut distinct = HashSet::new();
    for kind in 0..3 {
        let appearance = ProfileAppearance::from_tokens(kind, 0, 1).unwrap();
        let (baseline, baseline_stats) = extract(input(), appearance);
        let reference = colors(&baseline);
        let body = portrait_layers(appearance.kind, DoraExpression::Neutral, false, 0)[0];
        for main in 0..3 {
            for sub in 0..3 {
                let (actual, stats) = extract(
                    input(),
                    ProfileAppearance::from_tokens(kind, main, sub).unwrap(),
                );
                assert_eq!(stats, baseline_stats);
                for (a, b) in actual.frame().batches.iter().zip(&baseline.frame().batches) {
                    assert_eq!(a.transforms, b.transforms);
                    assert_eq!(
                        a.pixels
                            .iter()
                            .map(|p| (p.local, p.transform))
                            .collect::<Vec<_>>(),
                        b.pixels
                            .iter()
                            .map(|p| (p.local, p.transform))
                            .collect::<Vec<_>>()
                    );
                }
                let actual_colors = colors(&actual);
                for (n, (cell, _)) in body.body.cells().enumerate() {
                    if actual_colors[n] != reference[n] {
                        let index = local_index(&body.body, cell);
                        assert!(body.hair[index] || body.horns[index]);
                        if main == 0 {
                            assert!(body.horns[index]);
                        }
                        if sub == 1 {
                            assert!(body.hair[index]);
                        }
                    }
                }
                assert_eq!(
                    actual_colors[body.body.occupied_count()..],
                    reference[body.body.occupied_count()..]
                );
                let signature = actual
                    .frame()
                    .batches
                    .iter()
                    .flat_map(|b| {
                        b.pixels
                            .iter()
                            .map(|p| (p.local[0].to_bits(), p.local[1].to_bits(), p.color))
                    })
                    .collect::<Vec<_>>();
                assert!(
                    distinct.insert(signature),
                    "profile palettes or type body duplicated"
                );
            }
        }
    }
    assert_eq!(distinct.len(), 27);
}

#[test]
fn fractional_fit_keeps_every_cell_and_profile_dora_is_exact_guide_when_pan_renko() {
    for kind in 0..3 {
        let appearance = ProfileAppearance::from_tokens(kind, 0, 1).unwrap();
        let (_, baseline) = extract(input(), appearance);
        for rect in [
            [0.0, 0.0, 96.0, 128.0],
            [7.3, 9.7, 48.125, 64.375],
            [7.0, 9.0, 4.8, 6.4],
            [-7.3, -9.7, 4.8, 6.4],
        ] {
            let (actual, stats) = extract(DoraInput { rect, ..input() }, appearance);
            assert_eq!(stats.quads, baseline.quads);
            assert_eq!(stats.visible_cells, stats.quads);
            assert_eq!(stats.scale, (rect[2] / 96.0).min(rect[3] / 128.0));
            assert!(
                actual
                    .frame()
                    .batches
                    .iter()
                    .all(|b| b.transforms.iter().all(|t| t.b[2..] == [0.0, 0.0]))
            );
        }
    }
    let mut guide = FrameBuilder::new();
    let (mut pixels, mut work) = (60_000, 120_000);
    let guide_stats = append_dora(&mut guide, &input(), &mut pixels, &mut work);
    let (profile, profile_stats) =
        extract(input(), ProfileAppearance::from_tokens(0, 0, 1).unwrap());
    assert_eq!(guide_stats, profile_stats);
    same(&guide, &profile);
}

#[test]
fn partial_cells_clip_before_reflected_sheared_parent_and_still_charge_both_budgets() {
    let i = DoraInput {
        parent: AffineTransform {
            position: Vec2::new(33.0, -17.0),
            basis_x: Vec2::new(-1.1, 0.3),
            basis_y: Vec2::new(0.25, 1.7),
        },
        clip: Some([10.25, 12.75, 21.125, 35.375]),
        ..input()
    };
    for kind in 0..3 {
        let (frame, stats) = extract(i, ProfileAppearance::from_tokens(kind, 2, 0).unwrap());
        assert!(stats.visible_cells > 0 && stats.visible_cells < stats.quads);
        let clip = i.clip.unwrap();
        for batch in &frame.frame().batches {
            for pixel in &batch.pixels {
                if pixel.color >> 24 == 0 {
                    continue;
                }
                let gpu = batch.transforms[pixel.transform as usize];
                for corner in [
                    Vec2::splat(-0.5),
                    Vec2::new(0.5, -0.5),
                    Vec2::new(-0.5, 0.5),
                    Vec2::splat(0.5),
                ] {
                    let p = i
                        .parent
                        .inverse_point(gpu.point(Vec2::from_array(pixel.local) + corner));
                    assert!(p.x >= clip[0] - 0.0001 && p.x <= clip[0] + clip[2] + 0.0001);
                    assert!(p.y >= clip[1] - 0.0001 && p.y <= clip[1] + clip[3] + 0.0001);
                }
            }
        }
    }
}

#[test]
fn each_type_animates_actual_facial_states_but_reduced_or_disabled_is_exact_static() {
    for kind in 0..3 {
        let appearance = ProfileAppearance::from_tokens(kind, 1, 2).unwrap();
        let mut poses = HashSet::new();
        for expression in [
            DoraExpression::Neutral,
            DoraExpression::Focused,
            DoraExpression::Concerned,
            DoraExpression::Pleased,
        ] {
            let (frame, _) = extract(
                DoraInput {
                    expression,
                    ..input()
                },
                appearance,
            );
            assert!(poses.insert(colors(&frame)));
        }
        for (reduced_motion, disabled) in [(true, false), (false, true)] {
            let i = DoraInput {
                reduced_motion,
                disabled,
                speaking: true,
                ..input()
            };
            let (a, sa) = extract(i, appearance);
            let (b, sb) = extract(DoraInput { time: 4.4, ..i }, appearance);
            assert_eq!(sa, sb);
            same(&a, &b);
        }
        let i = DoraInput {
            speaking: true,
            reduced_motion: false,
            ..input()
        };
        let (a, sa) = extract(i, appearance);
        let (b, sb) = extract(DoraInput { time: 0.15, ..i }, appearance);
        assert_eq!(sa.quads, sb.quads);
        assert_ne!(sa.mouth_frame, sb.mouth_frame);
        assert_ne!(colors(&a), colors(&b));
    }
}

#[test]
fn each_profile_group_rejects_atomically_for_budget_or_invalid_geometry() {
    for kind in 0..3 {
        let appearance = ProfileAppearance::from_tokens(kind, 0, 1).unwrap();
        let (_, baseline) = extract(input(), appearance);
        for (mut pixels, mut work) in [(baseline.quads - 1, 120_000), (60_000, baseline.quads - 1)]
        {
            let before = (pixels, work);
            let mut frame = FrameBuilder::new();
            frame.push_pixel(Vec2::ZERO, 1.0, ICON_SHELL);
            let stats =
                append_profile_portrait(&mut frame, &input(), appearance, &mut pixels, &mut work);
            assert!(stats.budget_rejected);
            assert_eq!(stats.requested_quads, baseline.quads);
            assert_eq!((pixels, work), before);
            assert_eq!(frame.frame().pixel_count, 1);
        }
        for i in [
            DoraInput {
                rect: [0.0, 0.0, f32::NAN, 64.0],
                ..input()
            },
            DoraInput {
                clip: Some([0.0, 0.0, 0.0, 64.0]),
                ..input()
            },
            DoraInput {
                parent: AffineTransform {
                    basis_x: Vec2::ZERO,
                    ..AffineTransform::default()
                },
                ..input()
            },
        ] {
            let (frame, stats) = extract(i, appearance);
            assert_eq!(stats, DoraStats::default());
            assert_eq!(frame.frame().pixel_count, 0);
        }
    }
}
