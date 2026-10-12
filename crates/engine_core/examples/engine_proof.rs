//! Native extraction proof. This is CPU geometry evidence, not a GPU/mobile FPS benchmark.
use engine_core::{AffineTransform, FrameBuilder, PixelBody, Transform, Vec2};
use std::hint::black_box;
use std::time::Instant;

const WARMUP: usize = 60;
const SAMPLES: usize = 600;

fn filled_body(width: usize, height: usize) -> PixelBody {
    let rows = vec!["#".repeat(width); height];
    let rows: Vec<_> = rows.iter().map(String::as_str).collect();
    PixelBody::from_ascii(&rows, &[('#', 0xff_dac0a0)])
}

fn extract(builder: &mut FrameBuilder, enemy: &PixelBody, rock: &PixelBody, phase: usize) {
    builder.clear();
    for i in 0..160 {
        builder.push_body_colored(
            enemy,
            Transform {
                position: Vec2::new((i % 20) as f32 * 24.0, (i / 20) as f32 * 24.0),
                ..Transform::default()
            },
            |local, color| {
                if ((local.x + local.y) as i32 + phase as i32) & 7 == 0 {
                    color ^ 0x0000_001f
                } else {
                    color
                }
            },
        );
    }
    for i in 0..60 {
        builder.push_body(
            rock,
            Transform {
                position: Vec2::new((i % 10) as f32 * 48.0, (i / 10) as f32 * 48.0),
                rotation: i as f32 * 0.17,
                scale: Vec2::splat(1.3),
            },
        );
    }
    for i in 0..300 {
        let p = Vec2::new((i % 30) as f32 * 16.0, (i / 30) as f32 * 16.0);
        for (offset, size) in [(0.0, 2.0), (3.0, 1.5), (6.0, 1.0)] {
            builder.push_pixel(p - Vec2::X * offset, size, 0xff_c0eaff);
        }
    }
    for i in 0..2000 {
        builder.push_pixel(
            Vec2::new((i % 80) as f32 * 6.0, (i / 80) as f32 * 6.0),
            0.8 + i as f32 * 0.001,
            0x80_ff80a0,
        );
    }
    for i in 0..250 {
        for j in 0..6 {
            builder.push_pixel(
                Vec2::new((i % 25) as f32 * 20.0 + j as f32, (i / 25) as f32 * 20.0),
                1.5,
                0xff_90ffa0,
            );
        }
    }
}

fn quantile(samples: &[f64], p: f64) -> f64 {
    samples[((samples.len() - 1) as f64 * p).ceil() as usize]
}

fn main() {
    let enemy = filled_body(10, 10);
    let rock = filled_body(32, 32);
    let mut builder = FrameBuilder::new();
    for phase in 0..WARMUP {
        extract(&mut builder, &enemy, &rock, phase);
        black_box(builder.frame());
    }
    let mut timings = Vec::with_capacity(SAMPLES);
    for phase in 0..SAMPLES {
        let start = Instant::now();
        extract(&mut builder, &enemy, &rock, phase);
        black_box(builder.frame());
        timings.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    timings.sort_by(f64::total_cmp);
    let frame = builder.frame();
    assert_eq!(frame.pixel_count, 81_840);
    assert_eq!(
        frame.batches.iter().map(|b| b.pixels.len()).sum::<usize>(),
        frame.pixel_count
    );
    for batch in &frame.batches {
        assert!(batch.transforms.len() <= 128);
        assert!(
            batch
                .pixels
                .iter()
                .all(|p| p.transform_slot() < batch.transforms.len())
        );
    }
    println!(
        "{{\"scope\":\"native_cpu_extraction\",\"warmup\":{WARMUP},\"samples\":{SAMPLES},\"pixels\":{},\"batches\":{},\"instance_bytes\":{},\"transform_bytes_unpadded\":{},\"median_ms\":{:.6},\"p95_ms\":{:.6},\"p99_ms\":{:.6},\"max_ms\":{:.6}}}",
        frame.pixel_count,
        frame.batches.len(),
        frame.pixel_count * std::mem::size_of::<engine_core::PixelInstance>(),
        frame
            .batches
            .iter()
            .map(|b| b.transforms.len())
            .sum::<usize>()
            * std::mem::size_of::<engine_core::GpuTransform>(),
        quantile(&timings, 0.5),
        quantile(&timings, 0.95),
        quantile(&timings, 0.99),
        timings[timings.len() - 1],
    );

    let shear = Transform {
        position: Vec2::new(22.0, -9.0),
        rotation: 0.8,
        scale: Vec2::new(2.0, 0.5),
    }
    .compose_affine(Transform {
        rotation: -0.5,
        scale: Vec2::new(0.7, 1.6),
        ..Transform::default()
    });
    let mut perforated =
        PixelBody::from_ascii(&["##  ##", "      ", "##  ##"], &[('#', 0xff_ffffff)]);
    assert!(
        perforated
            .hit_segment(Vec2::new(-4.0, 0.0), Vec2::new(4.0, 0.0))
            .is_none()
    );
    let local_start = Vec2::new(-4.0, -1.0);
    let local_end = Vec2::new(4.0, -1.0);
    assert!(
        perforated
            .hit_segment_world(shear.point(local_start), shear.point(local_end), shear)
            .is_some()
    );
    let before = perforated.occupied_count();
    let debris = perforated.damage(Vec2::new(-2.5, -1.0), 0.1);
    assert_eq!(before - perforated.occupied_count(), debris.len());
    let collision_start = Instant::now();
    let rays = 100_000;
    let mut hits = 0;
    for i in 0..rays {
        let y = if i & 1 == 0 { 0.0 } else { 1.0 };
        hits += black_box(perforated.hit_segment(Vec2::new(-4.0, y), Vec2::new(4.0, y))).is_some()
            as usize;
    }
    println!(
        "{{\"scope\":\"local_swept_collision\",\"rays\":{rays},\"hits\":{hits},\"total_ms\":{:.6},\"damage_removed\":{}}}",
        collision_start.elapsed().as_secs_f64() * 1000.0,
        debris.len()
    );
    for coordinate in [1_000.0_f32, 100_000.0, 1_000_000.0, 16_000_000.0] {
        let spacing = f32::from_bits(coordinate.to_bits() + 1) - coordinate;
        let transform = AffineTransform {
            position: Vec2::splat(coordinate),
            ..AffineTransform::default()
        };
        let small_motion = transform.point(Vec2::new(0.1, 0.0)).x - coordinate;
        println!(
            "{{\"scope\":\"global_f32_precision\",\"coordinate\":{coordinate:.0},\"next_float_spacing\":{spacing},\"requested_motion\":0.1,\"represented_motion\":{small_motion}}}"
        );
    }
}
