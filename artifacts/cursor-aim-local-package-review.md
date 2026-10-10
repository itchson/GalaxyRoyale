# Cursor aim — independent local release review

Reviewed the final source and normal package after the touch-guide correction. No blocking source or package defect remains in this bounded review. The final local cache is **galaxy-37e3441c8c48302c**, with **18 game assets / 19 served files**, independently recomputed from sorted paths and bytes. Earlier `dd480` and `a112` packages are intermediate checkpoints, not this final package.

## Source coherence

- Desktop right hold keeps the cursor's client coordinates and reprojects them on every input sample using the latest presented, unshaken camera and world viewport. Left destination remains a one-shot command to the WASM boundary; Rust retains the course, including arrival braking. The fixed-step bridge retains a pending destination when a frame has no simulation step and consumes it only once.
- Native aim is transient, requires held fire, finite coordinates within ±1,000,000 world units, and a target more than four world units from the ship. Facing turns once per tick at the existing 720°/second limit. The first shot can occur during the turn; bullets follow the actual nose with the existing quarter-velocity inheritance.
- With aim held, keyboard thrust follows the stick direction and residual thrust follows velocity. It cannot curve the course toward the cursor. On release, existing heading control resumes. Aim does not teleport the ship or bypass sanctuary/pause checks.
- Pointer masks preserve simultaneous left/right channels, including a second button observed during a captured drag. Release, capture loss, cancellation, blur, reset and modal disable clear cursor aim. Held state is established before a Haven fire callback can disable/reset input. Touch fire carries no mouse aim and retains heading fire.
- Final HTML correctly labels desktop “Aim + fire” and touch “Fire weapons”; README and CONTRACTS describe the separate behavior. The mobile-label mistake found during review was corrected before the final web/PWA build.
- `InputFrame` keeps serde defaults for legacy inputs. The new field is absent from persisted profiles. Storage and campaign source hashes remain identical to the published 60590dd baseline; schema remains 3. The release also includes the previously reviewed Mote target adjustment and bounded Unicode text wrapper, without folding in separate warning/palette studies.

## Actual evidence inspected

`check-local-package.mjs` completed with exit 0 and wrote `local-package-evidence.json`. It parsed both actual normal WASM binaries, found 48 WebGPU / 44 WebGL2 exports, confirmed exact public/dist binary equality and absence of `checkpoint_contract` or test-fixture exports/glue. Studio, API, production records and work folders are absent from the game package. The PWA retains same-origin GET scope, API/Studio exclusions and explicit dock-only activation.

Pinned 78 source/document/test inputs and eight root logs. The recorded root Rust log contains 252 passing checks (37 engine, 131 game, 84 web); the shell log contains 87 passes and zero failures. These are inspected producer logs, not an independent full-suite rerun. Native and both WASM Clippy logs show successful completion; the typecheck log is empty, with exit 0 reported by the producer. The final web build log names the exact final cache. No runtime crate input is newer than either shipped WASM binary. Root's six actual Game tick tests cover smooth turn/current-nose shots, identical navigation/arrival, keyboard/coasting separation, invalid/inactive aim, sanctuary/pause and legacy input/save separation. The focused fifteen browser EventTarget checks and immutable source pins are linked in the earlier input handoff.

WebGPU WASM SHA256: `390bd749212822b51c641186e4dceb8aa6770c56622c2ae114ce8fb9986519ef`.

WebGL2 WASM SHA256: `883d989c8b7912e81366fb656ec747cbed723226e952a7db3f6303bc168fb2f7`.

## Limits and erratum

This checkpoint precedes the new runtime commit/archive and private deployment. Current root browser validation is separate evidence and will be reviewed when supplied. Actual binary parsing is not browser or GPU execution. The earlier Chrome stress capture applies only to the old 60590dd package. Human moving/crowded Mote readability, listening, natural full campaign play, physical phones and sustained soak remain pending; no owner-authenticated remote gameplay is inferred.

`INPUT_ADDENDUM_ERRATUM.md` corrects the earlier frozen input addendum's mistaken claim that `input.enabled = canRun` executes every animation frame. It is inside recovery. The equality guard is valid idempotent handling, without a demonstrated frame-allocation or timing benefit. Historical files remain unchanged.
