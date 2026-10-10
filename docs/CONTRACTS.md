# Runtime integration contracts

- PixelInstance is 16 bytes: local.xy, little-endian RGBA, transform index. GpuTransform is 32 bytes: position.xy, basis_x.xy, basis_y.xy, light response, emission. Up to 128 transforms share a 4,096-byte table. Materials clamp finite values to 0..1.
- PixelBody separates occupancy from color. Local swept hits test surviving cells; damage returns removed centers/colors. Exact affine composition preserves shear; FrameBuilder reuses capacity.
- Input contains optional joystick/new world destination and fire. Destination persists in Rust; no joystick means release. Desktop fire is right hold. Mouse mapping excludes shake.
- Game provides tick, render_into, HUD/events/profile, purchase, interact_site, density, pause and explicit QA scenes. At most eight interpolated world lights reach the renderer.
- Schema 3 profiles store revision/seed, currency/ranks/unlocks, kill/boss/pity/queues, discoveries/quests/relics and the champion campaign, per-ship XP, selected/crafted ships, bounded cargo/stash and last run. Storage migrates schemas 1/2, validates bounds, compares revisions and retains a previous snapshot. Optional Rust values cross the shared WASM serializer as explicit null. Live entities/drops/damage are transient.
- The five-generation content extension adds boss_generations[family][generation0..4]. Earlier missing/all-zero family rows preserve aggregate boss_kills as generation1. Nonzero rows obey previous-tier two-defeat promotion; boss_kills remains the u32-saturated row sum. Base pending flags reserve generation1 only. Same-schema normalization rewrites current/previous snapshots transactionally without changing reward credit or revision.
- Ordinary BossHud carries family, generation1..5, bounded phase text and telegraph state. Champion/league duels use generation0. Family progress exposes defeated and queued counts separately from the live encounter, so a completed kill cannot be confused with a pending/retried fight.
- Galaxy batches HUD/events/changed profile/counts/CPU timings across WASM. Browser APIs manage viewport/UI/insets/touch/settings/recovery.
- UI groups have viewport-relative parents; nodes have local rectangles/text/font/value/interaction/clips. Semantic DOM supplies layout/input/accessibility; Rust renders RoyaleMono and real pixel bodies after world layers. Console/dialogue share this contract.
- WebGPU/WebGL2 share gameplay. Initialization uses a fresh fallback canvas; recovery retains Game. Permanent recovery failure latches until an explicit retry.
- Studio mutations use its localhost service: atomic writes, expected revisions, append-only records and criteria/evidence requirements. PWA caching excludes Studio/API.

Circular arena wrapping is opt-in. Future authoritative multiplayer needs camera/history/destination rebasing and split collision seams. See UNIVERSE_TOPOLOGY.md and ROADMAP.md.

