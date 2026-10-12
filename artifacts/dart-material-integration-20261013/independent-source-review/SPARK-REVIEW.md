# Bounded source review - commit 4bbcd7b (parent 49e3e68) - Dart material asset pipeline

Reviewer: independent bounded source reviewer (no recursion, no builds/listeners, no runtime changes).
Scope: 8 changed files only. Read-only verification; notes written under work/local-dart-pipeline-review.

## Verdict
NO SOURCE BLOCKER within the bounded scope. Every claimed contract re-verified against actual committed bytes.

## Independently re-verified (actual executed, this session)
- Frozen commit/parent exact: 4bbcd7b / 49e3e68 confirmed; 8-file scope matches brief exactly.
- authored_indices: len=1024, occupied=426, mask_equal=True, changed=118 cells.
- Signal sites (indices 4/6/33): 51 cells, positions bit-unchanged old==new (positions_unchanged=True).
- pack.json: dart indices == recomputed candidate; palette == baseline 64-entry palette (exact).
- provenance hashes recomputed and matched: source.png SHA, manifest/pack LF digests,
  author_script_sha256_lf vs committed git blob, converter_sha256_lf vs scripts/asset-pack.py,
  candidate_indices_sha256, mask_sha256 -> all True.
- baseline reference: git show 49e3e68:atlas.json LF-normalized SHA == source_atlas_sha256_lf (True);
  baseline.json/pack/provenance/committed script all clean in git status --porcelain.
- surgical atlas: prefix/suffix byte-identical to reference; only dart span replaced;
  other sprites preserved; palette unchanged; idempotent (re-apply unchanged returns same bytes).
- Malformed input refusal confirmed: appended trailing byte -> AssetError "Non-Dart atlas bytes
  changed; review new baseline first" (verified path). Existing tests also cover version-tamper and
  unexpected-dart-mutation refusal.
- Provenance consistency: provenance.json records 426/51/118/64 matching my recomputation.

## Converter contract check (scripts/asset-pack.py)
- budgets(): group_transforms is a per-group parts cap (1..LIMITS=256), so 2 groups x 1 hull part
  with budget group_transforms:1 is valid (each group=1 part); manifest field set allowed. No defect.
- encode_index decodes provided-palette indices verbatim; decode-only validate confirms solid.
- animation_policy honored: SIGNALS {4,6,33} preserved verbatim; every new color asserted non-signal.

## Test coverage assessment (asset-dart-detail-tests.py)
Tests genuinely verify the contracts (not stubs): baseline==git-reference fidelity, exact-426 mask/
grid/palette + signal-site freeze, surgical byte-suffix retention + idempotency, tamper/unreviewed
refusal, full-PNG byte round-trip, CRLF->canonical reproducibility, and shipped-atlas final equality.

## Evidence-log consistency (work/enemy-art-quality-review)
- Timeline coherent: tests-first(7:31) -> canonical-policy FAIL crlf(7:37) -> lf-writer OK 6(7:38)
  -> final OK 7(7:45:07) -> commit(7:45:54). The preserved canonical-policy FAIL log shows an
  earlier writer iteration; final dart-tests-final.log records Ran 7 OK including the baseline test.
  Logs are transparent, not fabricated. No prior nebula binary loss touched/reinterpreted.
- Prior nebula correction preserved: PRIOR-NEBULA-ARTIFACT-CORRECTION.md intact; no relabel of
  the replacement executable as historical proof.
- No listeners/builds started; git status clean.

## Remaining limits / notes (non-blocking)
- Working-copy atlas carries 1 CRLF pair vs LF-only committed canonical; surgical normalizes CRLF->LF
  before compare and git reports clean, so reproducibility holds. Cosmetic only.
- author_script_sha256_lf is the only role pin not separately name-pinned in provenance alongside
  a committed-blob copy; value matches committed script, acceptable.
- Not evaluated (out of scope): visual/GPU approval, gameplay, audio, perf. Root owns those.

## Conclusion
No defects found. Reproducibility, exact-mask, signal-preservation, surgical-scope, and refusal
contracts all verified against actual bytes. Recommend accept for Root integration review.
