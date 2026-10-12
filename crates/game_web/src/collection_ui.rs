//! Native collection fittings. The host alone supplies discovered entries,
//! ownership, compatibility, ranks and recipes; this module owns no catalog.
use engine_core::{AffineTransform, FrameBuilder, PixelBody, Transform, Vec2};
use std::sync::LazyLock;

const SIZE: usize = 32;
const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    u32::from_le_bytes([r, g, b, 255])
}
const RECESS: u32 = rgb(8, 12, 17);
const GRAPHITE: u32 = rgb(28, 36, 46);
const ALLOY: u32 = rgb(91, 110, 125);
const PLATINUM: u32 = rgb(194, 207, 216);
const WHITE: u32 = rgb(234, 240, 242);
const GOLD: u32 = rgb(212, 172, 96);
const EMITTER: u32 = rgb(251, 249, 237);
const CATEGORY_COLORS: [u32; 3] = [rgb(255, 147, 119), rgb(92, 191, 255), rgb(159, 225, 138)];
const CORPORATION_COLORS: [u32; 3] = [CATEGORY_COLORS[2], CATEGORY_COLORS[0], CATEGORY_COLORS[1]];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolKind {
    Category,
    Corporation,
}

struct Symbol {
    body: PixelBody,
    accent: u32,
}

// Explicit category silhouettes: admitted pulse, shield lattice, annular rotor.
// Corporate marks are separate lettered circuit crests, never inferred from a
// hull's construction currency or a pilot's sponsor.
const GLYPHS: [[&str; 9]; 6] = [
    [
        "....##.", "...###.", "..###..", ".#####.", "#######", "..####.", "..###..", ".###...",
        ".##....",
    ],
    [
        "#######", "##...##", "#.###.#", "#.#.#.#", "#.###.#", ".#.#.#.", ".#.#.#.", "..###..",
        "...#...",
    ],
    [
        "..###..", ".##.##.", "##...##", "#..#..#", "#.###.#", "#..#..#", "##...##", ".##.##.",
        "..###..",
    ],
    [
        "######.", "##...##", "##...##", "######.", "##.....", "##.....", "##.....", "##.....",
        "##.....",
    ],
    [
        "######.", "##...##", "##...##", "######.", "##.##..", "##..##.", "##...##", "##...##",
        "##...##",
    ],
    [
        "##...##", "##...##", "##...##", "#######", "#######", "##...##", "##...##", "##...##",
        "##...##",
    ],
];

fn authored(index: usize) -> Symbol {
    let accent = if index < 3 {
        CATEGORY_COLORS[index]
    } else {
        CORPORATION_COLORS[index - 3]
    };
    let mut cells = [[' '; SIZE]; SIZE];
    for (y, row) in cells.iter_mut().enumerate() {
        for (x, cell) in row.iter_mut().enumerate() {
            let dx = (x as i32 * 2 - 31).abs();
            let dy = (y as i32 * 2 - 31).abs();
            let outer = dx <= 25 && dy <= 25 && dx + dy <= 42;
            let inner = dx <= 19 && dy <= 19 && dx + dy <= 32;
            if outer && !inner {
                *cell = if dx == 25 || dy == 25 || dx + dy >= 40 {
                    if x + y < 31 { 'w' } else { 'm' }
                } else if dx == 23 || dy == 23 {
                    'p'
                } else {
                    'd'
                };
            }
            // Hand-authored separated contacts and recessed fastener collars.
            if (x == 6 || x == 25) && (y == 8 || y == 23) {
                *cell = 'g';
            }
            if (x == 7 || x == 24) && (y == 8 || y == 23) {
                *cell = 'k';
            }
            if (14..=17).contains(&x) && y == 27 {
                *cell = 'e';
            }
            if (14..=17).contains(&x) && y == 26 {
                *cell = 'a';
            }
            if (9..23).contains(&x) && (7..25).contains(&y) {
                let gx = (x - 9) / 2;
                let gy = (y - 7) / 2;
                if GLYPHS[index][gy].as_bytes()[gx] == b'#' {
                    // Raised enamel motif has a metal leading facet and a
                    // dark recessed trailing seam at its actual source cells.
                    *cell = if x % 2 == 0 && y % 2 == 1 {
                        'w'
                    } else if x % 2 == 0 && y % 2 == 0 {
                        'd'
                    } else {
                        'a'
                    };
                }
            }
        }
    }
    let rows: Vec<String> = cells
        .into_iter()
        .map(|row| row.into_iter().collect())
        .collect();
    let body = PixelBody::from_ascii(
        &rows.iter().map(String::as_str).collect::<Vec<_>>(),
        &[
            ('k', RECESS),
            ('d', GRAPHITE),
            ('m', ALLOY),
            ('p', PLATINUM),
            ('w', WHITE),
            ('g', GOLD),
            ('a', accent),
            ('e', EMITTER),
        ],
    );
    Symbol { body, accent }
}

static SYMBOLS: LazyLock<[Symbol; 6]> = LazyLock::new(|| std::array::from_fn(authored));
static CELL: LazyLock<PixelBody> =
    LazyLock::new(|| PixelBody::from_ascii(&["#"], &[('#', RECESS)]));

pub struct Input {
    pub parent: AffineTransform,
    pub rect: [f32; 4],
    pub clip: Option<[f32; 4]>,
    pub active: bool,
    pub selected: bool,
    pub disabled: bool,
    pub time: f32,
    pub reduced: bool,
}

fn valid(input: &Input) -> bool {
    let rect = |r: [f32; 4]| {
        r.iter().all(|v| v.is_finite() && v.abs() <= 8192.0) && r[2] > 0.0 && r[3] > 0.0
    };
    let p = input.parent;
    rect(input.rect)
        && input.clip.is_none_or(rect)
        && p.position.is_finite()
        && p.basis_x.is_finite()
        && p.basis_y.is_finite()
        && p.basis_x.perp_dot(p.basis_y).is_finite()
        && p.basis_x.perp_dot(p.basis_y) != 0.0
}

fn muted(color: u32) -> u32 {
    let [r, g, b, _] = color.to_le_bytes();
    let gray = ((r as u16 + g as u16 + b as u16) / 4) as u8;
    rgb(gray, gray.saturating_add(5), gray.saturating_add(9))
}

fn emitter(accent: u32, input: &Input) -> u32 {
    if input.disabled {
        return muted(accent);
    }
    let strength = if input.selected {
        0.92
    } else if input.active && !input.reduced && input.time.is_finite() {
        0.86 + (input.time * 2.0).sin() * 0.08
    } else if input.active {
        0.86
    } else {
        0.55
    };
    let [r, g, b, _] = accent.to_le_bytes();
    rgb(
        (r as f32 * strength) as u8,
        (g as f32 * strength) as u8,
        (b as f32 * strength) as u8,
    )
}

fn finite_corners(
    parent: AffineTransform,
    local: AffineTransform,
    width: f32,
    height: f32,
) -> bool {
    let composed = parent.compose(local);
    [-0.5, 0.5].into_iter().all(|x| {
        [-0.5, 0.5].into_iter().all(|y| {
            let p = composed.point(Vec2::new(x * width, y * height));
            p.is_finite() && p.abs().max_element() <= 1_100_000.0
        })
    })
}

pub fn append_symbol(
    frame: &mut FrameBuilder,
    input: &Input,
    kind: SymbolKind,
    index: usize,
    pixels: &mut usize,
    work: &mut usize,
) -> usize {
    if index >= 3 || !valid(input) {
        return 0;
    }
    let symbol = &SYMBOLS[index
        + if kind == SymbolKind::Corporation {
            3
        } else {
            0
        }];
    let count = symbol.body.occupied_count();
    let r = input.rect;
    let scale = (r[2] / SIZE as f32).min(r[3] / SIZE as f32);
    let local = Transform {
        position: Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5),
        scale: Vec2::splat(scale),
        ..Default::default()
    }
    .affine();
    if count > *pixels
        || count > *work
        || !finite_corners(input.parent, local, SIZE as f32, SIZE as f32)
    {
        return 0;
    }
    let clip = input.clip.unwrap_or(r);
    let full = clip[0] <= r[0]
        && clip[1] <= r[1]
        && clip[0] + clip[2] >= r[0] + r[2]
        && clip[1] + clip[3] >= r[1] + r[3];
    let signal = emitter(symbol.accent, input);
    frame.push_body_affine_colored(&symbol.body, input.parent.compose(local), |cell, color| {
        if !full
            && [
                Vec2::splat(-0.5),
                Vec2::new(0.5, -0.5),
                Vec2::new(-0.5, 0.5),
                Vec2::splat(0.5),
            ]
            .into_iter()
            .map(|c| local.point(cell + c))
            .any(|p| {
                p.x < clip[0] || p.y < clip[1] || p.x > clip[0] + clip[2] || p.y > clip[1] + clip[3]
            })
        {
            0
        } else if input.disabled {
            muted(color)
        } else if color == EMITTER {
            signal
        } else {
            color
        }
    });
    *pixels -= count;
    *work -= count;
    count
}

/// Quiet entry plate, not an interactive button. Nine cells and nine transforms;
/// fixed geometry while tiny emitter colors signal actual focus/selection.
pub fn append_entry(
    frame: &mut FrameBuilder,
    input: &Input,
    category: Option<usize>,
    pixels: &mut usize,
    work: &mut usize,
) -> usize {
    const COUNT: usize = 9;
    if !valid(input) || category.is_some_and(|i| i >= 3) || COUNT > *pixels || COUNT > *work {
        return 0;
    }
    let r = input.rect;
    let c = 3.0_f32.min(r[2] * 0.25).min(r[3] * 0.25);
    let inset = c * 0.25;
    let accent = category.map_or(PLATINUM, |i| CATEGORY_COLORS[i]);
    let signal = emitter(accent, input);
    let cue = if input.selected {
        accent
    } else if input.active {
        signal
    } else {
        ALLOY
    };
    let parts = [
        ([r[0] + c * 0.4, r[1], r[2] - c * 0.8, r[3]], RECESS),
        ([r[0], r[1] + c * 0.4, r[2], r[3] - c * 0.8], RECESS),
        (
            [r[0] + c, r[1] + r[3] - inset, r[2] - 2.0 * c, inset],
            ALLOY,
        ),
        (
            [r[0] + c, r[1] + r[3] - 2.0 * inset, r[2] - 2.0 * c, inset],
            GRAPHITE,
        ),
        (
            [r[0] + inset, r[1] + c, inset, (r[3] - 2.0 * c).min(14.0)],
            cue,
        ),
        (
            [r[0] + c, r[1] + inset, (r[2] - 2.0 * c) * 0.24, inset],
            signal,
        ),
        ([r[0] + c, r[1] + 2.0 * inset, inset, inset], PLATINUM),
        (
            [
                r[0] + r[2] - c - inset,
                r[1] + r[3] - 3.0 * inset,
                inset,
                inset,
            ],
            GOLD,
        ),
        (
            [
                r[0] + r[2] - c - inset,
                r[1] + r[3] - 4.0 * inset,
                inset,
                inset,
            ],
            GRAPHITE,
        ),
    ];
    let affine = |r: [f32; 4]| {
        Transform {
            position: Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5),
            scale: Vec2::new(r[2], r[3]),
            ..Default::default()
        }
        .affine()
    };
    if parts
        .iter()
        .any(|(r, _)| !finite_corners(input.parent, affine(*r), 1.0, 1.0))
    {
        return 0;
    }
    let clip = input.clip.unwrap_or(r);
    for (part, color) in parts {
        let left = part[0].max(clip[0]);
        let top = part[1].max(clip[1]);
        let right = (part[0] + part[2]).min(clip[0] + clip[2]);
        let bottom = (part[1] + part[3]).min(clip[1] + clip[3]);
        let visible = right > left && bottom > top;
        let local = affine(if visible {
            [left, top, right - left, bottom - top]
        } else {
            part
        });
        frame.push_body_affine_colored(&CELL, input.parent.compose(local), |_, _| {
            if !visible {
                0
            } else if input.disabled {
                muted(color)
            } else {
                color
            }
        });
    }
    *pixels -= COUNT;
    *work -= COUNT;
    COUNT
}

#[cfg(test)]
#[path = "collection_ui_tests.rs"]
mod tests;
