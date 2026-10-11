//! Original cached sports-arcade chrome; semantic DOM bounds stay authoritative.
//! Fine corner objects and stretched metal rails are native occupied cells.
use engine_core::{AffineTransform, FrameBuilder, PixelBody, Transform, Vec2};
use std::sync::LazyLock;

const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    u32::from_le_bytes([r, g, b, 255])
}
pub const PLATINUM: u32 = rgb(194, 207, 216);
const SPECULAR: u32 = rgb(234, 240, 242);
const ALLOY: u32 = rgb(91, 110, 125);
const GRAPHITE: u32 = rgb(28, 36, 46);
const RECESS: u32 = rgb(8, 12, 17);
const ENAMEL: u32 = rgb(88, 222, 239);
const GOLD: u32 = rgb(212, 172, 96);

fn corner(size: usize) -> PixelBody {
    let rail = size / 3;
    let cut = size * 3 / 8;
    let rows: Vec<String> = (0..size)
        .map(|y| {
            (0..size)
                .map(|x| {
                    if x + y < cut {
                        return ' ';
                    }
                    // Recessed locator fastener is authored separately from
                    // the cut bevel, with a dark collar and gold pin.
                    let fastener = rail + 2;
                    let distance = x.abs_diff(fastener) + y.abs_diff(fastener);
                    if distance <= 2 {
                        return match distance {
                            0 => 'g',
                            1 => 'd',
                            _ => 'm',
                        };
                    }
                    if x >= rail && y >= rail {
                        return ' ';
                    }
                    if x + y == cut || x == 0 || y == 0 {
                        's'
                    } else if x + y <= cut + 2 || x < 2 || y < 2 {
                        'p'
                    } else if x == rail - 1 || y == rail - 1 {
                        'c'
                    } else if (x >= rail - 3 && y >= rail - 3)
                        || (x >= size / 2 && y == rail / 2)
                        || (y >= size / 2 && x == rail / 2)
                    {
                        'd'
                    } else {
                        'm'
                    }
                })
                .collect()
        })
        .collect();
    PixelBody::from_ascii(
        &rows.iter().map(String::as_str).collect::<Vec<_>>(),
        &[
            ('s', SPECULAR),
            ('p', PLATINUM),
            ('m', ALLOY),
            ('d', GRAPHITE),
            ('c', ENAMEL),
            ('g', GOLD),
        ],
    )
}
static PANEL_CORNER: LazyLock<PixelBody> = LazyLock::new(|| corner(24));
static BUTTON_CORNER: LazyLock<PixelBody> = LazyLock::new(|| corner(12));
static CELL: LazyLock<PixelBody> =
    LazyLock::new(|| PixelBody::from_ascii(&["#"], &[('#', RECESS)]));
static CHEVRON: LazyLock<PixelBody> = LazyLock::new(|| {
    PixelBody::from_ascii(
        &["##....", ".##...", "..##..", "...##.", "..##..", ".##..."],
        &[('#', ENAMEL)],
    )
});

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Panel,
    Button,
    Row,
}
pub struct Input {
    pub parent: AffineTransform,
    pub rect: [f32; 4],
    pub clip: Option<[f32; 4]>,
    pub kind: Kind,
    pub accent: u32,
    pub active: bool,
    pub selected: bool,
    pub disabled: bool,
}
#[derive(Clone, Copy)]
struct Part {
    body: &'static PixelBody,
    local: AffineTransform,
    color: Option<u32>,
}
fn rect_valid(r: [f32; 4]) -> bool {
    r.iter().all(|v| v.is_finite() && v.abs() <= 8192.0) && r[2] > 0.0 && r[3] > 0.0
}
fn subdued(color: u32) -> u32 {
    let [r, g, b, _] = color.to_le_bytes();
    let gray = (r as f32 * 0.21 + g as f32 * 0.72 + b as f32 * 0.07) * 0.55;
    rgb(gray as u8, (gray + 6.0) as u8, (gray + 11.0) as u8)
}
fn accent(color: u32, strong: bool) -> u32 {
    if strong {
        return color | 0xff000000;
    }
    let [r, g, b, _] = color.to_le_bytes();
    rgb(
        (r as f32 * 0.56) as u8,
        (g as f32 * 0.56) as u8,
        (b as f32 * 0.56) as u8,
    )
}

/// A complete chrome element is admitted before append. It does not reserve
/// labels or alter their measured rects; text remains the caller's independent
/// authoritative layer. Clipped alpha-zero cells still charge both quotas.
pub fn append(
    frame: &mut FrameBuilder,
    input: &Input,
    pixels: &mut usize,
    work: &mut usize,
) -> usize {
    let r = input.rect;
    if !rect_valid(r)
        || input.clip.is_some_and(|c| !rect_valid(c))
        || !input.parent.position.is_finite()
        || !input.parent.basis_x.is_finite()
        || !input.parent.basis_y.is_finite()
    {
        return 0;
    }
    let determinant = input.parent.basis_x.perp_dot(input.parent.basis_y);
    if !determinant.is_finite() || determinant == 0.0 {
        return 0;
    }
    let mut parts: [Option<Part>; 32] = [None; 32];
    let mut used = 0;
    let mut quad = |rect: [f32; 4], color: u32| {
        if rect[2] > 0.0 && rect[3] > 0.0 {
            parts[used] = Some(Part {
                body: &CELL,
                local: Transform {
                    position: Vec2::new(rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5),
                    scale: Vec2::new(rect[2], rect[3]),
                    ..Default::default()
                }
                .affine(),
                color: Some(color),
            });
            used += 1;
        }
    };
    let strong = (input.active || input.selected) && !input.disabled;
    let rail = if input.disabled {
        subdued(ENAMEL)
    } else {
        accent(input.accent, strong)
    };
    if input.kind == Kind::Row {
        if strong {
            quad(r, rgb(10, 15, 21));
        }
        quad([r[0], r[1] + r[3] - 0.6, r[2], 0.3], ALLOY);
        quad([r[0], r[1] + r[3] - 0.3, r[2], 0.3], RECESS);
    } else {
        let target = if input.kind == Kind::Panel {
            6.0_f32
        } else {
            3.0_f32
        };
        let c = target.min(r[2] * 0.25).min(r[3] * 0.25);
        let d = c * 0.12;
        // Two open cut-corner surfaces, not an opaque colored panel wash.
        let fill = if input.disabled {
            rgb(9, 12, 17)
        } else if strong {
            rgb(12, 18, 24)
        } else {
            RECESS
        };
        quad([r[0] + c * 0.4, r[1], r[2] - c * 0.8, r[3]], fill);
        quad([r[0], r[1] + c * 0.4, r[2], r[3] - c * 0.8], fill);
        for (offset, color) in [(0.0, PLATINUM), (d, ALLOY), (d * 2.0, GRAPHITE)] {
            quad([r[0] + c, r[1] + offset, r[2] - c * 2.0, d], color);
            quad(
                [r[0] + c, r[1] + r[3] - offset - d, r[2] - c * 2.0, d],
                color,
            );
            quad([r[0] + offset, r[1] + c, d, r[3] - c * 2.0], color);
            quad(
                [r[0] + r[2] - offset - d, r[1] + c, d, r[3] - c * 2.0],
                color,
            );
        }
        // A short recessed enamel rail signals action/category. Metal is
        // unchanged by hover; selected navigation keeps its independent bar.
        quad([r[0] + c, r[1] + d * 3.0, (r[2] - c * 2.0) * 0.22, d], rail);
        quad(
            [r[0] + r[2] - c - 1.25, r[1] + r[3] - d * 2.0, 1.25, d],
            GOLD,
        );
    }
    if input.kind == Kind::Row {
        if strong && r[2] >= 6.0 && r[3] >= 6.0 {
            let fit = (r[3] * 0.28).min(3.0) / 6.0;
            parts[used] = Some(Part {
                body: &CHEVRON,
                local: Transform {
                    position: Vec2::new(r[0] + 2.0, r[1] + r[3] * 0.5),
                    scale: Vec2::splat(fit),
                    ..Default::default()
                }
                .affine(),
                color: Some(rail),
            });
            used += 1;
        }
    } else {
        let body = if input.kind == Kind::Panel {
            &*PANEL_CORNER
        } else {
            &*BUTTON_CORNER
        };
        let c = (if input.kind == Kind::Panel {
            6.0_f32
        } else {
            3.0_f32
        })
        .min(r[2] * 0.25)
        .min(r[3] * 0.25);
        let scale = c / body.width as f32;
        for (x, y) in [(false, false), (true, false), (false, true), (true, true)] {
            parts[used] = Some(Part {
                body,
                local: Transform {
                    position: Vec2::new(
                        if x {
                            r[0] + r[2] - c * 0.5
                        } else {
                            r[0] + c * 0.5
                        },
                        if y {
                            r[1] + r[3] - c * 0.5
                        } else {
                            r[1] + c * 0.5
                        },
                    ),
                    scale: Vec2::new(
                        if x { -scale } else { scale },
                        if y { -scale } else { scale },
                    ),
                    ..Default::default()
                }
                .affine(),
                color: None,
            });
            used += 1;
        }
    }
    let parts = &parts[..used];
    let count = parts
        .iter()
        .flatten()
        .map(|p| p.body.occupied_count())
        .sum();
    if count > *pixels || count > *work {
        return 0;
    }
    for part in parts.iter().flatten() {
        let t = input.parent.compose(part.local);
        for x in [-0.5, 0.5] {
            for y in [-0.5, 0.5] {
                let point = t.point(Vec2::new(
                    x * part.body.width as f32,
                    y * part.body.height as f32,
                ));
                if !point.is_finite() || point.abs().max_element() > 1_100_000.0 {
                    return 0;
                }
            }
        }
    }
    let clip = input.clip.unwrap_or(r);
    let full = clip[0] <= r[0]
        && clip[1] <= r[1]
        && clip[0] + clip[2] >= r[0] + r[2]
        && clip[1] + clip[3] >= r[1] + r[3];
    for part in parts.iter().flatten() {
        // Flat backing/rail cells are rectangular surfaces: intersect their
        // geometry with the semantic clip, instead of erasing a whole large
        // surface when only one edge is scrolled. Fine corner cells retain
        // strict all-four-corner masking below.
        let quad = std::ptr::eq(part.body, &*CELL);
        let mut local = part.local;
        let mut empty = false;
        if quad && !full {
            let a = local.point(Vec2::splat(-0.5));
            let b = local.point(Vec2::splat(0.5));
            let left = a.x.max(clip[0]);
            let top = a.y.max(clip[1]);
            let right = b.x.min(clip[0] + clip[2]);
            let bottom = b.y.min(clip[1] + clip[3]);
            if right > left && bottom > top {
                local = Transform {
                    position: Vec2::new((left + right) * 0.5, (top + bottom) * 0.5),
                    scale: Vec2::new(right - left, bottom - top),
                    ..Default::default()
                }
                .affine();
            } else {
                empty = true;
            }
        }
        frame.push_body_affine_colored(part.body, input.parent.compose(local), |cell, color| {
            if empty
                || (!full
                    && !quad
                    && [
                        Vec2::splat(-0.5),
                        Vec2::new(0.5, -0.5),
                        Vec2::new(-0.5, 0.5),
                        Vec2::splat(0.5),
                    ]
                    .into_iter()
                    .map(|c| local.point(cell + c))
                    .any(|p| {
                        p.x < clip[0]
                            || p.y < clip[1]
                            || p.x > clip[0] + clip[2]
                            || p.y > clip[1] + clip[3]
                    }))
            {
                0
            } else if input.disabled {
                subdued(part.color.unwrap_or(color))
            } else if let Some(color) = part.color {
                color
            } else if color == ENAMEL {
                rail
            } else {
                color
            }
        });
    }
    *pixels -= count;
    *work -= count;
    count
}

#[cfg(test)]
#[path = "sports_ui_tests.rs"]
mod tests;
