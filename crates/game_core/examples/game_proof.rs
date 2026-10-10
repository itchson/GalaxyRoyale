//! Native fixed-step CPU proof. Browser GPU/input/audio and mobile thermals are separate gates.
use engine_core::FrameBuilder;
use game_core::{Game, InputFrame, Profile};
use std::hint::black_box;
use std::time::Instant;

const WARMUP: usize = 60;
const SAMPLES: usize = 600;
const DT: f32 = 1.0 / 60.0;

fn quantile(samples: &[f64], p: f64) -> f64 {
    samples[((samples.len() - 1) as f64 * p).ceil() as usize]
}

fn report(name: &str, stage: &str, samples: &mut [f64]) {
    samples.sort_by(f64::total_cmp);
    println!(
        "{{\"scope\":\"native_game_cpu\",\"scene\":\"{name}\",\"stage\":\"{stage}\",\"warmup\":{WARMUP},\"samples\":{SAMPLES},\"median_ms\":{:.6},\"p95_ms\":{:.6},\"p99_ms\":{:.6},\"max_ms\":{:.6}}}",
        quantile(samples, 0.5),
        quantile(samples, 0.95),
        quantile(samples, 0.99),
        samples[samples.len() - 1],
    );
}

fn run(name: &str, diagnostic: Option<&str>) {
    let mut game = Game::new(Profile::default());
    game.set_viewport(800.0, 360.0);
    if let Some(command) = diagnostic {
        game.diagnostic(command);
    }
    let mut frame = FrameBuilder::new();
    game.render_into(1.0, &mut frame);
    let initial_pixels = frame.frame().pixel_count;
    let input = InputFrame::default();
    for _ in 0..WARMUP {
        game.tick(&input, DT);
        game.render_into(1.0, &mut frame);
        black_box(frame.frame());
        black_box(game.take_events());
    }
    let mut ticks = Vec::with_capacity(SAMPLES);
    let mut extractions = Vec::with_capacity(SAMPLES);
    let mut total = Vec::with_capacity(SAMPLES);
    let mut pixels = (usize::MAX, 0);
    let mut entities = (usize::MAX, 0);
    let mut particles = (usize::MAX, 0);
    let mut max_batches = 0;
    for _ in 0..SAMPLES {
        let start = Instant::now();
        game.tick(black_box(&input), DT);
        let tick_ms = start.elapsed().as_secs_f64() * 1000.0;
        let extraction_start = Instant::now();
        game.render_into(1.0, &mut frame);
        black_box(frame.frame());
        let extraction_ms = extraction_start.elapsed().as_secs_f64() * 1000.0;
        ticks.push(tick_ms);
        extractions.push(extraction_ms);
        total.push(tick_ms + extraction_ms);
        let hud = game.hud();
        pixels.0 = pixels.0.min(frame.frame().pixel_count);
        pixels.1 = pixels.1.max(frame.frame().pixel_count);
        entities.0 = entities.0.min(hud.entities);
        entities.1 = entities.1.max(hud.entities);
        particles.0 = particles.0.min(hud.particles);
        particles.1 = particles.1.max(hud.particles);
        max_batches = max_batches.max(frame.frame().batches.len());
        assert!(hud.position.iter().all(|v| v.is_finite()));
        assert!(hud.camera.iter().all(|v| v.is_finite()));
        assert!(frame.frame().batches.iter().all(|b| {
            b.transforms.len() <= 128
                && b.pixels
                    .iter()
                    .all(|p| (p.transform as usize) < b.transforms.len())
        }));
        black_box(game.take_events());
    }
    if diagnostic == Some("stress") {
        assert!(initial_pixels >= 80_000);
        assert_eq!(particles, (2000, 2000));
    }
    report(name, "tick", &mut ticks);
    report(name, "extraction", &mut extractions);
    report(name, "tick_plus_extraction", &mut total);
    println!(
        "{{\"scope\":\"native_game_workload\",\"scene\":\"{name}\",\"logical_width\":800,\"logical_height\":360,\"initial_pixels\":{initial_pixels},\"pixels_min\":{},\"pixels_max\":{},\"entities_min\":{},\"entities_max\":{},\"particles_min\":{},\"particles_max\":{},\"batches_max\":{max_batches}}}",
        pixels.0, pixels.1, entities.0, entities.1, particles.0, particles.1,
    );
}

fn main() {
    run("dock", None);
    run("combat", Some("combat"));
    run("stress", Some("stress"));
}
