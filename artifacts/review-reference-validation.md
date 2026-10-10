# Studio review-reference validation

The engine specialist changed only `apps/studio/store.mjs` and its store tests. New reviews accept omitted or empty task references; supplied references must contain at most forty unique existing string task IDs. Historical append-only records remain intact. The producer reviewed the diff and reran all thirteen actual ProjectStore/HTTP tests: [review-reference-store-tests.log](review-reference-store-tests.log).

Four regressions cover malformed input (including sparse arrays), caller-array isolation, restart persistence, forty/forty-one boundaries, queue recovery and the previously misread task-creation response (`item.id`). Rejected edits leave memory, revision, activity, listeners, backup and disk unchanged.

The producer restarted the authoritative localhost API and compared its complete snapshot before and after: revision 192 and all records were unchanged. An actual malformed POST returned HTTP 400 without modifying that snapshot, project file or backup. Evidence: [review-reference-live-rejection.json](review-reference-live-rejection.json). This verifies the local production service; it does not imply a new deployed game build.
