# Radiation Mote engine integration review

The frozen **V2 / review repair1** is suitable for root's next production integration pass. No remaining blocking source defect was found in the reviewed cell geometry, current sanctuary ordering, shared projectile/reward paths or repaired pulse timing. This is acceptance of an isolated native candidate, not a claim that the Mote has been integrated, published, browser reviewed, balanced or tested on phones.

This review owns only `work/radiation-production-engine-review/`. The candidate and production were read without edits. Independent tests were appended to an owned copy using path-only rewrites; the real Game, controller and renderer extraction implementations were retained. Root's separately accepted early-death integration was deliberately **not** incorporated into these candidate runs.

## Exact source and commands

| Input | SHA256 |
|---|---|
| Final V2 freeze manifest | `8b2a1180863dfe9106576c5617a6fff4d2daaea898c98a774e7be8b854b260ed` |
| Actual-core candidate lib source | `f65a4a4ea80126325ff88343a65f45a011dd368cb9d656b02a72b35ce2151164` |
| Runtime module | `38150fa390c4e9ebee7bc88f56bab9c0b61663cf77597aef81479d860620f44f` |
| Controller | `c443549dedf4d2bf71316533b7e04a9d8ed6e59fff5193acd9f659c5acd0b130` |
| Frozen candidate rlib | `a46a1821641aaa1c68deb7eeb4805838129abf717bce2a863d3c04dadb6bdcdd` |
| Candidate baseline production lib | `0a574a35c9d23f955fff05246da2cf4bf9a8065f5db58c63c365124f982337e6` |

The preparer pins the final manifest, verifies every frozen candidate artifact and all linked modules except the separately changed production lib. `fingerprints.json` records the current production lib solely as a read-only observation; the compiled Game comes from the candidate copy. `final-evidence.json` records our copied sources, probes, executable, logs, Rust toolchain and existing dependency rlibs.

Executed from the repository root:

```powershell
./work/radiation-production-engine-review/build.ps1 -FrozenCandidateHash F65A4A4EA80126325FF88343A65F45A011DD368CB9D656B02A72B35CE2151164
./work/radiation-production-engine-review/engine-probes.exe independent_engine_review:: --nocapture
./work/radiation-production-candidate/mote-tests.exe radiation_enemies::tests:: --nocapture
```

Compilation passed. **4/4 independent engine probes** passed; **16/16 frozen actual-core tests** passed. Logs are `build-v2.log`, `engine-proof-v2.log` and `frozen-core-tests-v2.log`. Existing baseline tests compiled but were filtered. We did not rerun a broad production suite or the candidate's reported Clippy checks. Prior V1 review logs/identities remain in our `v1/`; Gameplay separately preserved V1 and intermediate timing V2.

## Occupied geometry, transforms and destruction

The original authored 40×40 indexed body has **712 occupied cells**, eleven used opaque colors and three disjoint occupied emitter masks containing **142/143/143 cells**. Runtime extraction uses the actual damaged `PixelBody`; it does not draw the pristine preview or restore missing cells. The 0.6 uniform scale yields a conservative occupied-quad radius of approximately **13.820534 world units**. Body collision and visual extraction both use heading plus PI/2.

The Mote is one rigid local pixel object. The three pylon masks are semantic ownership regions within that object, not independently articulated hitboxes. Every extracted body quad retains its local cell center and the same object transform. The material is light response0.75, emission0; warning cells are unlit presentation quads. The native proof compares packed GPU transform output with the actual body transform near origin and at approximately ±999,000 world units, within0.09 world-unit component-rounding distance. It also confirms exact occupied count after real local damage and finite transform tables. This tests packing; it is not a GPU screenshot claim.

Emitter ranking happens in local direction space with zero translation, avoiding far-world cancellation. A muzzle is a real surviving cell of its nominal mask transformed to world space. Erasing that entire mask disables its rays even when other pylons survive. The independent proof covers six authored ray angles, three headings and three world positions, verifies actual occupied ownership after inverse transformation, and tests complete owner removal. Far-world f32 still quantizes world positions; it is not continuous infinite-precision space.

The additional portable approach proof uses the real body-derived radius, a20-world-unit player, current/predicted camera fields and5-unit shake reserve at both533×300 and320×180 UI / zoom1.6. At near and ±999,000 positions,1/60 and0.05 steps all reach a real pulse without losing the conservative warning envelope. Its physical clearance assertion allows7.8 rather than8 units to disclose far-f32 rounding. This probe integrates the returned pose exactly once manually; the actual Game integration and stationary clearance evidence come from the separate16-test suite.

Maximum body-plus-charge-cue extraction is **732 cells per pristine Mote**:712 body cells, fourteen escape-wedge dots and at most six live-lane dots. A hypothetical48-Mote frame contributes35,136 such cells before other actors/scenery/UI. This is a geometry bound, not a frame-time measurement. There is no new texture or shader requirement. The update allocates at most ten sanctuary entries per actor and a shot queue bounded by the shared180/300 projectile cap; CPU/GPU costs under real encounter mixtures remain to be profiled.

## Shared physics, rewards and sanctuary ordering

The runtime replaces an eligible existing ordinary Dart, preserving its original hull, contact damage, tier and family. Eligibility uses the actual birth biome tile, seed and existing immutable `Brain.phase` bits; it consumes no new RNG and adds no saved state. Final V2 excludes stress-mode births in addition to arenas, bosses, champions, existing variants and ordinary QA spawning. The focused stress regression proves an eligible identity remains unconsumed and can still convert after leaving stress mode.

`update_motes` writes the pose before shared integration once and skips generic AI. Shared projectile dispatch, actual swept `PixelBody::hit_segment`, damage, shield/repulsor reflection, killed-entity removal and the existing ordinary reward branch remain authoritative. The actual-core tests exercise real player shots defeating a Mote for exactly five XP, the24th Dart reservation and an accepted bounty increment; actual hostile sweeps damage hull and remove ship cells; actual shield absorption triggers Nova reflection. Diagnostic defeats/retirement give no progress, and repeated kill callbacks do not duplicate rewards. Precise collision fixtures deliberately position targets on actual emitted rays; they do not establish natural dodge difficulty.

Remaining projectile capacity is reduced for commands from earlier Motes before the next actor updates. Already dispatched player/Riveter shots are counted, and `spawn_bullet` is still the final hard cap. Suppressed/saturated pulses are consumed rather than replayed. The shared-cap test uses two Motes with177 existing bullets and proves no overflow or debt replay.

The controller preguard cancels a displaced warning before commands; Game's existing post-integration guard catches subsequent inward motion. Current Haven/site circles are non-overlapping, both guards use the same order and protected radius+body radius+6, and Motes are excluded/purged from arenas. The independent test compares exact position, velocity and displacement across actual seeded circles, center fallback, tangency and both sides of the boundary. **No current parity defect was found.** These guards should not be replaced with an architecture rewrite merely because there are two checks. Future overlapping owned zones, arena filtering or changed sanctuary geometry would require a shared primitive or renewed parity review: the portable preguard removes inward velocity per circle whereas the current shared guard returns the final outward normal. Shared all-lifetime projectile sanctuary sweeps remain required even though emission also checks its first segment.

## Timing and camera admission

V1's observed20-tick second-pulse separation at60Hz was a real defect. Final V2 accumulates transient phase clocks asf64 and starts the pulse interval at the first scheduled pulse's actual tick. We independently ran the actual-projectile regression, rather than a duplicate timer simulation:

| Step | First/second emission tick from fresh charge | Actual minimum spacing |
|---|---:|---:|
| 1/120s | 132 / 174 | 0.350000018s |
| 1/60s | 66 / 87 | 0.350000018s |
| 0.05s | 22 / 29 | 0.350000005s |
| Declared irregular sequence | 43 / 57 | 0.353002999s |

All charge durations are at least1.1s, and no tick adds more than six rays. The original V1 result must not be substituted for these final identities.

At most five stable firing candidates are tested in the intersection of current and predicted follow-camera rectangles. The full conservative body-radius+35 cue envelope,2-unit guard and max(current shake,5) presentation allowance must fit; reduced motion removes the shake allowance. A0.125-world-unit corner inset addresses far-f32 addition/subtraction rounding. Approach is bounded, preserves actual physics, holds before charging and aborts on lost admission. It does not move the camera or weaken the offscreen gate.

The final candidate's twelve stationary actual-Game fixtures cover UI533×300,720×360,400×240 and320×180 at zoom0.6/1/1.6. All dispatch real projectiles after **112–205 ticks**. The narrow320×180 /1.6 fixture has a measured minimum physical gap9.4576 world units. These are cleared tier1 scenes, not crowded natural encounters. At that narrowest extent, the vertical firing rectangle is very thin; approach/pacing deserves human review.

The moving-player8-second maximum-zoom fixture produces **34 visible charge/attack ticks and zero emitting ticks at1/60**, versus48 visible ticks and two emitting ticks at0.05. The cancellation policy stays conservative, but encounter feel is step-sensitive and continuous movement may suppress this fixture's fire indefinitely. This remains a play-tuning question, not evidence that moving encounters universally succeed. Later unrelated collision displacement or larger same-tick shake can invalidate predicted framing; the tests do not prove every collision/impulse combination. Canvas admission also does not reserve semantic HUD/touch widgets, so browser review must check actual cue contrast and occlusion.

## Root integration gates

1. Apply the small runtime hooks to root's current source and preserve the accepted early-death helper. Copy final V2 modules/art/tests; adjust the sole atlas include to the chosen production asset path. Do not copy the old candidate lib over root's new lib or infer that the unchanged patch hash incorporates later modules.
2. Add `biome-mote` to the existing browser console help and scene whitelist if retaining the diagnostic route; current main.ts lists only `biome-riveter`. Keep diagnostic isolation and reset behavior. There is no new save/Hud field, radiation hazard or second spawning/reward system to integrate.
3. Run focused production tests plus required native/WASM checks after composition. Review WebGPU and WebGL2 actual warnings during zoom easing, crowded impacts, camera follow, pause/reload and reduced motion. Inspect pylon damage and shared lighting without restoring missing cells; verify real safe-zone retreat and actual ordinary reward collection.
4. Human combat pacing/telegraph readability and physical Pixel6a/iPhone12 performance remain Pending. Specifically observe a continuously moving player at narrow/max-zoom extents rather than approving only stationary admission. Record full-frame quads and measured frame-time under mixtures of Motes, other enemies, bullets and effects. No phone performance, browser rendering or deployment claim follows from this native review.
