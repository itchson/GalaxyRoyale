# Temporary co-op acceptance tools

Without `--dist`, preparation retains the historical transport-only page. Its declared admission fixture does not execute Game authority.

With an explicit absolute normal `dist` directory, preparation copies both actual content-addressed bindgen bundles, verifies their four-file content hashes and WASM header, and records individual SHA256/byte pins. It compiles the checkout's actual controller, ports, transport, wire, signaling and SaveStore with pinned TypeScript 5.9.3. It never imports the shipped main, rewrites production source or fabricates native replies. Use a fresh stage; an existing manifest is immutable.

```powershell
node scripts/qa/co-op-transport/prepare.mjs --stage work/co-op-transport/wasm-stage --dist C:/absolute/normal/project/dist
node scripts/qa/co-op-transport/server.mjs --stage work/co-op-transport/wasm-stage --port 4189 --max-ms 600000
```

Root owns launching/stopping this listener and browser actions. It binds loopback, fails if the port is occupied, stops on SIGINT/SIGTERM or its finite deadline and never stops an unrelated process. Stop any owned signaling service before this tool: it embeds the actual ephemeral service on the same listener. Do not change the preparing checkout's source while using the stage. The server validates source and staged byte pins before listening.

Open `http://localhost:4189/`. The page creates two real client iframes on `localhost:4189` and `guest.localhost:4189`; each loads the selected actual native bundle and creates its own Galaxy/canvas. The service is `127.0.0.1:4189`, with the two exact page origins allowlisted. The host and guest have separate origin-scoped `galaxy-royale` IndexedDB stores. Existing valid QA saves are retained; empty stores receive actual newly registered, unprogressed licenses. Profiles never cross RTC. Capabilities cross only the private parent/client message route with exact window/origin checks and never enter reports or displayed text.

Select a backend, then use Initialize, Create + join, Drive + fire, Host end, New session, Guest leave, Read local saves and Dispose. The page reports separate real transport-ready, native admission and successful native replica commit observations. Rejoin requires a new complete commit rather than an old cumulative counter. Drive performs 360 explicit fixed 1/60-second native steps per client, paced by browser timers; it is an actual Game input experiment, not a measurement of real-time frame rate. Ordinary presentation uses bounded timers for three minutes. Solo Game is paused throughout; only the host's actual existing Galaxy.step clock ticks shared Game. Guest rendering uses the native Replica.Scene.

Initialization has one 45-second total deadline and reports finite `init_stage` / `failure_code` markers, never raw native exceptions. A late Galaxy creation is freed after a timeout. A fresh Galaxy is paused before guarded license registration, and subsequent frame updates call `set_paused` only on actual pause transitions to preserve the native fixed-step accumulator. These are harness corrections; game runtime source and normal binaries are unchanged.

New session first leaves both controllers through their idempotent leave path, so it works after the explicit Host end action as well as while a session remains live. It then requires a distinct service session and a new actual native commit. Earlier failed browser attempts remain historical evidence; this correction alone does not establish browser rejoin acceptance.

The separate owned-loadout path requires an exact normal-build source commit and that checkout's normal `dist`. It reads the six adapter/storage sources from Git at that commit, checks their live equivalents, and verifies the complete runtime-source and package hashes against the retained normal build record before and after copying. A concurrent build, dirty runtime source, mismatched commit or package fails visibly. Later record-only commits may leave a valid retained build record; runtime bytes must still match. The temporary server repeats these provenance checks at launch. Existing factory manifests remain version 2; owned manifests/reports are version 3 and cannot be interchanged.

```powershell
node scripts/qa/co-op-transport/prepare.mjs --stage work/co-op-wasm-qa/owned-v2/fresh-stage --dist C:/absolute/normal/project/dist --owned-loadout --source-root C:/absolute/normal/project --source-commit EXACT_40_HEX_BUILD_COMMIT
```

Owned mode calls the actual staged `readOwnedLoadout` adapter against each real Galaxy. Only its bounded native equipment declaration accompanies seed/identity; no Profile or currency is sent. A guest's `owned_view` becomes true only after successful native replica commit of a header matching that local export, selected craft included. The host uses its authoritative native solo build and does not require a guest-style `owned_loadout` snapshot field. Reports include only export/commit flags, selected craft/slot counts and a declaration digest. Existing origin saves are retained; an empty store receives a legitimate unprogressed license. Thus a fresh run covers its actual starter declaration, not every advanced build. Equipment metadata acceptance does not establish all ability effects, balance or shared progression.

Save checks compare canonical valid local Profile digests and read the actual origin's persisted SaveStore state. No session frame is saved. Reports contain controlled enums, finite counts, booleans and digests only; invitation/token/SDP/ICE/Profile/position data are rejected. Per-action reports are immutable files under stage `captures`. Body-change counters compare complete native entity signatures within one session; rejoin clears the comparison baseline because ECS IDs may be reused. `unchanged_bodies` counts unchanged content signatures, not measured network cache hits. Same-count changes can remain zero if natural play did not produce one. Historical factory reports may include a cross-session comparison; that is not evidence of same-count destruction. The supplied v1 runtime uses factory guests and primary shots; owned behavior and shared rewards remain separate acceptance scopes.

The WASM mode CSP permits only self-hosted scripts plus `wasm-unsafe-eval` for compilation, exact service connections and exact client frames; general JavaScript eval remains disabled. This follows [Mozilla's script-src documentation](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy/script-src#unsafe_webassembly_execution). Header checks in Node do not prove browser enforcement: Root must verify actual WebGPU and WebGL2 initialization.

After Dispose, verify created PCs equal closed PCs, zero transport requests/timers and unchanged saves. Native Galaxies are freed after successful restoration. Close the browser page, then stop the listener; iframe context destruction closes remaining idle database connections. A failure remains visible and is not accepted as successful cleanup. Reload the tool for a second backend.

Drive preflight requires a foreground page and both full iframe rectangles inside the actual viewport. Resize or scroll to the frames before starting; a short viewport can leave the guest frame outside the visible area, where browser timers may be throttled. A visibility/scroll/resize event that moves a client out of view while Drive runs latches an interrupted observation, so it cannot later pass just because the frames became visible again. This does not bypass browser throttling or add a second native clock. Existing commands finish or time out; leave/dispose explicitly afterward. Timer asymmetry alone is not diagnosed as a native bug.

Focused checks:

```powershell
$env:COOP_QA_DIST = 'C:/absolute/normal/project/dist'
# Optional real owned-stage HTTP/provenance test (not browser execution):
$env:COOP_QA_SOURCE_ROOT = 'C:/absolute/normal/project'
$env:COOP_QA_SOURCE_COMMIT = 'EXACT_40_HEX_BUILD_COMMIT'
$env:COOP_QA_OWNED_DIST = 'C:/absolute/normal/project/dist'
node --experimental-transform-types --test scripts/test/coop-transport-qa.test.mjs scripts/test/coop-transport-wasm-qa.test.mjs
```

HTTP tests use OS-chosen ephemeral ports and stop all listeners. Without the explicit normal bundle input, only that actual-artifact HTTP case is skipped. Source-function tests use declared boundaries and do not execute WASM/RTC. No browser, separate-network/NAT/TURN, eight-player, mobile/background, latency, performance, advanced cosmetics or playable co-op release acceptance is implied by these tests.

## Optional fresh RENKO fifth-shot case

New WASM preparations include a small observation-only `fifth-shot.js` helper and `fifth_shot: true` manifest marker (19 files). Older immutable 18-file stages retain their original allowlist. The default PAN/HUGO scenario and version-2/3 reports remain unchanged. Select **Fresh RENKO fifth-shot** before Initialize in an owned source-pinned stage. This case produces bounded version-4 reports with `scenario: renko-fifth`; it is unavailable in factory mode.

Root must choose a fresh unused listener port (for example 4191) so both QA origins have independent empty stores. A new port alone is not sufficient: this case refuses any non-null existing SaveStore result before importing or creating Galaxy, registering or saving. It never clears a database or replaces a license. The host registers through actual native creation as RENKO and reads its genuine Pulse Lens R1 export; the guest receives its genuine HUGO starter. No Echo, Nova, advanced rank, XP or edited Profile is supplied.

After Create + join completes, keep both full iframes foreground and visible, then press **RENKO: depart + verify ten primaries** instead of ordinary Drive. The host naturally departs with fire false. Only the host then fires. Each real host `Galaxy.step` return is observed for an admitted-primary `shot` event; ambiguous multiple events or a saturated event queue are inconclusive. Immediately after one event, actual native simulation pauses while the existing controller transfers its complete publication over real RTC. The guest verifies the exact held tick/revision only after native commit. No additional acquire or JS gameplay simulation is introduced.

Both observers require one new owner/epoch-bound primary on ordinals 1–4 and 6–9, and a primary plus ×1.5 damage bolt on 5 and 10. The two bolts' velocity difference must be 70 along the actual host heading; common inherited ship velocity cancels. Seen IDs are internal and do not enter reports. A vanished projectile, refused bonus, stale membership, wrong pattern, deadline or interrupted visibility remains an explicit failure/inconclusive result. Do not interpret generic projectile totals, held-frame stepping or configured Node verifier tests as browser cadence/performance acceptance.

Finish Host end, New session, Guest leave, saved-profile verification and Dispose as usual. Local hashes and real PC/request/timer cleanup must remain unchanged. This source addition alone does not prove fifth-shot gameplay; Root must execute it against a new coherent normal-build stage. A guest-RENKO case, simultaneous firing pilots, Echo/Nova/full ability parity, eight peers and Internet/NAT remain separate scopes.
