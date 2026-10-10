# Two-phase browser recovery handoff

Production ownership was limited to `crates/game_web/src/browser.rs`. The frozen earlier gap review remains unchanged. No renderer, Game, manifest, distribution or publication change was made by this handoff.

API:

```ts
prepare_renderer_recovery(canvas: HTMLCanvasElement): Promise<RendererRecovery>
galaxy.install_recovery(candidate: RendererRecovery): void
```

`RendererRecovery` is an opaque owned Rust renderer. Async preparation is a free function and retains no Galaxy borrow or Game reference. Synchronous installation consumes the candidate, rejects a different canvas through a normal Result error, resizes the replacement to the canvas's current dimensions, resets the boss presentation bounds and updates the Game's visible world extent. The previous async `Galaxy.recover` method was removed. Profile, entities, damage occupancy, accumulator, UI scene, pause intent and zoom are not replaced by installation. The shell owns pausing, latest resize and modal/lifecycle intent.

The shell must not draw or resize the old renderer while preparing a replacement on its canvas, and must apply the latest intended dimensions after installation. Only one recovery request should be pending. A candidate is consumed exactly once and is not cached or retried. Calling a method with an already consumed/freed wasm-bindgen handle is outside its ownership contract: an isolated negative exploration threw a null-pointer exception and poisoned the target's active mutable borrow because JavaScript throws do not unwind Rust. Normal main integration does not reuse candidates, as confirmed by the parent. This is not a new supported misuse path or a live-game observation.

Validation actually completed:

- `cargo clippy --locked -p game_web --target wasm32-unknown-unknown --no-default-features --features webgpu -- -D warnings` — exit 0.
- The same command with `--features webgl` — exit 0.
- `rustfmt --edition 2024 --check crates/game_web/src/browser.rs` — exit 0.
- `node work/gpu-recovery-two-phase/binding-check.cjs` — exit 0 using the real locked wasm-bindgen 0.2.129 wrappers and matching installed rlibs. Independent preparation allows reads/writes during the await; synchronous consumption zeroes the generated candidate handle; rejection preserves the previously installed value and live state. This probe creates no Game, canvas or GPU. It does not assert disposal timing, canvas identity or real backend restoration.

The parent owns six actual shell-AST transition checks, full builds and subsequent browser smoke tests; those are not rerun or claimed as this specialist's evidence. Actual context/device callback, foreign canvas, restored world pixels and lifecycle browser acceptance remain separate gates. No new QA failure hook is present in the normal module.

`build-binding-probe.ps1` reproduces the isolated binding probe without adding a manifest or dependency. `frozen-hashes.json` pins the production browser change, Cargo.lock, installed matching rlibs and all owned evidence. The original acceptance proposal is `../gpu-recovery-gap-review/REVIEW.md`.
