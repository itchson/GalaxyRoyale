# Independent game runtime lease and Vite lifecycle review

Final verdict: **no blocking finding after Root repair 3d52739fb22e3fc83a01e82e314830bd394ae200**. Permanent regressions committed in **d4ca915eaf79428fb612b2d0169366b6da7ecaf6**, exactly one new tracked file: `scripts/test/game-runtime-vite.test.mjs`. Root owns all runtime implementation and integration.

## Concrete findings and repair

The initial committed hook 5aa920a77978ff970ea635d7a8801a2739fb4d5e acquired an independent OS lease in every `configureServer`. Installed Vite 7.3.7 creates a replacement server before closing its old server during restart (`config.js:26010–26044`). The replacement therefore collided with its own live lease and restart failed. Vite's never-listened close path also resolves without an HTTP `close` event (`config.js:25920–25940`), leaving an HTTP-event-only reservation held until process exit. Both defects were reported before Root's repair, and both were reproduced with actual Vite APIs at isolated ephemeral ports.

The repaired plugin keeps a port-keyed `Symbol.for` state across configuration module reimports, shares a pending open among owners, awaits final close before reopening, and makes every release idempotent. An explicit `server.close` finally releases owners even before HTTP listen; HTTP close/error also release the same owner safely. Vite replacement construction retains the lease while the old owner closes, preserving OS exclusion throughout the handoff.

The lease has no stale disk lock. An occupied lease port rejects with a controlled message and never kills its owner. Production uses the fixed localhost 4172 default; test ports are injectable without changing production Vite config. Same-process programmatic servers deliberately share the reservation. The singleton claim is cross-process CLI dev/preview exclusion, not a prohibition on every programmatic server in one Node process. The configured normal HTTP paths are reviewed; middleware-mode embedding is outside this acceptance.

`supervise` stops owned siblings after an exit or asynchronous startup error, waits for each child's close and escalates only owned surviving children after five seconds. Explicit stop is idempotent. Studio-only mode does not acquire the game lease. No concrete shutdown or preview lifecycle gap was found after repair in these supported paths.

## Actual executed evidence

Windows 11, Node 24.18.0, installed Vite 7.3.7. Source snapshots pin the exact original and repaired commits. The first investigation load failed because its local Windows import was not a file URL; `fixture-first-load-error.log` preserves that fixture-only error. Corrected execution is separate.

`actual-vite-review.log`: **7/7** investigation cases passed—two expected negative baseline reproductions plus five repaired cases. The baseline helper's fixed port was changed only inside the isolated loaded source copy; all other baseline hook behavior came from exact 5aa920a. No production 4172 or 4173 listener was used.

`permanent-vite-tests.log`: **5/5** committed positive regressions passed through actual Vite APIs: never-listened/idempotent close, two actual restarts, a separate real child denied by the held OS lease, independent module import owner sharing, preview close, and preview HTTP listen failure preserving the occupied test owner. The fixtures use empty temporary roots, disable filesystem watching/HMR, never load the game, and close every owned listener. Recursive cleanup validates its uniquely owned temporary directory before removal.

`lease-supervisor-tests.log`: **4/4** existing cases independently passed: idempotent OS lease/reuse, OS release after actual owned-child death, failed service closing its actual sibling, and explicit stop awaiting all owned children. These four plus the five permanent Vite cases are nine independently executed permanent regressions; the separate seven-case investigation is not an additional unique coverage count.

Both installed Vite copies have identical pinned implementation bytes. No persistent game/Studio server, browser, preview game, Rust build, source push or ledger update occurred. Source review and ephemeral Node fixtures establish the bounded lifecycle contract, not PWA/render/device acceptance. The permanent test commit is ready for Root cherry-pick.
