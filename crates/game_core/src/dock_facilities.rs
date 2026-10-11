//! Authored Haven facilities: packed object-local cells, no simulation or save state.
use engine_core::{FrameBuilder, PixelBody, PixelMaterial, Transform, Vec2};
use std::sync::LazyLock;

pub(super) const CENTERS: [[f32; 2]; 4] = [
    [-120.0, -80.0],
    [120.0, -80.0],
    [-120.0, 95.0],
    [120.0, 95.0],
];
const HIT_RADIUS: f32 = 24.0;
const ACCENTS: [[u8; 3]; 4] = [
    [240, 126, 112], // Forge: coral fabrication well and four work clamps.
    [100, 204, 224], // Hangar: cyan roof rails and an overhead shuttle bay.
    [133, 204, 144], // Journal: green radial archive pods and survey glass.
    [224, 185, 104], // Campaign: gold arena ring and score-control decks.
];
// All structures use the same overhead plane and fine world cells.
// There are no facade rows, cast tower shadows or compressed diamond bases.
const ART_SIDE: usize = 44;
const CELL_SCALE: f32 = 0.55;
struct Art {
    pads: [PixelBody; 4],
    buildings: [PixelBody; 4],
}
fn octagon(p: Vec2, half: f32, diagonal: f32) -> bool {
    let a = p.abs();
    a.max_element() <= half && a.x + a.y <= diagonal
}
fn roof_mask(id: usize, p: Vec2) -> bool {
    let a = p.abs();
    match id {
        // Four work clamps surround an octagonal fabrication deck.
        0 => octagon(p, 17.5, 30.0) || (a.x <= 20.5 && a.y <= 4.5) || (a.y <= 20.5 && a.x <= 4.5),
        // An open bay and a parked overhead shuttle, with its apron to the south.
        1 => {
            let roof = octagon(p, 20.5, 34.0) && a.y <= 16.5;
            let aperture = a.x < 7.5 && a.y < 11.5;
            let shuttle_width = if p.y < -3.5 {
                (p.y + 10.5) * 0.55 + 0.5
            } else if p.y < 4.5 {
                5.5
            } else if p.y < 7.5 {
                2.5
            } else {
                4.5
            };
            let shuttle = (-9.5..=9.5).contains(&p.y) && a.x <= shuttle_width;
            (roof && !aperture) || shuttle || (a.x <= 10.5 && (16.5..=20.5).contains(&p.y))
        }
        // Four radial archive pods surrounding a glass survey terminal.
        2 => (a.x <= 7.5 && a.y <= 20.5) || (a.y <= 7.5 && a.x <= 20.5) || a.x + a.y <= 23.0,
        // A circular arena aperture and four flat score-control decks.
        _ => {
            (8.0..=18.5).contains(&p.length())
                || (a.x <= 20.5 && a.x >= 16.5 && a.y <= 5.5)
                || (a.y <= 20.5 && a.y >= 16.5 && a.x <= 5.5)
        }
    }
}
fn roof_shade(id: usize, p: Vec2) -> u32 {
    let a = p.abs();
    let r = p.length();
    let ix = (p.x + 21.5) as usize;
    let iy = (p.y + 21.5) as usize;
    let edge = [Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y]
        .iter()
        .any(|offset| !roof_mask(id, p + *offset));
    if edge {
        return if p.x + p.y < 0.0 { 8 } else { 9 };
    }
    // Flush fasteners and recessed horizontal vents are roof details, not facades.
    if (a.x - 13.5).abs() < 0.1 && (a.y - 3.5).abs() < 0.1 {
        return 11;
    }
    if (12.5..=16.5).contains(&a.x) && a.y <= 2.5 {
        return if iy.is_multiple_of(2) { 10 } else { 0 };
    }
    match id {
        0 => {
            if r < 4.8 {
                4
            } else if r < 7.0 {
                0
            } else if r < 8.5 {
                3
            } else if a.min_element() < 1.0 {
                5
            } else if a.max_element() > 17.5 {
                2
            } else if (9.5..=15.5).contains(&a.x) && (7.5..=12.5).contains(&a.y) {
                if ix.is_multiple_of(3) { 10 } else { 1 }
            } else if (a.x - a.y).abs() < 0.6 {
                9
            } else {
                1
            }
        }
        1 => {
            if a.x < 6.0 && a.y < 10.0 {
                if a.x < 1.0 && (-4.5..=-0.5).contains(&p.y) {
                    4
                } else if p.y >= 7.5 {
                    5
                } else if a.x < 1.0 {
                    3
                } else {
                    1
                }
            } else if p.y >= 16.5 {
                if (a.x - (p.y - 15.5)).abs() < 0.6 {
                    3
                } else {
                    0
                }
            } else if (a.x - 8.5).abs() < 0.6 {
                5
            } else if (a.y - 13.5).abs() < 0.6 {
                3
            } else if a.x > 9.5 && ix.is_multiple_of(6) {
                9
            } else if a.x > 9.5 && (iy % 6 == 1 || iy % 6 == 2) {
                10
            } else {
                2
            }
        }
        2 => {
            if r < 4.0 {
                4
            } else if r < 5.7 {
                0
            } else if a.min_element() < 1.0 {
                5
            } else if a.max_element() >= 8.5 {
                let along = if a.x > a.y { ix } else { iy };
                if along.is_multiple_of(4) {
                    9
                } else if a.min_element() <= 4.5 {
                    3
                } else {
                    1
                }
            } else if (a.x - a.y).abs() < 0.6 {
                11
            } else {
                2
            }
        }
        _ => {
            if (12.0..=13.8).contains(&r) {
                5
            } else if r <= 9.7 {
                3
            } else if a.max_element() >= 17.5 && a.min_element() <= 3.5 {
                4
            } else if (a.x - a.y).abs() < 0.6 {
                9
            } else if r >= 16.0 {
                2
            } else {
                1
            }
        }
    }
}
fn cell_body(mut shade: impl FnMut(Vec2) -> Option<u32>) -> PixelBody {
    let rows: Vec<String> = (0..ART_SIDE)
        .map(|y| {
            (0..ART_SIDE)
                .map(|x| {
                    shade(Vec2::new(x as f32 - 21.5, y as f32 - 21.5))
                        .map_or(' ', |s| char::from_u32(0xe000 + s).unwrap())
                })
                .collect()
        })
        .collect();
    let borrowed: Vec<_> = rows.iter().map(String::as_str).collect();
    let palette: Vec<_> = (0..=12)
        .map(|s| (char::from_u32(0xe000 + s).unwrap(), s))
        .collect();
    PixelBody::from_ascii(&borrowed, &palette)
}
static ART: LazyLock<Art> = LazyLock::new(|| Art {
    // Flush service rails follow the individual roof/courtyard silhouettes.
    pads: std::array::from_fn(|id| {
        cell_body(|p| {
            let outer = [Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y]
                .iter()
                .any(|offset| roof_mask(id, p + *offset) || roof_mask(id, p + *offset * 2.0));
            if outer && !roof_mask(id, p) {
                Some(if p.x + p.y < 0.0 { 6 } else { 7 })
            } else {
                None
            }
        })
    }),
    buildings: std::array::from_fn(|id| cell_body(|p| roof_mask(id, p).then(|| roof_shade(id, p)))),
});

pub(super) fn at(player: Vec2, point: Vec2, home_radius: f32) -> Option<u8> {
    if !player.is_finite()
        || !point.is_finite()
        || !home_radius.is_finite()
        || home_radius <= 0.0
        || player.length_squared() > home_radius * home_radius
    {
        return None;
    }
    CENTERS
        .iter()
        .position(|center| {
            point.distance_squared(Vec2::from_array(*center)) <= HIT_RADIUS * HIT_RADIUS
        })
        .map(|id| id as u8)
}

fn color(
    id: usize,
    cell: Vec2,
    shade: u32,
    time: f32,
    theme: Option<crate::identity::Corporation>,
) -> u32 {
    let accent = ACCENTS[id];
    let corporate = theme.map_or(accent, crate::identity::Corporation::color);
    // Signal waves are object-local: conveyor traces, shuttle beacons and score
    // arcs change their lit cells while all roof/deck geometry remains stationary.
    let pulse = if shade == 4 || shade == 5 || shade == 12 {
        let phase = if id == 3 {
            cell.y.atan2(cell.x) * 2.0
        } else {
            cell.x * 0.22 + cell.y * 0.15
        };
        0.80 + (time * 1.8 - phase + id as f32 * 1.4).sin() * 0.20
    } else {
        1.0
    };
    let (rgb, alpha) = match shade {
        0 => ([12, 20, 29], 255),
        1 => ([41, 57, 67], 255),
        2 => (accent.map(|c| (c as f32 * 0.67) as u8), 255),
        3 => (accent.map(|c| (c as f32 * 0.92) as u8), 255),
        4 => (accent.map(|c| (c as f32 * pulse) as u8), 255),
        5 => (corporate.map(|c| (c as f32 * pulse) as u8), 255),
        6 => ([56, 76, 87], 255),
        7 => ([25, 37, 47], 255),
        8 => ([129, 156, 162], 255),
        9 => ([17, 27, 35], 255),
        10 => ([31, 43, 52], 255),
        11 => ([190, 173, 137], 255),
        _ => (corporate.map(|c| (c as f32 * pulse * 0.55) as u8), 255),
    };
    u32::from_le_bytes([rgb[0], rgb[1], rgb[2], alpha])
}

/// Uses the actual interpolated unshaken extraction camera and frozen cosmetic time.
pub(super) fn render(
    frame: &mut FrameBuilder,
    camera: Vec2,
    viewport: Vec2,
    time: f32,
    theme: Option<crate::identity::Corporation>,
) {
    if !camera.is_finite()
        || !viewport.is_finite()
        || viewport.min_element() <= 0.0
        || !time.is_finite()
    {
        return;
    }
    let half = viewport * 0.5;
    for (id, center) in CENTERS.iter().enumerate() {
        let center = Vec2::from_array(*center);
        if ((center - camera).abs() - half).max_element() > HIT_RADIUS {
            continue;
        }
        frame.push_body_colored_material(
            &ART.pads[id],
            Transform {
                position: center,
                scale: Vec2::splat(CELL_SCALE),
                ..Transform::default()
            },
            PixelMaterial {
                light_response: 0.35,
                emission: 0.0,
            },
            |cell, shade| color(id, cell, shade, time, theme),
        );
        frame.push_body_colored_material(
            &ART.buildings[id],
            Transform {
                position: center,
                scale: Vec2::splat(CELL_SCALE),
                ..Transform::default()
            },
            PixelMaterial {
                light_response: 0.75,
                emission: 0.0,
            },
            |cell, shade| color(id, cell, shade, time, theme),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Game, Motion, Profile, SAFE_RADIUS};
    fn packets(
        frame: &engine_core::RenderFrame,
    ) -> Vec<([f32; 2], u32, engine_core::GpuTransform)> {
        frame
            .batches
            .iter()
            .flat_map(|b| {
                b.pixels
                    .iter()
                    .map(|p| (p.local, p.color, b.transforms[p.transform as usize]))
            })
            .collect()
    }
    #[test]
    fn real_home_gate_ids_and_hit_edges_do_not_mutate_progress() {
        let game = Game::new(Profile::default());
        let before = serde_json::to_value(game.profile()).unwrap();
        for (id, center) in CENTERS.iter().enumerate() {
            assert!(Vec2::from_array(*center).length() + HIT_RADIUS < SAFE_RADIUS);
            assert_eq!(game.dock_facility_at(*center), Some(id as u8));
            assert_eq!(
                game.dock_facility_at([center[0] + 24.0, center[1]]),
                Some(id as u8)
            );
            assert_eq!(game.dock_facility_at([center[0] + 24.01, center[1]]), None);
        }
        assert_eq!(game.dock_facility_at([0.0, 0.0]), None);
        assert_eq!(game.dock_facility_at([f32::NAN, 0.0]), None);
        assert_eq!(game.dock_facility_at([0.0, f32::INFINITY]), None);
        game.world.get::<&mut Motion>(game.player).unwrap().position = Vec2::new(240.01, 0.0);
        for center in CENTERS {
            assert_eq!(game.dock_facility_at(center), None);
        }
        assert_eq!(serde_json::to_value(game.profile()).unwrap(), before);
    }
    #[test]
    fn authored_buildings_have_unique_live_cells_and_all_corners_fit_hit_radius() {
        for (id, body) in ART.buildings.iter().enumerate() {
            assert!(body.occupied_count() > 400 && body.occupied_count() < 2000);
            let shades: std::collections::HashSet<_> = body.cells().map(|(_, c)| c).collect();
            assert!(shades.contains(&8) && shades.contains(&9));
            assert!(shades.contains(&10) && shades.contains(&11));
            for (p, _) in body.cells() {
                for corner in [
                    Vec2::splat(-0.5),
                    Vec2::new(0.5, -0.5),
                    Vec2::splat(0.5),
                    Vec2::new(-0.5, 0.5),
                ] {
                    assert!(((p + corner) * CELL_SCALE).length() < HIT_RADIUS);
                }
            }
            for other in ART.buildings.iter().skip(id + 1) {
                assert_ne!(
                    body.cells().collect::<Vec<_>>(),
                    other.cells().collect::<Vec<_>>()
                );
            }
        }
        for (p, _) in ART.pads.iter().flat_map(PixelBody::cells) {
            for corner in [
                Vec2::splat(-0.5),
                Vec2::new(0.5, -0.5),
                Vec2::splat(0.5),
                Vec2::new(-0.5, 0.5),
            ] {
                assert!(((p + corner) * CELL_SCALE).length() < HIT_RADIUS);
            }
        }
    }
    #[test]
    fn overhead_roofs_have_planar_cardinal_silhouettes_without_facade_offsets() {
        let mut silhouettes = std::collections::HashSet::new();
        for id in 0..4 {
            let combined: Vec<_> = (0..ART_SIDE)
                .flat_map(|y| {
                    (0..ART_SIDE).map(move |x| {
                        ART.pads[id].is_occupied(x, y) || ART.buildings[id].is_occupied(x, y)
                    })
                })
                .collect();
            assert!(
                silhouettes.insert(combined),
                "service rails must preserve four distinct overhead silhouettes"
            );
        }
        for id in [0, 2, 3] {
            let body = &ART.buildings[id];
            for y in 0..ART_SIDE {
                for x in 0..ART_SIDE {
                    assert_eq!(
                        body.is_occupied(x, y),
                        body.is_occupied(ART_SIDE - 1 - y, x)
                    );
                }
            }
        }
        let hangar = &ART.buildings[1];
        for y in 0..ART_SIDE {
            for x in 0..ART_SIDE {
                assert_eq!(
                    hangar.is_occupied(x, y),
                    hangar.is_occupied(ART_SIDE - 1 - x, y)
                );
            }
        }
        let mut builder = FrameBuilder::new();
        render(&mut builder, Vec2::ZERO, Vec2::new(533.0, 300.0), 0.0, None);
        let frame = builder.finish();
        let transforms: Vec<_> = frame
            .batches
            .iter()
            .flat_map(|batch| batch.transforms.iter())
            .collect();
        assert_eq!(transforms.len(), 8);
        for (id, pair) in transforms.chunks_exact(2).enumerate() {
            for transform in pair {
                assert_eq!(
                    transform.a,
                    [CENTERS[id][0], CENTERS[id][1], CELL_SCALE, 0.0]
                );
                assert_eq!(transform.b[..2], [0.0, CELL_SCALE]);
            }
        }
    }
    #[test]
    fn extraction_is_bounded_culled_local_and_cosmetic_time_freezes_it() {
        let draw = |time, camera| {
            let mut f = FrameBuilder::new();
            render(&mut f, camera, Vec2::new(533.0, 300.0), time, None);
            f.finish()
        };
        let a = draw(0.0, Vec2::ZERO);
        assert!(a.pixel_count > 3200 && a.pixel_count < 8000);
        assert_eq!(
            a.batches.iter().map(|b| b.transforms.len()).sum::<usize>(),
            8
        );
        assert_eq!(
            a.pixel_count,
            ART.pads
                .iter()
                .map(PixelBody::occupied_count)
                .sum::<usize>()
                + ART
                    .buildings
                    .iter()
                    .map(PixelBody::occupied_count)
                    .sum::<usize>()
        );
        assert_eq!(packets(&a), packets(&draw(0.0, Vec2::ZERO)));
        assert_ne!(packets(&a), packets(&draw(1.0, Vec2::ZERO)));
        let later = draw(1.0, Vec2::ZERO);
        for (x, y) in packets(&a).iter().zip(packets(&later).iter()) {
            assert_eq!(x.0, y.0);
            assert_eq!(x.2, y.2);
        }
        assert_eq!(draw(0.0, Vec2::splat(999000.0)).pixel_count, 0);
        let mut invalid = FrameBuilder::new();
        render(&mut invalid, Vec2::ZERO, Vec2::splat(0.0), 0.0, None);
        assert_eq!(invalid.frame().pixel_count, 0);
    }
    #[test]
    fn game_extraction_uses_frozen_motion_without_advancing_rng_or_profile() {
        let mut game = Game::new(Profile::default());
        game.set_reduced_motion(true);
        let before = serde_json::to_value(game.profile()).unwrap();
        let rng = game.rng;
        let expected = {
            let mut f = FrameBuilder::new();
            render(&mut f, Vec2::ZERO, game.viewport, 0.0, None);
            packets(f.frame())
        };
        for time in [0.0, 1.0, 50.0] {
            game.time = time;
            let frame = game.render(1.0);
            assert!(
                packets(&frame)
                    .windows(expected.len())
                    .any(|w| w == expected)
            );
        }
        assert_eq!(game.rng, rng);
        assert_eq!(serde_json::to_value(game.profile()).unwrap(), before);
    }
}
