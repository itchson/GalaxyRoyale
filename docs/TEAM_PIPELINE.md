# Development and integration

GitHub repository: https://github.com/itchson/GalaxyRoyale. `main` is the integration branch. The existing project history is retained. The game remains a Rust/WASM desktop-first solo game; Mission Control remains private and local.

## Assignments and isolation

The producer scopes work against a real Mission Control task. Each specialist receives acceptance criteria, owned files, dependencies and a bounded objective. Feature branches use `codex/<department>-<feature>`. Use a separate managed worktree per active branch; do not switch a shared checkout underneath another agent or share its Git index. At most four native agents and two explicit Spark workers run concurrently. Spark tasks use the installed worker wrapper with an explicit UTF-8 task file, matching permissions and no recursive delegation.

For the presentation work already running when GitHub was connected, specialists initially used disjoint file ownership in the existing checkout. Their frozen changes are transferred to dedicated branches for review. This is a migration step, not a claim that those earlier edits were made in isolated worktrees.

## Review and merge

1. A specialist records changes, executed checks and remaining limits, then freezes a candidate commit.
2. Open and attach a pull request with exact Mission Control IDs and evidence. Dependencies may use stacked branches; rebase or merge the integrated dependency before final review.
3. Another actual agent reviews the candidate. Record its contribution, findings and resolutions; agents share one authorized GitHub identity, so internal review is not falsely represented as approval from another GitHub account.
4. GitHub Actions executes native and shell checks on Linux and Windows, plus the dual WebGPU/WebGL2 package, serialized WASM/IndexedDB save checks and WASM warning checks. Passing CI does not establish browser visuals, audio comfort, human balance or phone performance.
5. The producer reviews the combined source, resolves conflicts in the integration branch and runs checks appropriate to the result. Merge only a reviewed head with passing required checks. Preserve its exact SHA and link the PR/build in Mission Control.
6. Publish the exact checked source and archive through the existing private Sites workflow. GitHub build artifacts are candidates, not automatic deployments. Keep the shipped game independent of Studio.

The producer is the current integrator. Integration/review assignments rotate to a free specialist for independent scrutiny; a second imaginary always-running agent or fabricated meeting is not recorded.

## Production records

Mission Control at http://127.0.0.1:4175 owns task statuses and append-only reviews, decisions, builds and actual last-recorded run states. Only its localhost API serializes project-document writes and revision conflicts. Discussions are asynchronous design/review exchanges during actual active sessions. Feedback remains queued between sessions; this pipeline does not imply unattended agents or scheduled work.

Completion requires evidence for the task's actual acceptance criteria. Keep code implementation distinct from natural play, audible review, named physical hardware and soak tests. Reconcile the session's requests with existing tasks before creating duplicates. Future co-op, player-owned arenas and battle royale remain explicit backlog; a portable topology primitive does not complete those modes.
