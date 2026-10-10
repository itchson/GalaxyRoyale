# Root integration review — focus, boss overlay and Riveter

Read-only source review of `game_core/src/lib.rs`, `game_web/src/browser.rs`, `boss_overlay.rs`, the pixel UI/projection bridge, and final Riveter/console hooks. Production module and assets remain frozen. No compiler, test, browser or physical-device run was performed in this review; root owns both-renderer builds and presentation acceptance.

## Gameplay and progression separation

No new gameplay/progression mutation was found in `Game::render_focus` (`lib.rs:710`) or `BossOverlay::update` (`boss_overlay.rs:33`). Focus reads motion/actor/encounter data through an immutable game reference; it does not call RNG, collisions, damage, events, rewards, revision updates or profile sanitation. It uses the same previous/current position and shortest-heading interpolation as actor extraction, then the same `actor_transform` and authored body-local core metadata. Ordinary phase epochs derive from the actual encounter cycle/phase. Champion opponents deliberately use their center as a protected proxy, radius zero plus the layout's screen-space minimum, and epoch zero; no new vulnerable champion core is invented. Missing opponents return no focus and encounter IDs include ECS entity generation bits.

`browser.rs:144–239` advances presentation zoom/time separately, ticks gameplay only while unpaused, extracts world/focus with the same alpha and supplies the actual enabled-shake camera to both overlay and renderer. Layout output changes only the UI frame and diagnostic panel bounds. The pre-existing `update_world_viewport` path remains a real coupling: visible zoom changes `Game.viewport`, and that extent affects spawn distance, pursuit range, culling and Riveter warning admission. This is the existing visibility-based design, not a side effect of adaptive panel placement; it does not change world destinations, hull, queues or saved rewards.

## Projection and lifecycle

The adapter projects through the renderer's f32 `CameraView`, recenters projected points, and calls the layout with zero camera/unit zoom. Core/player radii are multiplied by current eased zoom once before screen-space padding/floors. Insets are neither zoomed nor applied twice: returned full-screen bounds subtract left/top, while the UI camera plus reduced viewport restores that safe-local origin on the full renderer surface.

Native biome/currency/meters/radar and touch hit rectangles are reserved. Semantic text/buttons use clipped group-local rectangles and then the same group translation as `ui_scene` rendering. Traversal limits match the renderer's 128 groups/512 nodes; the reservation array caps at 64 and hides the panel on overflow. Invalid or genuinely crowded inputs return no panel rather than reuse an unsafe placement. That is a safety behavior, not proof of readable essential-HUD fallback on every tiny viewport.

Encounter replacement, no boss, pause transitions, non-flight UI, resize, diagnostics, return-home and renderer recovery reset the layout controller. `last_boss_panel` intentionally survives ordinary menu opening as last-visible diagnostic telemetry; the console labels it `LAST BOSS UI`, and explicit home/diagnostic transitions clear it. Input continues mapping clicks against the unshaken interpolated `Hud.camera` and the current visible world extent.

## Riveter hooks and diagnostic route

`lib.rs:1293` calls `maybe_riveter(entity, regions::at(seed, spawn))` after the successful normal family-weight spawn. Thus weights still use the player biome, while variant identity uses its actual birth biome/grid. Boss spawning is separate. Tick dispatch invokes the specialized controller once; generic enemy updates skip `Riveter`; actor extraction dispatches `draw_riveter` with the same live damaged body and interpolated pose. No Profile/Hud/save schema field is added.

The browser console help and exact scene whitelist contain `biome-riveter`; the scene command is gated by QA mode. Root's URL startup path also dispatches `scene` only in QA mode, with unknown commands safely ignored by Rust. Actual QA mode is the presence of the `qa` query key, rather than checking its value equals `1`; the help's `?qa=1` is its conventional URL. The small QA toolbar intentionally retains only its older subset, so the new scene is reached through console or URL.

The Riveter setup uses an actual deterministic Iron Belt location and the existing whole-scene QA isolation flag. Its native real-projectile defeat/no-reward and standard kill/XP/bounty tests were already reported in `work/riveter-handoff.md`; this source review does not elevate those results into browser/human balance acceptance.

## Current integration dependency and pending limits

At review time, production `game_web/src/lib.rs:4` declared `pub mod boss_layout`, but neither `src/boss_layout.rs` nor `src/boss_layout/mod.rs` existed. Root was notified. The source controller reviewed here is still `work/boss-hud-layout/src/lib.rs`; copying/integrating that production module and passing root's builds remain required. This is a concrete current source dependency, not a test failure observed by this worker.

The existing prototype QA note records a deliberate worst-case exhaustive search cost around 9 ms on native desktop for 64 stripes at 4096×2160; I did not remeasure it. The normal fixed-anchor path is much smaller, but array/loop bounds alone do not establish a mobile frame budget. A supported tiny touch viewport can also be overconstrained and lose the boss panel; essential-HUD reflow/minimum viewport policy and representative browser captures remain root-owned acceptance questions. Whole armor/tail protection, human Riveter dodge timing, naturally tier-scaled Iron Belt difficulty and physical Pixel 6a/iPhone 12 performance were not established here.
