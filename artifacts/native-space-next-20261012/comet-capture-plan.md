# Actual moving-comet browser protocol

Prepared against exact source `22a8906cd668e92db0180d725da87b0a2c8408d5`; later Root evidence-only HEAD `69646456ac3e104c94ca27bc79aeb774dd9ab757` does not change the inspected files. Root reports the current normal runtime as `d749bc6972dee2cc0eb63330bb729f427ce2e749`. This is a source-derived plan, not browser execution. No source, profile, clock, listener or existing evidence was changed.

## Native field and camera

| Item | Derived value |
| --- | --- |
| Actual saved seed to confirm with PROFILE | 871238449 |
| Regional birth | layer 2, grid [-1,-3], hash 3887264754 |
| Stable reduced-density gate | Even hash: this object remains eligible |
| Unencoded birth origin | [-123.4034119,-3218.7492676] |
| Parallax | 0.48, encoded once in the parent |
| Heading | 0.1123184 radians (about 6.435 degrees) |
| Seeded phase offset | 11.7830658 seconds |
| Birth-centered WARP | -257.0904541 -6705.7280273 |
| Head-centered at native 80n | -130.3370056 -6691.4306641 |
| Recommended unobscured WARP | **-10.3370056 -6691.4306641** |

`derive.py` executes the source integer mixer and explicitly rounded f32 arithmetic in Python. Trig is a Python/libm calculation rounded to f32, not a new native extraction. Its small numerical approximation does not affect a camera pose with tens of world units of clearance. `derivation.json` records exact inputs, computed poses and source hashes.

The recommended pose is the head-centered camera plus 120 camera units in x. At every native `80n` the head is approximately 57.6 world units left of the craft. Around `80n ±0.5` its relative position ranges x=-68.53..-46.67, y=-1.23..1.23. The seeded dust extends at most about 90 world units behind it. At a settled 1280×720 / 533×300 logical / zoom1.6 view, the nominal world extent is 333.125×187.5, so this narrow capture interval retains the whole tail with modest left clearance. Confirm actual STATS world/UI values before using that claim. Do not generalize the fit to a phone/tiny viewport.

The true seeded pass begins at native time `68.2169342 + 80n`, fades in over 0.8 seconds, and becomes inactive at `86.2169342 + 80n`. A global `80n` boundary is inside the bright pass, not its legitimate cycle reset. Reduced Motion makes cosmetic time zero, so this same object freezes at phase11.7830658 and remains eligible for a visibility check at the recommended camera.

## Clock observation: DOM only

Current HELP/STATS/PROFILE do **not** display native `Game.time`. Opening the game console pauses simulation; UI/presentation clocks keep advancing separately. Do not leave it open during the moving test. WARP resets the run/player pose but does not assign `Game.time`.

The existing QA branch writes native `hud.time` into `#performance[data-report]` at roughly 500ms RAF intervals. This works even when the metrics element is hidden. Root's browser policy permits DOM-backed reads, so use only this diagnostic DOM record, not `window.galaxyQA` or other app globals:

```javascript
const metric = document.getElementById("performance");
const raw = metric?.getAttribute("data-report");
const report = raw ? JSON.parse(raw) : null;
return {
  nativeTime: report?.time ?? null,
  backend: report?.backend ?? null,
  position: report?.position ?? null,
  reportPaused: report?.paused ?? null,
  consoleHidden: document.getElementById("debug-console")?.hidden,
  pageHidden: document.hidden,
  densityReduced: document.getElementById("density-setting")?.checked,
  motionReduced: document.getElementById("motion-setting")?.checked,
  zoomSetting: document.getElementById("zoom-level")?.value,
};
```

Record the range's current DOM value alongside native console STATS after the zoom settles.

The record is cached: in a healthy continuously running solo session it can lag the native clock by approximately **0–500ms plus frame/tool latency**. This is an observation interval, not an exact timestamp for a screenshot. If it stops advancing, reports malformed/missing time, the page is hidden, or the console/dialogue/death/recovery gate is active, stop and label the moving gate Pending. `durationSeconds`, a stopwatch and RAF wall time are not substitutes for native time: the fixed-step host can discard backlog after five simulation steps per frame. Read the DOM record immediately before and after each retained frame; retain both values and the tool capture time. Equal repeated values are cached observations, not frozen simulation proof.

## Executable sequence for Root's owned normal browser

1. Use the already-owned normal backend tab/profile with `?qa=1` (and `renderer=webgl2` for an explicit GL2 run). Verify current loaded JS/hashed WASM/cache identity through normal DOM/network evidence. Do not change seed/storage, install a listener, invoke hidden ticks or manufacture elapsed time. Record PROFILE seed871238449 and STATS backend/world/UI dimensions. WARP is existing QA travel: it discards carried materials and disables scene progression; perform from Root's already banked QA state and do not present this as natural exploration.
2. In actual Settings, set zoom1.6, camera shake off, full cosmetic density and Reduced Motion on. Close Settings. Open the actual console, enter `WARP -10.3370056 -6691.4306641`, then close it. Leave all movement/fire controls neutral. First retain a **frozen visibility** screenshot. The comet should lie left of the player, with dust extending further left; no clock wait is needed because Reduced Motion sets cosmetic time0. If it is absent or obscured, the source formula alone is insufficient: record visibility/admission Pending rather than accepting an empty frame. Shared regional quota can reject a whole cosmetic group after earlier art; this protocol has not executed that full-scene admission.
3. In actual Settings turn Reduced Motion off, close Settings, and observe two advancing DOM clock values. Choose `B = 80 × ceil((observedTime + 10) / 80)` in the reviewer's local arithmetic, without writing it into the game. The 10-second lead allows normal panel/console exit; target any next global multiple, not only80. Keep the tab foreground, no console/dialogue/menu overlay, neutral controls and stable camera.
4. Start a short actual frame sequence when the reported native time reaches `B-1.5`, and continue until a fresh report is at least `B+1`. Because the report can lag500ms, this wider observation window is intended to include the real boundary. Retain frames plus pre/post DOM reads. An actual recording or frame sequence with **≤100ms gaps** around the crossing is useful; sparse screenshots separated by seconds cannot establish continuity and must remain static-form evidence. Do not create interpolated/mocked in-between frames.
5. Review the comet crop while preserving the full screenshots. The head should continue moving right by about22 world units per native second. Look for a collective abrupt dust shift while the head remains smooth (the old defect moved many still-visible cells by about16.6world units over the native79.999→80.001 regression). Individual dust recurrence at near-zero alpha is legitimate. Do not equate a500ms DOM timestamp with the original2ms native assertion, claim tracking of inaccessible particle IDs from pixels, or impose the native `<0.1world` threshold on a browser100ms gap.
6. Through actual Settings enable reduced cosmetic density, close it, verify advancing time and plan the next `80n` boundary with the same camera/zoom. Repeat the bounded sequence. Source eligibility is stable; the nucleus/coma are unchanged while dust falls from64 to32. Admission is atomic at884 full or852 reduced cells, four transforms, inside the existing regional6000/3000 budget. Actual pixels/visibility still need observation; a smaller total frame count does not prove this particular comet was admitted.
7. Through actual Settings enable Reduced Motion. Retain two comet-crop/full frames separated by at least1 native second as shown by advancing DOM reports. At stable camera/zoom the comet should remain in the phase0-derived pose despite the native simulation clock advancing. Do this for both density settings if the bounded session permits. Compare the comet, not unrelated moving actors or UI hover animation. Return Root's temporary settings, viewport and diagnostic browser state when finished.

Optional legitimate-cycle check: at `68.2169342 + 80n` this object is invisible on both sides of the reset/fade boundary (source inactive immediately before, charged alpha-zero submissions at reset), then fades in. That differs from an80n discontinuity. It is already covered by retained native tests and is not necessary to expand this browser pass; the source phase0 pose is roughly315 world units left of the recommended camera, so it is **not fully visible at this zoom**. A browser check of that separate boundary needs a different camera and is not claimed by this plan.

## Evidence and acceptance boundaries

The first continuity handoff/logs remain immutable: `work/deep-background-audit/` pins real baseline failure and12 final native tests at de8547c (Root10f84fe). Current shooting_star source is byte-identical to that committed helper. The new phase clock retains particle identities/palettes/materials/heads/parallax/costs and moves its reset to the invisible seeded boundary. This task did not rerun native tests, export a packet, drive a browser or alter current remnant evidence.

Bounded browser result labels: (a) frozen field visible, (b) full moving sequence brackets80n with continuous visible head/dust, (c) reduced moving sequence does the same, (d) Reduced Motion freezes visible comet while native clock advances. Missing/slow/stale capture or invisible field means the corresponding item remains Pending. Stills alone can establish form/placement, not continuity. All-seed, phone, performance, physical gas and extended-soak approval remain unclaimed.
