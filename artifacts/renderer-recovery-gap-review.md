# GPU fallback and recovery acceptance gaps

Source review of the published 95d0a4d39116971e generation, before the separately authorized recovery refactor. Production was unchanged during this review. This is a proposal and binding-contract proof, not a live browser failure reproduction or phone acceptance.

## Concrete findings

1. Startup already retries WebGL2 after a WebGPU error and replaces the canvas between attempts. However, `?qa=1&fail-webgpu=1` throws before module import or Rust initialization. It proves the shell catch/fallback path, not failure after a WebGPU canvas context has been acquired.
2. `Renderer::lose_device()` destroys the device **and directly sets** the loss flag. The device-lost callback is registered, but this control does not prove that the callback alone detects loss. An explicit `RECOVER` proves reconstruction, not automatic loss detection.
3. `Galaxy::recover(&mut self)` retains an exclusive wasm-bindgen borrow while awaiting `Renderer::new`. RAF is gated by `recovering`, but resize, visibility/profile saving, modal, settings and console callbacks can still invoke Galaxy methods. An isolated probe using the actual locked wasm-bindgen **0.2.129** confirms both readonly and mutable methods throw the recursive-use/unsafe-aliasing exception during such an await. This establishes the binding contract and a concrete source integration risk; it is not an observed failure of the running game.
4. No native WebGL context-lost/restored event listener is present in the shell. A WebGL loss test must exercise a real context event and renderer behavior. Calling the normal loss flag or dispatching a synthetic event is insufficient.
5. The recovery catch currently displays the original trigger error, dropping the actual reconstruction rejection. Successful recovery is same-backend only. Cross-backend migration of the live Game is outside this small change.

## Minimal implementation proposal

Keep the renderer and game ownership model. Replace the async exclusive method with an independent async `prepare_renderer_recovery(canvas) -> RendererRecovery`, followed by synchronous `Galaxy.install_recovery(candidate)`. The opaque candidate owns only the newly built Renderer, can be consumed once, and installation validates canvas identity. No Game reference crosses the await. The shell retains its one-at-a-time `recovering` flag, suspends audio/input, defers GPU resize until installation, then applies the latest dimensions and current launch/menu/death/visibility intent. A failure leaves the existing Game and checkpoint available and reports the actual rejection. This two-phase API was accepted by the parent for a separate scoped implementation after this review freeze.

For missing failure coverage, add at most two compile-gated QA hooks in a future reviewable candidate:

- A one-shot error immediately after `create_surface` succeeds in `Renderer::new`. Enable it after module initialization and before `Galaxy.create`. This tests real canvas replacement and fallback after context acquisition. Keep the current earlier shell failure as a separately named test.
- A callback-only device destruction control that calls `device.destroy()` without directly setting the loss flag. Verify the registered real device-lost callback and exactly one subsequent recovery. Intentional device destruction still does not simulate every driver or OS failure.

These hooks must be absent from normal release exports. No navigator replacement, fake context event, shader corruption, driver reset or general failure framework is needed. Neither hook is implemented by this review.

## Actual coverage and missing gates

| Evidence | What it establishes | Remaining gap |
| --- | --- | --- |
| Current production source and both backend compilation | Error handling and recreation code exist | Actual backend initialization/loss event behavior |
| Existing headless WASM checkpoint contract | Real serializer/checkpoint binding | No renderer or GPU initialization occurs |
| Older fallback/recovery images | Historical archived captures | No exact-current loss callback attribution |
| `work/desktop-stress-current/webgl2-recovery-*` | Explicit console RECOVER and limited pre/post state retention | No device/context loss induced |
| Peer-owned final 95d0 responsive session | WebGPU adapter unavailable, then actual WebGL2 through RENDERER | Availability-driven startup fallback; no forced failure or loss action |
| Owned deferred-Promise probe | Actual 0.2.129 async exclusive borrow rejects overlapping reads/writes, then releases access on settlement | No Game, DOM, GPU, lifecycle or disposal acceptance |

No renderer recovery tests were found in the shell suite. Storage recovery, AudioContext recovery, PWA cache tests and ordinary InputFrame tests exercise separate contracts. The source handles timeout/occlusion by skipping and outdated surfaces by reconfiguration; these are not device-loss tests.

## Repeatable acceptance matrix

Use a fresh, disposable QA profile on a foreground browser tab and pin module/assets/cache/backend. Record the actual canvas, zoom, viewport, profile and live encounter before each test. Do not infer blank-frame failures from hidden in-app tabs.

1. **Startup:** early shell WebGPU rejection; separately post-surface rejection; adapter unavailable; both backends unavailable. Each appropriate case produces one fallback, a replacement canvas, a working renderer and input bindings; total failure leaves an understandable startup screen and does not overwrite the checkpoint.
2. **WebGPU callback:** callback-only destruction during paused and active play. Capture callback reason, first failed presentation and successful reconstruction. Compare paused live Game state, damaged pixel occupancy, boss generation/phase and serialized progress. An active loss may follow one simulation step because `step` advances before `draw`; do not incorrectly require zero elapsed simulation before detection.
3. **WebGL real context:** use the existing canvas's `WEBGL_lose_context` extension when available. Record actual lost/restored events and `isContextLost()`. Test reconstruction while still lost, restoration and an explicit retry. If extension/restore is unavailable, mark the gate pending; do not replace it with a synthetic event. A reload with retained checkpoint is an acceptable separately stated fallback where automatic restoration cannot be demonstrated.
4. **Await/lifecycle ordering:** while preparation is deliberately pending, resize, hide/show, open/close console/menu/dialogue, return to title and enter death/retry states. Reads/save remain usable; no unsafe-borrow exception, ghost held input, unintended resume or duplicate reconstruction. The latest safe viewport and current intent determine settlement behavior.
5. **Failed reconstruction:** actual rejection reason is visible, RAF does not loop failed reconstruction, semantic UI/checkpoint remains usable, and a retry or reload is truthful. Test two overlapping requests and a stale/foreign candidate if supported by the chosen API.
6. **Rendered restoration:** at zoom 0.6 and 1.6, normal and narrow touch extents, verify world/UI buffers, materials, lighting, damaged holes, controls and cursor aim through actual frames. Snapshot equality alone does not establish visible restoration or correct hitboxes.

Physical Safari/Android lifecycle, background eviction, thermal behavior and genuine OS/driver loss remain pending. Intentional `GPUDevice.destroy` and a WebGL extension cover defined mechanisms, not all failures. No frame-time, phone, long-soak or human-playability claim is made here.

## Owned executable evidence

`borrow_probe.rs` and `borrow-check.cjs` instantiate the actual matching wasm-bindgen binding. A deferred Promise keeps an async `&mut self` method pending; both a read and write are rejected, then succeed after settlement. `borrow-result.log` records a clean Node exit 0. The probe does not instantiate the renderer or Game, and deliberately does not assert object disposal timing.

Build with the already installed matching release rlibs using `build-borrow-probe.ps1`; run `node work/gpu-recovery-gap-review/borrow-check.cjs`. This work directory has no shared manifest or added dependency. Evidence hashes pin the reviewed sources, installed dependency sources, historical records and owned probe.
