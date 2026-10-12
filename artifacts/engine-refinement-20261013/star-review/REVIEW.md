# Lazy star biome sampling: independent frozen review

Accept `49e3e686d5ea3e04523a8e40f4ef102756700ec2`, exact parent `8b0896d7ccd2c4dece1dc7a0d2cae6c4986505ab`. No source or evidence blocker found. Root's integrated `2a5a76a` has the same two affected source files. This review independently read source, checked actual compiler/probe evidence and recomputed result comparisons; it did not rerun a compiler, probe, native suite, browser, GPU or listener.

## Production behavior

The runtime change replaces eager `FieldBlend::new` at each star tile with a stack-local `Option` initialized once by the first seeded candidate surviving quality and viewport guards. The field constructor uses the same seed, tile corner, spacing and parallax factor. All four corner samples are retained and reused for later geometric survivors. Density rejection, interpolation, tint, twinkle, local coordinates, ordering, transforms and shared quotas remain on their original paths. `regions::at` is a pure bounded seeded-neighborhood calculation, so postponing its calls does not change combat RNG, saves or a mutable simulation cache.

Precise meaning of empty matters: a tile with no quality/viewport-surviving candidate skips all sampling. A tile with geometric candidates that later all fail density still needs one four-corner field. The source correctly initializes before density testing rather than skipping required density decisions. The test-build sample counter is thread-local and resets in each new focused case; both its declaration/increments and the new test module are under `#[cfg(test)]`. Shipped builds contain no counter.

The two focused regressions test an actually scanned, accepted 2×2 world-extent tile whose 32 points miss the viewport, and a larger visible tile with multiple stars. The latter requires exactly four total corner samples, exercises reduced detail with fewer positive survivors and compares every retained reduced star's exact position/color with the full result. The deliberately tiny world extent is a render-domain fixture; it is not a claim about normal game zoom limits or physical display resolution.

## Independent evidence verification

Both frozen source pins and 27 source-pinned evidence files plus the seven-entry manifest match exact raw bytes/hashes. Actual working/compiler bytes and canonical Git bytes are distinguished. The before/after/current compiler sets contain **108/109/108 entries**; their complete depfile unions, all declared source/asset/fixture bytes and all three native executable/depfile pairs were verified. Removing the two snapshots' declared query hook reproduces exact parent/candidate production source. The work-only probe source is byte-matched in both snapshots.

The after query snapshot includes the original visible-test definition that later failed its multiple-star precondition. Its only test-source differences from the final candidate are the full/reduced viewport setup. This function was not selected by the one-test query-probe filter. The final tracked suite separately compiled the corrected 800×800 world-extent fixture. These are distinct actual compilations; no probe result is presented as running the corrected new test before it existed.

Independent comparison of every before/after JSON row confirms 72 unique fixtures: two seeds, three cameras (including near the travel boundary), two detail levels, two reduced-motion settings and three nominal zoom factors. The raw f32-derived zoom numbers remain unchanged. All recorded pixel/star/motif/transform counts and ordered packet digests match, while every query count is nonincreasing and divisible by four. Recomputed totals are:

| Quality | Fixtures | Before samples | After samples | Avoided |
| --- | ---: | ---: | ---: | ---: |
| Full | 36 | 2,912 | 2,704 | 208 |
| Reduced | 36 | 2,912 | 2,456 | 456 |

The separate declared empty scanned tile records **4→0**. The probe counts actual four-corner FieldBlend biome samples; each current region lookup evaluates nine candidate centers. It does not count an allocation budget or measure elapsed frame time.

The retained FNV-1a64 digest consumes batch/pixel counts, each batch's affine transform raw f32 bits and each pixel's local bits, color and transform word. It supports ordered cell/affine instance-stream retention across those actual configured cases. It is not preserved full serialized packet bytes, a cryptographic equality proof or validation of every possible scene/unhashed frame metadata. The simple pure source change supplies the broader semantic reasoning; the digest evidence is accurately bounded.

Actual logs record one before and one after native probe pass; the final tracked cosmic suite has **34 passes, zero failures, five ignored explicit exporters**. Strict game-core Clippy, format and diff checks are recorded successful. Original failures are retained: missing include fixtures in the first snapshot compile, an absent after-snapshot manifest prerequisite, the initial visible-test precondition, and Studio route/timestamp dispatch attempts. Successful source/compiler inputs are checked against exact freezes rather than assuming those failed preparations were valid.

The actual Studio records distinguish author running/completed observations from task acceptance; no criteria were self-approved by this author record. This review's own first verifier counted both `@@` delimiters instead of unified-diff hunk headers and rejected the expected two test-only hunks. That read-only verifier error and the corrected result remain preserved. No author evidence/source changed for the correction.

## Limits

This is a measured CPU query-count reduction with unchanged source rendering semantics. Nebula gas morphology, shaders, game mechanics and saves are unchanged; it does not constitute new believable nebula art or a visual improvement. No normal WASM/bundle, actual GPU/browser/RTC/mobile, physical-device, elapsed-time, memory, allocation or FPS acceptance follows. Root's separate integrated checks and new normal build remain distinct work. Large compiler binaries are verified at original external paths; no artifact-writing probe was rerun over the freeze.

Canonical sources, original failures/results, compiled-input metadata and independent recomputation are pinned in this review's `MANIFEST.json`; earlier reviews remain unchanged.
