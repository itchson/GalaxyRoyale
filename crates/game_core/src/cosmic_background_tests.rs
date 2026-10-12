//! Actual star extraction and biome-query guards, with no GPU/FPS claim.
use super::*;

fn scene(reduced: bool) -> SceneInput {
    SceneInput {
        seed: 17,
        camera: Vec2::ZERO,
        viewport: Vec2::new(533.0, 300.0),
        time: 18.0,
        reduced,
        reduced_motion: false,
    }
}

#[test]
fn empty_star_tile_inside_render_scan_skips_all_biome_samples() {
    for reduced in [false, true] {
        let input = SceneInput {
            viewport: Vec2::splat(2.0),
            ..scene(reduced)
        };
        // This tile is genuinely visited by the actual layer's scan bounds,
        // although its 32 seeded points miss this small accepted viewport.
        let min = ((input.camera * 0.09 - input.viewport * 0.5 - Vec2::splat(0.7)) / STAR_TILE)
            .floor()
            .as_ivec2();
        let max = ((input.camera * 0.09 + input.viewport * 0.5 + Vec2::splat(0.7)) / STAR_TILE)
            .floor()
            .as_ivec2();
        assert!(min.x <= 0 && max.x >= 0 && min.y <= 0 && max.y >= 0);
        ATMOSPHERE_SAMPLES.with(|count| count.set(0));
        let cells = star_cells(input.seed, [0, 0], 0, 0.09, 0.7, &input);
        assert!(cells.into_iter().all(|cell| cell.is_none()));
        assert_eq!(ATMOSPHERE_SAMPLES.with(std::cell::Cell::get), 0);
    }
}

#[test]
fn visible_star_tile_samples_one_field_and_preserves_reduced_subset() {
    let full = SceneInput {
        viewport: Vec2::splat(800.0),
        ..scene(false)
    };
    ATMOSPHERE_SAMPLES.with(|count| count.set(0));
    let full_cells = star_cells(full.seed, [0, 0], 0, 0.09, 0.7, &full);
    let full_count = full_cells.iter().flatten().count();
    assert!(
        full_count > 1,
        "fixture exercises shared rather than per-star sampling"
    );
    assert_eq!(ATMOSPHERE_SAMPLES.with(std::cell::Cell::get), 4);
    let low = SceneInput {
        reduced: true,
        ..full
    };
    ATMOSPHERE_SAMPLES.with(|count| count.set(0));
    let low_cells = star_cells(low.seed, [0, 0], 0, 0.09, 0.7, &low);
    let low_count = low_cells.iter().flatten().count();
    assert!(low_count > 0 && low_count < full_count);
    assert_eq!(ATMOSPHERE_SAMPLES.with(std::cell::Cell::get), 4);
    // The same seeded survivors retain position, tint and twinkle exactly;
    // switching quality must not respawn or recolor the remaining stars.
    for (full_cell, low_cell) in full_cells.into_iter().zip(low_cells) {
        if low_cell.is_some() {
            assert_eq!(full_cell, low_cell);
        }
    }
}
