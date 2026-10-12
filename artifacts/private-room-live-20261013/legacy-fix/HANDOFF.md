# Legacy/current prepared allowlist correction

Root confirmed the live clients disposed and the owned listener stopped before this amendment. Only `scripts/qa/co-op-transport/prepare.mjs` and my existing `scripts/test/coop-private-room-provenance-qa.test.mjs` were edited. No commit was made. Root helpers, existing tests, frozen stages/captures, production runtime and normal bundles were untouched by this contributor.

`preparedNames` now admits the known `room-code` module only when the actual manifest has its own `files['modules/room-code.js']` entry. All other module names remain fixed. Historical transport manifests retain five files; current transport stages have six. Historical WASM/fifth-shot manifests retain 18/19 files; current stages with the pin retain 19/20. Actual preparation still unconditionally copies/pins room-code for current code, and the real transport import-closure test now also verifies served names exactly equal the prepared file map. No current import is dropped. Existing launch file count/hash checks remain authoritative; this narrow compatibility correction does not claim old provenance records can be launched by every current verifier.

Three new actual exported-function cases first failed against the uncorrected allowlist (exit 1, all three failures retained in `original-regression.log`). They cover historical/current transport, factory/owned/fifth WASM modes, and own-entry admission excluding inherited/unshipped fields. After the correction:

- `node --test scripts/test/coop-private-room-provenance-qa.test.mjs`: **9/9 pass**, exit 0 (`current-provenance-tests.log`). This includes the six prior carry/service/preparation cases, declared temporary Git fixtures, and an isolated transport-only stage with actual imports. No frozen/live stage was altered.
- `node --experimental-transform-types --test scripts/test/coop-transport-fifth-qa.test.mjs`: **10/10 pass**, exit 0 (`current-fifth-tests.log`). The original legacy 18/19 assertion now passes.
- `node --experimental-transform-types --test --test-name-pattern='^WASM allowlist' scripts/test/coop-transport-wasm-qa.test.mjs`: **1/1 pass**, exit 0 (`current-wasm-allowlist.log`). Root separately amended its current-manifest fixture to contain the room-code pin; this contributor did not edit it.
- Local Prettier formatted only the two owned files; scoped `git diff --check` exited 0 (logs retained).

Final exact working/LF source hashes:

- prepare.mjs: `2f5242434ea82793a0789efc8bff9070e451791e065ead6846612e7a3204ad51`
- coop-private-room-provenance-qa.test.mjs: `d3503c3fb286e82ea647d24821ac7f15da45df08fa38a769754b36bc74fab3a0`

`source-pins.json` pins actual final bytes, Root's current allowlist fixture, raw evidence and actual diff. Earlier preparation reviews/pins and native audio reviews/captures remain frozen. Root reported the post-live approval deadline/error-display changes separately; this amendment does not claim those changes were present in prior live captures or independently reapprove their implementation. Root owns the final combined suite and integration.

No listener, browser, normal build, native Game, RTC, saved-profile operation or negative live case was performed here.
