# Configured native Ion Wake fixture: independent review

Accept frozen `87d92f2cbf10c24dda585b47f05de2fad6fe178f` for the configured native fixture issuance step. No source or evidence blocker found. Exact parent is `f1afbbd09b1b9c932c548ea220e85c595a35a395`. This review inspected source and recorded execution evidence and independently checked bytes, dependency pins, identity seed derivation and recursive Profile differences. It did not run a native build/export, Node test suite, listener, browser, WASM or RTC session.

## Scope and native authority

The only source delta appends one explicit ignored test to `crates/game_web/src/co_op_bridge_wake_tests.rs`; the original module bytes are retained as a prefix. `co_op_bridge.rs` includes this module under `#[cfg(test)]`. The remaining changed files are its owned work evidence. No runtime, dispatcher, renderer, UI, audio, profile schema, API, manifest or QA helper is changed.

The generator executes `crypto.randomBytes(8)` independently for each role, validates two distinct lowercase 16-hex inputs, and writes a new input with create-new semantics. It never reads a user or retained QA save. The native exporter admits only the explicit host DORA/PAN/RENKO and guest DORO/HUGO/PAN role tuples, constructs real native identities, then captures each constructor-normalized fresh Profile before configuration. These are freshly issued test identities, not room capabilities or retained player identities. The generator source and actual pinned input/log are evidence of this configured setup, not proof that a player earned it.

The native test clones each fresh Profile, changes the four intended logical fields, normalizes it through the real Game constructor, and rejects any additional scalar difference. It serializes/deserializes the full native Profile and reconstructs a Game with exact Profile equality. It calls the real bridge `dispatch` with one `export_loadout` operation; the production dispatcher exports only outside an active session through `co_op_owned_loadout`, which validates the build using the native registry. Exact bridge response equality, read-only Profile equality and the reloaded native equipment export are asserted. No JS-authored response or partial Profile replaces that path.

Both roles validate completely before any output write. The output must already exist and be empty; every file is written with `create_new`. The actual command used explicit work-owned environment paths. This prevents overwriting prior evidence; it does not claim transactional all-or-nothing disk writes if a later I/O operation fails. The ordinary test suite ignores the exporter and no shipped game code automatically loads these Profiles.

## Independently checked Profile semantics

Both configured Profiles and their own native fresh baselines contain complete schema-7 fields/arrays. Independent recursive comparison confirms exactly these five scalar leaves:

| Field | Host before → after | Guest before → after |
| --- | --- | --- |
| `campaign.abilities[5].blueprint` | false → true | false → true |
| `campaign.abilities[5].rank` | 0 → 1 | 0 → 1 |
| `campaign.loadouts[0].slots[2]` | 2 → 5 | null → 5 |
| `ranks[8]` | 0 → 1 | 0 → 1 |
| `unlocked[2]` | false → true | false → true |

Every other key, value and array length is equal to its own fresh baseline. Identity, world seed, revision, zero currency/materials/components, all ten RAICHO XP entries, zero hardware, empty kills/defeats/quests/discoveries/relics, unearned patches/blueprints/hulls and incomplete onboarding remain unchanged. Scout is selected and only its owned mask bit is set. PAN's original Flux Drive stays in its ability inventory while the passive equipment selects Wake. HUGO keeps its original Guard Coil inventory and defensive equipment slot. The other nine hull loadouts remain empty.

The native seed formula was independently evaluated from the new hexadecimal identity bytes, matching host 2,934,182,754 and guest 1,813,293,345. Both are nonzero/distinct; they are native derived seeds, not invented fixture constants. The actual native exports have the exact ten required loadout fields, R1 ordinary Wake in the passive slot, zero XP/hardware/patches/relics, untrained starter and no victory. Host slots are `[null,null,{index:5,rank:1}]`; guest slots are `[null,{index:1,rank:1},{index:5,rank:1}]`. Echo/Nova/Wake evolution fields are absent. Native export filters evolution one to omission; co-op admission's missing-field default is E1. Explicit null is rejected by the reviewed parser and recorded Node control.

## Actual evidence verification

All 35 author manifest files, all nine compact native JSON outputs and the fixture-pins document match their raw size/SHA-256 values. The committed work files normalize to those originals with the documented Windows CRLF-to-LF text distinction. Native fixture JSON output itself is exact LF UTF-8 with one trailing newline. The full Profile objects have no inserted explanatory QA properties; provenance belongs to separate metadata.

All 119 raw and canonical source hashes match the exact frozen commit, with precisely one changed test source. The three actual native depfile source sets plus the declared lock/validator inputs independently produce the same 119-entry union. The native executable and three Cargo fingerprint pins match their existing target files. Third-party versions are locked; this is not a byte audit of installed toolchains/third-party crates or a normal WASM package pin. Selected semantic source snapshots and original depfile/fingerprint copies are preserved here.

Actual pinned command results record one explicit exporter pass; six bridge regression passes with two ignored exports; strict game-web Clippy success; and production Node validation of both native Profile/export outputs. The Node validator invokes `validProfile`, semantic-equal `normalizeProfile` and `ownedLoadout`; its stale-rank mirror, missing schema-7 components and present-null evolution controls reject. This is genuine recorded Node pure-validator execution, not browser storage or WASM execution. The native constructor/serde roundtrip is genuine recorded native execution, not an IndexedDB save-service roundtrip.

The first commit attempt's missing Git identity error is preserved separately and was not described as a failed test. This review's first byte verifier incorrectly required raw CRLF evidence text to equal canonical Git LF bytes; its failure is retained, then corrected to verify both representations explicitly. No author file or output was changed for that correction. Native Profile and loadout output values were not edited or reissued by the review.

## Remaining gates

Acceptance covers labelled **configured native fixture; NOT earned progression** issuance. Root must integrate the test-only source and select fixtures only on explicitly fresh isolated QA origins. The current integrated normal package is a separate checkpoint; no actual WASM constructor/export use, stored-save writes, real owner thrust/trail birth, page transfer/atomic guest commit, canvas rendering, expiry, sanctuary retreat, leave/end/rejoin or human appearance proof has run against these Profiles in this freeze. Naturally earned acquisition, damage/destruction, eight-client performance, champion/royal Wake, Internet/TURN and physical-device acceptance remain open.

`verification.json` records independent seed/diff checks and exact scope. `MANIFEST.json` pins raw review files; the author original raw manifest and separate canonical Git manifest are retained. Existing native executable/fingerprint files are verified external inputs, not redundant executable copies. Previous review records remain unchanged.
