# QA factory capability correction

Changed only `scripts/qa/audio-review/review.js` and the focused QA file `scripts/qa/audio-review/fixtures.test.mjs`. No commit was made. Root owns review/integration. Previous title reviews and the frozen V3 capture were preserved.

The real OfflineAudioContext adapter is now exported as `createOfflineAdapter`. A factory property is read from the real native receiver and wrapped only when it is callable. Absent or noncallable `createStereoPanner` remains absent/noncallable to the production engine's feature check. Supported factories still execute with the real native receiver, return the exact native object, register created nodes once, exclude AudioBuffers from node registration and register oscillator/buffer sources once. `renderFixture` calls this same exported adapter.

The focused file's existing AST/VM extraction now includes that exported function and removes its module-export modifier for isolated evaluation. It executes the actual source body rather than a copied implementation. Three new checks cover actual renderer capability absence before a declared no-PCM exit, supported factory receiver/argument/identity/registration behavior, and absent/noncallable factory values without phantom nodes. Existing declarations, missing-DSP and restoration checks still pass.

Actual results on Node `v24.18.0`:

```text
node --test --test-name-pattern='actual renderer preserves absent native panner capability' scripts/qa/audio-review/fixtures.test.mjs
BEFORE FIX: exit 1, one failure. Actual host capability was 'function'; expected 'undefined'.

node --test --test-skip-pattern='actual loopback HTTP|occupied-port CLI|autoquit closes' scripts/qa/audio-review/helper.test.mjs scripts/qa/audio-review/fixtures.test.mjs scripts/qa/audio-review/metrics.test.mjs
AFTER FIX: exit 0, 18 passed, 0 failed. Three server-opening checks excluded.

git diff --check -- scripts/qa/audio-review/review.js scripts/qa/audio-review/fixtures.test.mjs
exit 0

node node_modules/prettier/bin/prettier.cjs --write scripts/qa/audio-review/review.js scripts/qa/audio-review/fixtures.test.mjs
exit 0; only the two owned files formatted with local Prettier 3.9.9

node --test --test-name-pattern='actual renderer preserves absent native panner capability|exported offline adapter' scripts/qa/audio-review/fixtures.test.mjs
AFTER FORMATTING: exit 0, all three capability checks passed, no server tests executed
```

Raw evidence is in `regression-before.log`, `qa-tests.log`, `capability-after-format.log`, `format.log`, `diff-check.log`, and `diff.patch`; exact source, log, patch and prior-capture preservation hashes are in `source-pins.json`. The 18-check no-server run preceded the final formatting; the three new capability checks were rerun against the final formatted bytes. Root will run its combined current checks after integration.

Execution deviation: the first post-fix invocation attempted to exclude server-opening tests using a negative `--test-name-pattern`. That filter did not exclude them. All 21 QA tests ran and passed, including the three temporary loopback server tests, which completed their normal `finally`/autoquit cleanup. The Node test process exited zero and no continuing service was left by that run. This violated the task's no-server instruction; the incident was reported immediately to Root. Its unchanged raw log is `qa-tests-with-unintended-server-checks.log`. The explicit `--test-skip-pattern` rerun above then executed the intended 18 checks without starting a server.

The helper tests prepare isolated ignored QA modules as part of their existing fixtures. No production bundle, broad build, browser, runtime source edit, new native DSP render or listening pass was performed here. The three excluded server tests are omitted from the final scoped result rather than reported as skipped cases by Node.

V3 provenance remains tied to its earlier capable helper hash `85149988415a1e2a6f93284171c7b34555e13e22a6ba4b1e3337db18ceb523f7`. `review-before.js` is a raw-byte copy matching that hash. The captured report remains at SHA256 `5369b426b6d297ce48b9dc0321af7ac3eaa5db8d996b903980c1cc6afc37bba6`; each preserved WAV is checked against its existing report hash. The current QA helper has a new hash and requires fresh preparation for any future render. This correction does not relabel V3 as a post-amendment render or change its source pins, PCM, metrics or listening/fidelity status.
