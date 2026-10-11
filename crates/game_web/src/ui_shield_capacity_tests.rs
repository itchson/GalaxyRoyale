//! Capacity and actual emitted-quad proofs; no browser or recharge assertions.
use super::*;

type Cell = (u32, [Vec2; 4]);

fn packet(frame: &FrameBuilder) -> Vec<Cell> {
    frame
        .frame()
        .batches
        .iter()
        .flat_map(|batch| {
            batch.pixels.iter().map(|pixel| {
                let t = batch.transforms[pixel.transform as usize];
                assert_eq!(t.b[2..], [0.0; 2], "native shield remains unlit");
                (
                    pixel.color,
                    [
                        Vec2::splat(-0.5),
                        Vec2::new(-0.5, 0.5),
                        Vec2::new(0.5, -0.5),
                        Vec2::splat(0.5),
                    ]
                    .map(|corner| t.point(Vec2::from_array(pixel.local) + corner)),
                )
            })
        })
        .collect()
}

fn draw(capacity: f32, fill: f32, trail: f32) -> FrameBuilder {
    let mut frame = FrameBuilder::new();
    append_curved_vitals(
        &mut frame,
        AffineTransform::default(),
        [0.0, 0.0, 44.0, 12.0],
        VitalsPresentation {
            fill: [1.0, fill],
            trail: [1.0, trail],
        },
        [MINT, CYAN],
        [220.0, capacity],
    );
    frame
}

fn shield_active_right(frame: &FrameBuilder) -> f32 {
    packet(frame)
        .into_iter()
        .skip(154)
        .filter(|cell| cell.0 >> 24 == 0xff)
        .flat_map(|cell| cell.1.map(|p| p.x))
        .fold(f32::NEG_INFINITY, f32::max)
}

#[test]
fn actual_hud_draw_uses_three_base_capacity_marks_and_preserves_hull_cells() {
    let mut hud = game_core::Game::new(game_core::Profile::default()).hud();
    hud.dock = false;
    hud.max_hull = 220.0;
    hud.hull = hud.max_hull;
    hud.max_shield = 30.0;
    hud.shield = 30.0;
    let layout = FlightStatusLayout {
        viewport: [533.0, 300.0],
        vitals: Some([0.0, 0.0, 44.0, 12.0]),
        strip: None,
        stats: None,
        context: None,
        navigation: None,
    };
    let mut frame = FrameBuilder::new();
    // Camera at viewport/2 puts the actual UI origin at zero.
    draw_flight_status(
        &mut frame,
        &hud,
        [266.5, 150.0],
        &layout,
        None,
        0.0,
        true,
        false,
    );
    assert_eq!(frame.frame().pixel_count, 154 + 3 * 14);
    assert_eq!(&packet(&frame)[..154], packet(&draw(0.0, 1.0, 1.0)));
    let background = packet(&frame)
        .into_iter()
        .skip(154)
        .filter(|cell| cell.0 >> 24 == 0x90)
        .collect::<Vec<_>>();
    assert_eq!(background.len(), 3 * 7);
    for (_, corners) in background {
        assert!(corners.iter().all(|p| p.x >= 16.5 && p.x <= 27.5));
    }
}

#[test]
fn final_capacity_and_charge_slices_represent_actual_hit_points_separately() {
    // The sixth58HP mark holds8HP, and loses3HP without changing its capacity.
    let full = draw(58.0, 1.0, 1.0);
    let damaged = draw(58.0, 55.0 / 58.0, 1.0);
    let sixty = draw(60.0, 1.0, 1.0);
    assert!((shield_active_right(&sixty) - shield_active_right(&full) - 0.6).abs() < 0.0001);
    assert!((shield_active_right(&full) - shield_active_right(&damaged) - 0.9).abs() < 0.0001);
    let backgrounds = |f: &FrameBuilder| {
        packet(f)
            .into_iter()
            .skip(154)
            .filter(|cell| cell.0 >> 24 == 0x90)
            .collect::<Vec<_>>()
    };
    assert_eq!(backgrounds(&full), backgrounds(&damaged));
    assert!(packet(&damaged).iter().any(|cell| cell.0 >> 24 == 0xa0));
    assert!(packet(&full).iter().all(|cell| cell.0 >> 24 != 0xa0));
    // At58 max,50 current has exactly five full charges and an empty8HP mark.
    assert_eq!(
        shield_active_right(&draw(58.0, 50.0 / 58.0, 50.0 / 58.0)),
        29.5
    );
}

#[test]
fn current_sovereign_182_hp_has_nineteen_marks_with_true_two_hp_final_capacity() {
    let full = draw(182.0, 1.0, 1.0);
    let one_hp_lost = draw(182.0, 181.0 / 182.0, 1.0);
    let complete_nineteen = draw(190.0, 1.0, 1.0);
    let scale = 11.0 / 19.0;
    let background = |f: &FrameBuilder| {
        packet(f)
            .into_iter()
            .skip(154)
            .filter(|cell| cell.0 >> 24 == 0x90)
            .collect::<Vec<_>>()
    };
    // Eighteen 7-cell 10HP shapes plus two cells trimmed to 20% width. Geometry
    // is measured from the actual packet, not inferred from a rounded count.
    assert_eq!(background(&full).len(), 18 * 7 + 2);
    assert_eq!(full.frame().pixel_count, 154 + (18 * 7 + 2) * 2);
    assert_eq!(background(&full), background(&one_hp_lost));
    assert!(
        (shield_active_right(&complete_nineteen) - shield_active_right(&full) - 2.4 * scale).abs()
            < 0.0001
    );
    assert!(
        (shield_active_right(&full) - shield_active_right(&one_hp_lost) - 0.3 * scale).abs()
            < 0.0001
    );
    assert_eq!(&packet(&full)[..154], &packet(&draw(0.0, 1.0, 1.0)));
    assert!(packet(&one_hp_lost).iter().any(|cell| cell.0 >> 24 == 0xa0));
}

#[test]
fn valid_capacity_extremes_have_clear_gaps_and_bounded_nested_affine_corners() {
    let parent = Transform {
        position: Vec2::new(99.25, -33.5),
        rotation: 0.3,
        scale: Vec2::new(-0.8, 1.4),
    }
    .affine()
    .compose(
        Transform {
            rotation: -0.5,
            scale: Vec2::new(1.2, 0.7),
            ..Transform::default()
        }
        .affine(),
    );
    let rect = [7.5, 13.25, 44.0, 12.0];
    for capacity in [
        30.0, 50.0, 58.0, 120.0, 132.0, 142.0, 172.0, 180.0, 182.0, 190.0,
    ] {
        let count = (capacity / 10.0_f32).ceil() as usize;
        let scale = (11.0 / count as f32).min(1.0);
        let mut frame = FrameBuilder::new();
        append_curved_vitals(
            &mut frame,
            parent,
            rect,
            VitalsPresentation {
                fill: [1.0; 2],
                trail: [1.0; 2],
            },
            [MINT, CYAN],
            [220.0, capacity],
        );
        assert!(frame.frame().pixel_count <= 420);
        assert_eq!(frame.frame().batches.len(), 1);
        for (_, corners) in packet(&frame) {
            for p in corners {
                let p = parent.try_inverse_point(p).unwrap();
                assert!(
                    p.x >= rect[0] - 0.0001
                        && p.x <= rect[0] + rect[2] + 0.0001
                        && p.y >= rect[1] - 0.0001
                        && p.y <= rect[1] + rect[3] + 0.0001
                );
            }
        }
        // Assert actual emitted background quads stay outside each gap, rather
        // than checking only count/pitch from the construction formula.
        let plain = packet(&draw(capacity, 0.0, 0.0));
        let shield = &plain[154..];
        let span = (count as f32 * 4.0 - 1.0) * scale;
        let origin = (44.0 - span) * 0.5;
        for i in 0..count - 1 {
            let gap = [
                origin + (i as f32 * 4.0 + 3.0) * scale,
                origin + (i as f32 + 1.0) * 4.0 * scale,
            ];
            assert!(gap[1] > gap[0]);
            for (_, corners) in shield {
                let lo = corners.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
                let hi = corners
                    .iter()
                    .map(|p| p.x)
                    .fold(f32::NEG_INFINITY, f32::max);
                assert!(hi <= gap[0] + 0.0001 || lo >= gap[1] - 0.0001);
            }
        }
        println!(
            "capacity={capacity} chunks={count} quads={} transforms={}",
            frame.frame().pixel_count,
            frame.frame().batches[0].transforms.len()
        );
    }
}

#[test]
fn invalid_or_zero_capacity_is_absent_and_above_bound_work_is_fixed() {
    let hull = packet(&draw(0.0, 1.0, 1.0));
    assert_eq!(hull.len(), 154);
    for capacity in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0] {
        assert_eq!(packet(&draw(capacity, 1.0, 1.0)), hull);
    }
    assert_eq!(
        packet(&draw(f32::MAX, 1.0, 1.0)),
        packet(&draw(190.0, 1.0, 1.0))
    );
    for fill in [0.0, 0.001, 0.599, 0.601, 1.0, f32::NAN] {
        let frame = draw(182.0, fill, 1.0);
        assert!(frame.frame().pixel_count <= 426);
        assert_eq!(frame.frame().batches.len(), 1);
        assert!(
            packet(&frame)
                .iter()
                .flat_map(|c| c.1)
                .all(|p| p.is_finite())
        );
    }
    assert_ne!(
        packet(&draw(58.0, 0.599, 0.599)),
        packet(&draw(58.0, 0.601, 0.601))
    );
}

#[test]
fn capacity_upgrade_resets_phantom_damage_and_actual_shield_damage_still_holds() {
    let mut hud = game_core::Game::new(game_core::Profile::default()).hud();
    hud.dock = false;
    hud.max_shield = 30.0;
    hud.shield = 30.0;
    let mut animation = VitalsAnimation::default();
    animation.advance(&hud, 0.0, false);
    hud.max_shield = 50.0;
    let upgrade = animation.advance(&hud, 0.05, false);
    assert_eq!(upgrade.fill[1], 0.6);
    assert_eq!(upgrade.trail[1], 0.6, "no invented lost20HP tail");
    hud.shield = 20.0;
    let damaged = animation.advance(&hud, 0.02, false);
    assert_eq!(damaged.fill[1], 0.4);
    assert_eq!(damaged.trail[1], 0.6);
    assert!(
        packet(&draw(50.0, damaged.fill[1], damaged.trail[1]))
            .iter()
            .any(|cell| cell.0 >> 24 == 0xa0)
    );
    for _ in 0..30 {
        animation.advance(&hud, 0.1, false);
    }
    let settled = animation.advance(&hud, 0.0, false);
    assert_eq!(settled.fill, settled.trail);
    hud.shield = 45.0;
    let healing = animation.advance(&hud, 0.05, false);
    assert!(healing.fill[1] > 0.4 && healing.fill[1] < 0.9);
    let reduced = animation.advance(&hud, 0.05, true);
    assert_eq!(reduced.fill[1], 0.9);
    assert_eq!(reduced.fill, reduced.trail);
    assert_eq!(reduced, animation.advance(&hud, 99.0, true));
    hud.campaign.selected_ship = 1;
    assert_eq!(animation.advance(&hud, 0.0, false).fill[1], 0.9);
    hud.hull = 0.0;
    hud.shield = 0.0;
    assert_eq!(animation.advance(&hud, 0.0, false).trail, [0.0; 2]);
}

#[test]
fn shorter_and_compressed_shield_tracks_never_overlap_actual_hull_cells() {
    let bounds = |corners: &[Vec2; 4]| {
        let lo = corners
            .iter()
            .copied()
            .fold(Vec2::splat(f32::INFINITY), Vec2::min);
        let hi = corners
            .iter()
            .copied()
            .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
        (lo, hi)
    };
    for capacity in [
        30.0, 0.01, 10.0, 20.0, 50.0, 58.0, 110.0, 120.0, 132.0, 142.0, 172.0, 180.0, 182.0, 190.0,
    ] {
        let frame = draw(capacity, 1.0, 1.0);
        let cells = packet(&frame);
        let hull = &cells[..154];
        let shield = &cells[154..];
        for (_, shield_cell) in shield {
            let (slo, shi) = bounds(shield_cell);
            for (_, hull_cell) in hull {
                let (hlo, hhi) = bounds(hull_cell);
                assert!(
                    !(hlo.x < shi.x && hhi.x > slo.x && hlo.y < shi.y && hhi.y > slo.y),
                    "capacity {capacity}: hull {hlo:?}..{hhi:?} overlaps shield {slo:?}..{shi:?}"
                );
            }
        }
    }
}

#[test]
fn starter_hull_is_five_capacity_marks_and_upgrades_add_real_capacity() {
    let mut hud = game_core::Game::new(game_core::Profile::default()).hud();
    assert_eq!(hud.max_hull, 100.0);
    hud.dock = false;
    let render = |capacity, fill| {
        let mut f = FrameBuilder::new();
        append_curved_vitals(
            &mut f,
            AffineTransform::default(),
            [0.0, 0.0, 44.0, 12.0],
            VitalsPresentation {
                fill: [fill, 1.0],
                trail: [fill, 1.0],
            },
            [MINT, CYAN],
            [capacity, 0.0],
        );
        f
    };
    // Five seven-cell silhouettes, with capacity and charge passes.
    let base = render(hud.max_hull, 1.0);
    assert_eq!(base.frame().pixel_count, 5 * 7 * 2);
    let capacity_cells = |f: &FrameBuilder| {
        packet(f)
            .into_iter()
            .filter(|p| p.0 >> 24 == 0x90)
            .collect::<Vec<_>>()
    };
    assert_eq!(capacity_cells(&base).len(), 35);
    let upgraded = render(120.0, 1.0);
    assert_eq!(capacity_cells(&upgraded).len(), 42);
    let partial_upgrade = render(115.0, 1.0);
    let partial_damage = render(115.0, 109.0 / 115.0);
    assert_eq!(
        capacity_cells(&partial_upgrade),
        capacity_cells(&partial_damage)
    );
    assert_ne!(packet(&partial_upgrade), packet(&partial_damage));
    assert_ne!(packet(&render(115.0, 0.599)), packet(&render(115.0, 0.601)));

    // Current thirteen-chunk hull maximum remains under the same layout and
    // batching envelope, with no interaction or world-zoom dependence.
    for hull in [100.0, 115.0, 130.0, 175.0, 220.0, 260.0, f32::MAX] {
        let mut f = FrameBuilder::new();
        append_curved_vitals(
            &mut f,
            AffineTransform::default(),
            [0.0, 0.0, 44.0, 12.0],
            VitalsPresentation {
                fill: [0.43, 0.37],
                trail: [1.0; 2],
            },
            [MINT, CYAN],
            [hull, 182.0],
        );
        assert_eq!(f.frame().batches.len(), 1);
        assert!(f.frame().pixel_count <= 460);
        assert!(packet(&f).iter().flat_map(|p| p.1).all(|p| p.is_finite()
            && p.x >= 0.0
            && p.x <= 44.0
            && p.y >= 0.0
            && p.y <= 12.0));
    }
    assert_eq!(packet(&render(260.0, 1.0)), packet(&render(f32::MAX, 1.0)));
}

#[test]
fn actual_default_hud_draw_has_five_hull_chunks_and_three_shield_chunks() {
    let mut hud = game_core::Game::new(game_core::Profile::default()).hud();
    hud.dock = false;
    assert_eq!([hud.max_hull, hud.max_shield], [100.0, 30.0]);
    let layout = FlightStatusLayout {
        viewport: [533.0, 300.0],
        vitals: Some([0.0, 0.0, 44.0, 12.0]),
        strip: None,
        stats: None,
        context: None,
        navigation: None,
    };
    let mut f = FrameBuilder::new();
    draw_flight_status(
        &mut f,
        &hud,
        [266.5, 150.0],
        &layout,
        None,
        0.0,
        true,
        false,
    );
    assert_eq!(f.frame().pixel_count, (5 + 3) * 7 * 2);
    assert_eq!(
        packet(&f).iter().filter(|p| p.0 >> 24 == 0x90).count(),
        (5 + 3) * 7
    );
}
