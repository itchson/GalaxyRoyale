from pathlib import Path
import hashlib,json
work=Path('work/ion-wake-browser-boundary-review')
review='''# Ordinary Ion Wake browser boundary: independent frozen review

Reviewed Root commit `332ddd5ec0370e13ad346ea4ea988f35b61ca3e7`, parent `0f81bbbcc7c4ec854fd93652c9980511b9d05ce2`. This review is read-only source/evidence verification. No Root source/test/manifests were edited, no tests or builds were rerun by this reviewer, and no browser/listener/GPU/WebRTC execution was performed. The existing review of native gameplay `87de1f7` and the pending persistent-trail design/addendum remain distinct.

**Result: no source or provenance blocker in this narrow boundary.** The Wake browser shape agrees with intended native ordinary ID5 admission/export. Suitable for matched bundle compilation and later real-client checks; not a claim that co-op Wake is visually complete or that a rebuilt RTC runtime has passed.

The Root wire.ts change adds only the optional Wake interface field, allowlist entry, exact own-key count and validation arm. It requires a u32-shaped integer 1..5 plus a non-null passive slots[2] record with index5; slot shape/rank checks already run first. It distinguishes omission from a supplied undefined/null field. Unknown fields and missing required fields fail through the existing strict record/key-count validators. Both validatePayload(join) and publication(snapshot participants) invoke ownedLoadout, so the added field reaches both boundaries. readOwnedLoadout delegates through the actual expected native batch response framing without rewriting stored rank/evolution or importing a Profile. Registry/category/ship compatibility/XP-effective-rank and offline equipment claims remain native gates; these browser tests are not durable ownership proof.

Legacy/default and evolution-one exports omit Wake metadata; explicit numeric one remains a valid effect claim, matching native. Evolution2..5 exports retain the ordinary slot. Native present_evolution rejects explicit null and non-integer/u8-invalid JSON claims before admission; admission rejects0/6..255 and wrong slots. Existing Echo behavior is not changed by this increment. Older exact-key clients may reject evolved Wake exports, so matching rebuilt clients are required; no capability or mixed-build compatibility guarantee is introduced.

## Actual fixtures and test evidence

The independent verifier pinned eighteen exact canonical Git files and their working raw byte hashes, including Root wire/ports/controller, native co_op_loadout/wake/test/authority boundary, the tracked browser test, existing real publication fixture, new six fixtures/provenance and -text attribute. It compared each new fixture to the exact named native-wire.log serializer line plus newline, the final wake-final.log serializer line and the author's separately recorded raw fixture. All six match byte for byte, including184-byte legacy,200-byte evolution1 and219-byte evolution2..5 forms.

Provenance correctly identifies prepared test data: stored ordinary rank5, XP0/effective native rank1, not naturally earned; no boss_generations/wake_points/save container. The native serializer test actually uses Game::co_op_owned_loadout and validates temporary/native admission behavior. It does not produce a WASM envelope; the Node test explicitly supplies a fixture co_op_batch function returning that native value. Its positive calls exercise the imported real ownedLoadout, validatePayload(join), publication and readOwnedLoadout validators. These are meaningful strict-boundary fixture tests, not actual browser/Rust/WASM or RTC integration. The publication test intentionally replaces the peer's declaration while preserving its selected-ship field; it verifies browser shape, not native stats consistency for that modified peer.

The three new Node tests cover six byte-pinned actual values, all54 combinations of omitted/explicit Wake0? The loop actually uses Wake omitted or1..5, crossed with omitted/1/5 Echo and Nova (6*3*3=54 valid combinations), and invalid own-key undefined/null,0,6,255,256,negative,fraction,string,boolean,NaN,Infinity, wrong category/ID/empty passive equipment, missing required relic_count and arbitrary trails/history/currency. JSON erases an undefined own-property, so direct raw validation rejects it while export-wrapper validation deliberately skips that one representation; this is stated in the test and not hidden as native null equivalence. The negative publication path is source-reviewed through the shared validator, rather than advertised as a separately executed negative matrix.

Recorded Root evidence was read and copied exactly:

- browser-boundary.log:21 tests pass/0 fail/0 skipped, including three new tests and eighteen existing ports/boundary cases. Native delegates in these Node fixtures are supplied functions/recorded values, not WASM execution.
- native-wire.log:one actual named serializer test passes and emits all six canonical payloads.
- wake-final.log:thirteen actual final native Wake tests pass and emit the same six payloads.
- native-tests.log and native-tests-final.log:each actual full workspace native run aggregates914 passes/17 ignored over eight result records. This is recorded Root execution, not a rerun by this reviewer.
- clippy-native.log retains the original five actual findings: Option replacement and four redundant debug-format borrows. Root's final changes use hit_owner.replace(owner), preserving the same returned prior Option and restoration semantics; test borrow cleanup does not change debug comparisons. clippy-native-final.log records the subsequent strict pass. No suppressions were added by this increment. fmt/typecheck final logs are retained as emitted empty successful logs, with success reported by Root rather than independently rerun.

## Practical limits

The frozen native ordinary Wake samples/damage exist, but Pilot.wake still lacks the proposed host/guest persistent-trail publication/render path in this commit. This browser change concerns only owned equipment/evolution declarations. It adds no wake pages, effect reconstruction, light/audio parity or new save state. Actual matching dual bundles, guest admission/effective build, real RTC gameplay and final trail appearance are later gates owned by Root. The earlier Root real loopback approval/restore capture predates this rebuilt increment and cannot be silently reused as its runtime acceptance.

The review files, exact source snapshots, Root actual logs, raw author fixture copies and verification.json are independently pinned in MANIFEST.json. No source/passing status is inferred from a future build.
'''
review=review.replace('all54 combinations of omitted/explicit Wake0? The loop actually uses Wake omitted or1..5, crossed', 'all54 combinations using Wake omitted or1..5, crossed')
(work/'REVIEW.md').write_text(review,encoding='utf-8')
manifest={'version':1,'scope':'Independent read-only frozen Root source and actual recorded evidence verification only','root_frozen_commit':'332ddd5ec0370e13ad346ea4ea988f35b61ca3e7','source_pin_table':'verification.json','canonical_vs_raw':'source snapshots are exact Git blob bytes; verification.json separately records current working raw file bytes/hashes','files':[]}
for p in sorted(work.rglob('*')):
    if not p.is_file() or p.name=='MANIFEST.json':continue
    data=p.read_bytes()
    manifest['files'].append({'path':p.relative_to(work).as_posix(),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()})
(work/'MANIFEST.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
for name in ['REVIEW.md','MANIFEST.json','verification.json']:
    print(name,hashlib.sha256((work/name).read_bytes()).hexdigest())
