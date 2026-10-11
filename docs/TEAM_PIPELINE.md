# Development and integration

GitHub repository: https://github.com/itchson/GalaxyRoyale. `main` is the integration branch and `codex/testing` holds the latest integrated candidate. The existing project history is retained. The game remains a Rust/WASM desktop-first solo game. Mission Control's service is bound to localhost; its tracked project ledger and source evidence are available in this public repository.

## Assignments and isolation

The producer scopes work against a real Mission Control task. Each specialist receives acceptance criteria, owned files, dependencies and a bounded objective. Feature branches use `codex/<department>-<feature>`. Use a separate managed worktree per active branch; do not switch a shared checkout underneath another agent or share its Git index. At most four native agents and two explicit Spark workers run concurrently. Spark tasks use the installed worker wrapper with an explicit UTF-8 task file, matching permissions and no recursive delegation.

For the presentation work already running when GitHub was connected, specialists initially used disjoint file ownership in the existing checkout. Their frozen changes are transferred to dedicated branches for review. This is a migration step, not a claim that those earlier edits were made in isolated worktrees.

## Review and merge

1. A specialist records changes, executed checks and remaining limits, then freezes a candidate commit.
2. Open and attach a pull request with exact Mission Control IDs and evidence. Dependencies may use stacked branches; rebase or merge the integrated dependency before final review.
3. Another actual agent reviews the candidate. Record its contribution, findings and resolutions; agents share one authorized GitHub identity, so internal review is not falsely represented as approval from another GitHub account.
4. GitHub Actions executes native and shell checks on Linux and Windows, plus the dual WebGPU/WebGL2 package, serialized WASM/IndexedDB save checks and WASM warning checks. Passing CI does not establish browser visuals, audio comfort, human balance or phone performance.
5. The producer reviews the combined source, resolves conflicts in `codex/testing` and runs checks appropriate to the result. Merge only a reviewed head with passing required checks. Preserve its exact SHA and link the PR/build in Mission Control. Keep one project game-preview server; stop superseded project previews before starting another and serve the latest recorded candidate. Temporary QA tools have their own recorded lifetime and are stopped after their checks.
6. Publish the exact checked source and archive through the existing private Sites workflow. GitHub build artifacts are candidates, not automatic deployments. Keep the shipped game independent of Studio.

The producer is the current integrator. Integration/review assignments rotate to a free specialist for independent scrutiny; a second imaginary always-running agent or fabricated meeting is not recorded.

`main` now requires a pull request, resolved conversations and all three verified GitHub Actions checks with an up-to-date base, including for administrators. Force pushes and branch deletion are disabled. The review count remains zero because agents share one authorized GitHub account; actual independent review evidence is mandatory in Mission Control and source instead of inventing a second GitHub approval. The observed policy is retained in `artifacts/presentation-refinement-20261011/github-main-protection.json`.

## Local candidate preview

Build on `codex/testing` with `pnpm build`, then use `pnpm preview:game`. The launcher verifies the recorded commit, branch, runtime source/assets and packaged output, refuses an older or modified build, and refuses another owned project game Vite process even on a different port. Vite also uses a strict port: `http://127.0.0.1:4173`. The launcher never kills an unrelated process. Stop the existing preview with Ctrl+C before building and starting its replacement. Studio remains separate on 4174/4175. `work/game-preview.json` records the live owned PID and exact candidate; `work/preview-build.json` is local build evidence, not a shipped dependency.

Both Vite development and preview now reserve a single OS-held loopback lease on 4172 before listening. This also covers separate worktrees and explicit alternate game ports. A second launch fails without stopping the first or killing an unknown port owner. The lease accepts no application traffic and the OS releases it when its process exits. Studio can run independently with `node scripts/dev.mjs --studio-only`; its supervisor waits for all owned children to close after a stop or a child failure. Inspect actual listeners and recorded process identity before replacing a build. A stale PID record alone is not proof that a process belongs to this project.

Only the integrator starts the persistent game preview. Specialists use source checks and temporary test listeners with explicit cleanup, and report their actual last state. Avoid overlapping expensive full builds: specialists run bounded checks; the integrator performs the combined build. Browser checks reuse the one server, close agent-created test tabs afterward, and restore viewport overrides. A browser tab is also a running game client and is included in the cleanup pass.

## Production records

Mission Control at http://127.0.0.1:4175 owns task statuses and append-only reviews, decisions, builds and actual last-recorded run states. Only its localhost API serializes project-document writes and revision conflicts. Discussions are asynchronous design/review exchanges during actual active sessions. Feedback remains queued between sessions; this pipeline does not imply unattended agents or scheduled work.

Use canonical agent paths such as `/root/engine_research` for future run observations. Earlier short identifiers are retained as legacy records with their actual bounded scope completed; they are not extra running staff.

Completion requires evidence for the task's actual acceptance criteria. Keep code implementation distinct from natural play, audible review, named physical hardware and soak tests. Reconcile the session's requests with existing tasks before creating duplicates. Future co-op, player-owned arenas and battle royale remain explicit backlog; a portable topology primitive does not complete those modes.

The current parallel work is bounded: gameplay owns co-op signaling and admission tests; art owns the generated PAN dock's explicit conversion and native animation proof; engine reviews integrated node geometry, damage-aware fine quads and runtime lifecycle; root integrates, verifies the UI and maintains the ledger. Audio remains a separate listening/refinement queue until a worker is actually assigned. These are real assignments, not a claim of continuously running departments or completed meetings. Co-op signaling alone does not establish a shared Rust simulation.
