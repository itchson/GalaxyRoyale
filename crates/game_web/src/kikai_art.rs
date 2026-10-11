//! DORA-PAN-RENKO-1337: reviewed raster -> authored, immutable object-local cells.
//! This module has no game/save state. Host-provided expression/speech is explicit.
use engine_core::{AffineTransform, FrameBuilder, PixelBody, Transform, Vec2};
use serde::Deserialize;
use std::{collections::HashMap, sync::LazyLock};

pub const PORTRAIT_ID: usize = 8;
pub const PORTRAIT_SIZE: [usize; 2] = [96, 128];
pub const ICON_SIZE: usize = 32;
pub const MAX_PORTRAIT_QUADS: usize = 12_288;
pub const ICON_SHELL: u32 = u32::from_le_bytes([217, 234, 246, 255]);
pub const ICON_EDGE: u32 = u32::from_le_bytes([255, 255, 255, 255]);
pub const ICON_BEVEL: u32 = u32::from_le_bytes([95, 117, 126, 255]);
pub const ICON_RECESS: u32 = u32::from_le_bytes([42, 56, 65, 255]);
pub const ICON_STEEL: u32 = u32::from_le_bytes([99, 133, 164, 255]);
pub const ICON_ACCENT: u32 = u32::from_le_bytes([88, 222, 239, 255]);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DoraExpression {
    #[default]
    Neutral,
    Focused,
    Concerned,
    Pleased,
}
impl DoraExpression {
    pub fn from_label(label: &str) -> Self {
        match label {
            "focused" => Self::Focused,
            "concerned" => Self::Concerned,
            "pleased" => Self::Pleased,
            _ => Self::Neutral,
        }
    }
    fn eye_name(self) -> &'static str {
        match self {
            Self::Neutral => "dora_eyes_neutral",
            Self::Focused => "dora_eyes_focused",
            Self::Concerned => "dora_eyes_concerned",
            Self::Pleased => "dora_eyes_pleased",
        }
    }
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Corporation {
    #[default]
    Pan,
    Renko,
    Hugo,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CorporateAppearance {
    pub main: Corporation,
    pub sub: Corporation,
}
impl Default for CorporateAppearance {
    fn default() -> Self {
        Self {
            main: Corporation::Pan,
            sub: Corporation::Renko,
        }
    }
}
#[derive(Deserialize)]
struct Sprite {
    width: usize,
    height: usize,
    indices: Vec<u8>,
}
#[derive(Deserialize)]
struct PortraitRoles {
    hair_cells: Vec<usize>,
    horn_cells: Vec<usize>,
    signal_cells: Vec<usize>,
    source_size: [usize; 2],
    default_main: String,
    default_sub: String,
}
#[derive(Deserialize)]
struct Atlas {
    version: u32,
    palette: Vec<u32>,
    sprites: HashMap<String, Sprite>,
    portrait_roles: PortraitRoles,
}
struct Layer {
    body: PixelBody,
    hair: Box<[bool]>,
    horns: Box<[bool]>,
    signal: Box<[bool]>,
}
static ART: LazyLock<HashMap<String, Layer>> = LazyLock::new(|| {
    let atlas: Atlas = serde_json::from_str(include_str!("../../../assets/ui/kikai/atlas.json"))
        .expect("validated Kikai cell pack");
    assert_eq!(atlas.version, 1);
    assert!((2..=64).contains(&atlas.palette.len()));
    assert_eq!(atlas.palette[0], 0);
    assert!(atlas.palette[1..].iter().all(|color| color >> 24 == 255));
    assert_eq!(atlas.portrait_roles.source_size, PORTRAIT_SIZE);
    assert_eq!(atlas.portrait_roles.default_main, "PAN");
    assert_eq!(atlas.portrait_roles.default_sub, "RENKO");
    let key = |i: usize| char::from_u32(0xe000 + i as u32).unwrap();
    let palette: Vec<_> = atlas
        .palette
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, &color)| (key(i), color))
        .collect();
    atlas
        .sprites
        .into_iter()
        .map(|(name, sprite)| {
            let expected = if name.starts_with("icon_") {
                [ICON_SIZE; 2]
            } else {
                PORTRAIT_SIZE
            };
            assert_eq!([sprite.width, sprite.height], expected);
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
            let body = PixelBody::from_ascii(
                &rows.iter().map(String::as_str).collect::<Vec<_>>(),
                &palette,
            );
            let mut hair = vec![false; sprite.indices.len()].into_boxed_slice();
            let mut horns = hair.clone();
            let mut signal = hair.clone();
            if name == "dora_body" {
                for (mask, indices) in [
                    (&mut hair, &atlas.portrait_roles.hair_cells),
                    (&mut horns, &atlas.portrait_roles.horn_cells),
                    (&mut signal, &atlas.portrait_roles.signal_cells),
                ] {
                    for &i in indices {
                        assert!(i < sprite.indices.len() && sprite.indices[i] != 0 && !mask[i]);
                        mask[i] = true;
                    }
                }
            }
            assert!(hair.iter().zip(horns.iter()).all(|(&a, &b)| !(a && b)));
            (
                name,
                Layer {
                    body,
                    hair,
                    horns,
                    signal,
                },
            )
        })
        .collect()
});

#[derive(Debug, Clone, Copy)]
pub struct DoraInput {
    pub parent: AffineTransform,
    pub rect: [f32; 4],
    pub clip: Option<[f32; 4]>,
    pub time: f32,
    pub reduced_motion: bool,
    pub expression: DoraExpression,
    pub speaking: bool,
    pub disabled: bool,
}
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DoraStats {
    pub requested_quads: usize,
    pub quads: usize,
    pub visible_cells: usize,
    pub transforms: usize,
    pub scale: f32,
    pub blink: bool,
    pub mouth_frame: u8,
    pub budget_rejected: bool,
}
fn valid_rect(rect: [f32; 4]) -> bool {
    rect.iter().all(|v| v.is_finite() && v.abs() <= 8192.0) && rect[2] > 0.0 && rect[3] > 0.0
}
fn valid_parent(parent: AffineTransform) -> bool {
    parent.position.is_finite()
        && parent.position.abs().max_element() <= 1_100_000.0
        && parent.basis_x.is_finite()
        && parent.basis_y.is_finite()
        && parent.basis_x.abs().max_element() <= 64.0
        && parent.basis_y.abs().max_element() <= 64.0
        && parent.basis_x.perp_dot(parent.basis_y).is_finite()
        && parent.basis_x.perp_dot(parent.basis_y) != 0.0
}
fn full_cell_inside(local: Transform, cell: Vec2, rect: [f32; 4], clip: Option<[f32; 4]>) -> bool {
    [
        Vec2::splat(-0.5),
        Vec2::new(0.5, -0.5),
        Vec2::new(-0.5, 0.5),
        Vec2::splat(0.5),
    ]
    .into_iter()
    .all(|corner| {
        let p = local.point(cell + corner);
        [Some(rect), clip]
            .into_iter()
            .flatten()
            .all(|r| inside_axis(p.x, r[0], r[0] + r[2]) && inside_axis(p.y, r[1], r[1] + r[3]))
    })
}
fn inside_axis(value: f32, min: f32, max: f32) -> bool {
    // Fit/center arithmetic can disagree with the equivalent rectangle edge by
    // a few f32 rounding bits. This is at most 0.00001 local units, not a cell
    // margin; genuine partial cells outside either the fit or clip stay masked.
    let roundoff = (2.0 * f32::EPSILON * min.abs().max(max.abs()).max(1.0)).min(0.00001);
    value >= min - roundoff && value <= max + roundoff
}
fn local_index(body: &PixelBody, cell: Vec2) -> usize {
    let x = (cell.x + (body.width - 1) as f32 * 0.5).round() as usize;
    let y = (cell.y + (body.height - 1) as f32 * 0.5).round() as usize;
    y * body.width + x
}
fn corporate_color(base: u32, corporation: Corporation) -> u32 {
    let [r, g, b, a] = base.to_le_bytes();
    let [r, g, b] = match corporation {
        Corporation::Pan => [r, g, b],
        Corporation::Renko => [
            g,
            ((r as u16 + b as u16) * 3 / 10) as u8,
            (b as u16 * 4 / 5) as u8,
        ],
        Corporation::Hugo => [
            (b as u16 * 7 / 25) as u8,
            ((r as u16 * 13 + g as u16 * 6) / 20) as u8,
            g,
        ],
    };
    u32::from_le_bytes([r, g, b, a])
}
fn horn_color(base: u32, corporation: Corporation) -> u32 {
    let [r, g, b, a] = base.to_le_bytes();
    let [r, g, b] = match corporation {
        Corporation::Renko => [r, g, b],
        Corporation::Pan => [(g as u16 * 4 / 5) as u8, r, (b as u16 * 4 / 5) as u8],
        Corporation::Hugo => [(b as u16 * 4 / 5) as u8, (g as u16 * 4 / 5) as u8, r],
    };
    u32::from_le_bytes([r, g, b, a])
}

/// APR-1337 has PAN green hair and RENKO red crystals, independent of preview defaults.
pub fn append_dora(
    frame: &mut FrameBuilder,
    input: &DoraInput,
    pixels: &mut usize,
    work: &mut usize,
) -> DoraStats {
    append_corporate_portrait(
        frame,
        input,
        CorporateAppearance {
            main: Corporation::Pan,
            sub: Corporation::Renko,
        },
        pixels,
        work,
    )
}
/// Female long-hair preview geometry; recolor does not author DORO/TALO haircuts or an identity.
pub fn append_corporate_portrait(
    frame: &mut FrameBuilder,
    input: &DoraInput,
    appearance: CorporateAppearance,
    pixels: &mut usize,
    work: &mut usize,
) -> DoraStats {
    if !valid_rect(input.rect)
        || !valid_parent(input.parent)
        || input.clip.is_some_and(|r| !valid_rect(r))
    {
        return DoraStats::default();
    }
    let time = if input.time.is_finite() {
        input.time.max(0.0).rem_euclid(36.0)
    } else {
        0.0
    };
    let moving = !input.reduced_motion && !input.disabled;
    let blink = moving && time.rem_euclid(4.5) > 4.36;
    let mouth_frame = if input.speaking && moving {
        ((time * 7.0).floor() as u32 % 3) as u8
    } else {
        0
    };
    let eye = if blink {
        "dora_eyes_blink"
    } else {
        input.expression.eye_name()
    };
    let mouth = match mouth_frame {
        1 => "dora_mouth_small",
        2 => "dora_mouth_open",
        _ if input.expression == DoraExpression::Pleased => "dora_mouth_smile",
        _ => "dora_mouth_rest",
    };
    let layers = [&ART["dora_body"], &ART[eye], &ART[mouth]];
    let count = layers
        .iter()
        .map(|layer| layer.body.occupied_count())
        .sum::<usize>();
    let mut stats = DoraStats {
        requested_quads: count,
        blink,
        mouth_frame,
        ..DoraStats::default()
    };
    if count > MAX_PORTRAIT_QUADS || count > *pixels || count > *work {
        stats.budget_rejected = true;
        return stats;
    }
    let r = input.rect;
    let scale = (r[2] / PORTRAIT_SIZE[0] as f32).min(r[3] / PORTRAIT_SIZE[1] as f32);
    let local = Transform {
        position: Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5),
        scale: Vec2::splat(scale),
        ..Transform::default()
    };
    for layer in layers {
        frame.push_body_affine_colored(
            &layer.body,
            input.parent.compose(local.affine()),
            |cell, mut color| {
                if !full_cell_inside(local, cell, r, input.clip) {
                    return 0;
                }
                stats.visible_cells += 1;
                let index = local_index(&layer.body, cell);
                if layer.hair[index] {
                    color = corporate_color(color, appearance.main);
                }
                if layer.horns[index] {
                    color = horn_color(color, appearance.sub);
                }
                let [r, g, b, _] = color.to_le_bytes();
                if input.disabled {
                    let gray = ((r as u16 + g as u16 + b as u16) / 4) as u8;
                    return u32::from_le_bytes([gray, gray, gray, 255]);
                }
                if moving && layer.signal[index] {
                    let lift = ((time * 1.4 + cell.y * 0.16).sin() + 1.0) * 3.0;
                    let lift = lift as u8;
                    color = u32::from_le_bytes([
                        r.saturating_add(lift),
                        g.saturating_add(lift / 2),
                        b,
                        255,
                    ]);
                }
                color
            },
        );
    }
    *pixels -= count;
    *work -= count;
    stats.quads = count;
    stats.transforms = layers.len();
    stats.scale = scale;
    stats
}

pub fn icon(kind: &str) -> Option<&'static PixelBody> {
    let name = match kind {
        "status-icon" => "icon_status",
        "map-icon" => "icon_map",
        "settings-icon" => "icon_settings",
        "save-icon" => "icon_save",
        "close-icon" => "icon_close",
        "resume-icon" => "icon_resume",
        "restart-icon" => "icon_restart",
        "title-icon" => "icon_title",
        "quit-icon" => "icon_quit",
        "fullscreen-icon" => "icon_fullscreen",
        _ => return None,
    };
    Some(&ART[name].body)
}
/// Palette roles remain coherent under semantic action accents and hover/press.
pub fn icon_color(shade: u32, accent: u32, hovered: bool, pressed: bool, disabled: bool) -> u32 {
    if disabled {
        return u32::from_le_bytes([105, 122, 138, 255]);
    }
    let [r, g, b, _] = accent.to_le_bytes();
    let strength = if shade == ICON_EDGE {
        1.16
    } else if shade == ICON_BEVEL {
        0.68
    } else if shade == ICON_RECESS {
        0.22
    } else if shade == ICON_STEEL {
        0.48
    } else if shade == ICON_ACCENT {
        1.0
    } else {
        0.85
    };
    let strength = strength
        * if pressed {
            0.88
        } else if hovered {
            1.10
        } else {
            1.0
        };
    let channel = |c: u8| {
        (c as f32 * strength + if shade == ICON_RECESS { 7.0 } else { 0.0 }).min(255.0) as u8
    };
    u32::from_le_bytes([channel(r), channel(g), channel(b), 255])
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> DoraInput {
        DoraInput {
            parent: AffineTransform::default(),
            rect: [7.0, 9.0, 48.0, 64.0],
            clip: None,
            time: 0.0,
            reduced_motion: true,
            expression: DoraExpression::Neutral,
            speaking: false,
            disabled: false,
        }
    }
    fn draw(i: DoraInput, appearance: CorporateAppearance) -> (FrameBuilder, DoraStats) {
        let mut f = FrameBuilder::new();
        let (mut p, mut w) = (60_000, 120_000);
        let stats = append_corporate_portrait(&mut f, &i, appearance, &mut p, &mut w);
        assert_eq!((p, w), (60_000 - stats.quads, 120_000 - stats.quads));
        assert_eq!(stats.quads, f.frame().pixel_count);
        (f, stats)
    }
    fn colors(f: &FrameBuilder) -> Vec<u32> {
        f.frame()
            .batches
            .iter()
            .flat_map(|b| b.pixels.iter().map(|p| p.color))
            .collect()
    }
    #[test]
    fn real_cached_layers_have_disjoint_regions_and_fine_detail_below_group_budget() {
        assert_eq!(ART.len(), 20);
        let body = &ART["dora_body"];
        assert_eq!((body.body.width, body.body.height), (96, 128));
        assert!(body.body.occupied_count() > 7000);
        assert!(body.hair.iter().filter(|&&b| b).count() > 1000);
        assert!(body.horns.iter().filter(|&&b| b).count() > 100);
        for eye in ["neutral", "focused", "concerned", "pleased", "blink"] {
            for mouth in ["rest", "small", "open", "smile"] {
                let layers = [
                    &body.body,
                    &ART[&format!("dora_eyes_{eye}")].body,
                    &ART[&format!("dora_mouth_{mouth}")].body,
                ];
                let mut positions = std::collections::HashSet::new();
                for layer in layers {
                    for (p, color) in layer.cells() {
                        assert!(
                            positions.insert((p.x.to_bits(), p.y.to_bits())),
                            "overlapping authored layers"
                        );
                        assert_eq!(color >> 24, 255);
                    }
                }
                assert!(positions.len() <= MAX_PORTRAIT_QUADS);
            }
        }
    }
    #[test]
    fn default_and_narrow_fractional_fit_keep_every_occupied_cell() {
        let mut expected = None;
        for size in [[96.0, 128.0], [48.0, 64.0], [32.0, 48.0], [12.0, 16.0]] {
            let i = DoraInput {
                rect: [7.0, 9.0, size[0], size[1]],
                ..input()
            };
            let (_, s) = draw(i, CorporateAppearance::default());
            assert_eq!(s.visible_cells, s.quads);
            assert_eq!(s.transforms, 3);
            assert_eq!(s.scale, (size[0] / 96.0).min(size[1] / 128.0));
            assert_eq!(*expected.get_or_insert(s.quads), s.quads);
        }
    }
    #[test]
    fn tiny_fit_keeps_all_8656_opaque_cells_but_real_clip_still_masks_partial_cells() {
        for origin in [[0.0, 0.0], [7.0, 9.0], [-7.3, -9.7]] {
            let i = DoraInput {
                rect: [origin[0], origin[1], 4.8, 6.4],
                ..input()
            };
            let mut frame = FrameBuilder::new();
            let (mut pixels, mut work) = (60_000, 120_000);
            let stats = append_dora(&mut frame, &i, &mut pixels, &mut work);
            assert_eq!(stats.quads, 8656);
            assert_eq!(stats.visible_cells, 8656);
            assert_eq!(
                colors(&frame).iter().filter(|&&c| c >> 24 != 0).count(),
                8656
            );
            assert_eq!(stats.scale, 0.05);
            assert_eq!((pixels, work), (60_000 - 8656, 120_000 - 8656));

            let clip = [origin[0], origin[1], 2.4, 6.4];
            let (clipped, clipped_stats) = draw(
                DoraInput {
                    clip: Some(clip),
                    ..i
                },
                CorporateAppearance {
                    main: Corporation::Pan,
                    sub: Corporation::Renko,
                },
            );
            assert_eq!(clipped_stats.quads, 8656);
            assert!(clipped_stats.visible_cells > 0 && clipped_stats.visible_cells < 8656);
            for batch in &clipped.frame().batches {
                for pixel in &batch.pixels {
                    if pixel.color >> 24 == 0 {
                        continue;
                    }
                    let transform = batch.transforms[pixel.transform as usize];
                    for corner in [
                        Vec2::splat(-0.5),
                        Vec2::new(0.5, -0.5),
                        Vec2::new(-0.5, 0.5),
                        Vec2::splat(0.5),
                    ] {
                        let p = transform.point(Vec2::from_array(pixel.local) + corner);
                        assert!(p.x >= clip[0] - 0.00001 && p.x <= clip[0] + clip[2] + 0.00001);
                        assert!(p.y >= clip[1] - 0.00001 && p.y <= clip[1] + clip[3] + 0.00001);
                    }
                }
            }
        }
        // Even at the largest admitted local coordinates, real overhang larger
        // than the absolute tolerance cannot become visible.
        for edge in [0.0, 4.8, 8192.0] {
            assert!(!inside_axis(edge - 0.001, edge, edge + 1.0));
            assert!(!inside_axis(edge + 1.001, edge, edge + 1.0));
        }
    }
    #[test]
    fn finite_clipping_consumes_all_submitted_work_including_masked_cells() {
        let (_, full) = draw(input(), CorporateAppearance::default());
        let (f, s) = draw(
            DoraInput {
                clip: Some([7.0, 9.0, 24.0, 64.0]),
                ..input()
            },
            CorporateAppearance::default(),
        );
        assert_eq!(s.quads, full.quads);
        assert!(s.visible_cells > 0 && s.visible_cells < s.quads);
        assert_eq!(
            s.visible_cells,
            colors(&f).iter().filter(|&&c| c >> 24 != 0).count()
        );
        for b in &f.frame().batches {
            for p in &b.pixels {
                if p.color >> 24 != 0 {
                    let t = b.transforms[p.transform as usize];
                    for corner in [
                        Vec2::splat(-0.5),
                        Vec2::new(0.5, -0.5),
                        Vec2::new(-0.5, 0.5),
                        Vec2::splat(0.5),
                    ] {
                        let pt = t.point(Vec2::from_array(p.local) + corner);
                        assert!(pt.x >= 7.0 && pt.x <= 31.0 && pt.y >= 9.0 && pt.y <= 73.0);
                    }
                }
            }
        }
    }
    #[test]
    fn group_preflight_rejects_without_partial_geometry_or_budget_mutation() {
        let (_, baseline) = draw(input(), CorporateAppearance::default());
        for (mut p, mut w) in [(baseline.quads - 1, 120_000), (60_000, baseline.quads - 1)] {
            let before = (p, w);
            let mut f = FrameBuilder::new();
            let s = append_dora(&mut f, &input(), &mut p, &mut w);
            assert!(s.budget_rejected);
            assert_eq!(s.requested_quads, baseline.quads);
            assert_eq!((p, w), before);
            assert_eq!(f.frame().pixel_count, 0);
        }
    }
    #[test]
    fn animation_needs_explicit_speech_and_freezes_for_reduced_or_disabled() {
        let mut seen = std::collections::HashSet::new();
        for tick in 0..30 {
            let i = DoraInput {
                reduced_motion: false,
                speaking: true,
                time: tick as f32 / 30.0,
                ..input()
            };
            let (_, s) = draw(i, CorporateAppearance::default());
            seen.insert(s.mouth_frame);
            let (_, silent) = draw(
                DoraInput {
                    speaking: false,
                    ..i
                },
                CorporateAppearance::default(),
            );
            assert_eq!(silent.mouth_frame, 0);
        }
        assert_eq!(seen.len(), 3);
        let (_, blink) = draw(
            DoraInput {
                reduced_motion: false,
                time: 4.4,
                ..input()
            },
            CorporateAppearance::default(),
        );
        assert!(blink.blink);
        for (reduced, disabled) in [(true, false), (false, true)] {
            let i = DoraInput {
                speaking: true,
                reduced_motion: reduced,
                disabled,
                ..input()
            };
            let (a, sa) = draw(i, CorporateAppearance::default());
            let (b, sb) = draw(DoraInput { time: 4.4, ..i }, CorporateAppearance::default());
            assert_eq!(colors(&a), colors(&b));
            assert_eq!(sa, sb);
        }
    }
    #[test]
    fn expressions_change_actual_eye_and_mouth_colors_with_same_bounds() {
        let mut seen = std::collections::HashSet::new();
        for expression in [
            DoraExpression::Neutral,
            DoraExpression::Focused,
            DoraExpression::Concerned,
            DoraExpression::Pleased,
        ] {
            let (f, _) = draw(
                DoraInput {
                    expression,
                    ..input()
                },
                CorporateAppearance::default(),
            );
            assert!(
                seen.insert(colors(&f)),
                "expression did not change actual cells"
            );
        }
        assert_eq!(
            DoraExpression::from_label("unknown"),
            DoraExpression::Neutral
        );
    }
    #[test]
    fn main_hair_and_sub_horns_recolor_only_explicit_disjoint_roles() {
        let (reference, _) = draw(input(), CorporateAppearance::default());
        let original = colors(&reference);
        let body = &ART["dora_body"];
        for appearance in [
            CorporateAppearance {
                main: Corporation::Renko,
                sub: Corporation::Renko,
            },
            CorporateAppearance {
                main: Corporation::Pan,
                sub: Corporation::Hugo,
            },
            CorporateAppearance {
                main: Corporation::Hugo,
                sub: Corporation::Pan,
            },
        ] {
            let (f, _) = draw(input(), appearance);
            let actual = colors(&f);
            let mut changed = 0;
            for (n, (cell, _)) in body.body.cells().enumerate() {
                if actual[n] != original[n] {
                    let k = local_index(&body.body, cell);
                    assert!(body.hair[k] || body.horns[k]);
                    changed += 1;
                    if appearance.main == Corporation::Pan {
                        assert!(body.horns[k]);
                    }
                    if appearance.sub == Corporation::Renko {
                        assert!(body.hair[k]);
                    }
                }
            }
            assert!(changed > 100);
            assert_eq!(
                actual[body.body.occupied_count()..],
                original[body.body.occupied_count()..]
            );
        }
        let mut f = FrameBuilder::new();
        let (mut p, mut w) = (60_000, 120_000);
        append_dora(&mut f, &input(), &mut p, &mut w);
        let (guide, _) = draw(
            input(),
            CorporateAppearance {
                main: Corporation::Pan,
                sub: Corporation::Renko,
            },
        );
        assert_eq!(colors(&f), colors(&guide));
        assert_eq!(colors(&f), original);
        let actual_guide = colors(&f);
        let (mut green_hair, mut red_crystals) = (0, 0);
        for (n, (cell, _)) in body.body.cells().enumerate() {
            let index = local_index(&body.body, cell);
            let [r, g, _, _] = actual_guide[n].to_le_bytes();
            green_hair += usize::from(body.hair[index] && g > r);
            red_crystals += usize::from(body.horns[index] && r > g);
        }
        assert_eq!(green_hair, body.hair.iter().filter(|&&v| v).count());
        assert_eq!(red_crystals, body.horns.iter().filter(|&&v| v).count());
    }
    #[test]
    fn nonfinite_invalid_layouts_are_closed_and_bad_time_is_static_zero() {
        for bad in [
            DoraInput {
                rect: [0.0, 0.0, f32::NAN, 64.0],
                ..input()
            },
            DoraInput {
                rect: [0.0, 0.0, -1.0, 64.0],
                ..input()
            },
            DoraInput {
                clip: Some([0.0, 0.0, f32::INFINITY, 64.0]),
                ..input()
            },
            DoraInput {
                parent: AffineTransform {
                    basis_x: Vec2::ZERO,
                    ..AffineTransform::default()
                },
                ..input()
            },
        ] {
            let (f, s) = draw(bad, CorporateAppearance::default());
            assert_eq!(s, DoraStats::default());
            assert_eq!(f.frame().pixel_count, 0);
        }
        let i = DoraInput {
            reduced_motion: false,
            speaking: true,
            ..input()
        };
        let (a, _) = draw(i, CorporateAppearance::default());
        for time in [f32::NAN, f32::INFINITY, -1.0] {
            let (b, _) = draw(DoraInput { time, ..i }, CorporateAppearance::default());
            assert_eq!(colors(&a), colors(&b));
        }
    }
    #[test]
    fn reflected_sheared_parent_and_128_slot_boundary_preserve_nested_corners() {
        let i = DoraInput {
            parent: AffineTransform {
                position: Vec2::new(70.0, -18.0),
                basis_x: Vec2::new(-1.0, 0.3),
                basis_y: Vec2::new(0.25, 1.7),
            },
            ..input()
        };
        let mut f = FrameBuilder::new();
        let cell = PixelBody::from_ascii(&["#"], &[('#', ICON_SHELL)]);
        for n in 0..129 {
            f.push_body(
                &cell,
                Transform {
                    position: Vec2::new(n as f32, 0.0),
                    ..Transform::default()
                },
            );
        }
        let before = f.frame().pixel_count;
        let (mut p, mut w) = (60_000, 120_000);
        let s = append_dora(&mut f, &i, &mut p, &mut w);
        assert_eq!(f.frame().pixel_count - before, s.quads);
        assert!(f.frame().batches.len() > 1);
        let local = Transform {
            position: Vec2::new(31.0, 41.0),
            scale: Vec2::splat(0.5),
            ..Transform::default()
        };
        for b in &f.frame().batches {
            for pixel in &b.pixels {
                assert!((pixel.transform as usize) < b.transforms.len());
            }
        }
        let last = f.frame().batches.last().unwrap();
        let gpu = last.transforms.last().unwrap();
        for point in [Vec2::new(-47.5, -63.5), Vec2::new(47.5, 63.5)] {
            assert!((gpu.point(point) - i.parent.point(local.point(point))).length() < 0.0001);
        }
    }
    #[test]
    fn ten_original_32_cell_control_masks_are_distinct_detailed_and_contained() {
        let mut seen = std::collections::HashSet::new();
        for kind in [
            "status-icon",
            "map-icon",
            "settings-icon",
            "save-icon",
            "close-icon",
            "resume-icon",
            "restart-icon",
            "title-icon",
            "quit-icon",
            "fullscreen-icon",
        ] {
            let body = icon(kind).unwrap();
            assert_eq!((body.width, body.height), (32, 32));
            assert!((100..900).contains(&body.occupied_count()));
            let cells: Vec<_> = body
                .cells()
                .map(|(p, _)| (p.x.to_bits(), p.y.to_bits()))
                .collect();
            assert!(seen.insert(cells));
            assert!(
                body.cells()
                    .all(|(p, c)| p.abs().max_element() <= 14.5 && c >> 24 == 255)
            );
            assert!(
                body.cells()
                    .map(|(_, c)| c)
                    .collect::<std::collections::HashSet<_>>()
                    .len()
                    >= 4
            );
        }
        assert!(icon("bad-kind").is_none());
    }
    #[test]
    fn icon_hover_press_and_disabled_colors_preserve_opaque_role_contrast() {
        let accent = u32::from_le_bytes([88, 222, 239, 255]);
        for role in [
            ICON_SHELL,
            ICON_EDGE,
            ICON_BEVEL,
            ICON_RECESS,
            ICON_STEEL,
            ICON_ACCENT,
        ] {
            let normal = icon_color(role, accent, false, false, false);
            let hover = icon_color(role, accent, true, false, false);
            let press = icon_color(role, accent, true, true, false);
            assert_ne!(normal, hover);
            assert_ne!(hover, press);
            assert_eq!(normal >> 24, 255);
            assert_eq!(hover >> 24, 255);
            assert_eq!(press >> 24, 255);
            assert_eq!(
                icon_color(role, accent, true, true, true),
                u32::from_le_bytes([105, 122, 138, 255])
            );
        }
    }
}
