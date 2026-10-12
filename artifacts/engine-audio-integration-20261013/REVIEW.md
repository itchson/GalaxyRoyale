# Browser world-edge alignment: independent frozen review

Accept `4dc012cd4f899c392b52ac4d71402210917bf913` for integration. No source or provenance blocker found. Its exact parent is `f1afbbd09b1b9c932c548ea220e85c595a35a395`. This review read frozen Git blobs and actual author/native evidence and recomputed fixture hashes, body signatures and counts. It did not run a Node/native test suite, compiler, browser, RTC session or listener.

## Source decision

The sole production delta is 11 diff lines in `apps/game/src/coop/wire.ts`. It introduces the named 1,100,000 world-geometry position limit, applies it only to nonplayer body and projectile `position`/`previous`, retains 1,000,000 for player rows, and corrects projectile `velocity` from the old 1,100,000 bound to the existing default 1,000,000 bound. Body velocity and participant/input position, previous, velocity and heading domains remain unchanged. Camera derives from the strict participant domain; no camera source changed. The inherited Wake page raw/f32 coordinate halo 1,000,014, lifetime, 32 KiB quota and strict optional metadata are unchanged.

This matches the native `1b8c3ec7d20fba1dc69f14a3cc2f9b22945cd3f4` center-coordinate contract. Native publication uses the wider envelope for nonpilot geometry and the strict travel square for pilots. Native Replica stages geometry within the wider domain but completes a player body only when its pose matches the strictly validated participant header; the browser's early strict player-row check is consistent with that final invariant. Geometry maxima concern body centers/current and previous poses, not every scaled cell corner or a new world wrapping topology.

No schema, transport, controller, adapter, native runtime, shared manifest, production or QA helper source is changed by this candidate. Exact unchanged consumers and tests were compared with the parent. Coordinated current source/native compilation remains Root's integration gate; this author's branch itself still inherits the parent's native source and does not include the separate native edge implementation.

## Tests and original failures

The pinned original working wire normalizes exactly to the parent Git blob. The original real three-case Node log records one pass and two failures: actual native edge pages rejected with `invalid_vector`, and excess projectile velocity accepted instead of throwing. Those failures remain intact. The first three semantic cases remain in the expanded final suite; the initial test file's line layout is not claimed to be identical to the final file.

The actual final author logs record seven new focused passes, 69 existing controller/adapter/Wake passes and five selected pure transport-wire passes: 81 total, with no failed checks. The selected transport pattern matches only the five named pure wire functions; HTTP listener fixtures remain inside their unselected test callbacks. Typecheck, owned formatting and diff checks are separately recorded. This review verified these raw log pins and source semantics, and did not rerun those suites.

The new cases exercise public wire encode/decode of each native header/entity page; both signs/components and exact/next-f32/next-double body/player/projectile bounds; nonfinite and malformed DTO/frame limits; native shot owner/epoch; strict participant/input vectors; and the actual exported adapter's refusal before a declared native spy is invoked. These are meaningful parser/adapter gates. The adapter replies are explicitly declared unit responses and the case stops before body pages/complete commit, so its success is not new native Replica execution. The native complete-commit evidence comes from the separately recorded real exporter.

Raw next-double values just beyond a limit intentionally reject in JS even when they would round back inside f32. Native validators operate after f32 deserialization. The handoff correctly distinguishes this stricter raw-number browser gate from full native/raw-f64 equivalence.

## Independently verified native fixture provenance

The 19 fixture/provenance Git files consist of 18 exact native-written header/entity outputs plus the exact provenance document. Their producer is test-only exporter `84701ff7e0118654bc171276b122fc72df8a032b`, whose parent is native contract `1b8c3ec7d20fba1dc69f14a3cc2f9b22945cd3f4`. Its one-file append preserves the original edge tests and leaves runtime source unchanged. It uses actual Game admission, one real native tick, unfiltered publication, every actual entity/body page and a complete real native Replica commit before writing JSON Values with create-new output semantics.

All 27 full/small export size and SHA-256 pins were verified, including the nine full captures left at their original external paths. All 18 small files equal both the native original bytes and the committed browser fixtures. The 96 actual compiled-input source hashes, native depfile, test binary and linked engine-library pins were verified without execution. The exporter and its focused checks are actual recorded commands; the first successful full-only export and final small-file export remain distinguishable preserved logs, not invented failures.

Independent parsing checked all nine complete captures: 87 distinct rows in each; exactly two entity pages; 17/16/18/17/23/24/24/23 outlying asteroids in the eight edge/corner cases; matching body-page/storage/occupied totals; contiguous body-page offsets; exact unique row-major occupied cell indices; player-header pose equality; native f32 pose values within the correct domains; and all 783 surviving-body FNV-1a64 signatures recomputed from dimensions/count/index/RGBA. No edge body was omitted or repaired during the browser fixture copy. These fresh captures are distinct from the historical 99-row failure.

The ninth fixture has an actual guest-fired `ShotOwner` projectile at decoded native f32 position `[1000026.1875, 0]`, previous `[1000019, 0]`, velocity `[430, 0]`, owner matching the admitted guest's epoch 2. Participant owned loadouts are absent. This proves projectile attribution in a configured native edge test, not ownership/earning of an ability loadout or browser combat rewards.

The author source/evidence pins distinguish working bytes from canonical Git bytes. This review's first verifier attempt read a Unicode Node log with the Windows default encoding and failed its textual count assertion; that packaging error and the corrected explicit UTF-8 verification log are both retained. No author source or evidence was changed to resolve it.

## Remaining gates

Root must merge native 1b8 with the newer 111 persistent-trail implementation, preserve final confinement after late ability work, and compile/test matching browser/native packages. The reviewed pure JS/native-export evidence does not qualify current WASM or actual browser/RTC behavior, renderer parity, physical devices, eight clients, Internet/TURN, exact world/storage restoration, whole-game FPS or a wrapping world. Prior accepted source reviews remain separate frozen records. No new native or gameplay acceptance is inherited merely from the f1 parent.

`verification.json` records the exact checked inputs and case totals. `MANIFEST.json` pins the copied canonical sources, raw author/native evidence, original failed attempt and verification results. The nine large full native captures and compiler binaries remain external verified inputs; they were not redundantly copied or modified.
