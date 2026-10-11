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

Save checks compare canonical valid local Profile digests and read the actual origin's persisted SaveStore state. No session frame is saved. Reports contain controlled enums, finite counts, booleans and digests only; invitation/token/SDP/ICE/Profile/position data are rejected. Per-action reports are immutable files under stage `captures`. Body-change counters compare complete native entity signatures; `unchanged_bodies` counts unchanged content signatures, not measured network cache hits. Same-count changes can remain zero if natural play did not produce one. The supplied v1 runtime uses factory guests and primary shots; advanced owned builds, ability activation and shared rewards remain separate work.

The WASM mode CSP permits only self-hosted scripts plus `wasm-unsafe-eval` for compilation, exact service connections and exact client frames; general JavaScript eval remains disabled. This follows [Mozilla's script-src documentation](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy/script-src#unsafe_webassembly_execution). Header checks in Node do not prove browser enforcement: Root must verify actual WebGPU and WebGL2 initialization.

After Dispose, verify created PCs equal closed PCs, zero transport requests/timers and unchanged saves. Native Galaxies are freed after successful restoration. Close the browser page, then stop the listener; iframe context destruction closes remaining idle database connections. A failure remains visible and is not accepted as successful cleanup. Reload the tool for a second backend.

Focused checks:

```powershell
$env:COOP_QA_DIST = 'C:/absolute/normal/project/dist'
node --experimental-transform-types --test scripts/test/coop-transport-qa.test.mjs scripts/test/coop-transport-wasm-qa.test.mjs
```

HTTP tests use OS-chosen ephemeral ports and stop all listeners. Without the explicit normal bundle input, only that actual-artifact HTTP case is skipped. Source-function tests use declared boundaries and do not execute WASM/RTC. No browser, separate-network/NAT/TURN, eight-player, mobile/background, latency, performance, advanced cosmetics or playable co-op release acceptance is implied by these tests.
