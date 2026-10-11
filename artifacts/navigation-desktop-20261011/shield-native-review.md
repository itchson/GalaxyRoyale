# Frozen asteroid-shield native review

No blocking defect was reproduced. The current ordinary shield path remains finite, does not grant rewards, and passed the declared near-world-edge geometry probes. This review made no production or distribution edits and did not rebuild a public package.

## Pins and method

Current `lib.rs` SHA256 `46ae2e436719bb2a66825b4f7b36a009ae497a35fb84a920d3fff9b5819104a3`; `asteroid_zones.rs` `9f1ba2e68d403796e3ea64ec771c6fb433cfb1614c4091d1170beb28732e61f7`; `regions.rs` `a3dfd7561af5f03fd970de93c8caaed6e3ef51ee959394f28fb03643fa71aa01`. `source-pins.json` covers current core/engine source, assets, manifests and lockfile. All were checked before building, after compilation and after measurement. Among the prior 26-input gameplay handoff, only root's `universe_map.rs` formatting differs; that historical handoff is unchanged.

The existing production package was compiled with `cargo build -p game_core --release --locked --target-dir work/asteroid-shield-final-review/native-target`. A work-only copy of the exact `Game` implementation adds absolute module paths and a private probe; it changes no runtime branch or production API. This lets the probe directly exercise the real private guard and inspect real ECS motion. Engine and other dependencies are the fresh release rlibs, recorded in `conditions.json`. Both copied core and executable use opt-level 3/thin LTO. No workspace test rerun, WASM, renderer, browser, or physical phone claim is made.

## Source review and near-edge evidence

The solver expands canonical Home/site circles by each actor's conservative occupied-body radius. It computes earliest quadratic surface contact in f64 from f32 relative coordinates, refreshes canonical neighbourhoods after every redirected corridor, and bounds processing to four contacts. Any remaining untested fifth corridor is dropped. Final penetration correction runs again after ordinary contact impulses/pair separation. Contact/corrected previous positions keep presentation interpolation outside protection; there is no shield damage/reward/event call.

Ordinary trajectories use one 3×3 canonical neighbourhood per query. The 512-sample ceiling also covers the practical ±1,000,000 world extent: its largest diagonal is approximately 2,828,427, so consecutive sample centres are at most 5,525 units apart. A midpoint is at most about 2,763 units from the nearest point on that line; adding existing site/body radii stays within the neighbouring grid coverage. The present canonical site jitter lies inside its grid cell. This is a source-bound argument for that extent, not a proof for arbitrary externally injected huge finite positions or radii.

The new actual-helper experiment covers **172,800 corrected/interpolated pose samples**: four seeds, 32 canonical sites near all signed world edges, 360 directions, three declared modes and five interpolation alphas. Modes are a post-separation start 0.125 units inside the expanded shield, an inward 160-unit/s sweep over 0.05 seconds, and a declared extreme 20,000-unit/s sweep over 0.05 seconds. The actor is the real hash-31 asteroid body at scale 0.7, with conservative radius 14.677535. Every current/previous velocity was finite; no event was emitted. These directly call the actual guards on controlled ECS fixtures rather than claiming natural player encounters at the world boundary.

All sampled conservative-circle clearances were positive. Minimum **0.008560181 world units** occurred at interpolation alpha 0, seed `1196576601`, canonical site `-237:-237` at `[-992460.4375,-993534.25]`, site radius `222.406311`, direction 316°, extreme-sweep mode. Its recorded previous pose was `[-992289.9375,-993699.0]`. `results.json` records the exact reproducible setup, final pose and first-penetration list, which is empty. At these magnitudes f32 translation ULP is 0.0625; rounding each coordinate by half an ULP can reduce ideal 0.05 clearance substantially. The measured small residual is therefore meaningful. It is not a guarantee beyond the existing practical coordinate/radius contract, nor a claim about arbitrary future larger assets.

## Ordinary native CPU cost

Four declared scenes run three independent Game instances each: 120 untimed warmup ticks, then 600 advancing 60Hz ticks, followed by 300 repeated snapshot guard pairs (`swept=true`, then `false`). That is 7,200 timed advancing ticks and 3,600 separately timed guard pairs. Scenes use a starting Scout, neutral input, no firing, spawn/boss timers held far in the future, and declared long invulnerability. Natural streamed geometry is real; the fourth fixture deliberately places 112 real seeded orbit rocks in a nonoverlapping ring. All twelve profiles stayed identical during timing and emitted zero events. The preexisting full native suite already tests real collision impulses/projectile shields; these new cost fixtures deliberately exclude fighting.

An actual `System` allocation observer counts allocations/reallocations and requested bytes during each interval. Timing includes that instrumentation. It is not an uninstrumented frame benchmark, live heap/RSS figure, peak memory or leak claim. Requested byte totals include repeated reallocations and freed temporary strings/vectors. Wall-clock sample variation reflects ordinary uncontrolled desktop scheduling; affinity, frequency, thermals and other apps were not controlled.

| Fixture | Observed rocks throughout | Tick median range (ms) | Tick p95 range (ms) | Guard-pair p95 range (ms) | Calls per tick / guard pair | Requested bytes per guard pair |
|---|---:|---:|---:|---:|---:|---:|
| Natural Haven | 56 | .1911–.2219 | .2796–.3159 | .2591–.3365 | 2,872 / 2,866 | 264,928 |
| Natural Haven belt | 65 | .2210–.2549 | .2818–.3359 | .2705–.3258 | 3,334 / 3,327 | 317,016 |
| Natural remote belt | 44 | .1466–.1510 | .2006–.2248 | .1822–.2083 | 2,260 / 2,254 | 210,496 |
| Declared 112-rock ring | 112 | .3809–.3826 | .4696–.4945 | .4503–.5069 | 5,731 / 5,724 | 530,208 |

Maximum observed full tick was 0.7596 ms; the 112-rock full-tick p99 ranged 0.5234–0.5497 ms, guard-pair p99 0.5026–0.6160 ms. Guard and tick windows are separate samples at different simulation times; their noise is not additive stage accounting. `results.json` retains all per-trial figures. This is desktop native CPU work with no frame extraction/upload/render/audio/UI and no adversarial combat workload; it does not establish phone/browser 60 FPS or sustained-session acceptance.

The temporary allocation churn is concrete. In the ordinary no-contact path the swept call queries one neighbourhood and its final-penetration pass queries another; the post-collision call queries a third. `shield_circles` constructs a Vec/HashSet and `sites_near` regenerates formatted site IDs on each query. Most measured tick allocations/time therefore lie in the guards. Persistent memory is not growing in this observation. A future measured optimization could share immutable nearby shield geometry within a tick while retaining each redirected-path refresh, but no speculative refactor is proposed or integrated for this frozen release.

The 112 bound belongs to ordinary asteroid streaming. Explicit diagnostic stress setup may author additional rocks outside that streaming cap; this ordinary-cost review does not claim its figures cover that workload. Human contact balance, normal GPU/GL rendering, mobile DSP/inputs, sustained allocation behavior and physical devices remain separate acceptance gates.

`run.py` is an output-writing reproduction script and must not be rerun over this evidence freeze. `frozen-hashes.json` pins the owned source/evidence files; the production source remains unchanged.
