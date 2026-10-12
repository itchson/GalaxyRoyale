//! Authored overhead garage stage. Cached alloy/deck cells, no inventory or state.
use engine_core::{AffineTransform, FrameBuilder, PixelBody, Transform, Vec2};
use game_core::identity::Corporation;
use std::sync::LazyLock;
const WIDTH: usize = 128;
const HEIGHT: usize = 96;
const fn rgba(c: [u8; 3]) -> u32 {
    u32::from_le_bytes([c[0], c[1], c[2], 255])
}
struct Art {
    body: PixelBody,
    signals: Box<[bool]>,
}
static ART: LazyLock<[Art; 3]> = LazyLock::new(|| {
    std::array::from_fn(|corp| {
        let accent = [Corporation::Pan, Corporation::Renko, Corporation::Hugo][corp].color();
        let mut signals = vec![false; WIDTH * HEIGHT];
        let mut cells = Vec::new();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let ax = x.min(WIDTH - 1 - x);
                let ay = y.min(HEIGHT - 1 - y);
                if ax + ay < 8 {
                    continue;
                }
                let seam = x % 16 == 0 || y % 16 == 0;
                let side = ax < 21 && (12..84).contains(&y);
                let fastening = [12usize, 36, 60, 84].iter().any(|yy| y.abs_diff(*yy) <= 1)
                    && [8usize, 119].iter().any(|xx| x.abs_diff(*xx) <= 1);
                let lamp = (ax == 18 || ax == 19) && (18..78).contains(&y) && y % 12 < 5;
                let color = if fastening {
                    if y % 12 == 0 && (x == 8 || x == 119) {
                        [211, 181, 113]
                    } else {
                        [43, 49, 55]
                    }
                } else if ax + ay == 8 || ax == 0 || ay == 0 {
                    [195, 207, 209]
                } else if ax + ay < 11 || ax < 3 || ay < 3 {
                    [122, 139, 145]
                } else if ax == 3 || ay == 3 {
                    [30, 42, 47]
                } else if lamp {
                    signals[y * WIDTH + x] = true;
                    accent
                } else if side {
                    if ax == 4 || ax == 20 {
                        [131, 149, 151]
                    } else if (7..17).contains(&ax) && y % 12 < 6 {
                        if y % 2 == 0 {
                            [22, 32, 38]
                        } else {
                            [80, 96, 102]
                        }
                    } else if ax < 7 {
                        [69, 82, 88]
                    } else {
                        [99, 113, 115]
                    }
                } else if y < 13 {
                    if y == 6 && (40..88).contains(&x) {
                        [87, 165, 179]
                    } else if y % 3 == 0 {
                        [34, 46, 51]
                    } else {
                        [61, 78, 84]
                    }
                } else if y > 84 {
                    if (x + y) % 12 < 3 {
                        [152, 139, 92]
                    } else {
                        [43, 49, 53]
                    }
                } else if ((x == 29 || x == 98) && (22..76).contains(&y) && y % 12 < 6)
                    || ((y == 21 || y == 76) && (34..94).contains(&x) && x % 12 < 6)
                {
                    [106, 132, 140]
                } else if seam {
                    [24, 32, 37]
                } else if x % 16 == 1 || y % 16 == 1 {
                    [46, 57, 63]
                } else {
                    [35, 44, 50]
                };
                cells.push([(y * WIDTH + x) as u32, rgba(color)]);
            }
        }
        Art {
            body: PixelBody::from_indexed_rgba(WIDTH, HEIGHT, &cells, WIDTH * HEIGHT)
                .expect("authored garage"),
            signals: signals.into_boxed_slice(),
        }
    })
});
pub struct Input {
    pub parent: AffineTransform,
    pub rect: [f32; 4],
    pub clip: Option<[f32; 4]>,
    pub corporation: usize,
    pub time: f32,
    pub reduced_motion: bool,
    pub active: bool,
}
pub fn append(
    frame: &mut FrameBuilder,
    input: &Input,
    pixels: &mut usize,
    work: &mut usize,
) -> usize {
    let r = input.rect;
    let determinant = input.parent.basis_x.perp_dot(input.parent.basis_y);
    if input.corporation >= 3
        || r.iter().any(|v| !v.is_finite() || v.abs() > 8192.0)
        || r[2] <= 0.0
        || r[3] <= 0.0
        || input.clip.is_some_and(|c| {
            c.iter().any(|v| !v.is_finite() || v.abs() > 8192.0) || c[2] <= 0.0 || c[3] <= 0.0
        })
        || !input.parent.position.is_finite()
        || !input.parent.basis_x.is_finite()
        || !input.parent.basis_y.is_finite()
        || !determinant.is_finite()
        || determinant == 0.0
    {
        return 0;
    }
    let art = &ART[input.corporation];
    let count = art.body.occupied_count();
    if count > *pixels || count > *work {
        return 0;
    }
    let fit = (r[2] / WIDTH as f32).min(r[3] / HEIGHT as f32);
    let local = Transform {
        position: Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5),
        scale: Vec2::splat(fit),
        ..Default::default()
    };
    let parent = input.parent.compose(local.affine());
    if [
        Vec2::new(-64.0, -48.0),
        Vec2::new(-64.0, 48.0),
        Vec2::new(64.0, -48.0),
        Vec2::new(64.0, 48.0),
    ]
    .iter()
    .any(|p| {
        let p = parent.point(*p);
        !p.is_finite() || p.abs().max_element() > 1_100_000.0
    }) {
        return 0;
    }
    let time = if input.reduced_motion || !input.time.is_finite() {
        0.0
    } else {
        input.time.rem_euclid(100.0)
    };
    frame.push_body_affine_colored(&art.body, parent, |cell, color| {
        let p = local.point(cell);
        let half = fit * 0.5;
        if input.clip.is_some_and(|c| {
            p.x - half < c[0]
                || p.y - half < c[1]
                || p.x + half > c[0] + c[2]
                || p.y + half > c[1] + c[3]
        }) {
            return 0;
        }
        let index = (cell.y + HEIGHT as f32 * 0.5).floor() as usize * WIDTH
            + (cell.x + WIDTH as f32 * 0.5).floor() as usize;
        if art.signals.get(index).copied().unwrap_or(false) {
            let [r, g, b, _] = color.to_le_bytes();
            let pulse = if input.reduced_motion {
                0.86
            } else {
                0.78 + (time * 1.5 - cell.y * 0.09).sin() * 0.14
            };
            let strength = if input.active { 1.08 } else { 1.0 };
            rgba([r, g, b].map(|v| (v as f32 * pulse * strength).min(255.0) as u8))
        } else {
            color
        }
    });
    *pixels -= count;
    *work -= count;
    count
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cached_garage_keeps_complete_fine_cells_and_atomic_quotas_across_fits() {
        let count = ART[0].body.occupied_count();
        assert!(count > 10_000 && count < 13_000);
        for corp in 0..3 {
            for rect in [[0.0, 0.0, 320.0, 180.0], [4.125, 3.25, 39.5, 17.75]] {
                let input = Input {
                    parent: AffineTransform {
                        position: Vec2::new(3.0, 5.0),
                        basis_x: Vec2::new(-1.0, 0.25),
                        basis_y: Vec2::new(0.2, 0.9),
                    },
                    rect,
                    clip: None,
                    corporation: corp,
                    time: 1.0,
                    reduced_motion: false,
                    active: false,
                };
                let mut frame = FrameBuilder::new();
                let (mut p, mut w) = (count - 1, 60_000);
                assert_eq!(append(&mut frame, &input, &mut p, &mut w), 0);
                assert_eq!(frame.frame().pixel_count, 0);
                assert_eq!(p, count - 1);
                p = 60_000;
                assert_eq!(append(&mut frame, &input, &mut p, &mut w), count);
                assert_eq!(frame.frame().pixel_count, count);
                assert_eq!(p, 60_000 - count);
            }
        }
    }
}
