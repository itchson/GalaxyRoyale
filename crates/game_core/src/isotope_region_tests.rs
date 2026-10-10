use super::*;
use engine_core::{GpuTransform, RenderFrame};

fn packets(frame: &RenderFrame) -> Vec<([f32; 2], u32, GpuTransform)> {
    frame
        .batches
        .iter()
        .flat_map(|batch| {
            batch.pixels.iter().map(|pixel| {
                (
                    pixel.local,
                    pixel.color,
                    batch.transforms[pixel.transform as usize],
                )
            })
        })
        .collect()
}
fn seed_input(grid: [i32; 2], viewport: Vec2, reduced: bool) -> crate::isotope_scar::SceneInput {
    let seed = 17;
    let h = hash(seed, grid, 7919 ^ 0x706c_616e);
    let origin =
        (Vec2::new(grid[0] as f32, grid[1] as f32) + Vec2::new(unit(h), unit(mix(h)))) * 860.0;
    crate::isotope_scar::SceneInput {
        origin,
        camera: origin / 0.22,
        viewport,
        time: 18.0,
        rotation: unit(mix(h ^ 93)) * 0.6,
        scale: Vec2::splat(0.64 + unit(mix(h ^ 37)) * 0.08),
        reduced,
        reduced_motion: false,
    }
}
fn expected_packets(input: &crate::isotope_scar::SceneInput) -> Vec<([f32; 2], u32, GpuTransform)> {
    let mut frame = FrameBuilder::new();
    let mut budget = 6000;
    let stats = crate::isotope_scar::extract(&mut frame, input, &mut budget);
    assert_eq!(stats.quads, if input.reduced { 1411 } else { 4854 });
    packets(frame.frame())
}
fn contains(frame: &RenderFrame, needle: &[([f32; 2], u32, GpuTransform)]) -> bool {
    packets(frame)
        .windows(needle.len())
        .any(|window| window == needle)
}
fn assert_no_old_planet_transform(
    frame: &RenderFrame,
    input: &crate::isotope_scar::SceneInput,
    grid: [i32; 2],
) {
    let h = hash(17, grid, 7919 ^ 0x706c_616e);
    let kind = (mix(h ^ 12) % 3) as usize;
    let old: GpuTransform = Transform {
        position: input.origin + input.camera * 0.78,
        rotation: if kind == 1 {
            -0.2
        } else {
            unit(mix(h ^ 93)) * 0.6
        },
        scale: Vec2::splat(0.32 + unit(mix(h ^ 37)) * 0.4),
    }
    .into();
    assert!(
        frame
            .batches
            .iter()
            .flat_map(|batch| &batch.transforms)
            .all(|transform| *transform != old)
    );
}
#[test]
fn durable_seeded_region_contains_exact_authored_scar_packets_at_all_zoom_extents() {
    for (reduced, grid) in [(false, [-8, -2]), (true, [-1, -1])] {
        for zoom in [0.6, 1.03, 1.6] {
            let input = seed_input(grid, Vec2::new(533.0, 300.0) / zoom, reduced);
            assert_eq!(at(17, input.camera).kind, 4);
            let expected = expected_packets(&input);
            let mut frame = FrameBuilder::new();
            background_with_density(
                &mut frame,
                17,
                input.camera,
                input.viewport,
                input.time,
                reduced,
            );
            assert!(
                contains(frame.frame(), &expected),
                "actual seeded background omitted or changed authored packet geometry/colors"
            );
            assert!(frame.frame().pixel_count <= if reduced { 3000 } else { 6000 });
            assert_no_old_planet_transform(frame.frame(), &input, grid);
        }
    }
}
#[test]
fn durable_rejected_seeded_scar_does_not_restore_old_planet_or_rings() {
    for y in -10..=10 {
        for x in -10..=10 {
            let grid = [x, y];
            let h = hash(17, grid, 7919 ^ 0x706c_616e);
            if !h.is_multiple_of(3) {
                continue;
            }
            let input = seed_input(grid, Vec2::new(888.0, 500.0), false);
            if at(17, input.camera).kind != 4 {
                continue;
            }
            let expected = expected_packets(&input);
            let mut frame = FrameBuilder::new();
            background_with_density(
                &mut frame,
                17,
                input.camera,
                input.viewport,
                input.time,
                false,
            );
            if !contains(frame.frame(), &expected) {
                assert_no_old_planet_transform(frame.frame(), &input, grid);
                assert!(frame.frame().pixel_count <= 6000);
                return;
            }
        }
    }
    panic!("bounded fixture search found no shared-priority rejection");
}
#[test]
fn durable_zero_cosmetic_time_is_repeatable_with_actual_seeded_scar() {
    let mut input = seed_input([-8, -2], Vec2::new(533.0, 300.0), false);
    input.time = 0.0;
    let mut a = FrameBuilder::new();
    let mut b = FrameBuilder::new();
    background_with_density(&mut a, 17, input.camera, input.viewport, input.time, false);
    background_with_density(&mut b, 17, input.camera, input.viewport, input.time, false);
    assert_eq!(packets(a.frame()), packets(b.frame()));
    assert!(contains(a.frame(), &expected_packets(&input)));
}
