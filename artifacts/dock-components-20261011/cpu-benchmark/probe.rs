//! Work-only actual native extraction timing; no ticks, renderer, UI, or GPU.
use engine_core::FrameBuilder;
use game_core::{
    Game,
    identity::{self, Corporation},
};
use game_web::dock_art::{self, DockBudget, DockInput};
use serde_json::{Value, json};
use std::{hint::black_box, time::Instant};

const WARMUP: usize = 60;
const ITERATIONS: usize = 600;
const ALPHA: f32 = 1.0;

fn game(corp: &str, zoom: f32, reduced: bool) -> Game {
    let profile = identity::create_profile("dora", corp, "renko", "0123456789abcdef")
        .expect("actual licensed profile");
    let mut game = Game::new(profile);
    // Equivalent to a 1280x720 backing at UI scale2.4, then world zoom.
    // The Game receives a world viewport; projection/rendering is not timed.
    game.set_viewport((1280.0 / 2.4) / zoom, (720.0 / 2.4) / zoom);
    game.set_cosmetic_density(reduced);
    game
}

fn detailed(game: &Game, frame: &mut FrameBuilder) {
    let corporation = game.profile().identity.as_ref().map(|id| id.corporation);
    game.render_into_with_home(ALPHA, frame, |frame, parent, time| {
        let stats = dock_art::append(
            frame,
            &DockInput {
                parent,
                corporation,
                time,
                reduced_motion: false,
            },
            &mut DockBudget::default(),
        )
        .expect("valid admitted dock");
        assert_eq!(stats.cells, dock_art::CELL_COST);
        true
    });
    black_box(frame.frame());
}

fn counts(frame: &FrameBuilder) -> Value {
    json!({
        "quads": frame.frame().pixel_count,
        "transforms": frame.frame().batches.iter().map(|b| b.transforms.len()).sum::<usize>(),
        "batches": frame.frame().batches.len(),
    })
}

fn stats(samples: &mut [u128]) -> Value {
    samples.sort_unstable();
    // Nearest-rank quantiles, with nanosecond Instant differences retained.
    let quantile =
        |percent: usize| samples[(samples.len() * percent).div_ceil(100) - 1] as f64 / 1_000_000.0;
    json!({
        "iterations": samples.len(), "min_ms": samples[0] as f64 / 1_000_000.0,
        "p50_ms": quantile(50), "p95_ms": quantile(95), "p99_ms": quantile(99),
        "max_ms": samples[samples.len()-1] as f64 / 1_000_000.0,
        "total_ms": samples.iter().sum::<u128>() as f64 / 1_000_000.0,
    })
}

fn measure(game: &Game, frame: &mut FrameBuilder, candidate: bool) -> Value {
    for _ in 0..WARMUP {
        if candidate {
            detailed(game, frame);
        } else {
            game.render_into(ALPHA, frame);
        }
        black_box(frame.frame());
    }
    let packet = counts(frame);
    let mut samples = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let started = Instant::now();
        if candidate {
            detailed(game, frame);
        } else {
            game.render_into(ALPHA, frame);
        }
        black_box(frame.frame());
        samples.push(started.elapsed().as_nanos());
        assert_eq!(
            frame.frame().pixel_count,
            packet["quads"].as_u64().unwrap() as usize
        );
    }
    json!({"packet":packet, "timing":stats(&mut samples)})
}

fn main() {
    let trial = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "0".to_owned())
        .parse::<usize>()
        .unwrap();
    // Lazy dock cache has not been touched in this fresh process. Warm the
    // legacy path and builder first, so other Game art is outside the cold dock.
    let cold_game = game("pan", 0.6, false);
    let mut cold_frame = FrameBuilder::new();
    for _ in 0..WARMUP {
        cold_game.render_into(ALPHA, &mut cold_frame);
    }
    let baseline_packet = counts(&cold_frame);
    let mut callback_ns = 0;
    let mut callbacks = 0;
    let cold_started = Instant::now();
    cold_game.render_into_with_home(ALPHA, &mut cold_frame, |frame, parent, time| {
        let started = Instant::now();
        let stats = dock_art::append(
            frame,
            &DockInput {
                parent,
                corporation: Some(Corporation::Pan),
                time,
                reduced_motion: false,
            },
            &mut DockBudget::default(),
        )
        .unwrap();
        callback_ns = started.elapsed().as_nanos();
        callbacks += 1;
        assert_eq!(stats.cells, dock_art::CELL_COST);
        true
    });
    let cold_total_ns = cold_started.elapsed().as_nanos();
    black_box(cold_frame.frame());
    assert_eq!(callbacks, 1);
    let cold = json!({
        "total_game_render_ms":cold_total_ns as f64 / 1_000_000.0,
        "first_dock_callback_ms":callback_ns as f64 / 1_000_000.0,
        "baseline_packet":baseline_packet, "candidate_packet":counts(&cold_frame),
        "includes":"first dock parse/cache expansion and first larger pooled frame; Game construction and other art cache warmup excluded",
    });
    let mut cases = Vec::new();
    for (corp_index, corp) in ["pan", "renko", "hugo"].into_iter().enumerate() {
        for reduced in [false, true] {
            for (zoom_index, zoom) in [0.6, 1.6].into_iter().enumerate() {
                let fixture = game(corp, zoom, reduced);
                let before = serde_json::to_string(fixture.profile()).unwrap();
                let mut baseline_frame = FrameBuilder::new();
                let mut candidate_frame = FrameBuilder::new();
                // Alternate order across cases/trials to expose ordering bias.
                let candidate_first =
                    (trial + corp_index + zoom_index + usize::from(reduced)) % 2 == 1;
                let (baseline, candidate) = if candidate_first {
                    let c = measure(&fixture, &mut candidate_frame, true);
                    (measure(&fixture, &mut baseline_frame, false), c)
                } else {
                    let b = measure(&fixture, &mut baseline_frame, false);
                    (b, measure(&fixture, &mut candidate_frame, true))
                };
                assert_eq!(serde_json::to_string(fixture.profile()).unwrap(), before);
                cases.push(json!({
                    "corporation":corp,"cosmetics":if reduced {"reduced"} else {"full"},
                    "world_zoom_equivalent":zoom,"candidate_first":candidate_first,
                    "baseline":baseline,"candidate":candidate,
                }));
            }
        }
    }
    println!("{}", serde_json::to_string_pretty(&json!({
        "trial":trial,"warmup_per_path":WARMUP,"iterations_per_path":ITERATIONS,
        "alpha":ALPHA,"fixture":"stationary actual licensed Game at Haven, no simulation advance/time zero",
        "scope":"native optimized CPU extraction only; no browser/WASM/GPU/RSS or device acceptance",
        "cold":cold,"cases":cases,
    })).unwrap());
}
