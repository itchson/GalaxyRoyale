# Current pixel scenery and UI — actual browser observations

Producer operated the normal packaged game at `http://127.0.0.1:4186/`, cache **galaxy-86ea0d1b0d17f148**, through the Codex in-app browser on Windows. Query flags selected the normal WebGPU/WebGL2 bundles and enabled supported diagnostic controls. The WASM QA contract was absent. These are representative desktop observations, not phone or soak acceptance.

The existing Haven update button applied the waiting build and returned to the title screen; the current console help then exposed WARP. This establishes this bounded update interaction, not a complete offline or cross-browser PWA matrix.

## Declared fixture scope

`work/scar-browser-fixtures.mjs` validates all three fixtures using the actual current `validProfile`. They derive from the prior actual exported profile in `artifacts/mote-browser-profile.json`. `seed17.json` supplies seed 17; `emblems-mixed.json` supplies alternating earned/unearned patches and matching blueprints; `emblems-victory.json` supplies all eight patches and wins with victory claimed and credits already seen. Imports used the visible Settings / Import save control. These files test presentation; neither the patches nor the tournament wins were earned in this browser run. The temporary 4186 origin is separate from the user's 4173 origin.

## Actual distant scenery and zoom

The visible console WARP moved to seed 17 at `-28879.919921875, -6955.796875`. Its response explicitly disclosed isolated rewards and existing bank/forfeit bookkeeping. SCENE DOCK returned to Haven and cleared isolation. The four durable native tests separately establish malformed input, retained bookkeeping and quest reward behavior.

`scar-max-zoom-webgl2-current.jpg` and `scar-max-zoom-webgpu.jpg` show the new cratered purple planet, lime fractures, fragments and ion wisps with the player clearly in the foreground. Both depict layered stars and existing neighboring planetary rings. The initial capture immediately after WARP can precede the next interpolated ship frame; settled captures are used for foreground inspection.

Actual repeated wheel input reached both clamps. `scar-min-zoom-webgl2-stats.txt` reports **0.60 / world 888 x 500 / UI 533 x 300**. `scar-max-zoom-webgpu-stats.txt` reports **1.60 / world 333 x 188 / UI 533 x 300**. `scar-low-zoom-webgl2*` is an intermediate **0.99** capture, not proof of the lower limit. The actual ship position remained unchanged while scrolling. Desktop sampled FPS and p95 are recorded diagnostics, not sustained performance claims.

Reduced cosmic detail and Reduced motion were both visibly checked in Settings before the separate admitted location `-3024.44775390625, -690.4987182617188`. `scar-reduced-static-webgpu.jpg` shows the lower-density body with intact silhouette, and its stats record 3,876 combined quads. The setting state was read back as checked. A still image does not independently prove animation freeze over time; the actual native frozen-time tests cover that contract. Moving/crowded enemy warning contrast remains Pending.

## Actual UI boxes and clipping

`champion-emblems-webgpu.jpg` and `champion-emblems-webgl2.jpg` show the first earned and unearned freeform emblems at 1280 x 720, DPR 1.25 and UI scale 2.4. Paired DOM records measure all eight emblem boxes at approximately **24.00000095 logical units** on each axis. The shared ship boxes in `champion-ships-webgpu-dom.json` measure **32.00000127**; this is geometry evidence without a paired ship image. Mixed WebGPU captures had Reduced motion checked; native tests establish earned-only shimmer timing, while human/browser animation judgment remains Pending.

`champion-emblems-webgpu-scrolled.jpg` and `-scroll-dom.json` record actual scrollTop 324. The Nightrail/Prism icons are cropped at the measured upper scroll boundary and Windrunner/Ironwake at the lower boundary; neither paints across tabs or the fixed footer. The requested 853 x 480 override actually reported **854 x 480**, DPR approximately 1 and UI scale 1. The base `champion-emblems-fractional-webgpu` pair retained bottom scrollTop 389.6, with no icon visible; its `-scrolled` pair is actually scrollTop 0 and shows the first two. Positions are fractional at scale 1; measured dimensions are fractional at scale 2.4. `champion-emblems-narrow-webgpu` records **640 x 360**, scale 1 and the one-column roster. Viewport overrides were reset afterward.

`champion-royale-webgl2.jpg` / `-dom.json` show the actual ninth gold emblem at scrollTop 1008 inside the visible clip, active=true, with the supplied victory caption. The earlier `champion-royale-webgpu` pair is a deeply scrolled standings view: its ninth icon exists in the DOM but is above the clip. It is not evidence of the ninth icon's visible WebGPU appearance. Both renderer helper extraction is independently native-tested.

Independent production/native extraction and actual captured-observation reviews live separately in `work/champion-patch-engine-review/`; their original frozen candidate records are preserved. Human readability, natural campaign, physical touch and named-phone validation remain separate Pending gates.

## Actual audio and private release

Real browser DSP was exercised through the separate work-only page at 127.0.0.1:4188 using the unchanged hash-pinned production AudioEngine, native analyser and MediaRecorder after its existing compressor. Six declared scenarios completed: charge, pulse, music mix, master mute, suspend/resume and a 100-cue burst bounded to 16 effects voices. Actual disposal closed the context and cleared voices and graph nodes. `mote-browser-dsp-review.md` and `mote-browser-dsp-evidence.json` record measured windows and independent decoding of the six original native recordings, preserved under `mote-browser-dsp/`. Declared charge/pulse actions are not fake Game events. Human listening and physical-device recovery are not inferred from sample windows. This probe is excluded from the shipped package, and its owned service was stopped after testing.

The prior private deployment remains historical until the current exact committed source and game-only archive successfully publish. An unauthenticated login gate establishes protection only; owner-authenticated remote gameplay remains unmeasured.
