# Evidence-only canonical Git pin amendment

Runtime source remains frozen at `db15b91f3a1614ebd245fbab47787de737535882` (Root integrated it as 831f715). No source, build, browser, GPU, art, runtime or acceptance change is made by this amendment.

The original HANDOFF.md and MANIFEST.json are preserved byte for byte. Their SHA256 values remain 8b065e405cf1c24d6b23f0f00136c728e373153386426163eaa0413326fbacfc and 626cdf5ea5e9c443495b464632ab9771827c6a8ce795e7ebfaeef83ce94353d8 respectively.

The version1 manifest's `source.bytes`/`source.sha256` values were raw working-checkout bytes; `source.git_blob` named the canonical Git object. Version2 makes that distinction explicit: `source.bytes`/`source.sha256` now describe the exact binary output of `git cat-file blob <commit>:<path>`, while `source.working_checkout.bytes`/`sha256` preserve the original raw checkout pins. Inputs, evidence, handoff and preserved-manifest pins still mean exact raw file bytes. No text decoding or newline conversion is used to calculate a Git-source digest.

Actual verification checked all 13 source Git object identities and raw original source pins, 4 raw inputs and 34 original raw evidence files. Only pixel.wgsl differs between checkout and Git bytes: working 8850 bytes with 165 CRLF sequences / SHA 60a4e97870f8401cdb479e3a12ac57baf5e98b975aeef9139b7a30a6742580e8; canonical 8685 LF bytes / SHA e545658e1bb63a71193262acf1d9e39a3e5ef69310c41a38793949e31659a811. Converting working CRLF to LF matches the Git blob exactly. The other 12 source files match byte for byte. The tracked isolated worktree remains clean.

QA also noted that separate JSON decimal serializations can have a last-f64-digit tail for fields declared as native f32. Fixture pose/light/descriptor floats should compare after binary32 rounding on both sides (Math.fround in JavaScript), with finite/range validation retained. Cell indices, RGBA words, masks and integer headers stay exact integer comparisons. This annotation does not modify the fixtures, create a new numeric-shader test or claim GPU parity. Center-ray checks and all earlier hardware/performance limitations remain as documented in the original handoff.

MANIFEST-GIT-V2.json supersedes only the ambiguous source-byte interpretation. CANONICAL-GIT-VERIFY.json and the accompanying actual CLI log record the verification; verify-canonical-pins.py is its repeatable read-only producer. Original evidence and failure logs remain available.
