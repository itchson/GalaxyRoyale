# Adaptive boss interface

Boss health and phase text are pixel bodies in the Rust screen-space UI layer. They use RoyaleMono, fixed logical pixel sizes and the same packed quad renderer as the world. World zoom does not scale this layer.

The browser adapter projects interpolated player and reactor positions through the renderer's actual `CameraView`, including presentation shake and the current eased zoom. It reserves safe areas, hull/cargo, compass, touch hit regions and clipped semantic buttons/text. Pointer course selection continues using the camera without shake. Layout changes never mutate the simulation, saves or collision geometry.

The allocator first tries a 220×40 row beside the boss, then smaller 168, 140 and 100-pixel rows. Compact names and authored attack abbreviations retain the complete phase cue and health meter. It prefers stable placements, holds optional moves briefly, and moves immediately if an existing row becomes unsafe. Reactor clearance is at least 32 pixels with additional radius/padding; player clearance is at least 24 pixels.

When space becomes tight, optional biome, currency and home-bearing telemetry is omitted for that encounter. Hull and touch controls stay visible. The choice is latched to avoid flicker and resets on encounter completion, resize, modal changes and graphics recovery. Below a 220-pixel safe width, the interface starts in this minimal mode. Flight/touch painting and allocation share a 180×140 safe-area minimum; below that limit no invisible allocated row is reported.

Reservations are bounded to 64 rectangles. Exact-geometry no-fit results and successful free-search placements are cached, with fresh clearance checks. A genuinely overconstrained viewport can still have no safe row; the allocator never places it over protected targets. Two independent 320×180 fixtures with fractional asymmetric insets and minimum zoom remain conservatively overconstrained. A circular boss reactor can be offscreen, and fixed hull/touch controls do not dynamically move around world actors.

Forty-two standalone layout tests include integer tangency, a brute-force origin oracle, safe areas, hysteresis and cache invalidation. Six production adapter tests cover the real f32 projection, semantic reservations and critical controls. The independently reproduced 200×180 safe-area case now emits 1,011 actual HUD quads and 266 touch-control quads; the prior 220-pixel paint guard incorrectly emitted zero. The repair has a direct regression.

Independent native extraction checked 30 placed cases and 13,030 actual boss quads at both zoom limits, fractional insets and far cameras. At approximately one million world units, f32 projection can deviate by 0.0625 logical pixels from the ideal allocated edge; the measured protection padding still cleared physical cores and ships. These CPU checks do not prove unlimited-coordinate precision or phone performance.

Representative final-package browser captures show all three fifth-generation families on both backends, separated reactor/player/HUD placement, 720×360 landscape layouts and a real 100×40 compact row with visible touch controls at 320×180. These are reward-isolated diagnostic scenes, not full natural campaign acceptance or physical multitouch. The earlier Coil screenshot with its head below the hull panel is retained as a limited observation rather than an exposed-core proof.

`STATS` in the backtick console reports `LAST BOSS UI x / y / width / height` from the last rendered flight frame. The console pauses combat; this is last-frame evidence, not a fresh combat position while paused. See `artifacts/hud-riveter-browser-validation.json` and the independent review files for exact source/output hashes and remaining limits.
