//! Five cached, detailed overhead Haven service hubs. Cosmetic only.
use engine_core::{AffineTransform, FrameBuilder, PixelMaterial, Transform, Vec2};
use std::sync::LazyLock;
#[path = "dock_facility_art.rs"]
mod art;
use art::{Art, PARTS, SIDE};

pub(super) const CENTERS: [[f32; 2]; 5] = [
    [-120.0, -80.0],
    [120.0, -80.0],
    [-120.0, 95.0],
    [120.0, 95.0],
    [0.0, 170.0],
];
const HIT_RADIUS: f32 = 24.0;
const CELL_SCALE: f32 = 1.0 / 3.0;
const HUB_CELL_BUDGET: usize = 5_000;
const SCENE_CELL_BUDGET: usize = HUB_CELL_BUDGET * 5;
static ART: LazyLock<[Art; 5]> = LazyLock::new(|| {
    std::array::from_fn(|id| {
        let art = art::build(id);
        assert!(
            art.body.occupied_count() <= HUB_CELL_BUDGET,
            "hub{id} cells{}",
            art.body.occupied_count()
        );
        assert!(art.parts.iter().all(|p| !p.is_empty()));
        assert_eq!(
            art.parts.iter().map(|p| p.len()).sum::<usize>(),
            art.body.occupied_count()
        );
        art
    })
});
#[derive(Clone, Copy)]
struct Budget {
    cells: usize,
    transforms: usize,
}
impl Default for Budget {
    fn default() -> Self {
        Self {
            cells: SCENE_CELL_BUDGET,
            transforms: PARTS * 5,
        }
    }
}
pub(super) fn at(player: Vec2, point: Vec2, home_radius: f32) -> Option<u8> {
    if !player.is_finite()
        || !point.is_finite()
        || !home_radius.is_finite()
        || home_radius <= 0.0
        || player.length_squared() > home_radius * home_radius
    {
        return None;
    }
    CENTERS
        .iter()
        .position(|center| {
            point.distance_squared(Vec2::from_array(*center)) <= HIT_RADIUS * HIT_RADIUS
        })
        .map(|id| id as u8)
}

fn signal_color(
    id: usize,
    local: Vec2,
    color: u32,
    time: f32,
    theme: Option<crate::identity::Corporation>,
) -> u32 {
    let [r, g, b, alpha] = color.to_le_bytes();
    let phase = if id == 3 {
        local.y.atan2(local.x) * 2.0
    } else {
        local.x * 0.15 + local.y * 0.1
    };
    let gain = 0.82 + 0.18 * (time * 1.8 - phase + id as f32).sin();
    let rgb = match theme {
        None | Some(crate::identity::Corporation::Pan) => [r, g, b],
        Some(corporation) => {
            let strength = f32::from(r.max(g).max(b)) / 255.0;
            corporation
                .color()
                .map(|c| (f32::from(c) * strength).round() as u8)
        }
    };
    let rgb = rgb.map(|c| (f32::from(c) * gain).round() as u8);
    u32::from_le_bytes([rgb[0], rgb[1], rgb[2], alpha])
}
/// Atomic four-part admission; exact affine cells, never a minimum-size proxy.
fn append(
    frame: &mut FrameBuilder,
    id: usize,
    parent: AffineTransform,
    time: f32,
    theme: Option<crate::identity::Corporation>,
    budget: &mut Budget,
) -> Option<usize> {
    let asset = ART.get(id)?;
    let count = asset.body.occupied_count();
    if !time.is_finite()
        || budget.cells < count
        || budget.transforms < PARTS
        || !parent.position.is_finite()
        || !parent.basis_x.is_finite()
        || !parent.basis_y.is_finite()
        || parent.basis_x.perp_dot(parent.basis_y) == 0.0
    {
        return None;
    }
    for x in [-48.0, 48.0] {
        for y in [-48.0, 48.0] {
            let p = parent.point(Vec2::new(x, y));
            if !p.is_finite() || p.abs().max_element() > 1_100_000.0 {
                return None;
            }
        }
    }
    for (part, indices) in asset.parts.iter().enumerate() {
        let material = PixelMaterial {
            light_response: [0.7, 0.8, 0.35, 0.45][part],
            emission: [0.0, 0.0, 0.02, 0.08][part],
        };
        frame.push_cells_affine_material(
            parent,
            material,
            indices.iter().filter_map(|&index| {
                let index = usize::from(index);
                let local = Vec2::new((index % SIDE) as f32 - 47.5, (index / SIDE) as f32 - 47.5);
                let color = asset.body.color_at(index % SIDE, index / SIDE)?;
                Some((
                    local,
                    if part == 3 {
                        signal_color(id, local, color, time, theme)
                    } else {
                        color
                    },
                ))
            }),
        );
    }
    budget.cells -= count;
    budget.transforms -= PARTS;
    Some(count)
}
/// Actual unshaken extraction camera; caller supplies frozen cosmetic time.
pub(super) fn render(
    frame: &mut FrameBuilder,
    camera: Vec2,
    viewport: Vec2,
    time: f32,
    theme: Option<crate::identity::Corporation>,
) {
    if !camera.is_finite()
        || !viewport.is_finite()
        || viewport.min_element() <= 0.0
        || !time.is_finite()
    {
        return;
    }
    let mut budget = Budget::default();
    for (id, center) in CENTERS.iter().enumerate() {
        let center = Vec2::from_array(*center);
        if ((center - camera).abs() - viewport * 0.5).max_element() > HIT_RADIUS {
            continue;
        }
        let parent = Transform {
            position: center,
            scale: Vec2::splat(CELL_SCALE),
            ..Transform::default()
        }
        .affine();
        let _ = append(frame, id, parent, time, theme, &mut budget);
    }
}
#[cfg(test)]
#[path = "dock_facility_tests.rs"]
mod tests;
