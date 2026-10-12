//! Native sports league mark: cached cells, relative award face and arena light.
//! DOM owns the8:3 measured rectangle; no bitmap is submitted to the renderer.
use engine_core::{AffineTransform, FrameBuilder, PixelBody, Transform, Vec2};
use serde::Deserialize;
use std::{collections::HashMap, sync::LazyLock};

pub const SOURCE_SIZE: [f32; 2] = [512.0, 192.0];
pub const OCCUPIED_CELLS: usize = 44_985;
const NAMES: [&str; 9] = [
    "tile-0-0", "tile-0-1", "tile-0-2", "tile-0-3", "tile-1-0", "tile-1-1", "tile-1-2", "tile-1-3",
    "crest",
];

#[derive(Deserialize)]
struct Sprite {
    width: usize,
    height: usize,
    indices: Vec<usize>,
}
#[derive(Deserialize)]
struct Roles {
    energy_cells: Vec<usize>,
    scan_cells: Vec<usize>,
    award_cells: Vec<usize>,
}
#[derive(Deserialize)]
struct TitleRoles {
    source_size: [usize; 2],
    parts: HashMap<String, Roles>,
}
#[derive(Deserialize)]
struct Pack {
    version: usize,
    palette: Vec<u32>,
    sprites: HashMap<String, Sprite>,
    title_roles: TitleRoles,
}
struct Part {
    body: PixelBody,
    roles: Box<[u8]>,
    offset: Vec2,
}
struct Art {
    parts: [Part; 9],
}
fn load(source: &str) -> Art {
    let mut pack: Pack = serde_json::from_str(source).expect("compiled title pack JSON");
    assert_eq!(pack.version, 1);
    assert_eq!(pack.title_roles.source_size, [512, 192]);
    assert!((2..=64).contains(&pack.palette.len()) && pack.palette[0] == 0);
    assert!(pack.palette[1..].iter().all(|c| c >> 24 == 255));
    assert_eq!(pack.sprites.len(), NAMES.len());
    assert_eq!(pack.title_roles.parts.len(), NAMES.len());
    let key = |i: usize| char::from_u32(0xe000 + i as u32).unwrap();
    let palette: Vec<_> = pack
        .palette
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, &c)| (key(i), c))
        .collect();
    let parts = std::array::from_fn(|n| {
        let sprite = pack
            .sprites
            .remove(NAMES[n])
            .expect("expected title subobject");
        let expected = if n == 8 { [48, 64] } else { [128, 96] };
        assert_eq!([sprite.width, sprite.height], expected);
        assert_eq!(sprite.indices.len(), sprite.width * sprite.height);
        assert!(sprite.indices.iter().all(|&i| i < pack.palette.len()));
        let masks = pack
            .title_roles
            .parts
            .remove(NAMES[n])
            .expect("expected title roles");
        let mut roles = vec![0; sprite.indices.len()];
        for (role, indices) in [
            (1, masks.energy_cells),
            (2, masks.scan_cells),
            (3, masks.award_cells),
        ] {
            for i in indices {
                assert!(i < roles.len() && sprite.indices[i] != 0 && roles[i] == 0);
                roles[i] = role;
            }
        }
        let rows: Vec<String> = sprite
            .indices
            .chunks_exact(sprite.width)
            .map(|row| {
                row.iter()
                    .map(|&i| if i == 0 { ' ' } else { key(i) })
                    .collect()
            })
            .collect();
        let body = PixelBody::from_ascii(
            &rows.iter().map(String::as_str).collect::<Vec<_>>(),
            &palette,
        );
        let offset = if n == 8 {
            Vec2::new(0.0, -64.0)
        } else {
            Vec2::new((n % 4) as f32 * 128.0 - 192.0, (n / 4) as f32 * 96.0 - 48.0)
        };
        Part {
            body,
            roles: roles.into_boxed_slice(),
            offset,
        }
    });
    assert_eq!(
        parts.iter().map(|p| p.body.occupied_count()).sum::<usize>(),
        OCCUPIED_CELLS
    );
    Art { parts }
}
static ART: LazyLock<Art> =
    LazyLock::new(|| load(include_str!("../../../assets/ui/title-mark/atlas.json")));

pub struct Input {
    pub parent: AffineTransform,
    pub rect: [f32; 4],
    pub clip: Option<[f32; 4]>,
    pub time: f32,
    pub reduced: bool,
    pub disabled: bool,
}

fn valid_rect(r: [f32; 4]) -> bool {
    r.iter().all(|v| v.is_finite() && v.abs() <= 8192.0) && r[2] > 0.0 && r[3] > 0.0
}
fn tint(color: u32, strength: f32, disabled: bool) -> u32 {
    let [r, g, b, a] = color.to_le_bytes();
    let strength = strength * if disabled { 0.48 } else { 1.0 };
    u32::from_le_bytes([
        (r as f32 * strength).min(255.0) as u8,
        (g as f32 * strength).min(255.0) as u8,
        (b as f32 * strength).min(255.0) as u8,
        a,
    ])
}

/// Preflight complete pixels/work and every composed corner before any append.
/// All source cells, including those clipped to alpha0, charge both UI budgets.
pub fn append(
    frame: &mut FrameBuilder,
    input: &Input,
    pixels: &mut usize,
    work: &mut usize,
) -> usize {
    let r = input.rect;
    if !valid_rect(r)
        || input.clip.is_some_and(|c| !valid_rect(c))
        || !input.parent.position.is_finite()
        || input.parent.position.abs().max_element() > 1_100_000.0
        || !input.parent.basis_x.is_finite()
        || !input.parent.basis_y.is_finite()
        || input.parent.basis_x.abs().max_element() > 64.0
        || input.parent.basis_y.abs().max_element() > 64.0
        || OCCUPIED_CELLS > *pixels
        || OCCUPIED_CELLS > *work
    {
        return 0;
    }
    let scale = (r[2] / SOURCE_SIZE[0]).min(r[3] / SOURCE_SIZE[1]);
    if !scale.is_finite() || scale <= 0.0 {
        return 0;
    }
    let center = Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5);
    let time = if input.reduced || !input.time.is_finite() {
        0.0
    } else {
        input.time.rem_euclid(120.0)
    };
    let phase = time * std::f32::consts::TAU / 120.0;
    let local = |offset: Vec2| {
        Transform {
            position: center + offset * scale,
            scale: Vec2::splat(scale),
            ..Transform::default()
        }
        .affine()
    };
    let transforms: [AffineTransform; 9] = std::array::from_fn(|n| {
        let offset = ART.parts[n].offset
            + if n == 8 {
                Vec2::new(0.0, (phase * 15.0).sin() * 0.35)
            } else {
                Vec2::ZERO
            };
        local(offset)
    });
    // A finite parent can overflow after composition. Fail without mutating frame/quotas.
    for (n, t) in transforms.iter().enumerate() {
        let body = &ART.parts[n].body;
        let composed = input.parent.compose(*t);
        for x in [-0.5, 0.5] {
            for y in [-0.5, 0.5] {
                let corner =
                    composed.point(Vec2::new(x * body.width as f32, y * body.height as f32));
                if !corner.is_finite() || corner.abs().max_element() > 1_100_000.0 {
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
    let scan = time.rem_euclid(8.0) / 8.0 * 720.0 - 360.0;
    for (n, t) in transforms.iter().enumerate() {
        let body = &ART.parts[n].body;
        frame.push_cells_affine_material(
            input.parent.compose(*t),
            Default::default(),
            body.cells_indexed().map(|(i, cell, color)| {
                let outside = !full
                    && [
                        Vec2::splat(-0.5),
                        Vec2::new(0.5, -0.5),
                        Vec2::new(-0.5, 0.5),
                        Vec2::splat(0.5),
                    ]
                    .into_iter()
                    .map(|c| t.point(cell + c))
                    .any(|p| {
                        p.x < clip[0]
                            || p.y < clip[1]
                            || p.x > clip[0] + clip[2]
                            || p.y > clip[1] + clip[3]
                    });
                let strength = match ART.parts[n].roles[i] {
                    1 => {
                        1.0 + (phase * 20.0
                            + (cell.x + ART.parts[n].offset.x) * 0.025
                            + (cell.y + ART.parts[n].offset.y) * 0.04)
                            .sin()
                            * 0.07
                    }
                    2 if !input.reduced && (cell.x + ART.parts[n].offset.x - scan).abs() < 6.0 => {
                        1.12
                    }
                    3 => 1.0 + (phase * 15.0 + cell.x * 0.12).sin() * 0.06,
                    _ => 1.0,
                };
                (
                    cell,
                    if outside {
                        0
                    } else if input.reduced && !input.disabled {
                        color
                    } else {
                        tint(color, strength, input.disabled)
                    },
                )
            }),
        );
    }
    *pixels -= OCCUPIED_CELLS;
    *work -= OCCUPIED_CELLS;
    OCCUPIED_CELLS
}

#[cfg(test)]
#[path = "title_art_tests.rs"]
mod tests;
