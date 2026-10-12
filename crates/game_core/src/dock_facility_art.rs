//! Compact overhead service architecture, authored on the approved dock palette.
//! Each roof owns one packed field and four disjoint cached index groups.
use engine_core::{PixelBody, Vec2};

pub(super) const SIDE: usize = 96;
pub(super) const PARTS: usize = 4;
// Exact selected colors from pan-coop-dock/native256/pack.json. Neutral alloy
// and cyan instrumentation are independent of the manufacturer's green signals.
const PALETTE: [u32; 24] = [
    0xffd8e4eb, 0xffc9d5df, 0xffc0cdd6, 0xffbcc7d3, 0xffafbbc6, 0xffa1aeb6, 0xff889197, 0xff757c81,
    0xff5a5e62, 0xff4e4e4a, 0xff3e3d3d, 0xff353630, 0xff2e2c2c, 0xff262622, 0xff1d1d1d, 0xff0b0b0b,
    0xff88c480, 0xff57ba33, 0xff387c16, 0xff233c18, 0xffcbc51b, 0xffa89f31, 0xff82771c, 0xff373923,
];
pub(super) struct Art {
    pub body: PixelBody,
    pub parts: [Box<[u16]>; PARTS],
}
fn rect(p: Vec2, center: Vec2, half: Vec2) -> bool {
    let a = (p - center).abs();
    a.x <= half.x && a.y <= half.y
}
fn octagon(p: Vec2, half: f32, diagonal: f32) -> bool {
    let a = p.abs();
    a.max_element() <= half && a.x + a.y <= diagonal
}
fn circle(p: Vec2, center: Vec2, radius: f32) -> bool {
    p.distance_squared(center) <= radius * radius
}
fn shuttle(p: Vec2) -> bool {
    (-19.0..=19.0).contains(&p.y)
        && p.x.abs()
            <= if p.y < -7.0 {
                (p.y + 20.0) * 0.5
            } else if p.y < 7.0 {
                8.0
            } else {
                5.0
            }
}
fn roof(id: usize, p: Vec2) -> bool {
    let a = p.abs();
    match id {
        0 => {
            circle(p, Vec2::ZERO, 29.0)
                || (a.x <= 43.0 && a.y <= 7.0)
                || (a.y <= 43.0 && a.x <= 7.0)
        }
        1 => {
            (octagon(p, 32.0, 56.0) && (a.x >= 14.0 || p.y <= -25.0))
                || rect(p, Vec2::new(0.0, 36.0), Vec2::new(17.0, 6.0))
                || rect(p, Vec2::new(-38.0, 0.0), Vec2::new(5.0, 13.0))
                || rect(p, Vec2::new(38.0, 0.0), Vec2::new(5.0, 13.0))
                || shuttle(p)
        }
        2 => {
            octagon(p, 25.0, 40.0)
                || circle(p, Vec2::new(-34.0, 0.0), 10.0)
                || circle(p, Vec2::new(34.0, 0.0), 10.0)
                || rect(p, Vec2::new(0.0, -34.0), Vec2::new(9.0, 9.0))
                || rect(p, Vec2::new(0.0, 34.0), Vec2::new(14.0, 8.0))
        }
        3 => {
            octagon(p, 19.0, 29.0)
                || [
                    Vec2::new(-29.0, 0.0),
                    Vec2::new(29.0, 0.0),
                    Vec2::new(0.0, -29.0),
                    Vec2::new(0.0, 29.0),
                ]
                .iter()
                .any(|&c| circle(p, c, 13.0))
        }
        _ => {
            octagon(p, 30.0, 48.0)
                || (a.x <= 42.0 && a.x >= 28.0 && a.y <= 8.0)
                || (a.y <= 42.0 && a.y >= 28.0 && a.x <= 6.0)
        }
    }
}
fn plate(id: usize, p: Vec2) -> bool {
    let a = p.abs();
    let r = p.length();
    match id {
        0 => (18.0..=28.0).contains(&r) || (a.max_element() >= 29.0 && a.min_element() <= 7.0),
        1 => {
            (a.x >= 16.0 && a.x <= 32.0 && a.y <= 31.0)
                || (p.y <= -27.0 && a.x <= 30.0)
                || shuttle(p)
        }
        2 => {
            (17.0..=24.0).contains(&r)
                || a.x >= 27.0
                || (p.y <= -28.0 && a.x <= 8.0)
                || (p.y >= 29.0 && a.x <= 13.0)
        }
        3 => {
            [
                Vec2::new(-29.0, 0.0),
                Vec2::new(29.0, 0.0),
                Vec2::new(0.0, -29.0),
                Vec2::new(0.0, 29.0),
            ]
            .iter()
            .any(|&c| (8.0..=12.0).contains(&p.distance(c)))
                || (13.0..=18.0).contains(&a.max_element())
        }
        _ => {
            (octagon(p, 28.0, 44.0) && !octagon(p, 21.0, 34.0))
                || (a.x <= 16.0 && a.y <= 19.0)
                || a.x >= 33.0
                || a.y >= 33.0
        }
    }
}
fn sample(id: usize, p: Vec2) -> Option<(usize, u32)> {
    let a = p.abs();
    let r = p.length();
    let cardinal = [Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y];
    if !roof(id, p) {
        // Only a thin flush platform outline: no facade rows or tower shadows.
        if cardinal.iter().any(|&d| roof(id, p + d * 2.0)) {
            return Some((0, PALETTE[if p.x + p.y < 0.0 { 7 } else { 13 }]));
        }
        return None;
    }
    let x = (p.x + 47.5) as usize;
    let y = (p.y + 47.5) as usize;
    let interior_edge = cardinal.iter().any(|&d| !roof(id, p + d));
    if interior_edge {
        return Some((0, PALETTE[9]));
    }
    // Small fasteners, braces, coolant louvers and survey instrumentation are
    // authored independently per service, avoiding repeated facade rectangles.
    let detail = match id {
        0 => {
            if r < 8.5 {
                Some((
                    3,
                    if a.min_element() < 1.0 || (r - 6.5).abs() < 0.8 {
                        16
                    } else {
                        18
                    },
                ))
            } else if (10.0..=15.5).contains(&r) {
                Some((0, if (a.x - a.y).abs() < 1.2 { 6 } else { 14 }))
            } else if a.max_element() >= 36.0 && a.min_element() <= 4.0 {
                Some((1, if (x + y) % 4 <= 1 { 8 } else { 2 }))
            } else if a.max_element() >= 29.0 && a.min_element() <= 1.0 {
                Some((3, 17))
            } else if (a.x - 18.5).abs() < 3.0 && a.y < 5.0 || (a.y - 18.5).abs() < 3.0 && a.x < 5.0
            {
                Some((2, if (x + y).is_multiple_of(3) { 20 } else { 22 }))
            } else {
                None
            }
        }
        1 => {
            if shuttle(p) {
                if a.x <= 2.5 && (-12.0..=-4.0).contains(&p.y) {
                    Some((2, 20))
                } else if a.x <= 1.0 || p.y >= 14.0 {
                    Some((3, 17))
                } else if a.x >= 5.5 && p.y > -7.0 && p.y < 7.0 {
                    Some((1, 0))
                } else {
                    None
                }
            } else if (a.x - 14.5).abs() < 0.8 && a.y <= 22.0 {
                Some((3, if y % 8 < 3 { 16 } else { 19 }))
            } else if a.x >= 21.0 && a.x <= 28.0 && (p.y.abs() - 8.0).abs() <= 4.0 {
                Some((0, if y.is_multiple_of(3) { 6 } else { 14 }))
            } else if a.x >= 37.0 && a.y <= 8.0 {
                Some((2, if y.is_multiple_of(4) { 20 } else { 22 }))
            } else if p.y >= 33.0 && (a.x - (p.y - 27.0)).abs() < 1.0 {
                Some((3, 17))
            } else {
                None
            }
        }
        2 => {
            if r < 14.5 {
                if (r - 12.0).abs() < 0.8 || (a.x < 1.0 && p.y < 3.0) {
                    Some((3, 17))
                } else if x.is_multiple_of(5) || y.is_multiple_of(5) {
                    Some((2, 22))
                } else if (p - Vec2::new(5.0, -4.0)).length() < 2.0 {
                    Some((2, 20))
                } else {
                    Some((2, 23))
                }
            } else if a.x >= 29.0 && a.y <= 5.0 {
                Some((2, if y % 4 == 1 { 20 } else { 22 }))
            } else if p.y <= -31.0 && a.x <= 5.0 {
                Some((3, if x % 3 == 1 { 16 } else { 18 }))
            } else if p.y >= 31.0 && a.x <= 9.0 {
                Some((0, if y % 3 == 1 { 3 } else { 14 }))
            } else {
                None
            }
        }
        3 => {
            if r < 8.0 {
                Some((2, if (a.x - a.y).abs() < 1.0 { 20 } else { 23 }))
            } else if (a.x < 1.0 || a.y < 1.0) && a.max_element() < 28.0 {
                Some((3, 17))
            } else {
                let closest = [
                    Vec2::new(-29.0, 0.0),
                    Vec2::new(29.0, 0.0),
                    Vec2::new(0.0, -29.0),
                    Vec2::new(0.0, 29.0),
                ]
                .into_iter()
                .map(|c| p.distance(c))
                .fold(f32::INFINITY, f32::min);
                if closest < 5.0 {
                    Some((
                        2,
                        if x.is_multiple_of(3) || y.is_multiple_of(3) {
                            21
                        } else {
                            23
                        },
                    ))
                } else if (5.5..=7.0).contains(&closest) {
                    Some((3, 16))
                } else {
                    None
                }
            }
        }
        _ => {
            if a.x < 14.0 && a.y < 17.0 {
                Some((
                    1,
                    if y.is_multiple_of(4) {
                        7
                    } else if a.x < 1.0 {
                        14
                    } else {
                        4
                    },
                ))
            } else if (a.x - 21.5).abs() < 2.0 && (a.y - 10.5).abs() < 3.0 {
                Some((2, if y.is_multiple_of(3) { 20 } else { 22 }))
            } else if a.x < 5.0 && (a.y - 22.5).abs() < 1.0 {
                Some((3, 16))
            } else if a.x >= 35.0 && a.y < 5.0 || a.y >= 35.0 && a.x < 3.0 {
                Some((3, if (x + y) % 4 < 2 { 17 } else { 19 }))
            } else {
                None
            }
        }
    };
    if let Some((part, shade)) = detail {
        return Some((part, PALETTE[shade]));
    }
    if plate(id, p) {
        let bolt = match id {
            0 | 2 => [
                Vec2::new(23.5, 0.5),
                Vec2::new(0.5, 23.5),
                Vec2::new(16.5, 16.5),
            ]
            .into_iter()
            .map(|q| a.distance_squared(q))
            .fold(f32::INFINITY, f32::min),
            1 => [
                Vec2::new(23.5, 25.5),
                Vec2::new(23.5, 18.5),
                Vec2::new(29.5, 2.5),
            ]
            .into_iter()
            .map(|q| a.distance_squared(q))
            .fold(f32::INFINITY, f32::min),
            3 => [
                Vec2::new(37.5, 4.5),
                Vec2::new(4.5, 37.5),
                Vec2::new(15.5, 15.5),
            ]
            .into_iter()
            .map(|q| a.distance_squared(q))
            .fold(f32::INFINITY, f32::min),
            _ => [
                Vec2::new(25.5, 4.5),
                Vec2::new(4.5, 25.5),
                Vec2::new(21.5, 21.5),
            ]
            .into_iter()
            .map(|q| a.distance_squared(q))
            .fold(f32::INFINITY, f32::min),
        };
        if bolt < 5.0 {
            return Some((
                1,
                PALETTE[if bolt <= 0.25 {
                    0
                } else if bolt < 2.25 {
                    14
                } else {
                    5
                }],
            ));
        }
        let edge = cardinal.iter().any(|&d| !plate(id, p + d));
        let bevel = cardinal.iter().any(|&d| !plate(id, p + d * 2.0));
        let shade = if edge {
            7
        } else if bevel {
            if p.x + p.y < 0.0 { 0 } else { 4 }
        } else if matches!(id, 0 | 2)
            && (p.y.atan2(p.x) * 4.0 / std::f32::consts::PI).rem_euclid(1.0) < 0.045
        {
            7
        } else if (x + y).is_multiple_of(29) {
            5
        } else if x.is_multiple_of(19) || y.is_multiple_of(19) {
            6
        } else {
            2 + (x / 9 + y / 9) % 2
        };
        Some((1, PALETTE[shade]))
    } else {
        Some((
            0,
            PALETTE[if x.is_multiple_of(9) || y.is_multiple_of(9) {
                12
            } else {
                10 + (x / 7 + y / 7) % 2
            }],
        ))
    }
}
pub(super) fn build(id: usize) -> Art {
    let mut cells = Vec::new();
    let mut parts: [Vec<u16>; PARTS] = std::array::from_fn(|_| Vec::new());
    for y in 0..SIDE {
        for x in 0..SIDE {
            if let Some((part, color)) = sample(id, Vec2::new(x as f32 - 47.5, y as f32 - 47.5)) {
                let index = y * SIDE + x;
                cells.push([index as u32, color]);
                parts[part].push(index as u16);
            }
        }
    }
    Art {
        body: PixelBody::from_indexed_rgba(SIDE, SIDE, &cells, SIDE * SIDE)
            .expect("authored service field"),
        parts: parts.map(Vec::into_boxed_slice),
    }
}
