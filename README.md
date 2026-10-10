# Galaxy Royale

A single-player neo-retro space shooter built from portable Rust simulation and destructible pixel fields. The game and local production dashboard are separate applications.

## Run locally

Requirements: Rust with `wasm32-unknown-unknown`, Node 24+, and pnpm. Windows PowerShell is supported.

```powershell
pnpm install --frozen-lockfile
pnpm build:wasm
pnpm dev
```

The WASM script installs the exact `wasm-bindgen-cli` version in `Cargo.lock` when required, then builds separate WebGPU and WebGL2 bundles.

- Game: http://127.0.0.1:4173
- Mission Control: http://127.0.0.1:4175
- Local production API: http://127.0.0.1:4174

Use `pnpm build` for the complete release build, including both WASM backends, TypeScript checks, game assets, the game-only service worker, and the dashboard. The deployable game is `dist/`; the dashboard is `apps/studio/dist/` and is never included in that release.

## Fly

Desktop: left click sets a persistent destination; hold left to update it. Hold right to fire. Scroll the mouse wheel to smoothly zoom from 0.6× to 1.6×; aiming uses the current unshaken world projection, while pixel UI stays at screen size. WASD/arrows are available too; Space does not fire. Escape opens settings. Fire inside Haven opens dock services; E also opens services or interacts with a nearby character.

Mobile: rotate to landscape. The left joystick steers and applies proportional thrust; the right button fires independently. Release the stick to coast with a short throttle fade. Fullscreen and separate music/effects volume controls are available.

Haven protects the ship within 240 units. The asteroid belt begins at 700; combat begins beyond 1,400. Moving farther increases danger. Darts weave, Bastions strafe and fire, and Coils use segmented bodies. Twenty-four kills in one family queue its first boss; every two defeats of a boss generation queue its successor, through generation five. All fifteen encounters have authored pixel silhouettes, distinct attacks and charging/attack/exposed-core phases. One ordinary boss is active at a time. Retreat, death and reload preserve outstanding encounters and surplus kill credit.

Collected offensive, defensive, and passive points survive death. Return to Haven to buy one of six basic upgrades for ten matching points per rank, up to five ranks. Rank gates require ship levels 1/3/5/8/12; each ship earns its own XP. The Scout starts at 100 units/second and 260 acceleration, gaining 2.5% drive per level through level 12, plus purchased drive ranks. Family bosses can unlock Pulse Echo, Repulsor Nova, and Ion Wake; the fourth eligible defeat guarantees the family signature. Death restores the ship at Haven and shows an expedition report with Retry and Return to Title; arrival never automatically opens upgrades. Crafting materials occupy a 16-unit hold and bank at Haven or on death. Live enemies, uncollected loot, and damaged asteroids are transient.

Eight seeded arenas contain champions with authored pixel hulls and distinct attacks. Defeating a champion awards its patch and blueprint once. At Haven, crafting costs ten matching points and six stored materials; owned ships can be selected freely. All eight patches reveal the Galaxy Royale gate: eight player duels and 28 explicitly simulated NPC pairings form a nine-entrant round robin. First place unlocks the Sovereign and credits once; a losing season can be retried. See `docs/CAMPAIGN.md` for rules and checkpoint boundaries.

A seeded, jittered grid samples six biomes: Haven Expanse, Amethyst Drift, Aurora Reach, Iron Belt, Radial Wastes, and Cryo Rift. Protected discovery sites beyond Haven offer courier deliveries and enemy-family bounties. Accept an event, fulfil it, and return to its source to claim three matching points and a unique relic. Each relic adds one maximum hull, capped at twenty. Discoveries, quest progress, and claimed rewards persist in schema-3 saves; old saves migrate without losing progression. Courier cargo is retained on death, and reaching Haven after respawn or reload completes its delivery step. Biomes vary enemy-family pressure and cosmetic palettes; distinct additional biome enemies and radiation hazards remain future content.

Title, menu, HUD, sliders, toggles, and buttons render through Rust pixel quads. Generated artwork is converted into hard-alpha, palette-indexed cells; it is never drawn as a sprite texture. RoyaleMono uses authored 5×7 glyph bodies with fixed character spacing and grouped transforms. The invisible semantic DOM retains accessible controls and pointer capture; The generated RoyaleMono TTF matches the engine glyph metrics for semantic layout; Tiny5 and its OFL license remain bundled as fallback. Settings are available from the title and pause menu, with separate audio buses, camera shake, reduced motion, and cosmetic density. The Journal lists persistent events.

The compact flight HUD shows biome and threat, point totals, an active boss meter, hull/shield when away from the dock or damaged, and a home locator outside Haven. Backtick opens a black drawer at the top, sliding down over 180 ms; Escape or backtick closes it. Reduced motion disables the slide. `HELP`, `STATS`, `PROFILE`, and `RENDERER` report actual runtime data, `CLEAR` clears its output, and Up/Down recalls commands. Opening the console suspends flight input and simulation; closing restores the preceding play/pause state after the drawer finishes closing. `RECOVER` requests graphics recovery. The HUD hides while console or dialogue modals are open.

Press E at a discovery sanctuary to open its contextual conversation at the bottom, with an authored pixel portrait, the site's actual event description and an eligible accept/claim action. The panel pauses flight and restores the previous play state on close. Accepting an event and retaining its quest through offline reload were verified in the browser. Native tests verify courier/bounty rewards claim once and survive reload; completing and claiming the reward through this panel still needs a browser play test.

Pixel materials provide per-object light response and emission. A maximum of eight camera-relative local lights shades each occupied cell while UI and background materials stay unlit. Cells retain flat colors and nearest presentation; effects do not introduce a blurred sprite layer. Desktop browser acceptance is recorded for the console, HUD and material lighting, WebGPU and WebGL2 startup, initializer fallback, and device-loss recovery that retained expedition state. Physical device acceptance remains separate.

## Architecture

- `crates/engine_core`: packed cell occupancy, affine hierarchy, swept cell collision, local destruction, bounded pixel particles, reusable render batches.
- `crates/game_core`: `hecs` object ECS, fixed-step physics, deterministic streamed chunks/regions, protected sites, quests, enemies, bosses, loot, progression, compiled occupied pixel artwork.
- `crates/game_web`: `wgpu` instanced quads and low-resolution nearest presentation; browser WASM interface, fixed 60 Hz accumulator, interpolated extraction, device recovery.
- `apps/game`: pointer controls, HUD, procedural Web Audio score/SFX, versioned IndexedDB snapshots, import/export, and PWA lifecycle.
- `apps/studio`: localhost Node API, atomic versioned project document, six-state board, reviews, builds, decisions, activity, feedback queue, and agent CLI.

See `docs/CONTRACTS.md`, `docs/ART_DIRECTION.md`, `docs/ENGINE.md`, `docs/DEBUG_CONSOLE.md`, `docs/CAMPAIGN.md`, `docs/NARRATIVE.md`, `docs/ROADMAP.md`, and `apps/studio/README.md`. `production/project.json` contains actual recorded assignments, acceptance criteria, reviews, and evidence. Only the running API writes it. Queued feedback is read during active development sessions; the dashboard does not launch autonomous sessions.

## Validate

```powershell
cargo test --workspace --locked
cargo clippy -p engine_core -p game_core --all-targets -- -D warnings
pnpm test
pnpm test:wasm-contract
pnpm typecheck
pnpm build
```

Add `?qa=1` to enable explicit test scenes and the read-only `window.galaxyQA.report()` capture. The scene toolbar and performance overlay are hidden by default: add `&toolbar=1` or `&metrics=1` to show them. In QA mode, the backtick console accepts `SCENE dock|belt|combat|site|biome-riveter|boss-dart[-1..5]|boss-bastion[-1..5]|boss-coil[-1..5]|champion-0..7|tournament|stress|die`; close it to resume a running session. The boss aliases without a suffix select generation one. `?renderer=webgl2` forces fallback; `?qa=1&fail-webgpu=1` rejects the first initializer and tests automatic fallback. `?qa=1&scene=stress` exercises at least 80,000 visible quads, 160 enemy heads, 300 projectiles, and 2,000 particles. `?qa=1&touch=1` exposes mobile controls for desktop layout checks; this does not validate physical touch hardware. Diagnostics do not award free progression. The console's `STATS` command also reports last rendered boss panel bounds.

Desktop packaged-game offline reload and launch were verified after stopping the local preview server: the title, WASM runtime and flight loaded from the service-worker cache. Phone-specific installation, updates and lifecycle acceptance remain pending.

The WASM contract check builds isolated QA packages under `work/` and validates eight actual Rust fresh/champion/generation/legacy-boss snapshots through the TypeScript schema and IndexedDB transactions for both backends. Optional Rust values serialize as explicit `null`; a fresh ship's empty run history must remain a valid save. QA fixture exports are excluded from the shipped bundles.

## Validation boundaries and next content

The Iron Belt includes the original occupied-cell Riveter with destructible barrels and a locked triple burst. Boss health/phase panels now adapt around projected cores, ships and fixed controls, including compact minimal telemetry. See `docs/BIOME_ENEMIES.md`, `docs/BOSS_HUD.md` and `artifacts/hud-riveter-browser-validation.json` for implementation and exact current acceptance limits. Radiation enemy work remains an isolated prototype until integrated and tested in production.

All fifteen family bosses and five-generation promotion are implemented. Independent native projectile fights cover all fifteen, with full-profile diagnostic isolation and a naturally queued fifth-generation retreat/reload/death/retry. Human balance, a complete natural progression playthrough and full browser campaign acceptance remain pending. Invite-based co-op, additional biome enemies, and radiation hazards remain backlog. Accepted future scope also includes player-owned docks with upgradeable protection radii and arenas, plus a separate multiplayer battle royale: a compact seeded circular universe, equally spaced zones, arena sieges and a last-player-standing outcome. Its edge crossing must place a ship at the antipodal edge while preserving heading and speed and rebasing the camera. A portable, opt-in `CircularTopology` primitive implements and tests that motion/rebase behavior; the solo game does not use it. Owned docks, arena modes, seeded multiplayer zones, network authority and co-op remain future work. See `docs/ROADMAP.md` and Mission Control for the backlog and unresolved rules.

Disconnected asteroid islands do not yet become independent rigid-body fragments; removed cells become bounded debris. Simulation coordinates are bounded to a practical range; extreme exploration requires a future floating-origin migration.

Physical Pixel 6a/iPhone 12 performance, sustained 30-minute mobile soak, Safari compatibility, and device-specific audio/PWA lifecycle remain pending until measured on those devices. Desktop measurements are labelled separately in `artifacts/` and Mission Control. Cosmetic density should be reduced before gameplay budgets change.

The optional offline ACE-Step experiment is not required by the game. No generated audio candidate has been accepted. The shipped score and effects are authored procedural Web Audio; the generated concept sheet is a direction reference, not a gameplay screenshot.
