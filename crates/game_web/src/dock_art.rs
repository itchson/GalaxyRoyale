//! Cached native home artwork. The core retains all docking/safety semantics.
use engine_core::{
    AffineTransform, FrameBuilder, PixelBody, PixelRig, PixelRigPart, PixelRigState, Transform,
    Vec2,
};
use game_core::identity::Corporation;
use serde::Deserialize;
use std::{collections::HashMap, sync::LazyLock};

const SIDE: usize = 128;
const TILES: usize = 4;
const STATES: usize = 3;
pub const CELL_COST: usize = 28_549;
pub const TRANSFORM_COST: usize = 16;
const SOURCE_SCALE: f32 = 0.5;

#[derive(Deserialize)]
struct Sprite {
    width: usize,
    height: usize,
    indices: Vec<u8>,
}
#[derive(Deserialize)]
struct Tile {
    width: usize,
    height: usize,
    position: [f32; 2],
    roles: Vec<u8>,
    parts: Vec<PixelRigPart>,
    frame_sprites: Vec<String>,
}
#[derive(Deserialize)]
struct Tiles {
    source_size: [usize; 2],
    reference_footprint: [usize; 2],
    tiles: Vec<Tile>,
    lamp_cells_source: Vec<usize>,
}
#[derive(Deserialize)]
struct Pack {
    version: u32,
    palette: Vec<u32>,
    sprites: HashMap<String, Sprite>,
    pixel_rig_tiles: Tiles,
}
struct TileArt {
    position: Vec2,
    rig: PixelRig,
    // Three authored poses, three signal hues. No frame-time clone/recolor.
    bodies: [[PixelBody; STATES]; 3],
}
struct Art {
    tiles: [TileArt; TILES],
}

fn signal_color(color: u32, corporation: usize) -> u32 {
    if corporation == 0 {
        return color;
    } // PAN retains the authored palette.
    let [r, g, b, alpha] = color.to_le_bytes();
    let maximum = r.max(g).max(b) as f32 / 255.0;
    let accent = [Corporation::Pan, Corporation::Renko, Corporation::Hugo][corporation].color();
    u32::from_le_bytes([
        (accent[0] as f32 * maximum).round() as u8,
        (accent[1] as f32 * maximum).round() as u8,
        (accent[2] as f32 * maximum).round() as u8,
        alpha,
    ])
}

fn expand(sprite: &Sprite, palette: &[u32], roles: &[u8], corporation: usize) -> PixelBody {
    assert_eq!((sprite.width, sprite.height), (SIDE, SIDE));
    assert_eq!(sprite.indices.len(), SIDE * SIDE);
    assert!(
        sprite
            .indices
            .iter()
            .all(|&index| (index as usize) < palette.len())
    );
    // A separate character range distinguishes the same palette shade when
    // assigned to a signal instead of neutral alloy or cyan instrumentation.
    let key = |index: usize, signal: bool| {
        char::from_u32(0xe000 + index as u32 + u32::from(signal) * 64).unwrap()
    };
    let mut colors = Vec::with_capacity((palette.len() - 1) * 2);
    for (index, &color) in palette.iter().enumerate().skip(1) {
        colors.push((key(index, false), color));
        colors.push((key(index, true), signal_color(color, corporation)));
    }
    let rows: Vec<String> = sprite
        .indices
        .chunks_exact(SIDE)
        .enumerate()
        .map(|(y, row)| {
            row.iter()
                .enumerate()
                .map(|(x, &index)| {
                    if index == 0 {
                        ' '
                    } else {
                        key(index as usize, roles[y * SIDE + x] == 3)
                    }
                })
                .collect()
        })
        .collect();
    PixelBody::from_ascii(
        &rows.iter().map(String::as_str).collect::<Vec<_>>(),
        &colors,
    )
}

static ART: LazyLock<Art> = LazyLock::new(|| {
    let pack: Pack = serde_json::from_str(include_str!(
        "../../../assets/pipeline/pan-coop-dock/native256/pack.json"
    ))
    .expect("compiled dock pack");
    assert_eq!(pack.version, 1);
    assert_eq!(pack.palette.len(), 64);
    assert_eq!(pack.palette[0], 0);
    assert!(pack.palette[1..].iter().all(|color| color >> 24 == 255));
    let source = &pack.pixel_rig_tiles;
    assert_eq!(source.source_size, [256, 256]);
    assert_eq!(source.reference_footprint, [128, 128]);
    assert_eq!(source.tiles.len(), TILES);
    let mut lamps = vec![false; 256 * 256];
    for &index in &source.lamp_cells_source {
        assert!(index < lamps.len() && !lamps[index]);
        lamps[index] = true;
    }
    assert_eq!(source.lamp_cells_source.len(), 1528);
    let tiles = std::array::from_fn(|index| {
        let tile = &source.tiles[index];
        assert_eq!((tile.width, tile.height), (SIDE, SIDE));
        assert_eq!(tile.frame_sprites.len(), STATES);
        assert_eq!(tile.roles.len(), SIDE * SIDE);
        assert_eq!(tile.parts.len(), 4);
        assert_eq!(
            tile.position,
            [[-64.0, -64.0], [64.0, -64.0], [-64.0, 64.0], [64.0, 64.0]][index]
        );
        let initial = &pack.sprites[&tile.frame_sprites[0]];
        for state in 1..STATES {
            let next = &pack.sprites[&tile.frame_sprites[state]];
            for (cell, (&a, &b)) in initial.indices.iter().zip(&next.indices).enumerate() {
                assert_eq!(a != 0, b != 0, "animation cannot change occupancy");
                let global =
                    (cell / SIDE + (index / 2) * SIDE) * 256 + cell % SIDE + (index % 2) * SIDE;
                if a != b {
                    assert!(
                        lamps[global] && tile.roles[cell] == 3,
                        "only authored exterior lamps animate"
                    );
                }
            }
        }
        let bodies = std::array::from_fn(|corporation| {
            std::array::from_fn(|state| {
                expand(
                    &pack.sprites[&tile.frame_sprites[state]],
                    &pack.palette,
                    &tile.roles,
                    corporation,
                )
            })
        });
        TileArt {
            position: Vec2::from_array(tile.position),
            rig: PixelRig::new(
                tile.width,
                tile.height,
                tile.roles.clone(),
                tile.parts.clone(),
            )
            .expect("authored dock rig"),
            bodies,
        }
    });
    for corporation in 0..3 {
        for state in 0..STATES {
            assert_eq!(
                tiles
                    .iter()
                    .map(|t: &TileArt| t.bodies[corporation][state].occupied_count())
                    .sum::<usize>(),
                CELL_COST
            );
        }
    }
    Art { tiles }
});

#[derive(Debug, Clone, Copy)]
pub struct DockInput {
    /// Existing core home transform. Source cell size remains fractional.
    pub parent: Transform,
    pub corporation: Option<Corporation>,
    pub time: f32,
    pub reduced_motion: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DockBudget {
    pub cells: usize,
    pub transforms: usize,
}
impl Default for DockBudget {
    fn default() -> Self {
        Self {
            cells: 32_000,
            transforms: TRANSFORM_COST,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DockStats {
    pub cells: usize,
    pub transforms: usize,
    pub state: usize,
}

fn composed(parent: AffineTransform, tile: &TileArt) -> AffineTransform {
    parent
        .compose(
            Transform {
                scale: Vec2::splat(SOURCE_SCALE),
                ..Default::default()
            }
            .affine(),
        )
        .compose(
            Transform {
                position: tile.position,
                ..Default::default()
            }
            .affine(),
        )
}

/// Atomic local artwork admission; FrameBuilder has no global world quota.
/// False/None is safe for the core's old-home fallback because nothing appended.
pub fn append(
    frame: &mut FrameBuilder,
    input: &DockInput,
    budget: &mut DockBudget,
) -> Option<DockStats> {
    if !input.parent.position.is_finite()
        || !input.parent.scale.is_finite()
        || !input.parent.rotation.is_finite()
        || input.parent.scale.abs().min_element() <= 0.0
        || input.parent.scale.abs().max_element() > 64.0
        || budget.cells < CELL_COST
        || budget.transforms < TRANSFORM_COST
    {
        return None;
    }
    let parent = input.parent.affine();
    let time = if input.reduced_motion || !input.time.is_finite() {
        0.0
    } else {
        input.time.max(0.0)
    };
    let state = ((f64::from(time) / f64::from(0.7_f32))
        .floor()
        .rem_euclid(STATES as f64)) as usize;
    let corporation = match input.corporation {
        None | Some(Corporation::Pan) => 0,
        Some(Corporation::Renko) => 1,
        Some(Corporation::Hugo) => 2,
    };
    // Bounded tile rectangles cover all cell corners. Validate before any append.
    for tile in &ART.tiles {
        let transform = composed(parent, tile);
        for x in [-64.0, 64.0] {
            for y in [-64.0, 64.0] {
                let corner = transform.point(Vec2::new(x, y));
                if !corner.is_finite() || corner.abs().max_element() > 1_100_000.0 {
                    return None;
                }
            }
        }
    }
    let mut cells = 0;
    for tile in &ART.tiles {
        cells += tile.rig.draw_affine(
            frame,
            &tile.bodies[corporation][state],
            composed(parent, tile),
            PixelRigState {
                time,
                charge: 0.2,
                ..Default::default()
            },
        );
    }
    debug_assert_eq!(cells, CELL_COST);
    budget.cells -= CELL_COST;
    budget.transforms -= TRANSFORM_COST;
    Some(DockStats {
        cells,
        transforms: TRANSFORM_COST,
        state,
    })
}

#[cfg(test)]
#[path = "dock_art_tests.rs"]
mod tests;
