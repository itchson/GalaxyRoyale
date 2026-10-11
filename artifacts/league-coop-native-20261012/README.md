# Native league presentation and scoped co-op validation

Normal dual-renderer build: **1b754da4e09fb2b8e6bbe123d8ac834750a97d99**, `codex/testing`. `tested-build.json` records its actual runtime/package hashes and compilation time. Later evidence and Studio commits do not constitute recompilation.

## Native visual work

Detailed 64-cell flight icons now use graphite/platinum material planes, recessed fittings and restrained gold hardware. Silhouettes, occupancy, accent/emitter masks and animation contracts are retained. The actual generator tests passed 14/14; native icon checks passed 12/12. `toolbar-webgl2-848.jpg` is the earlier b34d build; `toolbar-webgl2-1280-final.jpg` is the final 1b normal flight view. `raicho-modal-webgpu-1280-final.jpg` is a RAICHO menu capture, not an unobstructed flight toolbar.

The ringed planet replaces a rectangular shadow mask with a curved projected shadow and penumbra, using the existing quantized material shades. Exact native geometry, occupied cells, layer ordering, transforms and admission ceilings remain unchanged. This is a cached cosmetic approximation, not a physical gas or 3D lighting simulation. Native before/after software comparison is explicitly separate from actual browser captures.

Actual final WebGPU and WebGL2 planet captures use native PROFILE seed **871238449**, QA `WARP -843.7285 5313.427`, zoom **1.6**, viewport **1280×720**. `nebula-webgl2-final.jpg` uses the same seed, QA `WARP -19485.7676 -41897.0117` and zoom. QA warps suppress rewards and discard carried run materials; no natural exploration or performance claim. Animation times differ between captures. The prior irregular-cloud work remains integrated; all-seed, moving-combat and subjective user art acceptance remain open.

The compact co-op card retains 44px interaction targets at the tested 848×480 viewport. Final secret-free measurements show a 300×414 card, content height/scrollHeight both324, and all actual hosting controls plus disclosure inside its content bounds. The last disclosure ends413.600 while content ends415.400. Earlier clipped and fractional-overflow evidence remains retained. No private invitation screenshot was taken. Actual fresh unwarped dock INVITE reached stable HOSTING1/8; END SESSION restored0/8. A prior invitation after QA warping ended with a generic failure and is preserved separately. Native begin rejects diagnostic-warp worlds even at the dock; this is a source-supported explanation, not an observed native-error attribution.

## Co-op evidence boundaries

`captures/` contains 13 corrected factory-loadout reports from immutable normal **48bec87** artifacts: local two-client WebGPU connection/lifecycle and WebGL2 drive/fire/lifecycle, including distinct-session rejoin and final disposal. These are not current owned-equipment tests.

`owned-captures/` contains 11 reports from immutable normal **b34d02c** artifacts and helper15b9e0f. The visible WebGPU retry passed native owned-starter export/guest metadata commit, 360-step movement/fire, host end, distinct-session rejoin, guest leave and disposal. There were1,084 guest commits, max16 shared projectiles, two PCs created/closed per client, zero remaining requests/timers and equal native/persisted save hashes. The earlier off-screen Drive timeout remains a failure, with successful cleanup. Exact browser timer causation is not established. Both runs predate the later visibility guard; they do not validate that guard through browser execution.

Native per-pilot RENKO/Pulse Lens fifth-shot behavior is implemented and tested. HUGO delayed recharge and PAN movement match corresponding solo builds in native parity tests. The browser owned-starter run uses PAN/HUGO, not RENKO fifth-shot acceptance or advanced owned builds. Ordinary Pulse Echo still has a separate co-op gap, including omitted earned evolution metadata; `echo-plan.md` describes the inspected next implementation boundary.

## Checks and remaining gates

Final full Rust workspace: **805 passed, zero failed, eight explicit exporters ignored**. Final shell suite with actual normal artifacts: **546 passed, zero failed, one existing Windows skip**. Strict workspace/all-target and wasm32 Clippy passed; normal WebGPU/WebGL2, TypeScript, game/PWA and Studio builds passed. Raw logs and earlier intermediate logs are retained with their source scopes.

Owned-loadout WebGL2 RTC, RENKO fifth-shot browser ownership, equal-count damage over RTC, eight clients, Internet/NAT/TURN, shared rewards/missions and complete automatic ability parity remain pending. Physical phone targets and30-minute performance soak remain unmeasured. One normal game preview4173 is retained; temporary QA4189 is stopped and regular4176 signaling is restored. No main merge or release-completion claim.
