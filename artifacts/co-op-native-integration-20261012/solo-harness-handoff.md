# Solo co-op lifecycle harness handoff

Commit: 71c801170fc336817a3ee892d480357da87c5ce2. Four tracked test files only; no runtime edits.

Null co-op controller values preserve existing solo assertions. Facility probes now use the actual four facility coordinates, outside the new central hub. A separate actual-handler test verifies the null-service hub page, focus/input safety and unchanged progression.

Actual results against Root's pending working integration: 65/65 focused tests and the full shell selection 528 passed, 1 existing Windows file-symlink privilege skip, 0 failed (529 total). Commands: node work/co-op-solo-harness/run-joined.mjs --focused; node work/co-op-solo-harness/run-joined.mjs. The ignored read-only preload substitutes only isolated application-source reads to Root; the full runner substitutes only these four test paths. Imported narrative source was byte-identical between checkouts.

Initial logs are preserved: runner-url-failure.log is a Windows --import path error in this work-only runner; joined-hub-fixture-failure.log captures obsolete facility coordinates; joined-concurrent-bridge-failure.log captures Root's subsequently repaired out-of-scope fake-DOM querySelector dependency. No passing claim is made for these earlier runs. Root's scoped bridge repair is included in the final successful run.

Root source pins were checked unchanged after the run. Browser rendering, native Game/replica behavior, IndexedDB durability and real networking remain outside these VM fixtures. No server or browser was started.
