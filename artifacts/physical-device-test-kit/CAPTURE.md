# Obtainable captures and their limits

Use the normal package with `?qa=1&metrics=1` for the measured natural run: **no scene parameter and no diagnostic scene actions**. If the overlay obstructs controls, use `?qa=1` and an attached inspector. The existing metadata's scenario label defaults to `dock` even when manually traveling; write the true workload/route in the session log. Diagnostic Mote/boss/stress scenes must be separately labeled.

## Platform connection

Pixel6a: existing desktop Chrome on Windows can inspect physical Android Chrome over USB through `chrome://inspect#devices`, after the human enables and approves device USB debugging. Use a data cable and the exact observed device/browser. No simulator or new tool installation is assumed. [Official Chrome remote debugging](https://developer.chrome.com/docs/devtools/remote-debugging).

IPhone12: Safari inspection requires a connected **Mac** with Safari developer features and the physical phone's Web Inspector enabled. Current Apple guidance locates the toggle under Apps→Safari→Advanced; settings paths depend on OS. A connected Mac/USB device has not been verified in this session. Windows Chrome alone does not provide this Safari path. If unavailable, keep inspector JSON/memory evidence Blocked and retain timed phone-screen observations. [Apple physical iOS inspection](https://developer.apple.com/documentation/safari-developer-tools/inspecting-ios).

## Existing read-only telemetry

At warm-up end and minutes1/5/10/15/20/25/30, save actual output of this inspector expression, without editing state:

```javascript
JSON.stringify({
  capturedAt: new Date().toISOString(),
  visibility: document.visibilityState,
  report: window.galaxyQA.report(),
  frameIntervalsMs: window.galaxyQA.frames(),
  layout: {
    css: [innerWidth,innerHeight], dpr: devicePixelRatio,
    scale: getComputedStyle(document.documentElement).getPropertyValue('--ui-scale'),
    canvas: [document.querySelector('canvas').width,document.querySelector('canvas').height],
    safe: ['paddingTop','paddingRight','paddingBottom','paddingLeft'].map(k=>getComputedStyle(document.querySelector('.safe-area-probe'))[k])
  }
}, null, 2)
```

This reads the implemented QA API/DOM. It does not start a capture reset, create a scene, or change progression. The game's normal Save→Export Backup provides profile evidence. Inspector readback of current SW/cache and loaded JS/CSS names must match `expected-build.json`; record stale/waiting build separately.

**Limits verified in main.ts:** at most3,600 active-flight intervals and corresponding simulation/render timings are retained. Reports are rolling windows, not all30minutes. Windows can overlap: do not concatenate them or average percentiles into a soak percentile. `durationSeconds` may include menu/background gaps. Hidden/recovery/failed frames are skipped, so a long recovery stall can be absent. FPS is sampled about once/second. Simulation/render fields are CPU scopes; render includes extraction/upload/submission, not GPU timestamps. Optional `memory` is JS heap only; null means unavailable, not zero/healthy. Backtick STATS pauses the game, so its pause is excluded from active-play timing.

## Frame and memory traces

If needed, record/save a real remote-browser Performance/Timelines trace spanning the required interval, or declared early/late60s windows with matched workloads. Export the actual trace; document profiling overhead and gaps. Only a complete trace with suitable frame records supports whole-soak stall counts. Keep an unprofiled visible-play run distinct. [Chrome Performance](https://developer.chrome.com/docs/devtools/performance), [Safari Timelines and export](https://webkit.org/web-inspector/timelines-tab/).

Chrome's Performance Memory capture and heap snapshots can expose JS/DOM trends; they are not automatically total game memory. On Android, if existing authorized ADB is available, save `adb shell dumpsys meminfo <observed-browser-package-or-pid>` at0/5/10/15/20/25/30. Retain package/process scope, PSS/PrivateDirty units and background-tab baseline; an aggregate Chrome process is not solely this game and shared GPU memory must not be added twice. No installation/authorization is performed by this kit. [Chrome memory tooling](https://developer.chrome.com/docs/devtools/memory-problems), [Android meminfo](https://developer.android.com/tools/dumpsys#meminfo).

Safari's connected-device Memory timeline can provide its reported page breakdown; that timeline and allocation snapshots can add overhead and are disabled by default. Export actual coverage/counters, not a guessed OS/GPU total. If neither platform exposes reliable total memory, mark that gate Pending and report available JS/page/process evidence by name. [WebKit memory/timeline scopes](https://webkit.org/web-inspector/timelines-tab/).

Raw screen recording is useful for touch/pacing/stall observation but is not a millisecond GPU trace. No screen recording or profiler result alone establishes acceptable human handling or sustained60FPS.