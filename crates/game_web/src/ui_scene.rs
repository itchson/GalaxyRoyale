//! Generated artwork and semantic DOM layout rendered as actual engine cells.
use crate::pixel_ui::RoyaleMono;
#[path = "collection_ui.rs"]
mod collection_ui;
#[path = "component_art.rs"]
mod component_art;
#[path = "meishi_art.rs"]
mod meishi_art;
#[path = "sports_ui.rs"]
mod sports_ui;
#[path = "title_art.rs"]
mod title_art;
use engine_core::{AffineTransform, FrameBuilder, PixelBody, Transform, Vec2};
use serde::Deserialize;
use std::{collections::HashMap, sync::LazyLock};

pub const MAX_UI_PIXELS: usize = 60_000;
const WHITE: u32 = 0xff_f4f1eb;
const CYAN: u32 = 0xff_efde58;
const DIM: u32 = 0xff_a29787;
const GOLD: u32 = 0xff_66bce8;

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

const MIN_FONT_SCALE: f32 = 0.25;

fn font_scale(requested: f32) -> f32 {
    if requested.is_finite() && requested > 0.0 {
        requested.clamp(MIN_FONT_SCALE, 4.0)
    } else {
        1.0
    }
}

fn text_capacity(rect: [f32; 4], font: f32) -> (usize, usize) {
    (
        ((rect[2] + font) / (RoyaleMono::ADVANCE * font)).floor() as usize,
        ((rect[3] + font) / (8.0 * font)).floor() as usize,
    )
}

fn fractional_text_rect(rect: [f32; 4], font: f32) -> [f32; 4] {
    if font.fract() == 0.0
        && rect
            .iter()
            .all(|coordinate| (coordinate * 4.0).fract() == 0.0)
    {
        return rect;
    }
    // Child translation and cell-corner composition can round in opposite
    // directions on an exact fractional fit. Reserve only f32 arithmetic
    // roundoff inside the measured box. Integer fonts on the existing exact
    // quarter-cell grid retain their placement and scale.
    let magnitude = rect[0]
        .abs()
        .max(rect[1].abs())
        .max((rect[0] + rect[2]).abs())
        .max((rect[1] + rect[3]).abs())
        .max(1.0);
    let gutter = magnitude * f32::EPSILON * 8.0;
    [
        rect[0] + gutter,
        rect[1] + gutter,
        (rect[2] - gutter * 2.0).max(0.0),
        (rect[3] - gutter * 2.0).max(0.0),
    ]
}

fn settings_font(rect: [f32; 4], text: &str, requested: f32) -> f32 {
    let requested = font_scale(requested);
    let complete = |font| {
        let (columns, rows) = text_capacity(fractional_text_rect(rect, font), font);
        columns > 0 && rows > 0 && wrap(text, columns).len() <= rows
    };
    if complete(requested) {
        return requested;
    }
    // Each candidate keeps a complete number of lines, but its cell scale is
    // continuous. The wrapper is stack-only and bounds names to 256 characters;
    // requested sizes that already fit avoid this bounded fallback search.
    let mut best = MIN_FONT_SCALE;
    for columns in 1..=text.chars().take(TEXT_CHARS).count().max(1) {
        let rows = wrap(text, columns).len().max(1);
        let mut font = requested
            .min(rect[2] / (columns as f32 * RoyaleMono::ADVANCE - 1.0))
            .min(rect[3] / (rows as f32 * 8.0 - 1.0));
        let fit = fractional_text_rect(rect, font);
        font = font
            .min(fit[2] / (columns as f32 * RoyaleMono::ADVANCE - 1.0))
            .min(fit[3] / (rows as f32 * 8.0 - 1.0));
        // A division on an exact column/row boundary may round up by one ULP.
        // Validate with the actual wrapper rather than assume candidate fit.
        if font >= MIN_FONT_SCALE && !complete(font) {
            font = font.next_down();
        }
        if font > best && complete(font) {
            best = font;
        }
    }
    best
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
    pub text_align: TextAlign,
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
    /// ID9 requires all three licensed-profile tokens; absent/unknown tokens draw nothing.
    #[serde(deserialize_with = "portrait_token")]
    pub portrait_kind: Option<u8>,
    #[serde(deserialize_with = "portrait_token")]
    pub portrait_main: Option<u8>,
    #[serde(deserialize_with = "portrait_token")]
    pub portrait_sub: Option<u8>,
}

/// Semantic horizontal alignment for ordinary text; controls keep their own layout.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}
impl<'de> Deserialize<'de> for TextAlign {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let token = serde_json::Value::deserialize(deserializer)?;
        Ok(match token.as_str() {
            Some("center") => Self::Center,
            Some("right") => Self::Right,
            _ => Self::Left,
        })
    }
}

fn portrait_token<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<u8>, D::Error> {
    let token = serde_json::Value::deserialize(deserializer)?;
    Ok(token
        .as_u64()
        .filter(|&value| value <= 2)
        .map(|value| value as u8))
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
}
struct Assets {
    sprites: HashMap<String, Art>,
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
                },
            )
        })
        .collect();
    Assets { sprites }
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
    fn control_icon(&mut self, node: &UiNode, time: f32, reduced: bool) {
        let Some(icon) = crate::flight_icons::icon(&node.kind) else {
            return;
        };
        let body = &icon.body;
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
                    icon.color(
                        cell,
                        shade,
                        crate::flight_icons::State {
                            accent,
                            hovered: node.active,
                            pressed: node.selected,
                            disabled: node.disabled,
                            busy: node.kind == "save-icon" && selection(node.value, 3) == 1,
                            time,
                            reduced_motion: reduced,
                        },
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
        let guide = node.value == crate::kikai_art::PORTRAIT_ID as f32;
        if guide || node.value == crate::kikai_art::PROFILE_PORTRAIT_ID as f32 {
            let input = crate::kikai_art::DoraInput {
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
            };
            if guide {
                crate::kikai_art::append_dora(
                    self.frame,
                    &input,
                    &mut self.remaining,
                    &mut self.work_remaining,
                );
            } else if let Some(appearance) = node
                .portrait_kind
                .zip(node.portrait_main)
                .zip(node.portrait_sub)
                .and_then(|((kind, main), sub)| {
                    crate::kikai_art::ProfileAppearance::from_tokens(kind, main, sub)
                })
            {
                crate::kikai_art::append_profile_portrait(
                    self.frame,
                    &input,
                    appearance,
                    &mut self.remaining,
                    &mut self.work_remaining,
                );
            }
            return;
        }
        // New licensed types must never silently use a champion or guide portrait.
        if !node.value.is_finite()
            || !(0.0..=7.0).contains(&node.value)
            || node.value.fract() != 0.0
        {
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
        let body = &CAMPAIGN_ASSETS.sprites
            [CAMPAIGN_SHIPS[selection(node.value, CAMPAIGN_SHIPS.len())]]
        .body;
        self.fitted_body(node, body, true);
    }
    fn licensed_ship(&mut self, node: &UiNode, time: f32, reduced: bool) {
        let corporation = [
            game_core::identity::Corporation::Pan,
            game_core::identity::Corporation::Renko,
            game_core::identity::Corporation::Hugo,
        ][selection(node.value, 3)];
        let body = game_core::identity::starter_body(corporation);
        let r = node.rect;
        let scale = (r[2] / body.width as f32).min(r[3] / body.height as f32);
        if !scale.is_finite() || scale <= 0.0 {
            return;
        }
        let local = Transform {
            position: Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5),
            scale: Vec2::splat(scale),
            ..Default::default()
        };
        let clip = self.clip;
        let disabled = node.disabled;
        let shade = |cell: Vec2, half: f32, color: u32| {
            let p = local.point(cell);
            let half = half * scale;
            if clip.is_some_and(|c| {
                p.x - half < c[0]
                    || p.y - half < c[1]
                    || p.x + half > c[0] + c[2]
                    || p.y + half > c[1] + c[3]
            }) {
                0
            } else if disabled {
                muted(color)
            } else {
                color
            }
        };
        let mut budget = game_core::identity::StarterPreviewBudget {
            pixels: self.remaining,
            work: self.work_remaining,
            transforms: 2,
        };
        if game_core::identity::append_starter_preview(
            self.frame,
            corporation,
            game_core::identity::StarterPreviewInput {
                parent: self.parent.compose(local.affine()),
                time,
                reduced_motion: reduced,
                ..Default::default()
            },
            &mut budget,
            |cell, color| shade(cell, 0.5, color),
            |cell, color| shade(cell, 0.25, color),
        )
        .is_ok()
        {
            self.remaining = budget.pixels;
            self.work_remaining = budget.work;
        }
    }
    fn fitted_body(&mut self, node: &UiNode, body: &PixelBody, highlight_active: bool) {
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
                let half = scale * 0.5;
                if clip.is_some_and(|c| {
                    p.x - half < c[0]
                        || p.y - half < c[1]
                        || p.x + half > c[0] + c[2]
                        || p.y + half > c[1] + c[3]
                }) {
                    0
                } else if node.disabled {
                    muted(color)
                } else if highlight_active && node.active {
                    brighten(color)
                } else {
                    color
                }
            },
        );
        self.remaining -= count;
        self.work_remaining -= count;
    }
    fn meishi_fittings(&mut self, node: &UiNode, time: f32, reduced: bool) {
        meishi_art::append(
            self.frame,
            meishi_art::Input {
                kind: &node.kind,
                rect: node.rect,
                parent: self.parent,
                clip: self.clip,
                value: node.value,
                // Only the saved earned state illuminates an anonymous socket.
                earned: node.kind != "meishi-socket" || node.selected,
                time,
                reduced,
            },
            &mut self.remaining,
            &mut self.work_remaining,
        );
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
    fn skin(&mut self, node: &UiNode, kind: sports_ui::Kind) {
        sports_ui::append(
            self.frame,
            &sports_ui::Input {
                parent: self.parent,
                rect: node.rect,
                clip: self.clip,
                kind,
                accent: button_accent(&node.kind),
                active: node.active,
                selected: node.kind == "nav-tab" && node.selected,
                disabled: node.disabled,
            },
            &mut self.remaining,
            &mut self.work_remaining,
        );
    }
    fn text(&mut self, rect: [f32; 4], text: &str, font: f32, color: u32, centered: bool) {
        self.text_aligned(
            rect,
            text,
            font,
            color,
            if centered {
                TextAlign::Center
            } else {
                TextAlign::Left
            },
            centered,
        );
    }
    fn text_aligned(
        &mut self,
        rect: [f32; 4],
        text: &str,
        font: f32,
        color: u32,
        alignment: TextAlign,
        centered: bool,
    ) {
        let bounds = rect;
        let font = font_scale(font)
            .min((rect[3] / RoyaleMono::HEIGHT).max(MIN_FONT_SCALE))
            .min((rect[2] / RoyaleMono::WIDTH).max(MIN_FONT_SCALE));
        let rect = fractional_text_rect(rect, font);
        let font = font
            .min((rect[3] / RoyaleMono::HEIGHT).max(MIN_FONT_SCALE))
            .min((rect[2] / RoyaleMono::WIDTH).max(MIN_FONT_SCALE));
        let (columns, rows) = text_capacity(rect, font);
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
            let width = (line.len() as f32 * RoyaleMono::ADVANCE - 1.0) * font;
            let left = match alignment {
                TextAlign::Left => rect[0],
                TextAlign::Center => rect[0] + (rect[2] - width) * 0.5,
                TextAlign::Right => rect[0] + rect[2] - width,
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
                let clip = self.clip.map_or(bounds, |clip| intersection(bounds, clip));
                self.frame
                    .push_body_affine_colored(body, letter_world, |cell, _| {
                        // Scroll clipping covers complete source cells, including
                        // fractional edge cells. Hidden cells still charge both
                        // budgets and retain their glyph's composed transform.
                        let outside = [
                            cell + Vec2::splat(-0.5),
                            cell + Vec2::new(0.5, -0.5),
                            cell + Vec2::new(-0.5, 0.5),
                            cell + Vec2::splat(0.5),
                        ]
                        .into_iter()
                        .map(|p| letter_local.point(p))
                        .any(|p| {
                            p.x < clip[0]
                                || p.x > clip[0] + clip[2]
                                || p.y < clip[1]
                                || p.y > clip[1] + clip[3]
                        });
                        if outside { 0 } else { color }
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
        let content = node
            .label_rect
            .and_then(|label| contained_region(label, r))
            .unwrap_or([r[0] + padding_x, r[1] + 2.0, width, height]);
        let mut font = font_scale(font)
            .min((width / units).max(MIN_FONT_SCALE))
            .min((height / RoyaleMono::HEIGHT).max(MIN_FONT_SCALE));
        let fit = fractional_text_rect(content, font);
        font = font
            .min((fit[2] / units).max(MIN_FONT_SCALE))
            .min((fit[3] / RoyaleMono::HEIGHT).max(MIN_FONT_SCALE));
        let mut columns = text_capacity(fractional_text_rect(content, font), font).0;
        if font > MIN_FONT_SCALE && columns < characters {
            font = font.next_down().max(MIN_FONT_SCALE);
            columns = text_capacity(fractional_text_rect(content, font), font).0;
        }
        if characters > columns {
            let visible = if columns >= 4 { columns - 3 } else { columns };
            if let Some((byte, _)) = label.char_indices().nth(visible) {
                label.truncate(byte);
            }
            if columns >= 4 {
                label.push_str("...");
            }
        }
        let previous_clip = self.clip;
        self.clip = Some(previous_clip.map_or(content, |clip| intersection(content, clip)));
        self.text(content, &label, font, color, true);
        self.clip = previous_clip;
    }
    fn settings_label(&mut self, rect: [f32; 4], text: &str, font: f32, color: u32) {
        if rect[2] < RoyaleMono::WIDTH * MIN_FONT_SCALE
            || rect[3] < RoyaleMono::HEIGHT * MIN_FONT_SCALE
        {
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
        // Open graphite rows retain measured label/control geometry.
        // Only focus receives a small native competitive chevron.
        self.skin(node, sports_ui::Kind::Row);
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
        self.skin(node, sports_ui::Kind::Button);
        if r[2] < 3.0 || r[3] < 3.0 {
            return;
        }
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
        let font = font_scale(node.font);
        let color = if node.disabled {
            DIM
        } else if node.active {
            CYAN
        } else {
            WHITE
        };
        match node.kind.as_str() {
            "garage-bay" => {
                crate::garage_art::append(
                    self.frame,
                    &crate::garage_art::Input {
                        parent: self.parent,
                        rect: r,
                        clip: self.clip,
                        corporation: selection(node.value, 3),
                        time,
                        reduced_motion: reduced,
                        active: node.active,
                    },
                    &mut self.remaining,
                    &mut self.work_remaining,
                );
            }
            "co-op-invite" | "co-op-join" | "co-op-roster" | "multiplayer-icon" => {
                use crate::co_op_art::{self, Kind};
                let kind = match node.kind.as_str() {
                    "co-op-invite" => Kind::Invite,
                    "co-op-join" | "multiplayer-icon" => Kind::Join,
                    _ if node.value.is_finite()
                        && node.value.fract() == 0.0
                        && (0.0..=8.0).contains(&node.value) =>
                    {
                        Kind::Roster(node.value as u8)
                    }
                    _ => return,
                };
                co_op_art::append(
                    self.frame,
                    &co_op_art::Input {
                        kind,
                        rect: r,
                        clip: self.clip,
                        parent: self.parent,
                        style: co_op_art::Style {
                            hovered: node.active,
                            busy: node.selected,
                            disabled: node.disabled,
                            time,
                            reduced_motion: reduced,
                        },
                    },
                    &mut self.remaining,
                    &mut self.work_remaining,
                );
            }
            "text" => self.text_aligned(r, &node.text, font, color, node.text_align, false),
            "collection-name" | "collection-state" | "collection-meta" | "collection-cost"
            | "collection-reason" => {
                // This is hierarchy only: all copy, ownership and exact costs
                // come from the discovered-only host HUD, never a local catalog.
                let color = if node.disabled {
                    BUTTON_DISABLED_TEXT
                } else {
                    match node.kind.as_str() {
                        "collection-name" if node.selected => CYAN,
                        "collection-name" | "collection-cost" => WHITE,
                        "collection-reason" => rgb(212, 172, 96),
                        "collection-state" => match node.value {
                            1.0 => GOLD,  // Known blueprint, not crafted.
                            2.0 => WHITE, // Owned; no inference of compatibility.
                            3.0 => CYAN,  // Actual active hull/equipped ability.
                            _ => DIM,
                        },
                        _ => DIM,
                    }
                };
                self.text_aligned(r, &node.text, font, color, node.text_align, false);
            }
            "collection-entry" | "category-symbol" | "corporation-symbol" => {
                let token = (node.value.is_finite()
                    && node.value.fract() == 0.0
                    && (0.0..3.0).contains(&node.value))
                .then_some(node.value as usize);
                let input = collection_ui::Input {
                    parent: self.parent,
                    rect: r,
                    clip: self.clip,
                    active: node.active,
                    selected: node.selected,
                    disabled: node.disabled,
                    time,
                    reduced,
                };
                if node.kind == "collection-entry" {
                    if node.value != -1.0 && token.is_none() {
                        return;
                    }
                    collection_ui::append_entry(
                        self.frame,
                        &input,
                        token,
                        &mut self.remaining,
                        &mut self.work_remaining,
                    );
                } else if let Some(token) = token {
                    collection_ui::append_symbol(
                        self.frame,
                        &input,
                        if node.kind == "category-symbol" {
                            collection_ui::SymbolKind::Category
                        } else {
                            collection_ui::SymbolKind::Corporation
                        },
                        token,
                        &mut self.remaining,
                        &mut self.work_remaining,
                    );
                }
            }
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
                crate::pixel_ui::micro_text_right(
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
            "offense-text" | "defense-text" | "passive-text" => self.text_aligned(
                r,
                &node.text,
                font,
                if node.disabled {
                    BUTTON_DISABLED_TEXT
                } else {
                    button_accent(&node.kind)
                },
                node.text_align,
                false,
            ),
            "title-backdrop" => self.rect(r, u32::from_le_bytes([1, 2, 5, 246])),
            "scene" => self.skin(node, sports_ui::Kind::Scene),
            "panel" => self.skin(node, sports_ui::Kind::Panel),
            "meishi-panel" | "meishi-portrait-frame" | "meishi-seal" | "meishi-socket" => {
                self.meishi_fittings(node, time, reduced);
            }
            "console-panel" => self.console_panel(r),
            "portrait" => self.portrait(node, time, reduced),
            "licensed-ship" => self.licensed_ship(node, time, reduced),
            "craft-component" => {
                if node.value.is_finite()
                    && node.value.fract() == 0.0
                    && node.value >= 0.0
                    && let Some(body) = component_art::body(node.value as usize)
                {
                    self.fitted_body(node, body, false);
                }
            }
            "campaign-patch" => self.campaign(node, false, time, reduced),
            "campaign-ship" => self.campaign(node, true, time, reduced),
            "close-icon" | "save-icon" | "settings-icon" | "resume-icon" | "restart-icon"
            | "title-icon" | "quit-icon" | "status-icon" | "map-icon" | "fullscreen-icon" => {
                self.control_icon(node, time, reduced)
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
                let font = font.min((height / RoyaleMono::HEIGHT).max(MIN_FONT_SCALE));
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
                self.skin(node, sports_ui::Kind::Button);
                self.button_label(node, font, color);
            }
            "nav-tab" | "primary-button" | "exit-button" | "offense-button" | "defense-button"
            | "passive-button" | "restart-button" => self.semantic_button(node, font),
            "logo" => {
                title_art::append(
                    self.frame,
                    &title_art::Input {
                        parent: self.parent,
                        rect: r,
                        clip: self.clip,
                        time,
                        reduced,
                        disabled: node.disabled,
                    },
                    &mut self.remaining,
                    &mut self.work_remaining,
                );
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

// MEISHI sockets and their later emblems share a group. Remember failed sockets
// without altering ordinary campaign scenes or introducing a new node payload.
fn paired_meishi_node(
    painter: &mut Painter<'_>,
    node: &UiNode,
    time: f32,
    reduced: bool,
    failed_sockets: &mut u8,
) {
    if node.kind == "campaign-patch" {
        let slot = selection(node.value, crate::champion_patches::COUNT);
        if slot < 8 && *failed_sockets & (1 << slot) != 0 {
            return;
        }
    }
    let previous = painter.remaining;
    painter.node(node, time, reduced);
    if node.kind == "meishi-socket"
        && node.value.is_finite()
        && node.value.fract() == 0.0
        && (0.0..24.0).contains(&node.value)
    {
        let bit = 1 << (node.value as usize % 8);
        if painter.remaining == previous {
            *failed_sockets |= bit;
        } else {
            *failed_sockets &= !bit;
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
        let mut failed_sockets = 0;
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
            paired_meishi_node(
                &mut painter,
                node,
                time,
                reduced_motion,
                &mut failed_sockets,
            );
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
    fn campaign_quads_fit_authored_rects_and_keep_unlit_material() {
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
                match kind {
                    "campaign-patch" => assert_eq!(t.a[2], 2.5),
                    "campaign-ship" => assert_eq!(t.a[2], 60.0 / 32.0),
                    _ => assert_eq!(t.a[2].fract(), 0.0),
                }
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
    fn licensed_ui_preview_keeps_authored_fine_cells_and_clips_both_cell_extents() {
        for (corp, count) in [(0, 1867), (1, 1864), (2, 2780)] {
            for extent in [15.125, 120.25] {
                for clipped in [false, true] {
                    let mut input = scene("licensed-ship");
                    let group = Vec2::from_array(input.groups[0].position);
                    let node = &mut input.groups[0].nodes[0];
                    node.value = corp as f32;
                    node.rect = [5.125, 9.25, extent, extent * 1.25];
                    let bounds = if clipped {
                        [5.125, 9.25, extent * 0.63, extent * 1.25]
                    } else {
                        node.rect
                    };
                    node.clip = clipped.then_some(bounds);
                    let mut frame = FrameBuilder::new();
                    draw(&mut frame, &input, [0.0; 2], [640.0, 360.0], 0.0, true);
                    assert_eq!(
                        frame.frame().pixel_count,
                        count,
                        "UI retains both authored source grids at every fit"
                    );
                    let mut visible = 0;
                    for batch in &frame.frame().batches {
                        for pixel in &batch.pixels {
                            if pixel.color >> 24 == 0 {
                                continue;
                            }
                            visible += 1;
                            let t = batch.transforms[pixel.transform_slot()];
                            for corner in [
                                Vec2::splat(-0.5),
                                Vec2::new(-0.5, 0.5),
                                Vec2::new(0.5, -0.5),
                                Vec2::splat(0.5),
                            ] {
                                let p = t.point(Vec2::from_array(pixel.local) + corner)
                                    + Vec2::new(320.0, 180.0)
                                    - group;
                                assert!(
                                    p.x >= bounds[0] - 0.0001
                                        && p.y >= bounds[1] - 0.0001
                                        && p.x <= bounds[0] + bounds[2] + 0.0001
                                        && p.y <= bounds[1] + bounds[3] + 0.0001
                                );
                            }
                        }
                    }
                    if clipped {
                        assert!(visible > 0 && visible < count);
                    } else {
                        assert_eq!(visible, count);
                    }
                }
            }
        }
    }
    #[test]
    fn campaign_ship_continuous_fit_retains_all_cells_below_authored_size() {
        for (index, name) in CAMPAIGN_SHIPS.iter().enumerate() {
            let body = &CAMPAIGN_ASSETS.sprites[*name].body;
            // 32 CSS pixels at UI scale 2.4, plus deliberately subpixel fits.
            for extent in [0.125, 32.0 / 2.4, 22.5, 32.0, 48.0, 60.0] {
                let mut input = scene("campaign-ship");
                let node = &mut input.groups[0].nodes[0];
                node.value = index as f32;
                node.rect = [5.125, 9.375, extent, extent * 0.75];
                let rect = node.rect;
                let mut frame = FrameBuilder::new();
                draw(&mut frame, &input, [0.0; 2], [640.0, 360.0], 0.0, true);
                assert_eq!(
                    frame.frame().pixel_count,
                    body.occupied_count(),
                    "{name}, {extent}"
                );
                let batch = &frame.frame().batches[0];
                assert_eq!(batch.transforms.len(), 1);
                let t = batch.transforms[0];
                let scale = extent * 0.75 / 32.0;
                assert_eq!(t.a[2], scale);
                assert_eq!(t.b[1], scale);
                assert_eq!(t.b[2..], [0.0, 0.0]);
                for pixel in &batch.pixels {
                    assert_ne!(pixel.color >> 24, 0);
                    let center = t.point(Vec2::from_array(pixel.local)) + Vec2::new(220.0, 130.0);
                    let half = scale * 0.5;
                    for corner in [
                        Vec2::new(-half, -half),
                        Vec2::new(half, -half),
                        Vec2::new(-half, half),
                        Vec2::new(half, half),
                    ] {
                        let point = center + corner;
                        assert!(
                            point.x >= rect[0] - 0.0001 && point.x <= rect[0] + rect[2] + 0.0001
                        );
                        assert!(
                            point.y >= rect[1] - 0.0001 && point.y <= rect[1] + rect[3] + 0.0001
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn campaign_ship_continuous_fit_preserves_state_colors_and_whole_cell_clip() {
        for (index, name) in CAMPAIGN_SHIPS.iter().enumerate() {
            let body = &CAMPAIGN_ASSETS.sprites[*name].body;
            for (active, disabled) in [(false, false), (true, false), (false, true), (true, true)] {
                let mut input = scene("campaign-ship");
                let node = &mut input.groups[0].nodes[0];
                node.value = index as f32;
                node.rect = [5.125, 9.375, 22.5, 17.25];
                node.active = active;
                node.disabled = disabled;
                let clip = [5.125, 9.375, 11.25, 17.25];
                node.clip = Some(clip);
                let mut frame = FrameBuilder::new();
                draw(&mut frame, &input, [0.0; 2], [640.0, 360.0], 17.0, false);
                assert_eq!(frame.frame().pixel_count, body.occupied_count());
                let scale = 17.25 / 32.0;
                let local = Transform {
                    position: Vec2::new(16.375, 18.0),
                    scale: Vec2::splat(scale),
                    ..Default::default()
                };
                let mut visible = 0;
                for batch in &frame.frame().batches {
                    for pixel in &batch.pixels {
                        let cell = Vec2::from_array(pixel.local);
                        let x = (cell.x + 15.5) as usize;
                        let y = (cell.y + 15.5) as usize;
                        let source = body.color_at(x, y).unwrap();
                        let center = local.point(cell);
                        let half = scale * 0.5;
                        let contained = center.x - half >= clip[0]
                            && center.y - half >= clip[1]
                            && center.x + half <= clip[0] + clip[2]
                            && center.y + half <= clip[1] + clip[3];
                        let expected = if !contained {
                            0
                        } else if disabled {
                            muted(source)
                        } else if active {
                            brighten(source)
                        } else {
                            source
                        };
                        assert_eq!(pixel.color, expected, "{name} cell {x},{y}");
                        if contained {
                            visible += 1;
                            let t = batch.transforms[pixel.transform_slot()];
                            let actual = t.point(cell) + Vec2::new(220.0, 130.0);
                            for corner in [
                                Vec2::new(-half, -half),
                                Vec2::new(half, -half),
                                Vec2::new(-half, half),
                                Vec2::new(half, half),
                            ] {
                                let point = actual + corner;
                                assert!(
                                    point.x >= clip[0] - 0.0001
                                        && point.x <= clip[0] + clip[2] + 0.0001
                                );
                                assert!(
                                    point.y >= clip[1] - 0.0001
                                        && point.y <= clip[1] + clip[3] + 0.0001
                                );
                            }
                        }
                    }
                }
                assert!(visible > 0 && visible < body.occupied_count());
            }
        }
    }

    #[test]
    fn campaign_ship_continuous_fit_debits_both_quotas_atomically_including_clipped_cells() {
        for (index, name) in CAMPAIGN_SHIPS.iter().enumerate() {
            let count = CAMPAIGN_ASSETS.sprites[*name].body.occupied_count();
            let mut input = scene("campaign-ship");
            let node = &mut input.groups[0].nodes[0];
            node.value = index as f32;
            node.rect = [5.125, 9.375, 13.25, 13.25];
            // No visible cells: clipping still charges the complete submitted body.
            let clip = Some([100.0, 100.0, 1.0, 1.0]);
            for (pixels, work, expected) in [
                (count - 1, count, 0),
                (count, count - 1, 0),
                (count, count, count),
            ] {
                let mut frame = FrameBuilder::new();
                frame.push_body(&CELL, Transform::default());
                let previous = frame.frame().clone();
                let mut painter = Painter {
                    frame: &mut frame,
                    parent: AffineTransform::default(),
                    remaining: pixels,
                    viewport: Vec2::new(640.0, 360.0),
                    clip,
                    work_remaining: work,
                };
                painter.node(node, 0.0, true);
                assert_eq!(painter.frame.frame().pixel_count, 1 + expected);
                assert_eq!(
                    (painter.remaining, painter.work_remaining),
                    (pixels - expected, work - expected)
                );
                if expected == 0 {
                    assert_eq!(
                        painter.frame.frame().batches[0].pixels,
                        previous.batches[0].pixels
                    );
                    assert_eq!(
                        painter.frame.frame().batches[0].transforms,
                        previous.batches[0].transforms
                    );
                } else {
                    assert!(
                        painter.frame.frame().batches[0]
                            .pixels
                            .iter()
                            .skip(1)
                            .all(|p| p.color == 0)
                    );
                }
            }
        }
    }

    #[test]
    fn semantic_text_alignment_positions_each_wrapped_line_without_vertical_change() {
        // The authored layout wraps into three differently sized lines at font .75.
        let lines = ["I", "WWWW", "WWW"];
        let count = lines
            .iter()
            .flat_map(|line| line.chars())
            .map(|c| RoyaleMono::body(c).occupied_count())
            .sum::<usize>();
        for kind in ["text", "offense-text", "defense-text", "passive-text"] {
            for alignment in [TextAlign::Left, TextAlign::Center, TextAlign::Right] {
                let mut input = scene(kind);
                let node = &mut input.groups[0].nodes[0];
                node.text = "I\nWWWW WWW".into();
                node.rect = [5.125, 9.375, 24.0, 24.0];
                node.font = 0.75;
                node.text_align = alignment;
                let mut frame = FrameBuilder::new();
                draw(&mut frame, &input, [0.0; 2], [640.0, 360.0], 0.0, true);
                assert_eq!(frame.frame().pixel_count, count);
                let batch = &frame.frame().batches[0];
                assert_eq!(batch.transforms.len(), 8);
                let mut index = 0;
                for (row, line) in lines.iter().enumerate() {
                    let width = (line.len() as f32 * 6.0 - 1.0) * 0.75;
                    let left = 5.125
                        + match alignment {
                            TextAlign::Left => 0.0,
                            TextAlign::Center => (24.0 - width) * 0.5,
                            TextAlign::Right => 24.0 - width,
                        };
                    for (column, _) in line.chars().enumerate() {
                        let t = batch.transforms[index];
                        assert_eq!(t.a[2], 0.75);
                        assert_eq!(t.b[1], 0.75);
                        assert!(
                            (t.a[0] + 220.0 - (left + column as f32 * 4.5 + 1.875)).abs() < 0.0001
                        );
                        assert!(
                            (t.a[1] + 130.0 - (9.375 + row as f32 * 6.0 + 2.625)).abs() < 0.0001
                        );
                        index += 1;
                    }
                }
                assert!(batch.pixels.iter().all(|p| p.color >> 24 != 0));
            }
        }
    }

    #[test]
    fn semantic_text_alignment_clips_whole_fractional_cells_and_keeps_work_charge() {
        let mut input = scene("text");
        let node = &mut input.groups[0].nodes[0];
        node.rect = [5.125, 9.375, 35.0, 18.0];
        node.text = "WWW\nI".into();
        node.font = 0.75;
        let source_count = "WWWI"
            .chars()
            .map(|c| RoyaleMono::body(c).occupied_count())
            .sum::<usize>();
        for alignment in [TextAlign::Left, TextAlign::Center, TextAlign::Right] {
            input.groups[0].nodes[0].text_align = alignment;
            let clip = [8.125, 10.125, 29.0, 8.25];
            input.groups[0].nodes[0].clip = Some(clip);
            let mut frame = FrameBuilder::new();
            draw(&mut frame, &input, [0.0; 2], [640.0, 360.0], 0.0, true);
            assert_eq!(frame.frame().pixel_count, source_count);
            let mut visible = 0;
            for batch in &frame.frame().batches {
                for pixel in &batch.pixels {
                    if pixel.color >> 24 == 0 {
                        continue;
                    }
                    visible += 1;
                    let t = batch.transforms[pixel.transform_slot()];
                    for corner in [
                        Vec2::splat(-0.5),
                        Vec2::new(0.5, -0.5),
                        Vec2::new(-0.5, 0.5),
                        Vec2::splat(0.5),
                    ] {
                        let p = t.point(Vec2::from_array(pixel.local) + corner)
                            + Vec2::new(220.0, 130.0);
                        assert!(p.x >= clip[0] - 0.0001 && p.x <= clip[0] + clip[2] + 0.0001);
                        assert!(p.y >= clip[1] - 0.0001 && p.y <= clip[1] + clip[3] + 0.0001);
                    }
                }
            }
            assert!(visible > 0 && visible < source_count);
            let mut frame = FrameBuilder::new();
            let mut painter = Painter {
                frame: &mut frame,
                parent: AffineTransform::default(),
                remaining: source_count,
                viewport: Vec2::new(640.0, 360.0),
                clip: Some(clip),
                work_remaining: source_count,
            };
            painter.node(&input.groups[0].nodes[0], 0.0, true);
            assert_eq!((painter.remaining, painter.work_remaining), (0, 0));
        }
    }

    #[test]
    fn semantic_text_alignment_defaults_invalid_payloads_without_losing_other_nodes() {
        for (token, expected) in [
            ("\"left\"", TextAlign::Left),
            ("\"center\"", TextAlign::Center),
            ("\"right\"", TextAlign::Right),
            ("\"justify\"", TextAlign::Left),
            ("null", TextAlign::Left),
            ("17", TextAlign::Left),
            ("{}", TextAlign::Left),
            ("[]", TextAlign::Left),
        ] {
            let json = format!(
                r#"{{"groups":[{{"nodes":[{{"kind":"text","text_align":{token}}},{{"kind":"text"}}]}}]}}"#
            );
            let input: UiScene = serde_json::from_str(&json).unwrap();
            assert_eq!(input.groups[0].nodes.len(), 2);
            assert_eq!(input.groups[0].nodes[0].text_align, expected);
            assert_eq!(input.groups[0].nodes[1].text_align, TextAlign::Left);
        }
    }

    #[test]
    fn semantic_text_alignment_does_not_change_centered_button_label_packets() {
        let render = |alignment| {
            let mut input = scene("primary-button");
            input.groups[0].nodes[0].rect = [5.125, 9.375, 80.0, 24.0];
            input.groups[0].nodes[0].text = "LAUNCH".into();
            input.groups[0].nodes[0].font = 0.75;
            input.groups[0].nodes[0].text_align = alignment;
            let mut frame = FrameBuilder::new();
            draw(&mut frame, &input, [0.0; 2], [640.0, 360.0], 0.0, true);
            frame.finish()
        };
        let baseline = render(TextAlign::Left);
        for alignment in [TextAlign::Center, TextAlign::Right] {
            let other = render(alignment);
            assert_eq!(baseline.pixel_count, other.pixel_count);
            for (a, b) in baseline.batches.iter().zip(&other.batches) {
                assert_eq!(a.pixels, b.pixels);
                assert_eq!(a.transforms, b.transforms);
            }
        }
    }

    #[test]
    fn component_icons_keep_source_detail_at_fractional_fit_and_clip_whole_cells() {
        for category in 0..3 {
            for extent in [17.25, 48.0, 73.5] {
                for clipped in [false, true] {
                    let mut input = scene("craft-component");
                    let node = &mut input.groups[0].nodes[0];
                    node.value = category as f32;
                    node.rect = [5.25, 9.5, extent, extent];
                    let clip = [5.25, 9.5, extent * 0.6, extent];
                    node.clip = clipped.then_some(clip);
                    let mut frame = FrameBuilder::new();
                    draw(&mut frame, &input, [0.0; 2], [640.0, 360.0], 1.5, false);
                    let full = component_art::body(category).unwrap().occupied_count();
                    assert!(frame.frame().pixel_count > 0);
                    assert_eq!(frame.frame().pixel_count, full);
                    let visible = frame
                        .frame()
                        .batches
                        .iter()
                        .flat_map(|b| &b.pixels)
                        .filter(|p| p.color >> 24 != 0)
                        .count();
                    if clipped {
                        assert!(visible > 0 && visible < full);
                    } else {
                        assert_eq!(visible, full);
                    }
                    for batch in &frame.frame().batches {
                        for pixel in &batch.pixels {
                            if pixel.color >> 24 == 0 {
                                continue;
                            }
                            let t = batch.transforms[pixel.transform_slot()];
                            assert_eq!(t.a[2], extent / 64.0);
                            assert_eq!(t.b[1], extent / 64.0);
                            assert_eq!(t.b[2..], [0.0, 0.0]);
                            let p =
                                t.point(Vec2::from_array(pixel.local)) + Vec2::new(220.0, 130.0);
                            let half = extent / 128.0;
                            let bounds = if clipped {
                                clip
                            } else {
                                [5.25, 9.5, extent, extent]
                            };
                            assert!(
                                p.x - half >= bounds[0] - 0.0001
                                    && p.y - half >= bounds[1] - 0.0001
                            );
                            assert!(
                                p.x + half <= bounds[0] + bounds[2] + 0.0001
                                    && p.y + half <= bounds[1] + bounds[3] + 0.0001
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn component_icon_rejects_invalid_ids_and_preserves_each_shared_budget() {
        for value in [-1.0, 1.5, 3.0, f32::NAN, f32::INFINITY, 1e20] {
            let mut input = scene("craft-component");
            input.groups[0].nodes[0].value = value;
            let mut frame = FrameBuilder::new();
            draw(&mut frame, &input, [0.0; 2], [640.0, 360.0], 0.0, true);
            assert_eq!(frame.frame().pixel_count, 0);
        }
        let input = scene("craft-component");
        let count = component_art::body(0).unwrap().occupied_count();
        for (pixels, work, expected) in [
            (count - 1, count, 0),
            (count, count - 1, 0),
            (count, count, count),
        ] {
            let mut frame = FrameBuilder::new();
            let mut painter = Painter {
                frame: &mut frame,
                parent: AffineTransform::default(),
                remaining: pixels,
                viewport: Vec2::new(640.0, 360.0),
                clip: None,
                work_remaining: work,
            };
            painter.node(&input.groups[0].nodes[0], 0.0, true);
            assert_eq!(painter.frame.frame().pixel_count, expected);
            assert_eq!(painter.remaining, pixels - expected);
            assert_eq!(painter.work_remaining, work - expected);
        }
    }
    #[test]
    fn meishi_fittings_keep_metal_neutral_corporate_enamel_and_whole_clipped_cells() {
        let camera = Vec2::new(75.5, -39.25);
        let group = Vec2::new(100.25, 50.5);
        let viewport = Vec2::new(640.0, 360.0);
        for kind in [
            "meishi-panel",
            "meishi-portrait-frame",
            "meishi-seal",
            "meishi-socket",
        ] {
            let mut palettes = Vec::new();
            for corporation in 0..3 {
                for clip in [None, Some([5.25, 9.5, 22.0, 27.0])] {
                    let mut input = scene(kind);
                    input.groups[0].position = group.to_array();
                    let node = &mut input.groups[0].nodes[0];
                    node.rect = [5.125, 9.375, 40.5, 40.5];
                    node.value = if kind == "meishi-socket" {
                        corporation * 8 + 3
                    } else {
                        corporation
                    } as f32;
                    node.selected = true;
                    node.clip = clip;
                    let mut frame = FrameBuilder::new();
                    draw(
                        &mut frame,
                        &input,
                        camera.to_array(),
                        viewport.to_array(),
                        19.0,
                        true,
                    );
                    assert!(frame.frame().pixel_count > 100);
                    let allowed = clip.unwrap_or([5.125, 9.375, 40.5, 40.5]);
                    let mut colors = std::collections::HashSet::new();
                    for batch in &frame.frame().batches {
                        for cell in &batch.pixels {
                            let transform = batch.transforms[cell.transform_slot()];
                            assert_eq!(transform.b[2..], [0.0, 0.0]);
                            if cell.color >> 24 == 0 {
                                continue;
                            }
                            colors.insert(cell.color);
                            for corner in [
                                Vec2::splat(-0.5),
                                Vec2::new(0.5, -0.5),
                                Vec2::new(-0.5, 0.5),
                                Vec2::splat(0.5),
                            ] {
                                let local = transform.point(Vec2::from_array(cell.local) + corner)
                                    - camera
                                    + viewport * 0.5
                                    - group;
                                assert!(local.is_finite());
                                assert!(
                                    local.x >= allowed[0] - 0.0001
                                        && local.x <= allowed[0] + allowed[2] + 0.0001
                                );
                                assert!(
                                    local.y >= allowed[1] - 0.0001
                                        && local.y <= allowed[1] + allowed[3] + 0.0001
                                );
                            }
                        }
                    }
                    if clip.is_none() {
                        palettes.push(colors);
                    }
                }
            }
            assert_ne!(palettes[0], palettes[1]);
            assert_ne!(palettes[1], palettes[2]);
            if kind == "meishi-panel" {
                for (colors, deck) in
                    palettes
                        .iter()
                        .zip([rgb(6, 11, 10), rgb(12, 8, 10), rgb(7, 10, 14)])
                {
                    assert!(
                        colors.contains(&deck),
                        "opaque deck must carry the main corporation theme"
                    );
                }
            }
            if kind != "meishi-seal" {
                for colors in palettes {
                    assert!(colors.contains(&rgb(162, 175, 176)));
                }
            }
        }
    }

    #[test]
    fn meishi_fittings_preflight_each_shared_quota_and_freeze_reduced_color() {
        for kind in [
            "meishi-panel",
            "meishi-portrait-frame",
            "meishi-seal",
            "meishi-socket",
        ] {
            let input = scene(kind);
            let mut complete = FrameBuilder::new();
            draw(&mut complete, &input, [0.0; 2], [640.0, 360.0], 0.0, true);
            let count = complete.frame().pixel_count;
            for (pixels, work, expected) in [
                (count - 1, 120000, 0),
                (60000, count - 1, 0),
                (count, count, count),
            ] {
                let mut frame = FrameBuilder::new();
                frame.push_body(&CELL, Transform::default());
                let previous = frame.frame().clone();
                let mut painter = Painter {
                    frame: &mut frame,
                    parent: AffineTransform::default(),
                    remaining: pixels,
                    viewport: Vec2::new(640.0, 360.0),
                    clip: None,
                    work_remaining: work,
                };
                painter.node(&input.groups[0].nodes[0], 0.0, true);
                assert_eq!(painter.frame.frame().pixel_count, 1 + expected);
                assert_eq!(painter.remaining, pixels - expected);
                assert_eq!(painter.work_remaining, work - expected);
                if expected == 0 {
                    assert_eq!(
                        painter.frame.frame().batches[0].pixels,
                        previous.batches[0].pixels
                    );
                    assert_eq!(
                        painter.frame.frame().batches[0].transforms,
                        previous.batches[0].transforms
                    );
                }
            }
            let mut later = FrameBuilder::new();
            draw(&mut later, &input, [0.0; 2], [640.0, 360.0], 3.7, true);
            assert_eq!(complete.frame().batches.len(), later.frame().batches.len());
            for (a, b) in complete.frame().batches.iter().zip(&later.frame().batches) {
                assert_eq!(a.pixels, b.pixels);
                assert_eq!(a.transforms, b.transforms);
            }
        }
    }

    #[test]
    fn meishi_short_quota_suppresses_paired_earned_patch_without_affecting_campaign() {
        let socket = UiNode {
            value: 0.0,
            selected: true,
            ..scene("meishi-socket").groups[0].nodes.remove(0)
        };
        let patch = UiNode {
            value: 0.0,
            active: true,
            ..scene("campaign-patch").groups[0].nodes.remove(0)
        };
        let submitted_cost = |node: &UiNode| {
            let mut frame = FrameBuilder::new();
            let mut painter = Painter {
                frame: &mut frame,
                parent: AffineTransform::default(),
                remaining: MAX_UI_PIXELS,
                viewport: Vec2::new(640.0, 360.0),
                clip: None,
                work_remaining: MAX_UI_PIXELS * 2,
            };
            painter.node(node, 0.0, true);
            painter.frame.frame().pixel_count
        };
        let socket_cost = submitted_cost(&socket);
        let patch_cost = submitted_cost(&patch);
        assert_eq!(socket_cost, 634); // 632 authored well cells plus two floor quads.
        assert_eq!(patch_cost, 165);
        let pair_cost = socket_cost + patch_cost;
        for (pixels, work, expected) in [
            (200, 120000, 0),
            (60000, 200, 0),
            (socket_cost - 1, 120000, 0),
            (60000, socket_cost - 1, 0),
            (pair_cost - 1, pair_cost - 1, socket_cost),
            (pair_cost, pair_cost, pair_cost),
        ] {
            let mut frame = FrameBuilder::new();
            let mut painter = Painter {
                frame: &mut frame,
                parent: AffineTransform::default(),
                remaining: pixels,
                viewport: Vec2::new(640.0, 360.0),
                clip: None,
                work_remaining: work,
            };
            let mut failed = 0;
            paired_meishi_node(&mut painter, &socket, 0.0, true, &mut failed);
            paired_meishi_node(&mut painter, &patch, 0.0, true, &mut failed);
            assert_eq!(painter.frame.frame().pixel_count, expected);
            assert_eq!(
                (painter.remaining, painter.work_remaining),
                (pixels - expected, work - expected)
            );
            if expected == 0 {
                assert_eq!(failed, 1);
            }
        }
        let mut frame = FrameBuilder::new();
        let mut painter = Painter {
            frame: &mut frame,
            parent: AffineTransform::default(),
            remaining: 200,
            viewport: Vec2::new(640.0, 360.0),
            clip: None,
            work_remaining: 200,
        };
        paired_meishi_node(&mut painter, &patch, 0.0, true, &mut 0);
        assert_eq!(painter.frame.frame().pixel_count, 165);
    }

    #[test]
    fn meishi_collectible_all_earned_card_admits_every_component_for_nine_profiles() {
        let node = |kind: &str, value, rect| UiNode {
            kind: kind.into(),
            value,
            rect,
            ..Default::default()
        };
        let packet = |frame: &engine_core::RenderFrame| {
            frame
                .batches
                .iter()
                .flat_map(|batch| {
                    batch.pixels.iter().map(|pixel| {
                        let t = batch.transforms[pixel.transform_slot()];
                        (pixel.local, pixel.color, t.a, t.b)
                    })
                })
                .collect::<Vec<_>>()
        };
        let render = |nodes| {
            let mut frame = FrameBuilder::new();
            draw(
                &mut frame,
                &UiScene {
                    groups: vec![UiGroup {
                        position: [20.25, 10.5],
                        nodes,
                    }],
                },
                [0.0; 2],
                [640.0, 360.0],
                0.0,
                true,
            );
            frame.finish()
        };
        for kind in 0..3 {
            for corp in 0..3 {
                let mut nodes = vec![
                    node("meishi-panel", corp as f32, [5.0, 5.0, 240.0, 324.0]),
                    node(
                        "meishi-portrait-frame",
                        corp as f32,
                        [14.0, 30.0, 80.0, 104.0],
                    ),
                    UiNode {
                        portrait_kind: Some(kind),
                        portrait_main: Some(corp),
                        portrait_sub: Some((corp + 1) % 3),
                        ..node("portrait", 9.0, [18.0, 34.0, 72.0, 96.0])
                    },
                    node("meishi-seal", corp as f32, [102.0, 34.0, 24.0, 24.0]),
                    node("licensed-ship", corp as f32, [102.0, 82.0, 32.0, 32.0]),
                    node("close-icon", 0.0, [215.0, 12.0, 22.0, 22.0]),
                ];
                for (index, text) in [
                    "MEISHI CARD",
                    "APR#0123456789ABCDEF",
                    "DORA-PAN-RENKO",
                    "FACTORY RAICHO LV 1",
                    "RENKO 13 HUGO 29 PAN 47",
                    "EIGHT ARENA PATCHES",
                ]
                .into_iter()
                .enumerate()
                {
                    nodes.push(UiNode {
                        text: text.into(),
                        font: 0.75,
                        ..node("text", 0.0, [14.0, 144.0 + index as f32 * 11.0, 220.0, 9.0])
                    });
                }
                for slot in 0..8 {
                    let x = 18.125 + (slot % 4) as f32 * 54.0;
                    let y = 219.375 + (slot / 4) as f32 * 49.0;
                    nodes.push(UiNode {
                        selected: true,
                        ..node(
                            "meishi-socket",
                            (corp as usize * 8 + slot) as f32,
                            [x, y, 32.0, 32.0],
                        )
                    });
                    // The previous DOM border/75% fit really measured22.5, while
                    // the corrected well measures24. Both retain the source cells.
                    let size = if slot % 2 == 0 { 22.5 } else { 24.0 };
                    let inset = (32.0 - size) * 0.5;
                    nodes.push(UiNode {
                        active: true,
                        ..node(
                            "campaign-patch",
                            slot as f32,
                            [x + inset, y + inset, size, size],
                        )
                    });
                }
                let mut joined = Vec::new();
                let mut expected = Vec::new();
                let mut isolated_counts = Vec::new();
                for node in nodes {
                    let mut scene = UiScene {
                        groups: vec![UiGroup {
                            position: [20.25, 10.5],
                            nodes: vec![node],
                        }],
                    };
                    let mut frame = FrameBuilder::new();
                    draw(&mut frame, &scene, [0.0; 2], [640.0, 360.0], 0.0, true);
                    assert!(frame.frame().pixel_count > 0);
                    assert!(
                        frame
                            .frame()
                            .batches
                            .iter()
                            .flat_map(|b| &b.pixels)
                            .all(|p| p.color >> 24 != 0),
                        "full-fit component lost a visible cell"
                    );
                    isolated_counts.push(frame.frame().pixel_count);
                    expected.extend(packet(frame.frame()));
                    joined.push(scene.groups[0].nodes.remove(0));
                }
                assert_eq!(isolated_counts[2], [34594, 28524, 24262][kind as usize]);
                // Retained collision cells plus the authored sparse96-grid detail.
                assert_eq!(isolated_counts[4], [1867, 1864, 2780][corp as usize]);
                assert_eq!(isolated_counts[0], 5818);
                assert_eq!(isolated_counts[1], 1677);
                for (slot, count) in [165, 321, 275, 219, 225, 323, 326, 202]
                    .into_iter()
                    .enumerate()
                {
                    assert_eq!(isolated_counts[12 + slot * 2], 634);
                    assert_eq!(isolated_counts[13 + slot * 2], count);
                }
                let decoration = isolated_counts[0]
                    + isolated_counts[1]
                    + isolated_counts[3]
                    + (0..8)
                        .map(|slot| isolated_counts[12 + slot * 2])
                        .sum::<usize>();
                assert_eq!(decoration, [12_949, 12_887, 12_957][corp as usize]);
                assert!(decoration <= 13_000);
                let complete = render(joined);
                assert!(complete.pixel_count < MAX_UI_PIXELS);
                assert_eq!(complete.pixel_count, expected.len());
                assert_eq!(
                    packet(&complete),
                    expected,
                    "late component suppressed or altered"
                );
                println!(
                    "MEISHI collectible type{kind}/corp{corp}: {} /60000 complete cells",
                    complete.pixel_count
                );
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
        let transform = batch.transforms[batch.pixels[0].transform_slot()];
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
            let background = batch.transforms[batch.pixels[0].transform_slot()];
            assert_eq!((background.a[2], background.b[1]), (60.0, 20.0));
            let cursor = batch.pixels.iter().find(|p| p.color == GOLD).unwrap();
            let transform = batch.transforms[cursor.transform_slot()];
            assert_eq!((transform.a[2], transform.b[1]), (1.0, 7.0));
            let center = transform.point(Vec2::from_array(cursor.local)) + Vec2::new(320.0, 180.0)
                - Vec2::new(100.0, 50.0);
            assert_eq!(center.x - 0.5, expected);
            for pixel in &batch.pixels {
                let transform = batch.transforms[pixel.transform_slot()];
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
                let aw = a.transforms[ap.transform_slot()].point(Vec2::from_array(ap.local));
                let bw = b.transforms[bp.transform_slot()].point(Vec2::from_array(bp.local))
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
                    let p = batch.transforms[pixel.transform_slot()]
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
                .all(|p| (p.transform_slot()) < b.transforms.len())
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

#[cfg(test)]
#[path = "ui_kikai_integration_tests.rs"]
mod kikai_integration_tests;

#[cfg(test)]
#[path = "ui_typography_tests.rs"]
mod typography_tests;

#[cfg(test)]
#[path = "ui_capped_settings_tests.rs"]
mod capped_settings_tests;

#[cfg(test)]
#[path = "ui_settings_skin_tests.rs"]
mod settings_skin_tests;
