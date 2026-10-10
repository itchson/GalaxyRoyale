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
    pub disabled: bool,
    pub clip: Option<[f32; 4]>,
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
            position: Vec2::new(rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5 + y_offset),
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
                rect[0] + (rect[2] - (line.chars().count() as f32 * 6.0 - 1.0) * font) * 0.5
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
            "panel" => self.frame("panel", r, node.active, node.disabled),
            "console-panel" => self.console_panel(r),
            "portrait" => self.portrait(node, time, reduced),
            "campaign-patch" => self.campaign(node, false, time, reduced),
            "campaign-ship" => self.campaign(node, true, time, reduced),
            "input" => {
                self.frame("button", r, node.active, node.disabled);
                let height = (r[3] - 6.0).max(RoyaleMono::HEIGHT);
                let width = (r[2] - 12.0).max(RoyaleMono::WIDTH);
                let font = font.min((height / RoyaleMono::HEIGHT).floor().max(1.0));
                let columns = ((width + font) / (RoyaleMono::ADVANCE * font)).floor() as usize;
                let chars: Vec<_> = node.text.chars().take(256).collect();
                let begin = chars.len().saturating_sub(columns.saturating_sub(3));
                let value: String = chars[begin..].iter().collect();
                let cursor =
                    if node.active && (reduced || (time * 2.0).floor().rem_euclid(2.0) == 0.0) {
                        '|'
                    } else {
                        ' '
                    };
                self.text(
                    [
                        r[0] + 6.0,
                        r[1] + (r[3] - RoyaleMono::HEIGHT * font) * 0.5,
                        width,
                        RoyaleMono::HEIGHT * font,
                    ],
                    &format!("> {value}{cursor}"),
                    font,
                    color,
                    false,
                );
            }
            "button" => {
                self.frame("button", r, node.active, node.disabled);
                let width = (r[2] - 12.0).max(5.0);
                let height = (r[3] - 6.0).max(7.0);
                let characters = node.text.chars().take(256).count();
                let units = (characters as f32 * 6.0 - 1.0).max(5.0);
                let font = font
                    .min((width / units).floor().max(1.0))
                    .min((height / 7.0).floor().max(1.0));
                let label: String = node
                    .text
                    .chars()
                    .take(((width + font) / (6.0 * font)).floor().max(1.0) as usize)
                    .collect();
                let label_height = height.max(7.0 * font);
                self.text(
                    [
                        r[0] + 6.0,
                        r[1] + (r[3] - label_height) * 0.5,
                        width,
                        label_height,
                    ],
                    &label,
                    font,
                    color,
                    true,
                );
            }
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
            "slider" => {
                let value = if node.value.is_finite() {
                    node.value.clamp(0.0, 1.0)
                } else {
                    0.0
                };
                if r[3] >= 20.0 {
                    self.text(
                        [r[0], r[1], r[2], 8.0 * font],
                        &node.text,
                        font,
                        color,
                        false,
                    );
                }
                let track = [r[0] + 4.0, r[1] + r[3] - 7.0, (r[2] - 8.0).max(0.0), 3.0];
                self.rect(track, 0xff_51402c);
                self.rect(
                    [track[0], track[1], track[2] * value, 3.0],
                    if node.disabled { DIM } else { CYAN },
                );
                self.rect(
                    [track[0] + track[2] * value - 3.0, track[1] - 2.0, 6.0, 7.0],
                    color,
                );
            }
            "toggle" => {
                let size = r[3].min(12.0).floor();
                self.rect([r[0], r[1], size, size], DIM);
                self.rect(
                    [
                        r[0] + 1.0,
                        r[1] + 1.0,
                        (size - 2.0).max(0.0),
                        (size - 2.0).max(0.0),
                    ],
                    0xff_26140d,
                );
                if node.value > 0.5 {
                    self.rect(
                        [
                            r[0] + 3.0,
                            r[1] + 3.0,
                            (size - 6.0).max(0.0),
                            (size - 6.0).max(0.0),
                        ],
                        if node.disabled { DIM } else { CYAN },
                    );
                }
                self.text(
                    [r[0] + size + 7.0, r[1], (r[2] - size - 7.0).max(0.0), r[3]],
                    &node.text,
                    font,
                    color,
                    false,
                );
            }
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
fn wrap(text: &str, columns: usize) -> Vec<String> {
    let text: String = text.chars().take(256).collect();
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let chars: Vec<_> = word.chars().collect();
            if !line.is_empty() && line.chars().count() + 1 + chars.len() > columns {
                lines.push(std::mem::take(&mut line));
            }
            for chunk in chars.chunks(columns) {
                if !line.is_empty() {
                    line.push(' ');
                }
                line.extend(chunk);
                if line.chars().count() >= columns {
                    lines.push(std::mem::take(&mut line));
                }
            }
        }
        if !line.is_empty() || paragraph.is_empty() {
            lines.push(line);
        }
    }
    lines
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
            node.font = 1.0;
            node.active = true;
            let mut frame = FrameBuilder::new();
            draw(&mut frame, &scene, [0.0; 2], [640.0, 360.0], time, reduced);
            frame.finish()
        };
        assert_eq!(
            extract(0.0, false).pixel_count - extract(0.75, false).pixel_count,
            7
        );
        assert_eq!(
            extract(0.0, true).batches[0].pixels,
            extract(0.75, true).batches[0].pixels
        );
        for character in ['|', '_', '?', '=', '#', ',', '\''] {
            assert!(RoyaleMono::body(character).occupied_count() > 0);
        }
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
