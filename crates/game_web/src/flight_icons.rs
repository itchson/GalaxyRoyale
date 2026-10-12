//! Authored control cells and explicit accent masks; no bitmap resizing.
use engine_core::{PixelBody, Vec2};
use serde::Deserialize;
use std::{collections::HashMap, sync::LazyLock};

pub const SIZE: usize = 64;
#[derive(Deserialize)]
struct Sprite {
    width: usize,
    height: usize,
    indices: Vec<u8>,
    accent_cells: Vec<usize>,
    emitter_cells: Vec<usize>,
}
#[derive(Deserialize)]
struct Atlas {
    version: u32,
    palette: Vec<u32>,
    sprites: HashMap<String, Sprite>,
}
pub struct Icon {
    pub body: PixelBody,
    accent: Box<[bool]>,
    emitters: Box<[bool]>,
}
static ART: LazyLock<HashMap<String, Icon>> = LazyLock::new(|| {
    let atlas: Atlas =
        serde_json::from_str(include_str!("../../../assets/ui/flight-icons/atlas.json"))
            .expect("committed flight icon atlas");
    assert_eq!(atlas.version, 1);
    assert!((2..=64).contains(&atlas.palette.len()));
    assert_eq!(atlas.palette[0], 0);
    assert!(atlas.palette[1..].iter().all(|c| c >> 24 == 255));
    assert_eq!(atlas.sprites.len(), 10);
    assert!(
        [
            "icon_status",
            "icon_map",
            "icon_settings",
            "icon_save",
            "icon_close",
            "icon_resume",
            "icon_restart",
            "icon_title",
            "icon_quit",
            "icon_fullscreen"
        ]
        .iter()
        .all(|name| atlas.sprites.contains_key(*name))
    );
    let key = |i: usize| char::from_u32(0xe000 + i as u32).unwrap();
    let palette: Vec<_> = atlas
        .palette
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, &c)| (key(i), c))
        .collect();
    atlas
        .sprites
        .into_iter()
        .map(|(name, sprite)| {
            assert_eq!([sprite.width, sprite.height], [SIZE; 2]);
            assert_eq!(sprite.indices.len(), SIZE * SIZE);
            assert!(
                sprite
                    .indices
                    .iter()
                    .all(|&i| (i as usize) < atlas.palette.len())
            );
            let mask = |cells: &[usize]| {
                let mut result = vec![false; SIZE * SIZE].into_boxed_slice();
                for &i in cells {
                    assert!(i < result.len() && sprite.indices[i] != 0 && !result[i]);
                    result[i] = true;
                }
                result
            };
            let accent = mask(&sprite.accent_cells);
            let emitters = mask(&sprite.emitter_cells);
            assert!(emitters.iter().zip(&accent).all(|(&e, &a)| !e || a));
            let rows: Vec<String> = sprite
                .indices
                .chunks_exact(SIZE)
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
            assert!((500..=3000).contains(&body.occupied_count()));
            (
                name,
                Icon {
                    body,
                    accent,
                    emitters,
                },
            )
        })
        .collect()
});
pub fn icon(kind: &str) -> Option<&'static Icon> {
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
    ART.get(name)
}
#[derive(Clone, Copy, Default)]
pub struct State {
    pub accent: u32,
    pub hovered: bool,
    pub pressed: bool,
    pub disabled: bool,
    pub busy: bool,
    pub time: f32,
    pub reduced_motion: bool,
}
impl Icon {
    fn index(cell: Vec2) -> usize {
        let x = (cell.x + SIZE as f32 * 0.5).floor() as usize;
        let y = (cell.y + SIZE as f32 * 0.5).floor() as usize;
        y * SIZE + x
    }
    pub fn color(&self, cell: Vec2, shade: u32, state: State) -> u32 {
        let i = Self::index(cell);
        let base = shade.to_le_bytes();
        if state.disabled {
            let l = (base[0] as f32 * 0.21 + base[1] as f32 * 0.72 + base[2] as f32 * 0.07) * 0.55;
            return u32::from_le_bytes([l as u8, (l + 9.0) as u8, (l + 17.0) as u8, 255]);
        }
        // Detailed neutral materials retain their actual authored colors.
        let marked = self.accent.get(i).copied().unwrap_or(false);
        let mut channels = if marked {
            let tint = state.accent.to_le_bytes();
            let strength = base[..3].iter().copied().max().unwrap_or(0) as f32 / 239.0;
            [
                tint[0] as f32 * strength,
                tint[1] as f32 * strength,
                tint[2] as f32 * strength,
            ]
        } else {
            [base[0] as f32, base[1] as f32, base[2] as f32]
        };
        let mut strength = if state.pressed && marked {
            0.88
        } else if state.hovered && marked {
            1.10
        } else {
            1.0
        };
        if marked
            && self.emitters.get(i).copied().unwrap_or(false)
            && (state.hovered || state.busy)
            && !state.pressed
            && !state.reduced_motion
        {
            let time = if state.time.is_finite() {
                state.time
            } else {
                0.0
            };
            let phase =
                time.rem_euclid(std::f32::consts::TAU / 3.0) * 3.0 + (i % SIZE) as f32 * 0.16;
            strength *= 0.94 + 0.08 * phase.sin();
        }
        for c in &mut channels {
            *c = (*c * strength).clamp(0.0, 255.0);
        }
        u32::from_le_bytes([channels[0] as u8, channels[1] as u8, channels[2] as u8, 255])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const KINDS: [&str; 10] = [
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
    ];
    #[test]
    fn material_masks_preserve_unmarked_authored_colors_and_only_emitter_cells_animate() {
        for kind in KINDS {
            let icon = icon(kind).unwrap();
            let state = State {
                accent: u32::from_le_bytes([106, 237, 193, 255]),
                hovered: true,
                time: 0.25,
                ..Default::default()
            };
            let later = State { time: 1.5, ..state };
            let mut animated = 0;
            let mut immutable = 0;
            for (index, cell, shade) in icon.body.cells_indexed() {
                assert_eq!(
                    Icon::index(cell),
                    index,
                    "mask coordinates must match row-major source cells"
                );
                let first = icon.color(cell, shade, state);
                let next = icon.color(cell, shade, later);
                assert_eq!(first >> 24, 255);
                if !icon.accent[index] {
                    assert_eq!(first, shade);
                    assert_eq!(next, shade);
                    assert_eq!(
                        icon.color(
                            cell,
                            shade,
                            State {
                                pressed: true,
                                ..state
                            }
                        ),
                        shade
                    );
                    immutable += 1;
                }
                if !icon.emitters[index] {
                    assert_eq!(first, next);
                } else if first != next {
                    animated += 1;
                }
                assert_eq!(
                    icon.color(
                        cell,
                        shade,
                        State {
                            reduced_motion: true,
                            ..state
                        }
                    ),
                    icon.color(
                        cell,
                        shade,
                        State {
                            reduced_motion: true,
                            ..later
                        }
                    )
                );
            }
            assert!(
                immutable > 500,
                "{kind}: neutral detailed material must remain authored"
            );
            assert!(animated > 0, "{kind}: relevant signal must animate");
        }
    }
    #[test]
    fn idle_save_and_disabled_materials_are_static_and_nonfinite_time_is_bounded() {
        let icon = icon("save-icon").unwrap();
        let normal = State {
            accent: u32::from_le_bytes([106, 237, 193, 255]),
            ..Default::default()
        };
        for (cell, shade) in icon.body.cells() {
            assert_eq!(
                icon.color(cell, shade, normal),
                icon.color(
                    cell,
                    shade,
                    State {
                        time: 9999.0,
                        ..normal
                    }
                )
            );
            let disabled = State {
                disabled: true,
                hovered: true,
                busy: true,
                ..normal
            };
            assert_eq!(
                icon.color(cell, shade, disabled),
                icon.color(
                    cell,
                    shade,
                    State {
                        time: 123.0,
                        ..disabled
                    }
                )
            );
            let busy = State {
                busy: true,
                ..normal
            };
            for time in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
                assert_eq!(
                    icon.color(cell, shade, busy),
                    icon.color(cell, shade, State { time, ..busy })
                );
            }
            assert_eq!(
                icon.color(
                    cell,
                    shade,
                    State {
                        time: f32::MAX,
                        ..busy
                    }
                ) >> 24,
                255
            );
        }
        assert!(super::icon("unrecognized").is_none());
    }
}
