# Current pixel UI integration acceptance

Root inspected the normal dual-renderer package built at `ec7504065e8830293c54f5c719ade368985973dd`, runtime input hash `76785526e166f2fb5dc0cdd0f1dc5ba135c80487f6186ba8a2cfe5fb589ed98f`. These are desktop in-app-browser observations, not physical-phone or human audio acceptance.

## Actual implementation and review

- Art runtime `773fc6c`: top-down planar Forge/Hangar/Journal/Arena facilities, dock sector details, unchanged facility centers and collision radii. Actual public Game packet extraction records 7,428 occupied cells and nine relative transforms; native Reduced Motion and animation checks pass. Evidence commit `54c2c3c` retains reproducible packet extraction, software-rendered contact sheets and logs separately from browser screenshots.
- Engine `8408b12`: north-up player-centered lower-left minimap above the text footer. Fixed 2,400-world-unit range, yellow Haven, purple actual tracked mission, edge bearings, heading and shared overlap reservations. Seven new native tests include compact boss/touch admission; real browser observations at Haven show the yellow home marker. There was no accepted mission in the observed profile, so no purple browser marker or naturally earned mission completion is claimed. Native authoritative mission-marker tests remain the evidence for purple tracking.
- Gameplay `e88e78e`: eleven original directly authored 24×24 native PixelBody control assets; separately measured settings labels and real input rectangles; Audio, Display & motion and Session grouping; coherent range-thumb geometry and unchanged nine setting IDs. Root bridge commits `a41196d`/`bc57036`, plus native ampersand `e5debe9`, integrate actual measured geometry and hover-only facility labels. Engine independently inspected the exact settings commit and ran 58 native UI tests and 21 integrated bridge/layout tests, with no scoped source blocker.

## Actual desktop browser observations

Both WebGL2 and WebGPU were inspected at 1280×720, 848×480 and 480×800. WebGL2 was additionally inspected at 640×480. Wide/compact landscape settings use two columns; the narrow portrait layout uses one independently scrollable column. Labels remain within their measured regions, and the long reduced-detail label fits without crossing its toggle. Header, close control and saved-state footer remain outside the scrolling content. Settings content reaching its scroll edge is clipped; existing text clipping remains center-based, whereas authored icon cells use full-corner clipping.

WebGL2: mouse click on Music set its value to 0.50; Home/End reached the range endpoints; keyboard input restored the original 0.45. Master volume ArrowRight/ArrowLeft and Reduced Motion check/uncheck responded and were restored. The cached previous build was replaced through the actual Apply update at Haven flow, without clearing saved progression or browser storage.

WebGPU: Master volume changed 0.54→0.55→0.54 through keyboard input. Camera shake toggled off/on. Session controls became accessible by scrolling, and Save & Backup opened its distinct backup page. No restore/import/export operation was performed. Reload retained the restored settings.

All four actual home facility routes opened their separate Forge, Hangar, Journal and Arena pages. Forge hover was inspected with `:hover=true`, focus=false and label visibility=visible after closing with Escape; idle visibility was hidden. This confirms actual cursor hover rather than relying on keyboard focus. Native semantic hit rectangles stay separate from text placement. Both renderer flight views show the map above the lower-left text, and the new native flight icons in the upper right. Browser screenshots are actual JPEGs; native packet contact sheets are explicitly PNG software previews.

## Reproducible checks

- Locked workspace native tests: 433 passed (37 engine unit, three engine integration, 231 gameplay, 131 browser/native UI, 31 boss-layout).
- Shell/service/audio helpers: 292 passed, one platform-specific skip, zero failed (293 total).
- Strict workspace all-target Clippy and TypeScript: passed.
- Normal WebGPU and WebGL2 package/PWA build: passed; game-only cache `galaxy-c40f91a071fafe05`.
- Actual Rust serializer through headless SaveStore: 18 passed across the two WASM backends, using fake-indexeddb. This is not physical browser storage evidence.

Raw root logs are retained under `artifacts/presentation-refinement-20261011/current-ui-*.log`. Independent immutable manifests describe the frozen specialist inputs; later integration changes do not rewrite those records. Short desktop timing receipts are contextual observations, not the requested 30-minute mobile performance soak.

Pending gates remain: physical touch/cancellation/device readability, sustained human listening, natural complete campaign and material/mission balance, and the separately recorded owner-private hosted PWA investigation. Bounded UI acceptance does not mark these gates complete.
