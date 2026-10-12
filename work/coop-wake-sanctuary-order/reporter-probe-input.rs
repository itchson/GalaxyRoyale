// Work-only source appended to the exact frozen native wake test module.
// Configured ordinary builds; these measurements are not earned gameplay or FPS.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::hint::black_box;
use std::time::Instant;

thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATION: Cell<(u64, u64)> = const { Cell::new((0, 0)) };
}
struct BudgetAllocator;
#[global_allocator]
static BUDGET_ALLOCATOR: BudgetAllocator = BudgetAllocator;
fn allocated(size: usize) {
    let _ = COUNTING.try_with(|enabled| {
        if enabled.get() {
            let _ = ALLOCATION.try_with(|count| {
                let (calls, bytes) = count.get();
                count.set((calls + 1, bytes + size as u64));
            });
        }
    });
}
unsafe impl GlobalAlloc for BudgetAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        allocated(layout.size());
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        allocated(layout.size());
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        allocated(size);
        unsafe { System.realloc(ptr, layout, size) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}
fn measure(mut action: impl FnMut(), iterations: u32) -> (f64, u64, u64) {
    ALLOCATION.with(|count| count.set((0, 0)));
    COUNTING.with(|enabled| enabled.set(true));
    let started = Instant::now();
    for _ in 0..iterations { action(); }
    let elapsed = started.elapsed().as_secs_f64();
    COUNTING.with(|enabled| enabled.set(false));
    let (allocations, bytes) = ALLOCATION.with(Cell::get);
    (elapsed * 1_000_000.0 / f64::from(iterations), allocations / u64::from(iterations), bytes / u64::from(iterations))
}
fn fixture(full: bool, target_count: usize, remote: bool) -> Game {
    let profiles = vec![profile(5, 5, 1560); 8];
    let (mut game, admissions) = start(&profiles);
    for a in admissions {
        let e = entity(&game, a.id);
        let position = Vec2::new(800.0, f32::from(a.slot) * 40.0);
        place(&mut game, e, position, 0.0);
        let mut pilot = game.world.get::<&mut Pilot>(e).unwrap();
        assert_eq!((pilot.wake_rank, pilot.wake_evolution), (5, 5));
        pilot.wake_timer = 10_000.0;
        if full {
            pilot.wake = (0..24).map(|index| (position - Vec2::X * index as f32 * 2.0, 3.4)).collect();
        }
    }
    quiet(&mut game);
    for index in 0..target_count {
        let mut position = Vec2::new(if remote { 30_000.0 } else { 1200.0 }, index as f32 * 8.0);
        while game.safe_zone_at(position).is_some() { position.x += 100.0; }
        assert_eq!(game.outside_site_cache(position), remote);
        target(&mut game, position, 1_000_000.0, Kind::Enemy(0));
    }
    game
}
fn resources(game: &Game) -> (usize, usize) {
    let mut pilots = game.world.query::<&Pilot>();
    let samples = pilots.iter().map(|pilot| pilot.wake.len()).sum();
    let links = pilots.iter().map(|pilot| pilot.wake.len().saturating_sub(1)).sum();
    (samples, links)
}
#[test]
fn work_only_native_budget_probe() {
    const ITERATIONS: u32 = 120;
    const BATCHES: u32 = 5;
    for (name, full, targets, remote) in [
        ("empty_local_no_targets", false, 0, false),
        ("full_local_no_targets", true, 0, false),
        ("full_local_48_misses", true, 48, false),
        ("empty_remote_48_misses", false, 48, true),
        ("full_remote_48_misses", true, 48, true),
    ] {
        let mut rows = Vec::new();
        for _ in 0..BATCHES {
            let mut game = fixture(full, targets, remote);
            for _ in 0..20 { advance_wakes(&mut game, TICK_SECONDS); }
            let before = resources(&game);
            let (us, allocations, bytes) = measure(|| {
                advance_wakes(&mut game, TICK_SECONDS);
                black_box(&game);
            }, ITERATIONS);
            let after = resources(&game);
            assert_eq!(before, after);
            assert_eq!(after, if full { (192, 184) } else { (0, 0) });
            assert!(game.transient_lights.len() <= 32 && game.events.len() <= 128);
            rows.push(json!({"us_per_wake_stage": us, "allocation_or_reallocation_calls_per_stage": allocations, "requested_bytes_per_stage": bytes}));
        }
        println!("NATIVE_BUDGET {}", json!({"case": name, "pilots": 8, "configured_samples": if full {192} else {0}, "candidate_links": if full {184} else {0}, "enemy_misses": targets, "remote_target_cache": remote, "warm_stages_per_batch": 20, "measured_stages_per_batch": ITERATIONS, "batches": BATCHES, "dt_seconds": TICK_SECONDS, "measurements": rows}));
    }
    let mut rows = Vec::new();
    for _ in 0..BATCHES {
        let mut game = fixture(true, 48, false);
        game.acquire_co_op_publication(session()).unwrap();
        let (us, allocations, bytes) = measure(|| {
            let capture = game.acquire_co_op_publication(session()).unwrap();
            black_box(capture);
        }, ITERATIONS);
        let capture = game.acquire_co_op_publication(session()).unwrap();
        let generation = capture.acquisition;
        let revision = capture.snapshot.revision;
        let header = serde_json::to_vec(&capture).unwrap();
        let snapshot_with_wake = serde_json::to_value(capture.snapshot).unwrap();
        let counts = capture.counts;
        assert_eq!(capture.snapshot.participants.len(), 8);
        assert_eq!(counts.entities, 56);
        assert!(counts.storage_cells <= co_op_publish::MAX_STORAGE_CELLS);
        let published = game.co_op_publication(session(), generation, revision).unwrap();
        let mut page_bytes = 0;
        let mut body_pages = 0;
        for row in published.entities() {
            let mut offset = 0;
            loop {
                let page = published.body_page(&row.id, offset).unwrap();
                page_bytes += serde_json::to_vec(&page).unwrap().len();
                body_pages += 1;
                if let Some(next) = page.next_offset { offset = next; } else { break; }
            }
        }
        assert_eq!(body_pages, counts.body_pages);
        for pilot in game.world.query::<&mut Pilot>().iter() { pilot.wake.clear(); }
        let without_wake = game.acquire_co_op_publication(session()).unwrap();
        assert_eq!(snapshot_with_wake, serde_json::to_value(without_wake.snapshot).unwrap());
        assert_eq!(counts.storage_cells, without_wake.counts.storage_cells);
        assert_eq!(counts.occupied_cells, without_wake.counts.occupied_cells);
        rows.push(json!({"us_per_acquisition":us, "allocation_or_reallocation_calls_per_acquisition":allocations, "requested_bytes_per_acquisition":bytes, "entities":counts.entities, "storage_cells":counts.storage_cells, "occupied_cells":counts.occupied_cells, "entity_pages":counts.entity_pages, "body_pages":counts.body_pages, "capture_json_bytes":header.len(), "all_body_page_json_bytes":page_bytes}));
    }
    println!("NATIVE_BUDGET {}", json!({"case":"packed_publication_full_192_wake_48_enemies", "measured_acquisitions_per_batch":ITERATIONS, "batches":BATCHES, "scope":"acquisition only; JSON lengths computed separately after timing", "measurements":rows}));
}
