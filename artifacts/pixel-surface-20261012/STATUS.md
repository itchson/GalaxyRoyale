# Pixel surfaces — first registered material integrated

## Implemented

Root compiled source1eca1c510325788624851a86527c2810b5e42df7 on codex/testing. The portable core, shared shader and native PAN adapter implement opt-in oct8 normals and linear luminance masks in existing16-byte pixel records. Affine transforms remain32 bytes/128 slots; the normal/light path uses the existing eight-light renderer. No texture/pass or runtime dependency was added. PAN caches14,036 bytes of material word payload.

The reviewed source has662 coarse hull cells and1,205 fine cells. Only31 original fine signals have mask255; all other fine/coarse masks are0. Rust repacking from the preserved semantic proposal exactly reproduced the committed33,702-byte map, SHA256903be7ec67fd236c9e2ec788a95da852779e22f05ad97dd2a066f1f9e77ea01c. Actual collision damage/holes, map identity/indexing, quotas, affine extraction and matching native co-op starter reconstruction passed meaningful native tests. Recent-hit timing and general surface wire transfer are not included.

## Actual integration evidence

876 native tests passed,14 explicit exporters ignored;564 shell tests passed,3 skipped. Strict native/all-target and WASM Clippy, fmt, normal dual-backend build and50 headless Rust/WASM serializer/save cases passed. The last use fake-indexeddb and do not establish real device persistence. Four standalone Rust proof consumers compile after transform_slot migration; the seven helpers were source-reviewed. See actual logs and independent registration/integration reviews.

Root applied the PWA update through the real Haven control and observed index-B5Nk0wRJ.js. Actual1280x720 screenshots show the healthy and turned PAN in both WebGPU and WebGL2; STATS confirms UI640x360/world400x225 at zoom1.60. The controller moved the ship from450,0 to542.6,0 in each isolated scene. Displayed PROFILE values remained identical: revision111/seed871238449, currency9/5/6, active XP81, cargo0, stash16 and training stash5/3/4. This is equality of displayed fields, not an independent full IndexedDB hash comparison.

Art inspected all five untouched screenshots and passed scoped healthy/turned appearance: crisp attached detail, recognizable canopy/reactor/lamps, no blanket wash or halo. QA independently recomputed the runtime/package hashes and verified saved logs/profile texts. Neither specialist reran the browser. Captured FPS/p95 were around1FPS/1000ms during tool-driven observation; they are not accepted performance measurements. The initial GL2 zero-quad sample is preserved as a pre-first-frame readout, followed by a real8,992-quad rendered scene.

Root exited with SCENE HOME, restored the observed previous zoom0.65/shake-off settings, reset the temporary viewport and left one normal preview on4173. Studio4174/4175 and signaling4176 are separate development services; the temporary4191 RTC stage was stopped.

## Remaining gates

The first small stills do not demonstrate controlled opposing-light or normal-versus-flat depth improvement. That art gate remains open, including browser hole appearance and any claimed idle/thrust emission gating. Wider asset authoring, real RTC material/Nova presentation, internet/eight-player co-op, GPU recovery with mapped content, physical-device performance and the30-minute soak remain pending. Shader f32 near-singular fallback is narrower than the portable f64 extreme-affine oracle. A renderable first material is not whole-game visual/performance completion.

## Reference and other evidence

User reference: https://x.com/moonfell_rpg/status/2109451515157233916?s=20 . Root read the public post and viewed its playing demonstration; reference-demonstration.jpg is an untouched later Luminance-view frame. No external code or artwork was imported. The first web403 was not successful media inspection.

The separately accepted ringed screenshots are from compiled303659 and do not prove the new surface shader. The real RENKO loopback RTC cadence is preserved in artifacts/renko-fifth-20261012, distinct from Nova presentation acceptance. Spark audio output here is an unaccepted source-only proposal: Root found its fixed-filter claim incorrect because advanceSound already updates cutoff with throttle/speed. No ear approval or new audio implementation is claimed.
