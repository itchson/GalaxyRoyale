//! Cached authored 96-cell detail over the retained, destructible 48-cell hull.
//! Presentation only: no collider, exhaust, progression or particle changes.
use crate::identity::{Corporation, starter_body};
use engine_core::{AffineTransform, FrameBuilder, PixelBody, PixelMaterial, Vec2};
use serde::Deserialize;
use std::{collections::BTreeMap, sync::LazyLock};

const GRID: usize = 96;
const COARSE_GRID: usize = 48;
const AREA: usize = GRID * GRID;
const NAMES: [&str; 3] = ["pan_hikari", "renko_hibana", "hugo_mamori"];
const ROLES: [&str; 7] = [
    "metal",
    "recess",
    "glass",
    "main_accent",
    "reactor",
    "exhaust",
    "fastener",
];
const ATLAS: &str = include_str!("../../../assets/pixels/raicho-fine-detail/atlas.json");

/// An object quota, not a global FrameBuilder/render-scene ceiling.
const PLAYER_CELL_BUDGET: usize = 3_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Budget {
    pub pixels: usize,
    pub transforms: usize,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            pixels: PLAYER_CELL_BUDGET,
            transforms: 2,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct DrawInput {
    /// Complete actor affine, including its existing world scale and interpolation.
    pub parent: AffineTransform,
    pub material: PixelMaterial,
    pub time: f32,
    pub reduced_motion: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Reject {
    InvalidAtlas,
    IncompatibleBody,
    InvalidTransform,
    Budget,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Stats {
    pub body_cells: usize,
    pub overlay_cells: usize,
    pub transforms: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Atlas {
    version: u32,
    palette: Vec<u32>,
    #[serde(deserialize_with = "unique_map")]
    sprites: BTreeMap<String, Sprite>,
    // Authoring/provenance metadata is not an animation or collision runtime API.
    asset_metadata: serde::de::IgnoredAny,
    raicho_detail: Detail,
}

#[derive(Deserialize)]
struct Detail {
    source_size: [usize; 2],
    relative_cell_scale: f32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Sprite {
    width: usize,
    height: usize,
    indices: Vec<u8>,
    #[serde(deserialize_with = "unique_map")]
    material_cells: BTreeMap<String, Vec<usize>>,
    fine_cells: Vec<usize>,
    signal_cells: Vec<usize>,
}

// Map deserialization must not silently replace a repeated sprite/material key.
fn unique_map<'de, D, T>(deserializer: D) -> Result<BTreeMap<String, T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct Unique<T>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for Unique<T> {
        type Value = BTreeMap<String, T>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a map with unique sprite or material keys")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut map: M,
        ) -> Result<Self::Value, M::Error> {
            let mut values = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, T>()? {
                if values.insert(key, value).is_some() {
                    return Err(serde::de::Error::custom("duplicate sprite or material key"));
                }
            }
            Ok(values)
        }
    }
    deserializer.deserialize_map(Unique(std::marker::PhantomData))
}

struct FineCell {
    local: Vec2,
    coarse_x: usize,
    coarse_y: usize,
    color: u32,
    signal: bool,
}

struct FineArt {
    cells: Box<[FineCell]>,
}

static ASSETS: LazyLock<Result<[FineArt; 3], &'static str>> = LazyLock::new(|| parse(ATLAS));

fn corporation(index: usize) -> Corporation {
    [Corporation::Pan, Corporation::Renko, Corporation::Hugo][index]
}

fn art_index(corporation: Corporation) -> usize {
    match corporation {
        Corporation::Pan => 0,
        Corporation::Renko => 1,
        Corporation::Hugo => 2,
    }
}

fn occupied(sprite: &Sprite) -> impl Iterator<Item = usize> + '_ {
    sprite
        .indices
        .iter()
        .enumerate()
        .filter_map(|(i, &color)| (color != 0).then_some(i))
}

fn ordered_mask(mask: &[usize], sprite: &Sprite) -> bool {
    mask.windows(2).all(|p| p[0] < p[1])
        && mask
            .iter()
            .all(|&i| i < AREA && sprite.indices.get(i).is_some_and(|&p| p != 0))
}

fn validate_sprite(
    sprite: &Sprite,
    palette: &[u32],
    reference: &PixelBody,
) -> Result<Vec<u8>, &'static str> {
    if sprite.width != GRID || sprite.height != GRID || sprite.indices.len() != AREA {
        return Err("96x96 exact indexed grid required");
    }
    if sprite.indices.iter().any(|&i| i as usize >= palette.len()) {
        return Err("palette index outside provided palette");
    }
    let count = occupied(sprite).count();
    if count == 0 || count > 4_000 {
        return Err("source surface occupancy outside declared limit");
    }
    if !ordered_mask(&sprite.fine_cells, sprite) || !ordered_mask(&sprite.signal_cells, sprite) {
        return Err("role mask must be sorted, unique and occupied");
    }
    if sprite.material_cells.len() != ROLES.len()
        || ROLES
            .iter()
            .any(|name| !sprite.material_cells.contains_key(*name))
    {
        return Err("seven exact material roles required");
    }
    let mut roles = vec![u8::MAX; AREA];
    for (role, name) in ROLES.iter().enumerate() {
        let mask = &sprite.material_cells[*name];
        if mask.is_empty() || !ordered_mask(mask, sprite) {
            return Err("material mask must be nonempty, sorted and occupied");
        }
        for &i in mask {
            if roles[i] != u8::MAX {
                return Err("material roles overlap");
            }
            roles[i] = role as u8;
        }
    }
    for i in occupied(sprite) {
        if roles[i] == u8::MAX || !reference.is_occupied((i % GRID) / 2, (i / GRID) / 2) {
            return Err("unpartitioned cell or detail outside retained hull");
        }
    }
    if sprite
        .signal_cells
        .iter()
        .any(|&i| !matches!(roles[i], 3..=5))
    {
        return Err("signals must belong to accent, reactor or exhaust");
    }
    Ok(roles)
}

fn parse(json: &str) -> Result<[FineArt; 3], &'static str> {
    let atlas: Atlas = serde_json::from_str(json).map_err(|_| "invalid exact atlas JSON")?;
    let _authoring_metadata = atlas.asset_metadata;
    if atlas.version != 1
        || atlas.raicho_detail.source_size != [GRID, GRID]
        || atlas.raicho_detail.relative_cell_scale != 0.5
        || !(2..=64).contains(&atlas.palette.len())
        || atlas.palette[0] != 0
        || atlas.palette[1..].iter().any(|c| c >> 24 != 255)
        || atlas
            .palette
            .iter()
            .enumerate()
            .any(|(i, c)| atlas.palette[..i].contains(c))
    {
        return Err("invalid version, scale or hard-alpha palette");
    }
    if atlas.sprites.len() != 6
        || NAMES.iter().any(|name| {
            !atlas.sprites.contains_key(*name)
                || !atlas.sprites.contains_key(&format!("{name}_fine"))
        })
    {
        return Err("three exact surface/overlay pairs required");
    }
    let mut arts = Vec::with_capacity(3);
    for (index, name) in NAMES.iter().enumerate() {
        let reference = starter_body(corporation(index));
        if reference.width != COARSE_GRID || reference.height != COARSE_GRID {
            return Err("retained hull grid changed");
        }
        let full = &atlas.sprites[*name];
        let fine = &atlas.sprites[&format!("{name}_fine")];
        let full_roles = validate_sprite(full, &atlas.palette, reference)?;
        let fine_roles = validate_sprite(fine, &atlas.palette, reference)?;
        let fine_positions: Vec<_> = occupied(fine).collect();
        if fine_positions.len() > 1_800
            || fine_positions != full.fine_cells
            || fine_positions != fine.fine_cells
            || full.signal_cells != fine.signal_cells
            || full.signal_cells.is_empty()
            || full
                .signal_cells
                .iter()
                .any(|i| fine_positions.binary_search(i).is_err())
        {
            return Err("sparse overlay/explicit signal contract changed");
        }
        if fine_positions
            .iter()
            .any(|&i| full.indices[i] != fine.indices[i] || full_roles[i] != fine_roles[i])
        {
            return Err("overlay colors/materials differ from authored surface");
        }
        if reference.cells_indexed().any(|(i, _, _)| {
            let x = i % COARSE_GRID;
            let y = i / COARSE_GRID;
            (0..2).all(|dy| (0..2).all(|dx| full.indices[(y * 2 + dy) * GRID + x * 2 + dx] == 0))
        }) {
            return Err("fine source drops a complete retained collision cell");
        }
        arts.push(FineArt {
            cells: fine_positions
                .into_iter()
                .map(|i| FineCell {
                    local: Vec2::new(
                        (i % GRID) as f32 + 0.5 - 48.0,
                        (i / GRID) as f32 + 0.5 - 48.0,
                    ),
                    coarse_x: (i % GRID) / 2,
                    coarse_y: (i / GRID) / 2,
                    color: atlas.palette[fine.indices[i] as usize],
                    signal: fine.signal_cells.binary_search(&i).is_ok(),
                })
                .collect(),
        });
    }
    arts.try_into()
        .map_err(|_| "three cached overlays required")
}

fn finite_cell(parent: AffineTransform, center: Vec2, half: f32) -> bool {
    [
        Vec2::new(-half, -half),
        Vec2::new(half, -half),
        Vec2::new(-half, half),
        Vec2::new(half, half),
    ]
    .into_iter()
    .all(|corner| parent.point(center + corner).is_finite())
}

fn signal_color(cell: &FineCell, time: f32, reduced_motion: bool) -> u32 {
    if !cell.signal || reduced_motion {
        return cell.color;
    }
    // RGB-only local lamp breathing. Fixed geometry/alpha; no new light emitter.
    let phase = time.rem_euclid(4.0) * std::f32::consts::TAU / 4.0
        + cell.local.x * 0.11
        + cell.local.y * 0.07;
    let factor = 1.0 + phase.sin() * 0.08;
    let [r, g, b, a] = cell.color.to_le_bytes();
    u32::from_le_bytes([
        (r as f32 * factor).round().clamp(0.0, 255.0) as u8,
        (g as f32 * factor).round().clamp(0.0, 255.0) as u8,
        (b as f32 * factor).round().clamp(0.0, 255.0) as u8,
        a,
    ])
}

/// Preflight the complete retained-body + sparse-detail pair before either append.
/// On rejection the caller must draw its existing gameplay body as the baseline.
/// Fine color callbacks receive the aligned 48-grid local center, after signal
/// modulation. Use the same real damage tint as the base, not broad hue remapping.
pub(super) fn append(
    frame: &mut FrameBuilder,
    live: &PixelBody,
    manufacturer: Corporation,
    input: DrawInput,
    remaining: &mut Budget,
    coarse_color: impl FnMut(Vec2, u32) -> u32,
    mut fine_color: impl FnMut(Vec2, u32) -> u32,
) -> Result<Stats, Reject> {
    let assets = ASSETS.as_ref().map_err(|_| Reject::InvalidAtlas)?;
    let index = art_index(manufacturer);
    let reference = starter_body(manufacturer);
    if live.width != COARSE_GRID
        || live.height != COARSE_GRID
        || live
            .cells_indexed()
            .any(|(i, _, _)| !reference.is_occupied(i % COARSE_GRID, i / COARSE_GRID))
    {
        return Err(Reject::IncompatibleBody);
    }
    let parent = input.parent;
    let determinant = parent.basis_x.perp_dot(parent.basis_y);
    if !parent.position.is_finite()
        || !parent.basis_x.is_finite()
        || !parent.basis_y.is_finite()
        || !determinant.is_finite()
        || determinant == 0.0
        || !input.time.is_finite()
        || live.cells().any(|(p, _)| !finite_cell(parent, p, 0.5))
    {
        return Err(Reject::InvalidTransform);
    }
    let art = &assets[index];
    let survives = |cell: &&FineCell| live.is_occupied(cell.coarse_x, cell.coarse_y);
    let fine_parent = parent.compose(AffineTransform {
        basis_x: Vec2::X * 0.5,
        basis_y: Vec2::Y * 0.5,
        ..AffineTransform::default()
    });
    let mut count = 0;
    for cell in art.cells.iter().filter(survives) {
        if !finite_cell(fine_parent, cell.local, 0.5) {
            return Err(Reject::InvalidTransform);
        }
        count += 1;
    }
    let body_cells = live.occupied_count();
    let needed = body_cells.checked_add(count).ok_or(Reject::Budget)?;
    let transforms = usize::from(body_cells > 0) + usize::from(count > 0);
    if needed > remaining.pixels || needed > PLAYER_CELL_BUDGET || transforms > remaining.transforms
    {
        return Err(Reject::Budget);
    }
    // No callbacks or writes happened before all guards, including both layers.
    frame.push_body_affine_colored_material(live, parent, input.material, coarse_color);
    let emitted = frame.push_cells_affine_material(
        fine_parent,
        input.material,
        art.cells.iter().filter(survives).map(|cell| {
            (
                cell.local,
                fine_color(
                    cell.local * 0.5,
                    signal_color(cell, input.time, input.reduced_motion),
                ),
            )
        }),
    );
    debug_assert_eq!(emitted, count);
    remaining.pixels -= needed;
    remaining.transforms -= transforms;
    Ok(Stats {
        body_cells,
        overlay_cells: emitted,
        transforms,
    })
}

#[cfg(test)]
#[path = "raicho_fine_tests.rs"]
mod tests;
