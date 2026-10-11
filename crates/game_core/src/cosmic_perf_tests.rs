//! Explicit native CPU measurements only; excluded from production builds.
use super::*;
use std::{hint::black_box, time::Instant};

fn transform_count(frame: &FrameBuilder) -> usize {
    frame
        .frame()
        .batches
        .iter()
        .map(|b| b.transforms.len())
        .sum()
}

#[test]
#[ignore = "explicit fresh-process native cache timing; run alone in release mode"]
fn measure_cold_cosmic_caches() {
    let started = Instant::now();
    black_box(LazyLock::force(&ART));
    let art_ns = started.elapsed().as_nanos();
    let started = Instant::now();
    black_box(LazyLock::force(&GAS_ART));
    let gas_ns = started.elapsed().as_nanos();
    let pairs = [
        &ART.galaxy_arms,
        &ART.galaxy_core,
        &ART.galaxy_disk,
        &ART.nebula,
        &ART.nebula_thread,
        &ART.nova_shell,
        &ART.nova_core,
        &GAS_ART.veil,
        &GAS_ART.billows,
    ];
    let occupied: [usize; 2] =
        [false, true].map(|reduced| pairs.iter().map(|p| p.body(reduced).occupied_count()).sum());
    println!(
        "COSMIC_BENCH_JSON:{}",
        serde_json::json!({"mode":"cold_cache_force", "art_ns":art_ns,"gas_ns":gas_ns,
            "total_ns":art_ns+gas_ns,"occupied_full_low":occupied,"cached_bodies":pairs.len()*2,
            "scope":"fresh-process ART then GAS_ART initialization only; excludes startup/extraction"})
    );
}

#[derive(Clone, Copy)]
enum Path {
    Motif,
    Custom,
    RegionalGas,
}

impl Path {
    fn name(self) -> &'static str {
        match self {
            Self::Motif => "nebula_motif",
            Self::Custom => "custom_background",
            Self::RegionalGas => "regional_gas",
        }
    }
}

fn extract(frame: &mut FrameBuilder, m: Motif, scene: SceneInput, path: Path) {
    frame.clear();
    match path {
        Path::Motif => {
            let mut quota = if scene.reduced { BUDGET / 2 } else { BUDGET };
            black_box(draw_motif(frame, m, &scene, &mut quota));
        }
        Path::Custom => {
            black_box(render(frame, scene));
        }
        Path::RegionalGas => {
            let mut quota = if scene.reduced { 3_000 } else { 6_000 };
            black_box(render_gas(frame, scene, &mut quota));
        }
    }
    black_box(frame.frame());
}

#[test]
#[ignore = "explicit bounded warm native extraction timing; run alone in release mode"]
fn measure_warm_cosmic_extraction() {
    black_box(LazyLock::force(&ART));
    black_box(LazyLock::force(&GAS_ART));
    let seed = 871_238_449;
    let m = (-8..=8)
        .flat_map(|y| (-8..=8).map(move |x| [x, y]))
        .filter_map(|grid| motif(seed, grid))
        .find(|m| m.kind == MotifKind::Nebula)
        .expect("actual seeded nebula within bounded grid search");
    let camera = m.origin / MOTIF_PARALLAX;
    let mut results = Vec::new();
    for path in [Path::Motif, Path::Custom, Path::RegionalGas] {
        for reduced in [false, true] {
            for zoom in [0.6_f32, 1.0, 1.6] {
                let mut frame = FrameBuilder::new();
                let mut samples = Vec::with_capacity(256);
                let mut cells = [usize::MAX, 0];
                let mut transforms = [usize::MAX, 0];
                for i in 0..288 {
                    let scene = black_box(SceneInput {
                        seed,
                        camera,
                        viewport: Vec2::new(533.0, 300.0) / zoom,
                        time: 20.0 + i as f32 / 60.0,
                        reduced,
                        reduced_motion: false,
                    });
                    let started = Instant::now();
                    extract(&mut frame, m, scene, path);
                    let elapsed = started.elapsed().as_nanos();
                    if i >= 32 {
                        samples.push(elapsed);
                        let count = frame.frame().pixel_count;
                        cells[0] = cells[0].min(count);
                        cells[1] = cells[1].max(count);
                        let count = transform_count(&frame);
                        transforms[0] = transforms[0].min(count);
                        transforms[1] = transforms[1].max(count);
                    }
                    assert!(frame.frame().pixel_count > 0);
                    assert!(frame.frame().pixel_count <= if reduced { BUDGET / 2 } else { BUDGET });
                    if matches!(path, Path::Motif) {
                        assert_eq!(frame.frame().pixel_count, motif_cost(m.kind, reduced));
                        assert_eq!(transform_count(&frame), 4);
                    }
                }
                results.push(
                    serde_json::json!({"path":path.name(),"reduced":reduced,"zoom":zoom,
                    "viewport":(Vec2::new(533.0,300.0)/zoom).to_array(),"warmup_frames":32,
                    "samples_ns":samples,"cells_min_max":cells,"transforms_min_max":transforms}),
                );
            }
        }
    }
    println!(
        "COSMIC_BENCH_JSON:{}",
        serde_json::json!({"mode":"warm_extraction","seed":seed,"camera":camera.to_array(),
            "motif_hash":m.hash,"motif_origin":m.origin.to_array(),"cases":results,
            "scope":"release CPU FrameBuilder clear/reuse+native extraction; excludes GPU, cache initialization, startup and serialization"})
    );
}
