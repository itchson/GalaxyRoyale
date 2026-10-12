

## Bounded source review — commit `4bbcd7b` (parent `49e3e68`), Dart material asset pipeline

**Verdict: No source blocker within this bounded scope.** Every claimed contract was independently re-verified against the actual committed bytes this session. Notes: [REVIEW.md](C:/Users/mitch/.codex/worktrees/galaxy-audio-refinement/galaxy-royale/work/local-dart-pipeline-review/REVIEW.md).

### Verified by execution (read-only; no builds, listeners, or runtime changes)

- **Scope/snapshot:** 8-file diff exactly matches the brief; `git status --porcelain` clean for all owned files.
- **Exact mask/grid:** recomputed `authored_indices` = 1024 cells, **426 occupied**, bit-identical occupancy mask, **118 color changes** (matches `changed_color_cells`).
- **Signal freeze:** **51** animated sites (indices 4/6/33) are position- and value-unchanged; converter's decode-only validate passes on `pack.json`.
- **Exact conversion:** `pack.json` dart indices equal the recomputed candidate; palette exactly equals the baseline 64-entry palette.
- **Hash integrity:** I recomputed `source_sha256`, `manifest_sha256_lf`, `pack_sha256_lf`, `mask_sha256`, `candidate_indices_sha256`, `author_script_sha256_lf` (vs the committed git blob) and `converter_sha256_lf` (vs `scripts/asset-pack.py`) — **all match** `provenance.json`. `source_atlas_sha256_lf` matches `git show 49e3e68:assets/pixels/atlas.json` normalized.
- **Surgical atlas boundary:** prefix/suffix bytes byte-identical to the frozen reference; only the dart span replaced; other sprites and palette preserved; idempotent on re-apply.
- **Malformed refusal:** a trailing-byte corruption raises `AssetError: "Non-Dart atlas bytes changed; review new baseline first"`; version-tamper and unexpected-dart-mutation refusals also exercised by tests.
- **Converter contracts:** `group_transforms: 1` is legal — it caps parts *per group* (2 groups × 1 hull part; limit 1..256), not total groups ([asset-pack.py:206](C:/Users/mitch/.codex/worktrees/galaxy-audio-refinement/galaxy-royale/scripts/asset-pack.py)).
- **Tests are real contract proofs** (7 tests): baseline↔git-reference fidelity, mask/palette/signal freeze, surgical retention + idempotency, tamper refusal, byte-level PNG round-trip, CRLF→canonical reproducibility, shipped-atlas equality.
- **Author evidence:** log timeline is transparent and internally consistent (`dart-tests-final.log`: Ran 7, OK, timestamped ~2 min before `dart-commit.log`); an earlier preserved failing CRLF-writer run is honestly retained, not overwritten. The prior nebula binary-loss correction is intact and unmodified — no new executable was relabelled as historical proof.

### Non-blocking notes

- The working-copy `atlas.json` carries one CRLF pair vs the LF-only canonical git blob (39,598 vs 39,597 bytes); `surgical_atlas` normalizes CRLF→LF before digest and git reports the file clean, so reproducibility holds. Cosmetic only.

### Limits

No visual/GPU, gameplay, progression, or performance claims assessed; human art approval and integration remain with the main OpenAI agent.