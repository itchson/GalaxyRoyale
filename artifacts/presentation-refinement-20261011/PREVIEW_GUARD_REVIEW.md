# Preview guard repair addendum

Actual fixed Root source is pinned in `evidence.json` with byte-identical snapshots. The previous negative report and fixtures remain unchanged. No production edits or package/build/browser actions were made by this reviewer.

All three bounded repairs now pass through the actual guard exports:

- Source identity captured before the fixture's artifact production rejects a later source change at `writePreviewBuild(root, expected)`; the existing stamp stays byte-identical. `scripts/build.mjs` now passes that initial identity through to the end check.
- Relative Vite plus an absolute project output anchor matches; a bare relative command is separately recognized as an unverified blocker; an unrelated absolute project command does not match.
- An actual spawned held-child fixture is terminated when the record path is a directory and record writing fails. No fixture child remains, and SIGINT/SIGTERM listener counts return to their prior values.

The two current Root preview tests also pass (`test.log`). This validates the narrow fixes, not a real Vite/WASM build or cross-OS/browser preview. Source freezing is still a required operational rule; this check is not an atomic lock against concurrent independent builds or launchers. No remaining concrete failure was found in the bounded three-scenario follow-up. Root remains the integrator.
