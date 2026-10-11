//! Generated artwork and semantic DOM layout rendered as actual engine cells.
use crate::pixel_ui::RoyaleMono;
use engine_core::{AffineTransform, FrameBuilder, PixelBody, Transform, Vec2};
use serde::Deserialize;
use std::{collections::HashMap, sync::LazyLock};

pub const MAX_UI_PIXELS: usize = 60_000;
const WHITE: u32 = 0xff_f6ead9;
const CYAN: u32 = 0xff_f4e887;
const DIM: u32 = 0xff_a48563;
const GOLD: u32 = 0xff_85d4ff;

const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    u32::from_le_bytes([r, g, b, 255])
}
const BUTTON_TEXT: u32 = rgb(224, 236, 241);
const BUTTON_DISABLED_TEXT: u32 = rgb(139, 155, 166);
/// Semantic action colors are independent of transient hover and tab selection.
fn button_accent(kind: &str) -> u32 {
    match kind {
        "primary-button" => rgb(106, 237, 193),
        "exit-button" => rgb(220, 125, 130),
        "restart-button" => rgb(255, 215, 131),
        "offense-button" | "offense-text" => rgb(255, 147, 119),
        "defense-button" | "defense-text" => rgb(92, 191, 255),
        "passive-button" | "passive-text" => rgb(159, 225, 138),
        _ => rgb(88, 222, 239),
    }
}
fn button_tone(color: u32, strength: f32) -> u32 {
    let [r, g, b, _] = color.to_le_bytes();
    rgb(
        (7.0 + r as f32 * strength).min(255.0) as u8,
        (12.0 + g as f32 * strength).min(255.0) as u8,
        (18.0 + b as f32 * strength).min(255.0) as u8,
    )
}
fn intersection(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let left = a[0].max(b[0]);
    let top = a[1].max(b[1]);
    [
        left,
        top,
        ((a[0] + a[2]).min(b[0] + b[2]) - left).max(0.0),
        ((a[1] + a[3]).min(b[1] + b[3]) - top).max(0.0),
    ]
}

fn contained_region(region: [f32; 4], bounds: [f32; 4]) -> Option<[f32; 4]> {
    // CSS BCR arithmetic may differ by a subpixel. Recover only that small
    // measurement roundoff and intersect before emitting any native geometry.
    const ROUNDING: f32 = 1.0 / 64.0;
    (region.iter().all(|v| v.is_finite() && v.abs() <= 8192.0)
        && region[2] > 0.0
        && region[3] > 0.0
        && region[0] >= bounds[0] - ROUNDING
        && region[1] >= bounds[1] - ROUNDING
        && region[0] + region[2] <= bounds[0] + bounds[2] + ROUNDING
        && region[1] + region[3] <= bounds[1] + bounds[3] + ROUNDING)
        .then(|| intersection(region, bounds))
}

fn setting_regions(node: &UiNode) -> Option<([f32; 4], [f32; 4])> {
    let r = node.rect;
    let (label, control) = if node.kind == "slider" {
        let pad = if r[3] >= 35.0 { 6.0 } else { 2.0 };
        let gap = (r[3] * 0.1).min(6.0);
        let height = (r[3] - pad * 2.0 - gap - RoyaleMono::HEIGHT).min(11.0);
        (
            [
                r[0] + pad,
                r[1] + pad,
                r[2] - pad * 2.0,
                r[3] - pad * 2.0 - gap - height,
            ],
            [
                r[0] + pad,
                r[1] + r[3] - pad - height,
                r[2] - pad * 2.0,
                height,
            ],
        )
    } else {
        let pad = 6.0_f32.min(r[3] * 0.15);
        let size = 12.0_f32.min(r[2] - pad * 2.0).min(r[3] - pad * 2.0);
        (
            [
                r[0] + pad,
                r[1] + pad,
                r[2] - size - pad * 2.0 - 4.0,
                r[3] - pad * 2.0,
            ],
            [
                r[0] + r[2] - pad - size,
                r[1] + (r[3] - size) * 0.5,
                size,
                size,
            ],
        )
    };
    let label = contained_region(node.label_rect.unwrap_or(label), r)?;
    let control = contained_region(node.control_rect.unwrap_or(control), r)?;
    let overlap = intersection(label, control);
    if overlap[2] > 0.0 && overlap[3] > 0.0 {
        return None;
    }
    Some((label, control))
}

fn settings_font(rect: [f32; 4], text: &str, requested: f32) -> f32 {
    let requested = if requested.is_finite() {
        requested.round().clamp(1.0, 4.0) as u32
    } else {
        1
    };
    for size in (1..=requested).rev() {
        let font = size as f32;
        let columns = ((rect[2] + font) / (RoyaleMono::ADVANCE * font)).floor() as usize;
        let rows = ((rect[3] + font) / (8.0 * font)).floor() as usize;
        if columns > 0 && rows > 0 && wrap(text, columns).len() <= rows {
            return font;
        }
    }
    1.0
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct UiScene {
    pub groups: Vec<UiGroup>,
}
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct UiGroup {
    pub position: [f32; 2],
    pub nodes: Vec<UiNode>,
}
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct UiNode {
    pub kind: String,
    pub rect: [f32; 4],
    pub text: String,
    pub font: f32,
    pub value: f32,
    pub active: bool,
    /// Persistent navigation state; hover/focus remains `active`.
    pub selected: bool,
    pub disabled: bool,
    pub clip: Option<[f32; 4]>,
    /// Actual semantic label/input bounds, in the same group-local units as rect.
    /// Older clients may omit either region or explicitly send null.
    pub label_rect: Option<[f32; 4]>,
    pub control_rect: Option<[f32; 4]>,
    pub portrait_expression: u8,
    pub portrait_speaking: bool,
}

#[derive(Deserialize)]
struct Atlas {
    version: u32,
    palette: Vec<u32>,
    sprites: HashMap<String, Sprite>,
}
#[derive(Deserialize)]
struct Sprite {
    width: usize,
    height: usize,
    indices: Vec<u8>,
}
struct Art {
    body: PixelBody,
    sprite: Sprite,
}
struct Assets {
    palette: Vec<u32>,
    sprites: HashMap<String, Art>,
}

/// Original corner silhouettes remain intact. Ornament cells are sampled once
/// at the center; the remaining rails use a neutral two-cell metal strip.
#[derive(Clone, Copy)]
struct FrameGeometry {
    size: [usize; 2],
    source: [usize; 2],
    rail: [usize; 2],
    corner: [usize; 2],
    neutral: [usize; 2],
}
impl FrameGeometry {
    fn new(name: &str, size: [usize; 2], source: [usize; 2]) -> Self {
        let panel = name == "panel";
        let mut geometry = Self {
            size,
            source,
            rail: if panel { [8, 8] } else { [4, 4] },
            corner: if panel { [24, 16] } else { [8, 8] },
            neutral: if panel { [28, 16] } else { [16, 8] },
        };
        for axis in 0..2 {
            geometry.corner[axis] = geometry.corner[axis]
                .min(size[axis] / 2)
                .min(source[axis] / 2);
            geometry.rail[axis] = geometry.rail[axis].min(geometry.corner[axis]);
            geometry.neutral[axis] = geometry.neutral[axis].min(source[axis].saturating_sub(2));
        }
        geometry
    }

    fn source_at(self, p: [usize; 2]) -> [usize; 2] {
        let edge = |n: usize, axis: usize| {
            if n < self.size[axis] / 2 {
                n
            } else {
                self.source[axis] - (self.size[axis] - n)
            }
        };
        let corner_x = p[0] < self.corner[0] || p[0] >= self.size[0] - self.corner[0];
        let corner_y = p[1] < self.corner[1] || p[1] >= self.size[1] - self.corner[1];
        if corner_x && corner_y {
            [edge(p[0], 0), edge(p[1], 1)]
        } else if p[1] < self.rail[1] || p[1] >= self.size[1] - self.rail[1] {
            let ornament = 20.min(self.size[0].saturating_sub(self.corner[0] * 2));
            let begin = (self.size[0] - ornament) / 2;
            let sx = if (begin..begin + ornament).contains(&p[0]) {
                (self.source[0] - ornament) / 2 + p[0] - begin
            } else {
                self.neutral[0] + p[0] % 2
            };
            [sx, edge(p[1], 1)]
        } else {
            [edge(p[0], 0), self.neutral[1] + p[1] % 2]
        }
    }
}
static CELL: LazyLock<PixelBody> = LazyLock::new(|| PixelBody::from_ascii(&["#"], &[('#', WHITE)]));
// Original 24-cell controls: final-resolution silhouettes, insets and metal details.
// A chart marker is its own compact occupied object. DOM hit regions stay
// larger than this art; selecting a marker never adds a generic button frame.
static MAP_HOME: LazyLock<PixelBody> = LazyLock::new(|| {
    PixelBody::from_ascii(
        &[
            "....#....",
            "....#....",
            "..#####..",
            "..#...#..",
            "###.+.###",
            "..#...#..",
            "..#####..",
            "....#....",
            "....#....",
        ],
        &[('#', WHITE), ('+', DIM)],
    )
});
static MAP_PLAYER: LazyLock<PixelBody> = LazyLock::new(|| {
    PixelBody::from_ascii(
        &[
            ".........",
            ".##......",
            "..###....",
            "...####..",
            "...++####",
            "...####..",
            "..###....",
            ".##......",
            ".........",
        ],
        &[('#', WHITE), ('+', DIM)],
    )
});
static MAP_SITES: LazyLock<[PixelBody; 6]> = LazyLock::new(|| {
    [
        [
            "....#....",
            "...###...",
            "....#....",
            ".#..#..#.",
            "#########",
            ".#..#..#.",
            "....#....",
            "...###...",
            "....#....",
        ],
        [
            "....#....",
            "...#.#...",
            "..#...#..",
            ".#..+..#.",
            "#..+++..#",
            ".#..+..#.",
            "..#...#..",
            "...#.#...",
            "....#....",
        ],
        [
            "....##...",
            "...##....",
            "..##..##.",
            ".##..##..",
            "##..##...",
            "...##..##",
            "..##..##.",
            ".##..##..",
            "....##...",
        ],
        [
            "..#####..",
            ".#.....#.",
            "#..#.#..#",
            "#..#.#..#",
            "#..+++..#",
            "#..#.#..#",
            "#..#.#..#",
            ".#.....#.",
            "..#####..",
        ],
        [
            "....#....",
            "...###...",
            "..#####..",
            ".........",
            "##..+..##",
            "##.+++.##",
            ".#..+..#.",
            "..#...#..",
            ".........",
        ],
        [
            "#...#...#",
            ".#..#..#.",
            "..#.#.#..",
            "...###...",
            "#########",
            "...###...",
            "..#.#.#..",
            ".#..#..#.",
            "#...#...#",
        ],
    ]
    .map(|rows| PixelBody::from_ascii(&rows, &[('#', WHITE), ('+', DIM)]))
});
fn map_color(kind: usize) -> u32 {
    [
        rgb(88, 222, 239),
        rgb(194, 142, 244),
        rgb(118, 230, 177),
        rgb(244, 180, 105),
        rgb(216, 224, 122),
        rgb(141, 193, 252),
    ][kind.min(5)]
}

/// Midpoint perimeter, eight octants with diagonal/axis duplicates removed.
/// It streams at most 8*(radius+1) local cells without a temporary bitmap.
fn map_ring_cells(radius: i32) -> impl Iterator<Item = Vec2> {
    std::iter::successors(Some((radius, 0, 1 - radius)), |&(x, y, error)| {
        let y = y + 1;
        let (x, error) = if error < 0 {
            (x, error + 2 * y + 1)
        } else {
            (x - 1, error + 2 * (y - (x - 1)) + 1)
        };
        (y <= x).then_some((x, y, error))
    })
    .flat_map(|(x, y, _)| {
        let points = [
            (x, y),
            (y, x),
            (-y, x),
            (-x, y),
            (-x, -y),
            (-y, -x),
            (y, -x),
            (x, -y),
        ];
        points.into_iter().enumerate().filter_map(move |(i, p)| {
            (!points[..i].contains(&p)).then_some(Vec2::new(p.0 as f32, p.1 as f32))
        })
    })
}
// Original comms portrait, authored here as object-local cells. The dark suit,
// helmet, brows, headset and face remain readable at its native 24×32 size.
const PORTRAIT_MASK: [&str; 32] = [
    "",
    "kkkkkkkk",
    "kkHHHHHHHHkk",
    "kHHcccccccccHk",
    "kHchhhhhhhhhhHck",
    "kHchhhhhhhhhhhhHck",
    "kHchhhkkkkkkhhhHck",
    "kHchhkFFFFFFFFkhHck",
    "kHchkFlFFFFFFlFkhHck",
    "khhkFllFFFFFFllFkhhk",
    "krhkFkkkFFkkkFFkhrrk",
    "krhkFpeFFFFepFFkhrck",
    "krhkFlFFFFFFFlFkhrrk",
    "khhkFlllFFlllFFkhhhk",
    "khkFllFffFllFFkhk",
    "kkFllFffFllFFkk",
    "kFlFFFFFFFlFk",
    "kFFmmmFFFk",
    "kFFFmFFFFk",
    "kFFFFFFk",
    "kkFFFFkk",
    "kwkffffkwk",
    "kJJkwwwwkJJk",
    "kJJJJkwwkJJJJk",
    "kJJJJjjkkjjJJJJk",
    "kJJJJjjjggjjjJJJJk",
    "kJJJJjjjjggjjjjJJJJk",
    "kJJJjjjjjggjjjjjJJJk",
    "kJJjjjjjwwwwjjjjjJJk",
    "kJJjjjjjwwwwjjjjjJJk",
    "kJJjjjjjjjjjjjjjjJJk",
    "kkkkkkkkkkkkkkkkkkkkkkkk",
];
struct CommsPortrait {
    body: PixelBody,
    eyes: u32,
    pupil: u32,
    skin: u32,
}
static PORTRAITS: LazyLock<[CommsPortrait; 8]> = LazyLock::new(|| {
    std::array::from_fn(|style| {
        if style >= 3 {
            let names = [
                "campaign_portrait_3",
                "campaign_portrait_4",
                "campaign_portrait_5",
                "campaign_portrait_6",
                "campaign_portrait_7",
            ];
            let body = CAMPAIGN_ASSETS.sprites[names[style - 3]].body.clone();
            let eye_x = if style == 3 { 9 } else { 8 };
            return CommsPortrait {
                eyes: body.color_at(eye_x, 11).expect("authored portrait eye"),
                pupil: u32::from_le_bytes([12, 25, 43, 255]),
                skin: body.color_at(12, 16).expect("authored portrait face"),
                body,
            };
        }
        let (skin, shade, highlight, helmet, eyes, accent) = match style {
            1 => (
                [128, 184, 190],
                [57, 100, 119],
                [192, 231, 226],
                [70, 124, 137],
                [255, 207, 105],
                [92, 230, 164],
            ),
            2 => (
                [182, 122, 107],
                [104, 61, 75],
                [226, 168, 137],
                [113, 73, 130],
                [234, 228, 245],
                [214, 116, 198],
            ),
            _ => (
                [229, 173, 122],
                [146, 86, 75],
                [255, 220, 168],
                [69, 103, 134],
                [229, 244, 255],
                [73, 223, 228],
            ),
        };
        let rgba = |rgb: [u8; 3]| u32::from_le_bytes([rgb[0], rgb[1], rgb[2], 255]);
        let palette = [
            ('k', rgba([8, 19, 33])),
            ('h', rgba([37, 55, 77])),
            ('H', rgba(helmet)),
            ('c', rgba(accent)),
            ('f', rgba(shade)),
            ('F', rgba(skin)),
            ('l', rgba(highlight)),
            ('e', rgba(eyes)),
            ('p', rgba([12, 25, 43])),
            ('m', rgba([96, 47, 61])),
            ('j', rgba([25, 43, 64])),
            ('J', rgba(helmet)),
            ('w', rgba([201, 210, 202])),
            ('g', rgba([226, 180, 107])),
            ('r', rgba(accent)),
        ];
        let rows: Vec<String> = PORTRAIT_MASK
            .iter()
            .map(|row| {
                assert!(row.len() <= 24);
                let left = (24 - row.len()) / 2;
                format!("{}{:width$}", " ".repeat(left), row, width = 24 - left)
            })
            .collect();
        let rows: Vec<_> = rows.iter().map(String::as_str).collect();
        CommsPortrait {
            body: PixelBody::from_ascii(&rows, &palette),
            eyes: rgba(eyes),
            pupil: rgba([12, 25, 43]),
            skin: rgba(skin),
        }
    })
});
fn load_assets(source: &str) -> Assets {
    let atlas: Atlas = serde_json::from_str(source).expect("compiled UI atlas must be valid JSON");
    assert_eq!(atlas.version, 1);
    assert_eq!(atlas.palette.len(), 64);
    assert_eq!(atlas.palette[0], 0);
    let key = |index: usize| char::from_u32(0xe000 + index as u32).unwrap();
    let palette: Vec<_> = atlas
        .palette
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, &color)| (key(i), color))
        .collect();
    let sprites = atlas
        .sprites
        .into_iter()
        .map(|(name, sprite)| {
            assert!((1..=512).contains(&sprite.width) && (1..=512).contains(&sprite.height));
            assert_eq!(sprite.indices.len(), sprite.width * sprite.height);
            assert!(
                sprite
                    .indices
                    .iter()
                    .all(|&i| (i as usize) < atlas.palette.len())
            );
            let rows: Vec<String> = sprite
                .indices
                .chunks_exact(sprite.width)
                .map(|row| {
                    row.iter()
                        .map(|&i| if i == 0 { ' ' } else { key(i as usize) })
                        .collect()
                })
                .collect();
            let borrowed: Vec<_> = rows.iter().map(String::as_str).collect();
            (
                name,
                Art {
                    body: PixelBody::from_ascii(&borrowed, &palette),
                    sprite,
                },
            )
        })
        .collect();
    Assets {
        palette: atlas.palette,
        sprites,
    }
}
static ASSETS: LazyLock<Assets> =
    LazyLock::new(|| load_assets(include_str!("../../../assets/ui/atlas.json")));
static CAMPAIGN_ASSETS: LazyLock<Assets> =
    LazyLock::new(|| load_assets(include_str!("../../../assets/ui/campaign-atlas.json")));
const CAMPAIGN_SHIPS: [&str; 10] = [
    "campaign_ship_0",
    "campaign_ship_1",
    "campaign_ship_2",
    "campaign_ship_3",
    "campaign_ship_4",
    "campaign_ship_5",
    "campaign_ship_6",
    "campaign_ship_7",
    "campaign_ship_8",
    "campaign_ship_9",
];
#[cfg(test)]
const CAMPAIGN_PATCHES: [&str; 8] = [
    "campaign_patch_0",
    "campaign_patch_1",
    "campaign_patch_2",
    "campaign_patch_3",
    "campaign_patch_4",
    "campaign_patch_5",
    "campaign_patch_6",
    "campaign_patch_7",
];
fn selection(value: f32, count: usize) -> usize {
    if value.is_finite() {
        value.floor().clamp(0.0, (count - 1) as f32) as usize
    } else {
        0
    }
}

struct Painter<'a> {
    frame: &'a mut FrameBuilder,
    parent: AffineTransform,
    remaining: usize,
    viewport: Vec2,
    clip: Option<[f32; 4]>,
    work_remaining: usize,
}
impl Painter<'_> {
    fn map_field(&mut self, node: &UiNode) {
        let r = node.rect;
        let old_clip = self.clip;
        self.clip = Some(old_clip.map_or(r, |c| intersection(r, c)));
        let mut strip = |rect, color| {
            if self.work_remaining > 0 {
                self.work_remaining -= 1;
                self.rect(rect, color);
            }
        };
        strip(r, rgb(4, 10, 17));
        // Rectangular local quad strips remain in the parent group. A fixed
        // logical grid supports chart orientation without inventing world zones.
        for x in (16..r[2].ceil() as usize).step_by(16) {
            strip([r[0] + x as f32, r[1], 1.0, r[3]], rgb(13, 25, 34));
        }
        for y in (16..r[3].ceil() as usize).step_by(16) {
            strip([r[0], r[1] + y as f32, r[2], 1.0], rgb(13, 25, 34));
        }
        for border in [
            [r[0], r[1], r[2], 1.0],
            [r[0], r[1] + r[3] - 1.0, r[2], 1.0],
            [r[0], r[1], 1.0, r[3]],
            [r[0] + r[2] - 1.0, r[1], 1.0, r[3]],
        ] {
            strip(border, rgb(38, 67, 80));
        }
        self.clip = old_clip;
    }

    fn map_ring(&mut self, node: &UiNode) {
        let r = node.rect;
        let radius = ((r[2].min(r[3]) - 1.0) * 0.5).floor() as i32;
        if radius < 1 || self.remaining == 0 || self.work_remaining == 0 {
            return;
        }
        let center = Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5);
        let clip = self.clip.map_or(r, |c| intersection(r, c));
        let local = Transform {
            position: center,
            ..Transform::default()
        }
        .affine();
        let color = match selection(node.value, 3) {
            1 => rgb(168, 123, 62),
            2 => rgb(61, 135, 153),
            _ => rgb(72, 169, 187),
        };
        let mut attempted = 0;
        let cells = map_ring_cells(radius)
            .take(self.work_remaining)
            .filter_map(|p| {
                attempted += 1;
                let c = p + center;
                (c.x - 0.5 >= clip[0]
                    && c.x + 0.5 <= clip[0] + clip[2]
                    && c.y - 0.5 >= clip[1]
                    && c.y + 0.5 <= clip[1] + clip[3])
                    .then_some((p, color))
            })
            .take(self.remaining);
        let count = self.frame.push_cells_affine_material(
            self.parent.compose(local),
            engine_core::PixelMaterial::default(),
            cells,
        );
        self.remaining -= count;
        self.work_remaining -= attempted;
    }

    fn map_marker(&mut self, node: &UiNode) {
        let (body, accent, rotation) = match node.kind.as_str() {
            "map-home" => (&*MAP_HOME, map_color(0), 0.0),
            "map-player" => (
                &*MAP_PLAYER,
                rgb(255, 215, 131),
                if node.value.is_finite() {
                    node.value.rem_euclid(std::f32::consts::TAU)
                } else {
                    0.0
                },
            ),
            _ => {
                let kind = selection(node.value, 6);
                (&MAP_SITES[kind], map_color(kind), 0.0)
            }
        };
        let count = body.occupied_count();
        if count > self.remaining || count > self.work_remaining {
            return;
        }
        let r = node.rect;
        // The player rotates as one real object. Fit its transformed cell
        // corners, rather than rotating a pre-fitted square beyond the hitbox.
        let basis = Transform {
            rotation,
            ..Transform::default()
        };
        let extent = body
            .cells()
            .flat_map(|(p, _)| {
                [
                    p + Vec2::splat(-0.5),
                    p + Vec2::new(0.5, -0.5),
                    p + Vec2::new(-0.5, 0.5),
                    p + Vec2::splat(0.5),
                ]
            })
            .map(|p| basis.point(p).abs())
            .fold(Vec2::ZERO, Vec2::max)
            * 2.0;
        let fit = (r[2] / extent.x).min(r[3] / extent.y).min(1.0);
        if !fit.is_finite() || fit <= 0.0 {
            return;
        }
        let local = Transform {
            position: Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5),
            rotation,
            scale: Vec2::splat(fit),
        };
        let clip = self.clip.map_or(r, |c| intersection(r, c));
        let color = if node.disabled {
            BUTTON_DISABLED_TEXT
        } else if node.selected {
            rgb(255, 215, 131)
        } else if node.active {
            brighten(accent)
        } else {
            accent
        };
        self.frame.push_body_affine_colored(
            body,
            self.parent.compose(local.affine()),
            |p, shade| {
                let outside = [
                    p + Vec2::splat(-0.5),
                    p + Vec2::new(0.5, -0.5),
                    p + Vec2::new(-0.5, 0.5),
                    p + Vec2::splat(0.5),
                ]
                .into_iter()
                .map(|p| local.point(p))
                .any(|p| {
                    p.x < clip[0]
                        || p.x > clip[0] + clip[2]
                        || p.y < clip[1]
                        || p.y > clip[1] + clip[3]
                });
                if outside {
                    0
                } else if shade == DIM && !node.disabled {
                    button_tone(color, 0.55)
                } else {
                    color
                }
            },
        );
        self.remaining -= count;
        self.work_remaining -= count;
    }
    fn control_icon(&mut self, node: &UiNode) {
        let Some(body) = crate::kikai_art::icon(&node.kind) else {
            return;
        };
        let accent = match node.kind.as_str() {
            "close-icon" => WHITE,
            "resume-icon" | "map-icon" => rgb(106, 237, 193),
            "restart-icon" => rgb(255, 215, 131),
            "title-icon" | "status-icon" => rgb(182, 201, 219),
            "quit-icon" => rgb(220, 125, 130),
            "save-icon" => [rgb(106, 237, 193), rgb(88, 222, 239), rgb(220, 125, 130)]
                [selection(node.value, 3)],
            _ => rgb(88, 222, 239),
        };
        let count = body.occupied_count();
        if count > self.remaining || count > self.work_remaining {
            return;
        }
        let r = node.rect;
        let fit = (r[2] / body.width as f32).min(r[3] / body.height as f32);
        if !fit.is_finite() || fit <= 0.0 {
            return;
        }
        let scale = fit;
        let local = Transform {
            position: Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5),
            scale: Vec2::splat(scale),
            ..Transform::default()
        };
        let clip = self.clip;
        self.frame.push_body_affine_colored(
            body,
            self.parent.compose(local.affine()),
            |cell, shade| {
                let p = local.point(cell);
                let half = scale * 0.5;
                // Keep entire visible icon cells inside a scrolling clip.
                if clip.is_some_and(|c| {
                    p.x - half < c[0]
                        || p.x + half > c[0] + c[2]
                        || p.y - half < c[1]
                        || p.y + half > c[1] + c[3]
                }) {
                    0
                } else {
                    crate::kikai_art::icon_color(
                        shade,
                        accent,
                        node.active,
                        node.selected,
                        node.disabled,
                    )
                }
            },
        );
        self.remaining -= count;
        self.work_remaining -= count;
    }
    fn pixel(&mut self, position: Vec2, size: f32, color: u32) {
        if self.remaining == 0 || self.work_remaining == 0 {
            return;
        }
        self.work_remaining -= 1;
        if !self.inside_clip(position + Vec2::splat(size * 0.5)) {
            return;
        }
        self.frame.push_pixel(
            self.parent.point(position + Vec2::splat(size * 0.5)),
            size,
            color,
        );
        self.remaining -= 1;
    }
    fn rect(&mut self, rect: [f32; 4], color: u32) {
        if self.remaining == 0 || rect[2] <= 0.0 || rect[3] <= 0.0 {
            return;
        }
        let rect = if let Some(clip) = self.clip {
            let left = rect[0].max(clip[0]);
            let top = rect[1].max(clip[1]);
            [
                left,
                top,
                (rect[0] + rect[2]).min(clip[0] + clip[2]) - left,
                (rect[1] + rect[3]).min(clip[1] + clip[3]) - top,
            ]
        } else {
            rect
        };
        if rect[2] <= 0.0 || rect[3] <= 0.0 {
            return;
        }
        let transform = self.parent.compose(
            Transform {
                position: Vec2::new(rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5),
                scale: Vec2::new(rect[2], rect[3]),
                ..Transform::default()
            }
            .affine(),
        );
        self.frame
            .push_body_affine_colored(&CELL, transform, |_, _| color);
        self.remaining -= 1;
    }
    fn inside_clip(&self, position: Vec2) -> bool {
        self.clip.is_none_or(|r| {
            position.x >= r[0]
                && position.x < r[0] + r[2]
                && position.y >= r[1]
                && position.y < r[1] + r[3]
        })
    }
    fn artwork(
        &mut self,
        name: &str,
        rect: [f32; 4],
        rotation: f32,
        y_offset: f32,
        shimmer: Option<f32>,
        disabled: bool,
    ) {
        let Some(art) = ASSETS.sprites.get(name) else {
            return;
        };
        if art.body.occupied_count() > self.remaining {
            return;
        }
        let fit = (rect[2] / art.body.width as f32).min(rect[3] / art.body.height as f32);
        let scale = if fit >= 1.0 {
            fit.floor()
        } else {
            1.0 / (1.0 / fit).ceil()
        };
        let local = Transform {
            position: Vec2::new(
                rect[0] + rect[2] * 0.5,
                rect[1]
                    + rect[3] * 0.5
                    + y_offset.clamp(
                        -(rect[3] - art.body.height as f32 * scale).max(0.0) * 0.5,
                        (rect[3] - art.body.height as f32 * scale).max(0.0) * 0.5,
                    ),
            ),
            rotation,
            scale: Vec2::splat(scale),
        };
        let transform = self.parent.compose(local.affine());
        let clip = self.clip;
        self.frame
            .push_body_affine_colored(&art.body, transform, |cell, color| {
                let p = local.point(cell);
                if clip.is_some_and(|r| {
                    p.x < r[0] || p.x >= r[0] + r[2] || p.y < r[1] || p.y >= r[1] + r[3]
                }) {
                    0
                } else if disabled {
                    muted(color)
                } else if shimmer.is_some_and(|sweep| (cell.x - cell.y * 0.25 - sweep).abs() < 2.0)
                {
                    brighten(color)
                } else {
                    color
                }
            });
        self.remaining -= art.body.occupied_count();
    }
    fn portrait(&mut self, node: &UiNode, time: f32, reduced: bool) {
        if node.value == 8.0 {
            crate::kikai_art::append_dora(
                self.frame,
                &crate::kikai_art::DoraInput {
                    rect: node.rect,
                    parent: self.parent,
                    clip: self.clip,
                    time,
                    reduced_motion: reduced,
                    expression: crate::kikai_art::DoraExpression::from_label(
                        ["neutral", "focused", "concerned", "pleased"]
                            [node.portrait_expression.min(3) as usize],
                    ),
                    speaking: node.portrait_speaking,
                    disabled: node.disabled,
                },
                &mut self.remaining,
                &mut self.work_remaining,
            );
            return;
        }
        let style = selection(node.value, PORTRAITS.len());
        let art = &PORTRAITS[style];
        let body = &art.body;
        let count = body.occupied_count();
        let r = node.rect;
        let scale = (r[2] / body.width as f32)
            .min(r[3] / body.height as f32)
            .floor();
        if scale < 1.0 || count > self.remaining || count > self.work_remaining {
            return;
        }
        let local = Transform {
            position: Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5),
            scale: Vec2::splat(scale),
            ..Transform::default()
        };
        let clip = self.clip;
        let blink = node.active && !reduced && time.rem_euclid(4.5) > 4.35;
        self.frame.push_body_affine_colored(
            body,
            self.parent.compose(local.affine()),
            |cell, color| {
                let p = local.point(cell);
                if clip.is_some_and(|r| {
                    p.x < r[0] || p.x >= r[0] + r[2] || p.y < r[1] || p.y >= r[1] + r[3]
                }) {
                    0
                } else if node.disabled {
                    muted(color)
                } else if blink && cell.y == -4.5 && (color == art.eyes || color == art.pupil) {
                    art.skin
                } else {
                    color
                }
            },
        );
        self.remaining -= count;
        self.work_remaining -= count;
    }
    fn campaign(&mut self, node: &UiNode, ship: bool, time: f32, reduced_motion: bool) {
        if !ship {
            crate::champion_patches::extract(
                self.frame,
                &crate::champion_patches::PatchInput {
                    index: selection(node.value, crate::champion_patches::COUNT),
                    rect: node.rect,
                    parent: self.parent,
                    clip: self.clip,
                    earned: node.active,
                    disabled: node.disabled,
                    time,
                    reduced_motion,
                },
                &mut self.remaining,
                &mut self.work_remaining,
            );
            return;
        }
        let names: &[&str] = &CAMPAIGN_SHIPS;
        let body = &CAMPAIGN_ASSETS.sprites[names[selection(node.value, names.len())]].body;
        let count = body.occupied_count();
        let r = node.rect;
        let scale = (r[2] / body.width as f32)
            .min(r[3] / body.height as f32)
            .floor();
        if scale < 1.0 || count > self.remaining || count > self.work_remaining {
            return;
        }
        let local = Transform {
            position: Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5),
            scale: Vec2::splat(scale),
            ..Transform::default()
        };
        let clip = self.clip;
        self.frame.push_body_affine_colored(
            body,
            self.parent.compose(local.affine()),
            |cell, color| {
                let p = local.point(cell);
                if clip.is_some_and(|r| {
                    p.x < r[0] || p.x >= r[0] + r[2] || p.y < r[1] || p.y >= r[1] + r[3]
                }) {
                    0
                } else if node.disabled {
                    muted(color)
                } else if node.active {
                    brighten(color)
                } else {
                    color
                }
            },
        );
        self.remaining -= count;
        self.work_remaining -= count;
    }
    fn licensed_ship(&mut self, node: &UiNode) {
        let corporation = [
            game_core::identity::Corporation::Pan,
            game_core::identity::Corporation::Renko,
            game_core::identity::Corporation::Hugo,
        ][selection(node.value, 3)];
        let body = game_core::identity::starter_body(corporation);
        let count = body.occupied_count();
        let r = node.rect;
        let scale = (r[2] / body.width as f32).min(r[3] / body.height as f32);
        if !scale.is_finite()
            || scale <= 0.0
            || count > self.remaining
            || count > self.work_remaining
        {
            return;
        }
        let local = Transform {
            position: Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5),
            scale: Vec2::splat(scale),
            ..Transform::default()
        };
        let clip = self.clip;
        self.frame.push_body_affine_colored(
            body,
            self.parent.compose(local.affine()),
            |cell, color| {
                let p = local.point(cell);
                if clip.is_some_and(|c| {
                    p.x < c[0] || p.y < c[1] || p.x >= c[0] + c[2] || p.y >= c[1] + c[3]
                }) {
                    0
                } else if node.disabled {
                    muted(color)
                } else {
                    color
                }
            },
        );
        self.remaining -= count;
        self.work_remaining -= count;
    }
    fn console_panel(&mut self, r: [f32; 4]) {
        self.rect(r, 0xff_000000);
        self.rect([r[0], r[1] + r[3] - 1.0, r[2], 1.0], CYAN);
        // Sparse fixing pins leave the console's reading surface unadorned.
        if r[2] >= 16.0 && r[3] >= 8.0 {
            for x in [r[0] + 5.0, r[0] + r[2] - 6.0] {
                self.pixel(Vec2::new(x, r[1] + 4.0), 1.0, 0xff_554733);
            }
        }
    }
    fn frame(&mut self, name: &str, rect: [f32; 4], active: bool, disabled: bool) {
        let Some(art) = ASSETS.sprites.get(name) else {
            return;
        };
        self.rect(rect, 0xf5_26140d);
        let width = rect[2].round().max(1.0) as usize;
        let height = rect[3].round().max(1.0) as usize;
        let geometry =
            FrameGeometry::new(name, [width, height], [art.sprite.width, art.sprite.height]);
        if geometry.rail.contains(&0) {
            return;
        }
        for y in 0..height {
            let outer = y < geometry.rail[1] || y >= height - geometry.rail[1];
            let corner = y < geometry.corner[1] || y >= height - geometry.corner[1];
            let side = if corner {
                geometry.corner[0]
            } else {
                geometry.rail[0]
            };
            let left_end = if outer { width } else { side };
            for x in 0..left_end {
                self.frame_cell(art, rect, [x, y], geometry, (active, disabled));
                if self.remaining == 0 || self.work_remaining == 0 {
                    return;
                }
            }
            if !outer {
                for x in width - side..width {
                    self.frame_cell(art, rect, [x, y], geometry, (active, disabled));
                    if self.remaining == 0 || self.work_remaining == 0 {
                        return;
                    }
                }
            }
        }
    }
    fn frame_cell(
        &mut self,
        art: &Art,
        rect: [f32; 4],
        p: [usize; 2],
        geometry: FrameGeometry,
        state: (bool, bool),
    ) {
        if self.work_remaining == 0 {
            return;
        }
        self.work_remaining -= 1;
        let (active, disabled) = state;
        let [sx, sy] = geometry.source_at(p);
        let index = art.sprite.indices[sy * art.sprite.width + sx] as usize;
        if index == 0 {
            return;
        }
        let color = ASSETS.palette[index];
        self.pixel(
            Vec2::new(rect[0] + p[0] as f32, rect[1] + p[1] as f32),
            1.0,
            if disabled {
                muted(color)
            } else if active {
                brighten(color)
            } else {
                color
            },
        );
    }
    fn text(&mut self, rect: [f32; 4], text: &str, font: f32, color: u32, centered: bool) {
        let font = font
            .round()
            .clamp(1.0, 4.0)
            .min((rect[3] / 7.0).floor().max(1.0))
            .min((rect[2] / 5.0).floor().max(1.0));
        let columns = ((rect[2] + font) / (6.0 * font)).floor() as usize;
        let rows = ((rect[3] + font) / (8.0 * font)).floor() as usize;
        if columns == 0 || rows == 0 {
            return;
        }
        let lines = wrap(text, columns);
        let line_count = lines.len().min(rows);
        let top = if centered {
            rect[1] + (rect[3] - line_count as f32 * 8.0 * font + font) * 0.5
        } else {
            rect[1]
        };
        for (row, line) in lines.iter().take(rows).enumerate() {
            let left = if centered {
                rect[0] + (rect[2] - (line.len() as f32 * 6.0 - 1.0) * font) * 0.5
            } else {
                rect[0]
            };
            let text_local = Transform {
                position: Vec2::new(left, top + row as f32 * 8.0 * font),
                scale: Vec2::splat(font),
                ..Transform::default()
            }
            .affine();
            for (column, character) in line.chars().enumerate() {
                let body = RoyaleMono::body(character);
                let count = body.occupied_count();
                if count == 0 {
                    continue;
                }
                if count > self.remaining || count > self.work_remaining {
                    return;
                }
                let letter_local = RoyaleMono::letter_transform(text_local, column);
                let letter_world = self.parent.compose(letter_local);
                let clip = self.clip;
                self.frame
                    .push_body_affine_colored(body, letter_world, |cell, _| {
                        let p = letter_local.point(cell);
                        if clip.is_some_and(|r| {
                            p.x < r[0] || p.x >= r[0] + r[2] || p.y < r[1] || p.y >= r[1] + r[3]
                        }) {
                            0
                        } else {
                            color
                        }
                    });
                self.remaining -= count;
                self.work_remaining -= count;
            }
        }
    }
    fn button_label(&mut self, node: &UiNode, font: f32, color: u32) {
        let r = node.rect;
        let padding_x = if r[2] < 40.0 { 2.0 } else { 6.0 };
        let width = (r[2] - padding_x * 2.0).max(0.0);
        let height = (r[3] - if node.kind == "nav-tab" { 5.0 } else { 4.0 }).max(0.0);
        if width < RoyaleMono::WIDTH || height < RoyaleMono::HEIGHT {
            return;
        }
        let mut label: String = node
            .text
            .chars()
            .take(TEXT_CHARS)
            .map(|c| if c.is_whitespace() { ' ' } else { c })
            .collect();
        let characters = label.chars().count();
        let units = (characters as f32 * RoyaleMono::ADVANCE - 1.0).max(RoyaleMono::WIDTH);
        let font = font
            .min((width / units).floor().max(1.0))
            .min((height / RoyaleMono::HEIGHT).floor().max(1.0));
        let columns = ((width + font) / (RoyaleMono::ADVANCE * font)).floor() as usize;
        if characters > columns {
            let visible = if columns >= 4 { columns - 3 } else { columns };
            if let Some((byte, _)) = label.char_indices().nth(visible) {
                label.truncate(byte);
            }
            if columns >= 4 {
                label.push_str("...");
            }
        }
        let content = [r[0] + padding_x, r[1] + 2.0, width, height];
        let previous_clip = self.clip;
        self.clip = Some(previous_clip.map_or(content, |clip| intersection(content, clip)));
        self.text(content, &label, font, color, true);
        self.clip = previous_clip;
    }
    fn settings_label(&mut self, rect: [f32; 4], text: &str, font: f32, color: u32) {
        if rect[2] < RoyaleMono::WIDTH || rect[3] < RoyaleMono::HEIGHT {
            return;
        }
        // DOM text can wrap differently from the native bitmap font. Select the
        // largest complete RoyaleMono layout inside the measured label box.
        // Do not reserve an invented right gutter or truncate a setting name.
        let font = settings_font(rect, text, font);
        let previous_clip = self.clip;
        self.clip = Some(previous_clip.map_or(rect, |clip| intersection(rect, clip)));
        self.text(rect, text, font, color, false);
        self.clip = previous_clip;
    }
    fn setting(&mut self, node: &UiNode, font: f32) {
        let Some((label, control)) = setting_regions(node) else {
            return;
        };
        let r = node.rect;
        let previous_clip = self.clip;
        self.clip = Some(previous_clip.map_or(r, |clip| intersection(r, clip)));
        let accent = if node.disabled {
            BUTTON_DISABLED_TEXT
        } else {
            CYAN
        };
        let border = if node.active && !node.disabled {
            button_tone(CYAN, 0.65)
        } else {
            rgb(41, 59, 68)
        };
        // One quiet card outline. The label and interactive strip are both
        // independently measured and retain their own padding inside it.
        self.rect(r, rgb(10, 18, 24));
        self.rect([r[0], r[1], r[2], 0.5], border);
        self.rect([r[0], r[1] + r[3] - 0.5, r[2], 0.5], border);
        self.rect([r[0], r[1], 0.5, r[3]], border);
        self.rect([r[0] + r[2] - 0.5, r[1], 0.5, r[3]], border);
        self.settings_label(
            label,
            &node.text,
            font,
            if node.disabled {
                BUTTON_DISABLED_TEXT
            } else {
                BUTTON_TEXT
            },
        );
        // Native painted controls never extend beyond the actual input BCR.
        self.clip = Some(previous_clip.map_or(control, |clip| intersection(control, clip)));
        if node.kind == "slider" {
            let value = if node.value.is_finite() {
                node.value.clamp(0.0, 1.0)
            } else {
                0.0
            };
            let thumb_w = control[2].min(6.0);
            let thumb_h = control[3].min(10.0);
            let track = [
                control[0] + thumb_w * 0.5,
                control[1] + control[3] * 0.5 - 1.0,
                (control[2] - thumb_w).max(0.0),
                control[3].min(2.0),
            ];
            self.rect(track, rgb(36, 58, 69));
            self.rect(
                [track[0], track[1], track[2] * value, track[3]],
                button_tone(accent, 0.65),
            );
            let thumb = [
                control[0] + (control[2] - thumb_w) * value,
                control[1] + (control[3] - thumb_h) * 0.5,
                thumb_w,
                thumb_h,
            ];
            self.rect(thumb, accent);
            if thumb_w >= 3.0 && thumb_h >= 4.0 {
                self.rect(
                    [thumb[0] + 1.0, thumb[1] + 1.0, thumb_w - 2.0, 1.0],
                    BUTTON_TEXT,
                );
                self.rect(
                    [thumb[0] + thumb_w * 0.5, thumb[1] + 3.0, 0.5, thumb_h - 4.0],
                    button_tone(accent, 0.30),
                );
            }
        } else {
            let size = control[2].min(control[3]);
            let x = control[0] + (control[2] - size) * 0.5;
            let y = control[1] + (control[3] - size) * 0.5;
            let edge = size.min(1.0);
            self.rect([x, y, size, size], border);
            self.rect(
                [
                    x + edge,
                    y + edge,
                    (size - edge * 2.0).max(0.0),
                    (size - edge * 2.0).max(0.0),
                ],
                rgb(8, 15, 20),
            );
            if node.value > 0.5 && size >= 5.0 {
                // A stepped check has open space around it; no solid square
                // obscures the checkbox state or its actual semantic hitbox.
                let unit = size / 9.0;
                for (cx, cy) in [
                    (2.0, 4.0),
                    (3.0, 5.0),
                    (4.0, 6.0),
                    (5.0, 5.0),
                    (6.0, 4.0),
                    (7.0, 3.0),
                ] {
                    self.rect([x + cx * unit, y + cy * unit, unit, unit], accent);
                }
            }
        }
        self.clip = previous_clip;
    }
    fn semantic_button(&mut self, node: &UiNode, font: f32) {
        let r = node.rect;
        let accent = button_accent(&node.kind);
        let selected = node.kind == "nav-tab" && node.selected;
        let active = node.active && !node.disabled;
        let fill = if node.disabled {
            rgb(10, 16, 22)
        } else {
            button_tone(
                accent,
                if selected {
                    0.20
                } else if active {
                    0.14
                } else {
                    0.07
                },
            )
        };
        let rail = if node.disabled {
            rgb(51, 64, 74)
        } else if selected || active {
            accent
        } else {
            button_tone(accent, 0.40)
        };
        if r[2] < 3.0 || r[3] < 3.0 {
            self.rect(r, fill);
            return;
        }
        // Stepped corners, continuous rails and one quiet inner bevel.
        self.rect([r[0] + 1.0, r[1], r[2] - 2.0, r[3]], fill);
        self.rect([r[0], r[1] + 1.0, r[2], r[3] - 2.0], fill);
        self.rect([r[0] + 1.0, r[1], r[2] - 2.0, 1.0], rail);
        self.rect([r[0] + 1.0, r[1] + r[3] - 1.0, r[2] - 2.0, 1.0], rail);
        self.rect([r[0], r[1] + 1.0, 1.0, r[3] - 2.0], rail);
        self.rect([r[0] + r[2] - 1.0, r[1] + 1.0, 1.0, r[3] - 2.0], rail);
        if selected && r[2] >= 10.0 && r[3] >= 10.0 {
            self.rect(
                [r[0] + 3.0, r[1] + r[3] - 3.0, r[2] - 6.0, 2.0],
                if node.disabled {
                    BUTTON_DISABLED_TEXT
                } else {
                    accent
                },
            );
        }
        self.button_label(
            node,
            font,
            if node.disabled {
                BUTTON_DISABLED_TEXT
            } else if selected {
                accent
            } else {
                BUTTON_TEXT
            },
        );
    }
    fn node(&mut self, node: &UiNode, time: f32, reduced: bool) {
        let r = node.rect;
        let font = if node.font.is_finite() {
            node.font.round().clamp(1.0, 4.0)
        } else {
            1.0
        };
        let color = if node.disabled {
            DIM
        } else if node.active {
            CYAN
        } else {
            WHITE
        };
        match node.kind.as_str() {
            "text" => self.text(r, &node.text, font, color, false),
            "status-text" | "warning-text" => {
                // The measured status slot is distinct from its accessible
                // 44px hitbox. One logical pixel font stays stable on zoom;
                // clipping covers full glyph cells, not only their centers.
                let color = if node.disabled {
                    BUTTON_DISABLED_TEXT
                } else if node.kind == "warning-text" {
                    crate::pixel_ui::warning_color(
                        if node.value.is_finite() {
                            node.value.clamp(0.0, 2.0) as u8
                        } else {
                            0
                        },
                        time,
                        reduced,
                    )
                } else if node.active {
                    CYAN
                } else {
                    DIM
                };
                crate::pixel_ui::micro_text(
                    self.frame,
                    self.parent,
                    r,
                    &node.text,
                    color,
                    self.clip,
                    &mut self.remaining,
                    &mut self.work_remaining,
                );
            }
            "offense-text" | "defense-text" | "passive-text" => self.text(
                r,
                &node.text,
                font,
                if node.disabled {
                    BUTTON_DISABLED_TEXT
                } else {
                    button_accent(&node.kind)
                },
                false,
            ),
            "panel" => self.frame("panel", r, node.active, node.disabled),
            "console-panel" => self.console_panel(r),
            "portrait" => self.portrait(node, time, reduced),
            "licensed-ship" => self.licensed_ship(node),
            "campaign-patch" => self.campaign(node, false, time, reduced),
            "campaign-ship" => self.campaign(node, true, time, reduced),
            "close-icon" | "save-icon" | "settings-icon" | "resume-icon" | "restart-icon"
            | "title-icon" | "quit-icon" | "status-icon" | "map-icon" | "fullscreen-icon" => {
                self.control_icon(node)
            }
            "map-field" => self.map_field(node),
            "map-ring" => self.map_ring(node),
            "map-home" | "map-player" | "map-site" => self.map_marker(node),
            "facility-label" => {
                // The shell selects a safe label side without moving the real
                // facility hit area. Nonfinite values retain the below default.
                let placement = selection(node.value, 3);
                if placement == 2 {
                    return;
                }
                let label = [
                    r[0] + r[2] * 0.5 - 50.0,
                    if placement == 1 {
                        r[1] - 12.0
                    } else {
                        r[1] + r[3] + 2.0
                    },
                    100.0,
                    10.0,
                ];
                let previous_clip = self.clip;
                self.clip = Some(previous_clip.map_or(label, |clip| intersection(label, clip)));
                self.text(
                    label,
                    &node.text,
                    1.0,
                    if node.disabled {
                        BUTTON_DISABLED_TEXT
                    } else if node.active {
                        CYAN
                    } else {
                        rgb(98, 143, 156)
                    },
                    true,
                );
                self.clip = previous_clip;
            }
            "input" => {
                // A console command line belongs to the black console surface.
                // It has no button frame; the DOM input still owns all editing.
                let previous_clip = self.clip;
                self.clip = Some(previous_clip.map_or(r, |clip| intersection(r, clip)));
                self.rect(r, rgb(0, 0, 0));
                self.rect([r[0], r[1], r[2], 1.0], rgb(38, 55, 63));
                let height = (r[3] - 6.0).max(RoyaleMono::HEIGHT);
                let font = font.min((height / RoyaleMono::HEIGHT).floor().max(1.0));
                let advance = RoyaleMono::ADVANCE * font;
                let columns = ((r[2] + font) / advance).floor() as usize;
                let chars: Vec<_> = node.text.chars().take(256).collect();
                let caret = if node.value.is_finite() {
                    (node.value.max(0.0) as usize).min(chars.len())
                } else {
                    chars.len()
                };
                // Leave room for the prompt and caret; scroll toward the edited
                // position rather than always displaying the end of the command.
                let capacity = columns.saturating_sub(3);
                let begin = caret.saturating_sub(capacity);
                let end = chars.len().min(begin + capacity);
                let y = r[1] + (r[3] - RoyaleMono::HEIGHT * font) * 0.5;
                for (column, character) in ['>', ' '].iter().chain(&chars[begin..end]).enumerate() {
                    let mut encoded = [0; 4];
                    self.text(
                        [
                            r[0] + column as f32 * advance,
                            y,
                            RoyaleMono::WIDTH * font,
                            RoyaleMono::HEIGHT * font,
                        ],
                        character.encode_utf8(&mut encoded),
                        font,
                        color,
                        false,
                    );
                }
                if node.active && (reduced || (time * 2.0).floor().rem_euclid(2.0) == 0.0) {
                    self.rect(
                        [
                            r[0] + (2 + caret - begin) as f32 * advance,
                            y,
                            font,
                            RoyaleMono::HEIGHT * font,
                        ],
                        GOLD,
                    );
                }
                self.clip = previous_clip;
            }
            "button" => {
                self.frame("button", r, node.active, node.disabled);
                self.button_label(node, font, color);
            }
            "nav-tab" | "primary-button" | "exit-button" | "offense-button" | "defense-button"
            | "passive-button" | "restart-button" => self.semantic_button(node, font),
            "logo" => {
                let width = ASSETS
                    .sprites
                    .get("logo")
                    .map_or(256.0, |a| a.body.width as f32);
                let settle = if reduced {
                    0.0
                } else {
                    (6.0 * (-time.max(0.0) * 4.0).exp()).round()
                };
                let shimmer = (!reduced)
                    .then_some((time * 24.0).rem_euclid(width + 40.0) - width * 0.5 - 20.0);
                self.artwork("logo", r, 0.0, settle, shimmer, node.disabled);
            }
            "seal" => {
                let rotation = if reduced {
                    0.0
                } else {
                    (time * 0.3).sin() * 0.08
                };
                let shimmer = (!reduced).then_some((time * 15.0).rem_euclid(60.0) - 30.0);
                self.artwork("seal", r, rotation, 0.0, shimmer, node.disabled);
            }
            "slider" | "toggle" => self.setting(node, font),
            "rank" => {
                for i in 0..5 {
                    let x = r[0] + i as f32 * 10.0;
                    if x + 7.0 > r[0] + r[2] {
                        break;
                    }
                    self.rect([x, r[1] + 2.0, 7.0, 7.0], DIM);
                    self.rect(
                        [x + 1.0, r[1] + 3.0, 5.0, 5.0],
                        if node.value.is_finite() && i < node.value.floor().clamp(0.0, 5.0) as usize
                        {
                            GOLD
                        } else {
                            0xff_26140d
                        },
                    );
                }
            }
            _ => {}
        }
    }
}

/// Adds at most 60k cells for 512 nodes. All transforms compose with each group.
pub fn draw(
    frame: &mut FrameBuilder,
    scene: &UiScene,
    camera: [f32; 2],
    viewport: [f32; 2],
    time: f32,
    reduced_motion: bool,
) {
    let camera = Vec2::from_array(camera);
    let viewport = Vec2::from_array(viewport);
    if !camera.is_finite() || !viewport.is_finite() || viewport.min_element() <= 0.0 {
        return;
    }
    let time = if time.is_finite() { time } else { 0.0 };
    let origin = camera - viewport * 0.5;
    let mut painter = Painter {
        frame,
        parent: AffineTransform::default(),
        remaining: MAX_UI_PIXELS,
        viewport,
        clip: None,
        work_remaining: MAX_UI_PIXELS * 2,
    };
    let mut nodes = 0;
    for group in scene.groups.iter().take(128) {
        let position = Vec2::from_array(group.position);
        if !position.is_finite() || position.abs().max_element() > 8192.0 {
            continue;
        }
        painter.parent = Transform {
            position: origin + position,
            ..Transform::default()
        }
        .affine();
        for node in &group.nodes {
            if nodes == 512 || painter.remaining == 0 || painter.work_remaining == 0 {
                return;
            }
            nodes += 1;
            let r = node.rect;
            if r.iter().any(|v| !v.is_finite() || v.abs() > 8192.0) || r[2] <= 0.0 || r[3] <= 0.0 {
                continue;
            }
            if node.clip.is_some_and(|c| {
                c.iter().any(|v| !v.is_finite() || v.abs() > 8192.0) || c[2] <= 0.0 || c[3] <= 0.0
            }) {
                continue;
            }
            painter.clip = node.clip;
            let top = position + Vec2::new(r[0], r[1]);
            if top.x > painter.viewport.x
                || top.y > painter.viewport.y
                || top.x + r[2] < 0.0
                || top.y + r[3] < 0.0
            {
                continue;
            }
            painter.node(node, time, reduced_motion);
        }
    }
}

fn muted(color: u32) -> u32 {
    let [r, g, b, _] = color.to_le_bytes();
    let gray = ((r as u16 + g as u16 + b as u16) / 4) as u8;
    u32::from_le_bytes([gray, gray, gray, 180])
}
fn brighten(color: u32) -> u32 {
    let [r, g, b, a] = color.to_le_bytes();
    u32::from_le_bytes([
        r.saturating_add(28),
        g.saturating_add(28),
        b.saturating_add(28),
        a,
    ])
}
// Private bounded layout; public UI nodes and cached glyph objects are unchanged.
const TEXT_CHARS: usize = 256;
const TEXT_LINES: usize = TEXT_CHARS + 1;

#[derive(Clone, Copy, Default)]
struct LineSpan {
    start: u16,
    end: u16,
}
struct WrappedLines {
    chars: [char; TEXT_CHARS],
    spans: [LineSpan; TEXT_LINES],
    used: usize,
    lines: usize,
}
struct WrappedLine<'a>(&'a [char]);
impl WrappedLine<'_> {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.0.iter().copied()
    }
}
impl WrappedLines {
    fn new() -> Self {
        Self {
            chars: ['\0'; TEXT_CHARS],
            spans: [LineSpan::default(); TEXT_LINES],
            used: 0,
            lines: 0,
        }
    }
    fn len(&self) -> usize {
        self.lines
    }
    fn iter(&self) -> impl Iterator<Item = WrappedLine<'_>> {
        self.spans[..self.lines]
            .iter()
            .map(|span| WrappedLine(&self.chars[span.start as usize..span.end as usize]))
    }
    fn append(&mut self, chars: &[char]) {
        self.chars[self.used..self.used + chars.len()].copy_from_slice(chars);
        self.used += chars.len();
    }
    fn line(&mut self, start: usize) {
        self.spans[self.lines] = LineSpan {
            start: start as u16,
            end: self.used as u16,
        };
        self.lines += 1;
    }
}

fn wrap(text: &str, columns: usize) -> WrappedLines {
    // Painter::text returns before calling us when columns is zero.
    assert!(columns > 0);
    let mut input = ['\0'; TEXT_CHARS];
    let mut length = 0;
    for character in text.chars().take(TEXT_CHARS) {
        input[length] = character;
        length += 1;
    }
    let mut output = WrappedLines::new();
    let mut paragraph_start = 0;
    for paragraph_end in 0..=length {
        if paragraph_end < length && input[paragraph_end] != '\n' {
            continue;
        }
        let mut cursor = paragraph_start;
        let mut line_start = output.used;
        while cursor < paragraph_end {
            if input[cursor].is_whitespace() {
                cursor += 1;
                continue;
            }
            let word_start = cursor;
            while cursor < paragraph_end && !input[cursor].is_whitespace() {
                cursor += 1;
            }
            if output.used > line_start
                && output.used - line_start + 1 + cursor - word_start > columns
            {
                output.line(line_start);
                line_start = output.used;
            }
            for chunk in input[word_start..cursor].chunks(columns) {
                if output.used > line_start {
                    output.append(&[' ']);
                }
                output.append(chunk);
                if output.used - line_start >= columns {
                    output.line(line_start);
                    line_start = output.used;
                }
            }
        }
        if output.used > line_start || paragraph_start == paragraph_end {
            output.line(line_start);
        }
        paragraph_start = paragraph_end + 1;
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authored_campaign_ship_and_patch_outlines_are_distinct_occupied_objects() {
        use std::collections::HashSet;
        let occupied = |body: &PixelBody| {
            (0..body.height)
                .flat_map(|y| (0..body.width).map(move |x| body.is_occupied(x, y)))
                .collect::<Vec<_>>()
        };
        for (names, dimensions) in [
            (&CAMPAIGN_SHIPS[..], (32, 32)),
            (&CAMPAIGN_PATCHES[..], (24, 24)),
        ] {
            let mut silhouettes = HashSet::new();
            for name in names {
                let body = &CAMPAIGN_ASSETS.sprites[*name].body;
                assert_eq!((body.width, body.height), dimensions);
                assert!(body.occupied_count() > 100);
                assert!(body.occupied_count() < body.width * body.height);
                assert!(
                    silhouettes.insert(occupied(body)),
                    "duplicate silhouette: {name}"
                );
            }
        }
        let new_heads: HashSet<_> = PORTRAITS
            .iter()
            .skip(3)
            .map(|art| occupied(&art.body))
            .collect();
        assert_eq!(new_heads.len(), 5, "new comms silhouettes must differ");
        assert_eq!(selection(f32::NAN, 10), 0);
        assert_eq!(selection(8.9, 10), 8);
        assert_eq!(selection(100.0, 10), 9);
    }

    #[test]
    fn campaign_quads_fit_integer_scale_and_keep_unlit_material() {
        for kind in ["campaign-ship", "campaign-patch", "portrait"] {
            for value in 0..8 {
                let mut scene = scene(kind);
                let node = &mut scene.groups[0].nodes[0];
                node.rect = [5.0, 9.0, 60.0, 70.0];
                node.value = value as f32;
                let mut frame = FrameBuilder::new();
                draw(&mut frame, &scene, [0.0; 2], [640.0, 360.0], 0.0, true);
                assert!(frame.frame().pixel_count > 100);
                let batch = &frame.frame().batches[0];
                assert_eq!(batch.transforms.len(), 1);
                let t = batch.transforms[0];
                assert_eq!(t.b[2..], [0.0, 0.0]);
                assert_eq!(t.a[2].fract(), 0.0);
                assert_eq!(t.a[2], t.b[1]);
                for pixel in &batch.pixels {
                    let p = t.point(Vec2::from_array(pixel.local)) + Vec2::new(320.0, 180.0)
                        - Vec2::new(100.0, 50.0);
                    assert!(p.x >= 5.0 && p.x < 65.0 && p.y >= 9.0 && p.y < 79.0);
                }
            }
        }
    }
    #[test]
    fn console_panel_is_exact_opaque_black_with_only_a_rail_and_pins() {
        let mut scene = scene("console-panel");
        scene.groups[0].nodes[0].rect = [0.0, 0.0, 300.0, 100.0];
        let mut frame = FrameBuilder::new();
        draw(&mut frame, &scene, [0.0; 2], [640.0, 360.0], 0.0, true);
        let batch = &frame.frame().batches[0];
        assert_eq!(batch.pixels[0].color, 0xff_000000);
        let transform = batch.transforms[batch.pixels[0].transform as usize];
        assert_eq!(transform.a[2], 300.0);
        assert_eq!(transform.b[1], 100.0);
        assert_eq!(transform.b[2..], [0.0, 0.0]);
        assert_eq!(frame.frame().pixel_count, 4);
        assert_eq!(batch.pixels[1].color, CYAN);
    }

    #[test]
    fn cached_portraits_fit_integer_cells_and_preserve_group_camera_transform() {
        assert_eq!(
            (PORTRAITS[0].body.width, PORTRAITS[0].body.height),
            (24, 32)
        );
        assert!(PORTRAITS[0].body.occupied_count() > 300);
        assert!(PORTRAITS[0].body.occupied_count() <= 24 * 32);
        assert!(std::ptr::eq(&PORTRAITS[0].body, &PORTRAITS[0].body));
        let extract = |camera, value| {
            let mut scene = scene("portrait");
            let node = &mut scene.groups[0].nodes[0];
            node.rect = [5.0, 9.0, 60.0, 70.0];
            node.value = value;
            let mut frame = FrameBuilder::new();
            draw(&mut frame, &scene, camera, [640.0, 360.0], 0.0, true);
            frame.finish()
        };
        let a = extract([0.0; 2], 0.0);
        let b = extract([1000.0, -1200.0], 1.0);
        assert_eq!(a.pixel_count, b.pixel_count);
        assert_eq!(a.batches[0].transforms.len(), 1);
        let transform = a.batches[0].transforms[0];
        assert_eq!((transform.a[2], transform.b[1]), (2.0, 2.0));
        for (ap, bp) in a.batches[0].pixels.iter().zip(&b.batches[0].pixels) {
            let aw = transform.point(Vec2::from_array(ap.local));
            let bw = b.batches[0].transforms[0].point(Vec2::from_array(bp.local))
                - Vec2::new(1000.0, -1200.0);
            assert!(aw.distance(bw) < 0.001);
            let screen = aw + Vec2::new(320.0, 180.0) - Vec2::new(100.0, 50.0);
            assert!(screen.x >= 5.0 && screen.x < 65.0 && screen.y >= 9.0 && screen.y < 79.0);
        }
        assert_ne!(
            a.batches[0].pixels, b.batches[0].pixels,
            "palette changes the speaker's style"
        );
        let mut frame = FrameBuilder::new();
        let many = UiScene {
            groups: vec![UiGroup {
                position: [0.0; 2],
                nodes: (0..600)
                    .map(|_| UiNode {
                        kind: "portrait".into(),
                        rect: [0.0, 0.0, 48.0, 64.0],
                        ..Default::default()
                    })
                    .collect(),
            }],
        };
        draw(&mut frame, &many, [0.0; 2], [640.0, 360.0], 0.0, true);
        assert!(frame.frame().pixel_count <= MAX_UI_PIXELS);
    }

    #[test]
    fn active_portrait_blinks_without_changing_geometry_and_freezes_when_reduced() {
        let extract = |time, reduced| {
            let mut scene = scene("portrait");
            scene.groups[0].nodes[0].active = true;
            let mut frame = FrameBuilder::new();
            draw(&mut frame, &scene, [0.0; 2], [640.0, 360.0], time, reduced);
            frame.finish()
        };
        let a = extract(0.0, false);
        let b = extract(4.4, false);
        assert_eq!(a.pixel_count, b.pixel_count);
        assert_eq!(a.batches[0].transforms, b.batches[0].transforms);
        assert_ne!(a.batches[0].pixels, b.batches[0].pixels);
        assert_eq!(
            extract(0.0, true).batches[0].pixels,
            extract(4.4, true).batches[0].pixels
        );
    }

    #[test]
    fn rails_keep_original_corners_and_one_centered_ornament() {
        for (name, source) in [("panel", [96, 48]), ("button", [96, 20])] {
            for width in [96, 300, 600] {
                let geometry = FrameGeometry::new(name, [width, 120], source);
                let ornaments: Vec<_> = (geometry.corner[0]..width - geometry.corner[0])
                    .filter(|&x| (38..58).contains(&geometry.source_at([x, 0])[0]))
                    .collect();
                assert_eq!(
                    ornaments,
                    (width / 2 - 10..width / 2 + 10).collect::<Vec<_>>()
                );
                assert_eq!(geometry.source_at([3, 3]), [3, 3]);
                assert_eq!(geometry.source_at([width - 3, 118]), [93, source[1] - 2]);
                assert_eq!(geometry.source_at([0, 60])[1], geometry.neutral[1]);
            }
        }
    }

    #[test]
    fn text_letters_have_individual_composed_object_transforms() {
        let mut scene = scene("text");
        let node = &mut scene.groups[0].nodes[0];
        node.text = "A A".into();
        node.rect = [5.0, 9.0, 100.0, 14.0];
        let mut frame = FrameBuilder::new();
        draw(&mut frame, &scene, [0.0; 2], [640.0, 360.0], 0.0, true);
        let batch = &frame.frame().batches[0];
        assert_eq!(batch.transforms.len(), 2);
        assert_eq!(batch.transforms[0].a[0], -210.0);
        assert_eq!(batch.transforms[0].a[1], -114.0);
        assert_eq!(batch.transforms[1].a[0], -186.0);
        assert_eq!(batch.transforms[0].a[2], 2.0);
        assert_eq!(batch.transforms[0].b[1], 2.0);
        assert_eq!(
            batch.pixels.iter().filter(|p| p.transform == 0).count(),
            RoyaleMono::body('A').occupied_count()
        );
    }

    #[test]
    fn console_input_cursor_blinks_and_reduced_motion_keeps_it_static() {
        let extract = |time, reduced| {
            let mut scene = scene("input");
            let node = &mut scene.groups[0].nodes[0];
            node.rect = [0.0, 0.0, 160.0, 20.0];
            node.text = "HELP".into();
            node.value = 4.0;
            node.font = 1.0;
            node.active = true;
            let mut frame = FrameBuilder::new();
            draw(&mut frame, &scene, [0.0; 2], [640.0, 360.0], time, reduced);
            frame.finish()
        };
        assert_eq!(
            extract(0.0, false).pixel_count - extract(0.75, false).pixel_count,
            1
        );
        assert_eq!(
            extract(0.0, true).batches[0].pixels,
            extract(0.75, true).batches[0].pixels
        );
        for character in ['|', '_', '?', '=', '#', ',', '\''] {
            assert!(RoyaleMono::body(character).occupied_count() > 0);
        }
    }
    #[test]
    fn console_input_is_black_and_caret_tracks_middle_and_scrolled_tail() {
        let extract = |text: &str, caret: f32| {
            let mut scene = scene("input");
            let node = &mut scene.groups[0].nodes[0];
            node.rect = [5.0, 9.0, 60.0, 20.0];
            node.text = text.into();
            node.value = caret;
            node.font = 1.0;
            node.active = true;
            let mut frame = FrameBuilder::new();
            draw(&mut frame, &scene, [0.0; 2], [640.0, 360.0], 0.0, true);
            frame.finish()
        };
        for (text, caret, expected) in [("HELP", 1.0, 23.0), ("0123456789ABCDEF", 16.0, 59.0)] {
            let frame = extract(text, caret);
            let batch = &frame.batches[0];
            assert_eq!(batch.pixels[0].color, rgb(0, 0, 0));
            let background = batch.transforms[batch.pixels[0].transform as usize];
            assert_eq!((background.a[2], background.b[1]), (60.0, 20.0));
            let cursor = batch.pixels.iter().find(|p| p.color == GOLD).unwrap();
            let transform = batch.transforms[cursor.transform as usize];
            assert_eq!((transform.a[2], transform.b[1]), (1.0, 7.0));
            let center = transform.point(Vec2::from_array(cursor.local)) + Vec2::new(320.0, 180.0)
                - Vec2::new(100.0, 50.0);
            assert_eq!(center.x - 0.5, expected);
            for pixel in &batch.pixels {
                let transform = batch.transforms[pixel.transform as usize];
                let center = transform.point(Vec2::from_array(pixel.local))
                    + Vec2::new(320.0, 180.0)
                    - Vec2::new(100.0, 50.0);
                assert!(center.x >= 5.0 && center.x <= 65.0 && center.y >= 9.0 && center.y <= 29.0);
            }
        }
        // Spaces occupy real columns rather than being collapsed by paragraph wrapping.
        let spaced = extract("  A", 0.0);
        let batch = &spaced.batches[0];
        let glyph = batch
            .transforms
            .iter()
            .find(|t| t.a[2] == 1.0 && t.b[1] == 1.0 && t.a[0] > -200.0)
            .unwrap();
        assert_eq!(glyph.a[0] - RoyaleMono::WIDTH * 0.5 + 320.0 - 100.0, 29.0);
    }
    fn scene(kind: &str) -> UiScene {
        UiScene {
            groups: vec![UiGroup {
                position: [100.0, 50.0],
                nodes: vec![UiNode {
                    kind: kind.into(),
                    rect: [5.0, 9.0, 300.0, 120.0],
                    text: "GALAXY ROYALE".into(),
                    font: 2.0,
                    ..Default::default()
                }],
            }],
        }
    }
    #[test]
    fn composed_group_translation_keeps_text_screen_fixed() {
        let extract = |camera: [f32; 2]| {
            let mut frame = FrameBuilder::new();
            draw(
                &mut frame,
                &scene("text"),
                camera,
                [640.0, 360.0],
                0.0,
                true,
            );
            frame.finish()
        };
        let a = extract([0.0; 2]);
        let b = extract([900.25, -1400.5]);
        assert!(a.pixel_count > 100);
        assert_eq!(a.pixel_count, b.pixel_count);
        for (a, b) in a.batches.iter().zip(&b.batches) {
            for (ap, bp) in a.pixels.iter().zip(&b.pixels) {
                let aw = a.transforms[ap.transform as usize].point(Vec2::from_array(ap.local));
                let bw = b.transforms[bp.transform as usize].point(Vec2::from_array(bp.local))
                    - Vec2::new(900.25, -1400.5);
                assert!(aw.distance(bw) < 0.001);
            }
        }
        assert!(
            wrap("ONE TWO THREE LONGWORD", 5)
                .iter()
                .all(|line| line.len() <= 5)
        );
    }
    #[test]
    fn reduced_motion_freezes_generated_art_and_normal_motion_changes_it() {
        let extract = |time, reduced| {
            let mut frame = FrameBuilder::new();
            draw(
                &mut frame,
                &scene("seal"),
                [0.0; 2],
                [640.0, 360.0],
                time,
                reduced,
            );
            frame.finish()
        };
        let a = extract(0.0, true);
        let b = extract(10.0, true);
        assert!(a.pixel_count > 0);
        assert_eq!(a.batches[0].pixels, b.batches[0].pixels);
        assert_eq!(a.batches[0].transforms, b.batches[0].transforms);
        assert_ne!(
            extract(2.0, false).batches[0].transforms,
            extract(10.0, false).batches[0].transforms
        );
    }
    #[test]
    fn short_button_rects_shrink_labels_without_hiding_them() {
        let pixels = |label: &str| {
            let mut scene = scene("button");
            scene.groups[0].nodes[0].rect = [0.0, 0.0, 120.0, 12.0];
            scene.groups[0].nodes[0].font = 4.0;
            scene.groups[0].nodes[0].text = label.into();
            let mut frame = FrameBuilder::new();
            draw(&mut frame, &scene, [0.0; 2], [640.0, 360.0], 0.0, true);
            frame.frame().pixel_count
        };
        assert!(pixels("LAUNCH") > pixels(""));
    }

    #[test]
    fn scroll_clip_keeps_visible_cells_inside_group_relative_bounds() {
        for kind in [
            "text",
            "logo",
            "panel",
            "portrait",
            "campaign-patch",
            "campaign-ship",
        ] {
            let mut scene = scene(kind);
            let clip = [140.0, 40.0, 32.0, 16.0];
            scene.groups[0].nodes[0].clip = Some(clip);
            let mut frame = FrameBuilder::new();
            draw(&mut frame, &scene, [0.0; 2], [640.0, 360.0], 2.0, true);
            for batch in &frame.frame().batches {
                for pixel in &batch.pixels {
                    if pixel.color >> 24 == 0 {
                        continue;
                    }
                    let p = batch.transforms[pixel.transform as usize]
                        .point(Vec2::from_array(pixel.local))
                        + Vec2::new(320.0, 180.0)
                        - Vec2::new(100.0, 50.0);
                    assert!(
                        p.x >= clip[0]
                            && p.x < clip[0] + clip[2]
                            && p.y >= clip[1]
                            && p.y < clip[1] + clip[3],
                        "{kind}: {p:?}"
                    );
                }
            }
        }
        let decoded: UiScene = serde_json::from_str(
            r#"{"groups":[{"nodes":[{"kind":"text","rect":[0,0,100,20],"text":"DEFAULTS"}]}]}"#,
        )
        .unwrap();
        assert!(decoded.groups[0].nodes[0].clip.is_none());
        assert_eq!(decoded.groups[0].position, [0.0; 2]);
    }

    #[test]
    fn malformed_geometry_and_large_scenes_stay_bounded() {
        let mut invalid = scene("text");
        invalid.groups[0].nodes[0].rect[0] = f32::NAN;
        let mut frame = FrameBuilder::new();
        draw(&mut frame, &invalid, [0.0; 2], [640.0, 360.0], 0.0, false);
        assert_eq!(frame.frame().pixel_count, 0);
        let nodes = (0..600)
            .map(|_| UiNode {
                kind: "panel".into(),
                rect: [0.0, 0.0, 640.0, 360.0],
                ..Default::default()
            })
            .collect();
        draw(
            &mut frame,
            &UiScene {
                groups: vec![UiGroup {
                    position: [0.0; 2],
                    nodes,
                }],
            },
            [0.0; 2],
            [640.0, 360.0],
            0.0,
            false,
        );
        assert!(frame.frame().pixel_count <= MAX_UI_PIXELS);
        assert!(frame.frame().batches.iter().all(|b| {
            b.pixels
                .iter()
                .all(|p| (p.transform as usize) < b.transforms.len())
        }));
    }
}

#[cfg(test)]
#[path = "ui_wrap_tests.rs"]
mod wrapping_tests;

#[cfg(test)]
#[path = "ui_button_tests.rs"]
mod semantic_button_tests;

#[cfg(test)]
#[path = "ui_icon_tests.rs"]
mod icon_tests;

#[cfg(test)]
#[path = "ui_map_tests.rs"]
mod map_tests;

#[cfg(test)]
#[path = "ui_facility_label_tests.rs"]
mod facility_label_tests;

#[cfg(test)]
#[path = "ui_settings_tests.rs"]
mod settings_tests;
