# Pixel engine implementation and proof

The game uses portable engine_core, game_core with a hecs World, and game_web with custom wgpu rendering. TypeScript handles input, semantic accessibility controls, audio, saves and PWA lifecycle. Input/HUD/events cross WASM in batches; individual pixels never call JavaScript.

## Occupancy, hierarchy and materials

Generated ship/station and UI artwork is thresholded, nearest-resampled and palette indexed into actual PixelBodies. Packed local RGBA cells and occupancy are separate. Removed cells leave collision holes. Extraction scans original rectangular storage even after damage.

Each PixelInstance is 16 bytes; each affine GpuTransform is 32 bytes, including light response/emission. Up to 128 transforms share a 4,096-byte uniform table compatible with WebGL2. FrameBuilder and GPU buffers reuse capacity. Submission is capped at 384,000 cells.

Exact affine composition preserves nonuniform parent scale, child rotation and shear. RoyaleMono caches original 5×7 letter bodies in text/letter/group transforms. Generated UI art and authored 24×32 comms portraits use the same quad extraction. The console is an opaque black drawer with one bottom rail. Low-resolution rendering uses integer nearest-neighbor presentation; rotating pixels can cover changing logical display cells, without blur filtering.

A 272-byte uniform supplies eight bounded camera-relative radial lights from recent effects, nearby bullets, engine thrust and a station beacon. Each cell receives one color; UI/background defaults remain unlit. This does not implement shadows, normal maps or volumetric scattering.

## Simulation and destruction

Swept segments traverse occupied cells in body-local coordinates. Candidate hits are ordered by actual cell contact after broad-phase rejection. Circular damage removes cells, emits their original colors as debris and applies impulses. Asteroid contacts exchange momentum and separate overlap. Disconnected pixel groups can be identified, but complete independently simulated rigid-body fracture remains future work.

Fixed 60 Hz simulation interpolates extraction, caps catch-up, smooths the camera and applies presentation-only shake. Hidden tabs pause/reset input. Normal caps are 48 enemy roots, 180 projectiles, 1,100 particles, 112 asteroids and 96 pickups; extra drops merge with matching pickups. Coil segments add ECS entities. Collision candidates use bounded CPU loops.

## Streamed universe

512-unit asteroid chunks retain at most 25 chunk records. Transient rock damage resets after unload. A seeded 4,200-unit jittered region network provides six biome palettes/family weights, sanctuary discoveries, courier/family-bounty events and once-only persistent relic rewards.

Cached 96×96 planets use small relative transforms, craters, veins, storms and rings. Fine gas wisps curl/fade analytically by seed/time. Regional cosmetic budgets are 6,000 cells full or 3,000 reduced. Star layers have independent parallax; density does not change gameplay limits.

Global simulation remains f32, bounded to ±1,000,000 per axis; adjacent floats there are 0.0625 units apart. CPU camera subtraction improves extraction precision but is not a floating-origin simulation. CircularTopology is opt-in and does not wrap solo play. Its seam/interpolation/camera constraints are documented in [UNIVERSE_TOPOLOGY.md](UNIVERSE_TOPOLOGY.md).

## Measurement

Run cargo test --workspace --locked, native/WASM strict Clippy, pnpm typecheck, pnpm test and pnpm build. Native engine_proof/game_proof examples warm up 60 frames and measure 600 samples.

[Native results](../artifacts/native-performance.jsonl), measured on the Windows Ryzen 9 9950X3D workstation on 2026-10-11 local time, exclude WASM/browser/GPU work and light-uniform extraction.

| Workload | Median | p95 | p99 |
| --- | ---: | ---: | ---: |
| Synthetic 81,840-cell extraction / 18 batches | 0.1655 ms | 0.2309 ms | 0.2751 ms |
| Dock tick + extraction | 0.0764 ms | 0.0978 ms | 0.1217 ms |
| Combat tick + extraction | 0.1008 ms | 0.1126 ms | 0.1558 ms |
| Stress tick + extraction | 0.6357 ms | 0.8202 ms | 0.9653 ms |

Native stress measured 91,776–125,759 cells, 2,000 particles and 699–708 ECS entities. Initial setup has 160 enemy roots, 300 projectiles and 128,043 cells; motion/destruction/culling changes visible counts.

[Desktop WebGPU capture](../artifacts/browser-stress-webgpu.json) at 1280×720 measured 83,996–131,546 submitted cells and 2,000 particles, with 4.3 ms p95/p99 presentation interval under a high-refresh cadence. Render timing is CPU extraction/upload/submission, not GPU timestamp duration. Memory is JS heap only. This short capture is not a phone pass or 30-minute soak.

## Remaining gates

Physical Pixel 6a/iPhone 12-class 60 FPS testing, Safari, device audio/PWA lifecycle and 30-minute soak remain pending. Record named hardware/backend, resolution, visible counts, frame percentiles, stalls and total-memory trends. Reduce cosmetics before changing gameplay limits.

Both renderer bundles share gameplay; browser verification must exercise initialization fallback and recovery. The current content has three families with five authored boss generations each. Nested armor/core/emitter rigs extract surviving occupied cells without restoring damaged holes. The fifteen-boss native combat sweep and eight real WASM snapshot scenarios are recorded in [QA_BOSSES.md](QA_BOSSES.md); these do not establish human balance or phone performance. Networking/owned arenas and balancing remain backlog. Build records distinguish automated tests, desktop browser checks and unmeasured acceptance.

