# Append-only prior artifact correction

Observed 2026-10-12T21:44:20+00:00.

The Dart native-loader check mistakenly reused the previous nebula cargo-current target. Its earlier frozen candidate executable at `C:/Users/mitch/.codex/worktrees/galaxy-audio-refinement/galaxy-royale/work/nebula-render-refinement/cargo-current/release/deps/game_core-2b707a8b4a4ebce6.exe` is no longer physically retained. Expected old SHA256 `aa01b158e9b356f18b336c1a3d39be6c602ad61b442ff2a7602f233b2114c04c`; actual Dart overwrite `e341c13539df6a979151947910b0ce3437acb9722bc7568efa29739fe3bb44c3`. The Dart executable and its actual compiled inputs are preserved separately under `native-loader-freeze/`; it is not relabeled as nebula evidence.

One bounded attempt temporarily restored the exact baseline atlas bytes and reran the same pinned-source/path/release/offline/locked cosmic test command. Actual34pass/5explicitignored; resulting SHA256 `83f996bdd8c5419d41ee8b0ac18ab081fcf10bb407a914bdeda452d2501a7e51` does not match the old executable. The exact old physical binary remains unavailable. No further regeneration is claimed. Final Dart atlas bytes were restored after that attempt.

Original nebula source/evidence manifests and logs were not edited. Original before/after instrumented probe targets remain retained. Original logged49 source/test observations and completed independent review are historical evidence, not an assertion that this mutable executable path still has its old hash. Root was given the exact path/hashes/outcome for its board correction. The independent Root checks do not depend on this old executable.
