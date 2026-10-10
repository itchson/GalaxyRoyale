# Primary cadence correction — isolated, ready for integration review

Production remains unchanged. The candidate is an actual copy of the reviewed `Game` core, linked to unchanged production modules/assets and existing native dependencies. It adds no manifest, dependency, profile field, HUD field, renderer change or new firing system.

## Exact minimal change

[integration.patch](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/work/weapon-cadence-fix/integration.patch) contains three narrow hunks in `crates/game_core/src/lib.rs`:

1. Decrement a **positive** cooldown. Normalize an absolute residue ≤one microsecond to zero, preventing exact f32 tick boundaries from accumulating an extra frame.
2. Keep the existing single `if input.fire && !safe && ready` gate. When input is inactive or sanctuary protection applies, discard a negative residual while retaining a positive pending cooldown.
3. After one primary invocation, **add** its authored period rather than resetting the timer, preserving held-fire fractional overshoot.

This preserves short-release/pause cooldown, gives a ready weapon one immediate shot after idle, prevents idle/sanctuary burst debt, and retains the one-primary-per-tick policy. Echo/ship signature calls and `shot_count` increments remain in the existing firing invocation. Current authored maximum is nine shots/s, below the fixed-tick limit even at the accepted 0.05s step. There is no catch-up loop or idle priming.

## Actual native evidence

[tests.rs](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/work/weapon-cadence-fix/tests.rs) supplies **seven focused tests of real Game::tick**, not a duplicated clock algorithm. Profile fixtures explicitly grant inherited survival hardware/XP; diagnostics suppress production rewards. Existing 75 tests were compiled but filtered from execution, so this is not a claim that all production tests were rerun.

| Firing rank | Existing steady shots/s | Candidate primary shots / 30s | Authored shots/s |
| ---: | ---: | ---: | ---: |
| 0 | 4.615 | 150 | 5.0 |
| 1 | 5.455 | 165 | 5.5 |
| 2 | 5.455 | 180 | 6.0 |
| 3 | 6.000 | 195 | 6.5 |
| 4 | 6.667 | 210 | 7.0 |
| 5 | 6.667 | 225 | 7.5 |

Raw candidate ticks are retained in [steady-results.json](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/work/weapon-cadence-fix/steady-results.json); the existing-release raw measurements remain in [Scout results](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/work/scout-progression-audit/results.json). Candidate intervals are the adjacent floor/ceiling of the ideal fixed-tick period; every rank improves its long-window throughput.

Tests also pass long idle → immediate first shot → full new period, short release → retained positive remainder, paused time → unchanged remainder, held fire throughout Haven → zero shots/debt, explicit death-return/retry, variable accepted dt/invalid dt → at most one primary per tick, and **actual newly created Echo projectiles** on every eighth primary through pause/release. No XP change occurred in cadence diagnostic fixtures. Death-return uses a disclosed regen-zero fixture: the pre-existing `diagnostic("die")` sets zero hull without resetting damage time, so active R5 regen can heal that forced diagnostic before death processing. Natural lethal hits reset damage time and are outside that helper issue; this patch does not alter death handling.

Both candidate library and test compilation pass strict native `clippy-driver -D warnings`. The independent QA agent linked the frozen candidate's public API and separately confirmed all six 30s rates, eight crafted R5 ships, Sovereign nine shots/s, idle/release/reset/pause boundaries and E1/E5 real +3/+7 Echo volleys on primary 8/6. Its separately owned report records exact evidence and limits.

## Reproduce without a new manifest

From repository root:

```powershell
.\work\weapon-cadence-fix\build.ps1
.\work\weapon-cadence-fix\cadence-tests.exe cadence_audit:: --nocapture
```

`prepare.py` requires the original production `lib.rs` SHA256 `E2EF687C9F14D8B75AF6B36FBB220B8244979FF35A042D8B0CE6D579FABD6702`, writes only inside this directory and emits the minimal patch plus the compiled candidate source. It rejects changed source rather than silently rebasing. Read-only inputs and candidate identity are in `source-hashes.json`. `build.ps1` pins existing root native rlibs and produces `libgame_core_cadence_candidate.rlib` plus `cadence-tests.exe`; independent probes can link that actual core with `-L dependency=target/debug/deps`.

Tests write proof JSON only when the task-specific `GALAXY_CADENCE_PROOF_FILE` environment variable is set. Their assertions do not depend on a work directory or proof file, so they can be copied into a new private test module at production integration. The work-only candidate's absolute module paths and `cadence_audit` declaration are build scaffolding; **only integration.patch's timer hunks belong in production**.

## Remaining acceptance

Root should apply/review the three hunks, install the focused test module, then run the required core/WASM and dual-renderer build checks for the resulting source. This isolated candidate has not been published or tested in a browser/phone. The corrected authored primary rate increases real throughput over the old release, including baseline; enemy difficulty and point costs were intentionally untouched. Human onboarding/Coil tuning remains Pending and should use the corrected cadence rather than the old plateaus.
