# WebGL lifecycle handoff

Owned production files are only `apps/game/src/webgl-lifecycle.ts` and `scripts/test/webgl-lifecycle.test.mjs`. No main, Rust, renderer, manifest, distribution or publication edit was made by this handoff. The parent owns integration and real browser execution.

## Primary specification

Khronos requires cancelling the loss event to permit restoration. The extension's restoration request queues work; the context is usable when the restored callback runs. Calling restoration before the loss event has completed is invalid. The extension can request restoration only for a loss initiated through its loss API. Previous GPU objects and state require reconstruction after restoration. These constraints come from the [ratified WEBGL_lose_context specification](https://registry.khronos.org/webgl/extensions/WEBGL_lose_context/) and [WebGL context event specification](https://registry.khronos.org/webgl/specs/1.0.3/#5.15.2), inspected on 2026-10-11. The separately inspected latest WebGL text is an editor's draft; it is not treated as a ratified revision.

## Exact integration contract

```ts
const lifecycle = bindWebglLifecycle(canvas, {
  lost(event: WebGLContextEvent) { /* shell suspends input/audio/presentation */ },
  restored(event: WebGLContextEvent) { /* shell rebuilds renderer resources */ },
});
// lifecycle.lost, lifecycle.restoreReady, lifecycle.disposed are readonly.
// lifecycle.dispose() removes both listeners and pending readiness work.

const probe = createWebglLossProbe(canvas, lifecycle); // optional QA use only
probe?.requestLoss();
// In a later task after the native lost event completes:
probe?.requestRestore();
// Do not resume/render until native restored + resource reconstruction settles.
```

Bind once to the successfully selected WebGL2 canvas. Binding never acquires or creates a context, and has no Game or recovery ownership. Loss is cancelled before the shell callback. The loss flag is cleared only by the native restored listener. A zero-delay **task**, rather than a microtask, makes `restoreReady` true after the entire loss dispatch; disposal or earlier restoration cancels this pending readiness work.

Call the optional probe factory only after a working WebGL2 renderer has claimed that canvas: `getContext("webgl2")` then returns its existing context. An unrelated unclaimed canvas is outside the helper's calling contract. Unsupported context/extension returns null. Extension exceptions propagate for truthful reporting. The helper neither dispatches events nor changes lifecycle flags. Request statuses are `requested`, `disposed`, `already-lost`, `not-lost`, `not-requested`, `await-lost-event` and `already-requested`. A requested restoration does not establish success; wait for the actual restored callback. The helper refuses to claim restoration of a natural loss that it did not request.

The parent will guard frames and GPU resize while `contextLost`, defer retry if independent renderer preparation is still pending, dispose a stale prepared candidate and rebuild once after restoration. The parent also retains current launch/menu/dialogue/death/visibility intent. Clearing graphics failure or resuming input solely because `requestRestore()` returned `requested` would violate this contract.

## Executed checks and scope

`node --experimental-transform-types --test scripts/test/webgl-lifecycle.test.mjs` passed **11/11**. Tests use Node EventTarget/extension fixtures and explicitly do not claim native WebGL loss, restoration, GPU resources or browser rendering. They cover cancellation-before-callback, later-task readiness, restoration clearing, stale timer cancellation, idempotent disposal/removal, independent replacement canvases, unsupported extension, real extension method dispatch without fabricated lifecycle changes, same-callback restoration rejection, natural-loss refusal, extension error propagation and repeated cycles.

`node node_modules/typescript/bin/tsc --noEmit --strict --target ES2022 --module ESNext --moduleResolution bundler --lib ES2022,DOM --skipLibCheck apps/game/src/webgl-lifecycle.ts` passed, as did formatting of both owned files. No dependency was added. Full shell/normal build and GPU verification belong to the parent.

Actual acceptance remains pending: native trusted lost/restored events, extension availability and support, whole renderer reconstruction, preserved live gameplay/checkpoint state, latest resize and modal intent, actual restored pixels/materials/control behavior, and physical-device lifecycle. Intentionally using the extension tests its specified mechanism; it does not reproduce every OS or driver loss. No helper code synthesizes a browser event or replaces navigator/context capabilities.
