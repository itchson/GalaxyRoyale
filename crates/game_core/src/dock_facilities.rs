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
    [240, 126, 112], // Upgrade fabricator: coral tools and reactor.
    [100, 204, 224], // Hangar: cyan gantry and parked shuttle.
    [133, 204, 144], // Journal: green archive stack and terminal.
    [224, 185, 104], // Campaign: gold observatory and pennant.
];
const PAD: &[&str] = &[
    "        ppp        ",
    "      ppppppp      ",
    "    ppqqqqqqqpp    ",
    "  ppqqqqqqqqqqqpp  ",
    "ppqqqqqqqqqqqqqqqpp",
    "  ppqqqqqqqqqqqpp  ",
    "    ppqqqqqqqpp    ",
    "      ppppppp      ",
    "        ppp        ",
];
const UPGRADE: &[&str] = &[
    "          t          ",
    "         aaa         ",
    "        awwwa        ",
    "        awlwa        ",
    "       sawwwas       ",
    "     ssaaaaaaass     ",
    "    sbbsaasasbbs     ",
    "    sbbsaasasbbs     ",
    "    sbbsswssbbs      ",
    "    sbbaaabaabbs     ",
    "   ssbbbaaaabbbss    ",
    "   saaaaaaaaaaas     ",
    "   swssbbbbbssws     ",
    "   swssblllbssws     ",
    "   saasbbbbbsaas     ",
    "   ssaaaaaaaasss     ",
    "  ssssbbbbbsssss     ",
    "  saasbbbbbsaas      ",
    "  ssssbbbbbsssss     ",
    "   sssssssssss       ",
];
const HANGAR: &[&str] = &[
    "   aaaaaaaaaaaaaaaaa   ",
    "  aswwwwwwwwwwwwwwwsa  ",
    "  asssssssssssssssssa  ",
    "  asbbbs       sbbbsa  ",
    "  asbbbs   w   sbbbsa  ",
    "  asbbbs  waw  sbbbsa  ",
    "  asbbbs  waw  sbbbsa  ",
    "  asbbbs wwaww sbbbsa  ",
    "  asbbs wwlaww sb bsa  ",
    "  asbbswwwwwwwsb bsa   ",
    "  asbbs  as a  sbbbsa  ",
    "  asbbs  tl t  sbbbsa  ",
    "  assbs  bbb   sbbbsa  ",
    "  aasbs        sbbsaa  ",
    "  ssasbbbbbbbbbsasss   ",
    "   ssaaaaaaaaaaass    ",
    "     sssssssssss     ",
];
const JOURNAL: &[&str] = &[
    "       aaaaaaa       ",
    "      aswwwwwsa      ",
    "      aswlllwsa      ",
    "      aswwwwwsa      ",
    "      asssssssa      ",
    "   aaaaabbbaaaaaa    ",
    "   aswwwwwwwwwwsa    ",
    "   aswllbblllwwsa    ",
    "   aswwwwwwwwwwsa    ",
    "   assssssssssssa    ",
    "    asbbbbbbbbsa     ",
    "  aaaswwwwwwwwsaaa   ",
    "  aswwllbbllbbwwsa   ",
    "  aswwwwwwwwwwwwsa   ",
    "  assssssssssssssa   ",
    "   saasbbbbbsaas     ",
    "   ssssbbbbbssss     ",
    "    sssssssssss      ",
];
const CAMPAIGN: &[&str] = &[
    "            aaa        ",
    "            awaa       ",
    "            awwwa      ",
    "            aaaa       ",
    "            s          ",
    "      aa    s    aa    ",
    "     awwa   s   awwa   ",
    "    awwssaassasswwa    ",
    "     asbsaallassbsa    ",
    "      sbbawwwabbss     ",
    "      sbbaawaa bb s    ",
    "      sbbaa aabb s     ",
    "     ssbbbaaabbbss     ",
    "    sasbbbbbbbbbsa     ",
    "   saaassbbbbbssaaa    ",
    "   swssaabbbb aassw    ",
    "   saassaaaaaaassa     ",
    "    sssssssssssss      ",
    "     sssssssssss       ",
];
struct Art {
    pad: PixelBody,
    buildings: [PixelBody; 4],
}
/// Fine object-local metalwork: chamfered corners, recessed vents, staggered
/// fasteners and two-tone screens. The half-sized cells retain the world bounds.
fn detail_building(rows: &[&str]) -> PixelBody {
    let source: Vec<Vec<char>> = rows.iter().map(|r| r.chars().collect()).collect();
    let width = source.iter().map(Vec::len).max().unwrap_or(0);
    let sample = |x: isize, y: isize| {
        if x < 0 || y < 0 {
            return ' ';
        }
        source
            .get(y as usize)
            .and_then(|r| r.get(x as usize))
            .copied()
            .unwrap_or(' ')
    };
    let mut detail = Vec::with_capacity(rows.len() * 2);
    for y in 0..rows.len() * 2 {
        let mut row = String::with_capacity(width * 2);
        for x in 0..width * 2 {
            let sx = (x / 2) as isize;
            let sy = (y / 2) as isize;
            let original = sample(sx, sy);
            if original == ' ' {
                row.push(' ');
                continue;
            }
            let left = sample(sx - 1, sy) == ' ';
            let right = sample(sx + 1, sy) == ' ';
            let top = sample(sx, sy - 1) == ' ';
            let bottom = sample(sx, sy + 1) == ' ';
            let shade = if (left && top && x % 2 == 0 && y % 2 == 0)
                || (right && bottom && x % 2 == 1 && y % 2 == 1)
            {
                ' '
            } else if (left && x % 2 == 0) || (top && y % 2 == 0) {
                'h'
            } else if (right && x % 2 == 1) || (bottom && y % 2 == 1) {
                'd'
            } else if original == 'b' && y % 6 == 3 {
                'v'
            } else if original == 'b' && x % 8 == 2 && y % 6 == 1 {
                'r'
            } else if original == 'l' && y % 2 == 0 {
                'e'
            } else if original == 'a' && x % 6 == 3 {
                'd'
            } else {
                original
            };
            row.push(shade);
        }
        detail.push(row);
    }
    let borrowed: Vec<_> = detail.iter().map(String::as_str).collect();
    PixelBody::from_ascii(
        &borrowed,
        &[
            ('s', 0),
            ('b', 1),
            ('a', 2),
            ('w', 3),
            ('l', 4),
            ('t', 5),
            ('p', 6),
            ('q', 7),
            ('h', 8),
            ('d', 9),
            ('v', 10),
            ('r', 11),
            ('e', 12),
        ],
    )
}
static ART: LazyLock<Art> = LazyLock::new(|| Art {
    pad: detail_building(PAD),
    buildings: [UPGRADE, HANGAR, JOURNAL, CAMPAIGN].map(detail_building),
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

fn color(id: usize, shade: u32, time: f32) -> u32 {
    let accent = ACCENTS[id];
    let pulse = 0.94 + (time * 0.8 + id as f32 * 1.4).sin() * 0.06;
    let (rgb, alpha) = match shade {
        0 => ([24, 28, 34], 255),
        1 => ([49, 58, 65], 255),
        2 => (accent.map(|c| (c as f32 * 0.67) as u8), 255),
        3 => (accent.map(|c| (c as f32 * 0.92) as u8), 255),
        4 | 5 => (accent.map(|c| (c as f32 * pulse) as u8), 255),
        6 => ([41, 47, 52], 125),
        7 => ([27, 33, 38], 110),
        8 => ([160, 183, 183], 255),
        9 => ([17, 27, 35], 255),
        10 => ([31, 43, 52], 255),
        11 => ([190, 173, 137], 255),
        _ => (accent.map(|c| (c as f32 * pulse * 0.55) as u8), 255),
    };
    u32::from_le_bytes([rgb[0], rgb[1], rgb[2], alpha])
}

/// Uses the actual interpolated unshaken extraction camera and frozen cosmetic time.
pub(super) fn render(frame: &mut FrameBuilder, camera: Vec2, viewport: Vec2, time: f32) {
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
            &ART.pad,
            Transform {
                position: center + Vec2::new(0.0, 8.0),
                scale: Vec2::new(0.75, 0.5),
                ..Transform::default()
            },
            PixelMaterial {
                light_response: 0.35,
                emission: 0.0,
            },
            |_, shade| color(id, shade, time),
        );
        frame.push_body_colored_material(
            &ART.buildings[id],
            Transform {
                position: center + Vec2::new(0.0, -3.0),
                scale: Vec2::splat(0.55),
                ..Transform::default()
            },
            PixelMaterial {
                light_response: 0.75,
                emission: 0.0,
            },
            |_, shade| color(id, shade, time),
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
                    assert!(((p + corner) * 0.55 + Vec2::new(0.0, -3.0)).length() < HIT_RADIUS);
                }
            }
            for other in ART.buildings.iter().skip(id + 1) {
                assert_ne!(
                    body.cells().collect::<Vec<_>>(),
                    other.cells().collect::<Vec<_>>()
                );
            }
        }
        for (p, _) in ART.pad.cells() {
            for corner in [
                Vec2::splat(-0.5),
                Vec2::new(0.5, -0.5),
                Vec2::splat(0.5),
                Vec2::new(-0.5, 0.5),
            ] {
                assert!(
                    ((p + corner) * Vec2::new(0.75, 0.5) + Vec2::new(0.0, 8.0)).length()
                        < HIT_RADIUS
                );
            }
        }
    }
    #[test]
    fn extraction_is_bounded_culled_local_and_cosmetic_time_freezes_it() {
        let draw = |time, camera| {
            let mut f = FrameBuilder::new();
            render(&mut f, camera, Vec2::new(533.0, 300.0), time);
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
            ART.pad.occupied_count() * 4
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
        render(&mut invalid, Vec2::ZERO, Vec2::splat(0.0), 0.0);
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
            render(&mut f, Vec2::ZERO, game.viewport, 0.0);
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
