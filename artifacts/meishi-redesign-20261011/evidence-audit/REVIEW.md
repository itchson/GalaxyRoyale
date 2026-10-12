# Final MEISHI evidence audit

No mismatch found in the requested evidence. Root HEAD is `f3a723081033a2cdd9bab15e37241fe334530716`.

- The native log records **680 passed, 0 failed, 1 ignored**: 37 + 3 + 368 + 241 + 31. The shell log records **425 passed, 0 failed, 1 skipped** out of 426. Its skip explicitly concerns unavailable Windows file-symlink privilege. These are shell tests, not a count of TypeScript checks.
- The Clippy log records successful completion with no error. It does not embed command flags, so this audit does not independently authenticate the reported strict flags. No checks were rerun.
- Read-only `verifyPreviewBuild` passed at 2026-10-11T14:14:44.934Z. The build and preview records agree with current HEAD/branch, actual runtime input hash `adfb3ad34b0c4499b98fc76ba089386a0d0bd5d099ff43210e86298410b26d3c`, and actual dist hash `0edfe1a2d7112494f4d504b68e1564bb0e16a5b6fa7f0442516f7a81d73894b5`.
- At 2026-10-11T14:14:44.9919481Z, the relevant Node/Vite inventory contained **one game preview**, PID **4912**, launched from Root with `preview --config apps/game/vite.config.ts`. It owned loopback **4173** and singleton lease **4172**. Its creation time matches the preview record's start time to millisecond precision.

Only this review directory was written. No source, dist, service, browser, or Studio state changed. This audit confirms captured logs and the current owned preview identity; it does not provide new GPU, visual, device, or performance acceptance. Exact input byte pins and observations are in `evidence.json`.
