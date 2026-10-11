# Independent Studio DevBible review

Reviewed clean feature commit `e1eaad369c7eec7a06a25c6a3aa8df215bfdb389` in its dedicated Wiki checkout. Source snapshots/pins and executed check results are recorded in `evidence.json`. No production source, records, browser or build output was changed by this review.

No blocking file-access, rendering-injection or revision/persistence defect was found. Exact-listed source/media routes reject traversal, hidden/absolute/unlisted paths and link ancestors; source responses are plain text with nosniff/sandbox headers. Catalog prose/attributes are escaped and link URLs are constructed locally. The phase transaction admits only two metadata fields and checks the global revision inside the existing queued atomic persistence path; invalid/stale edits preserve tasks/history and disk state.

Executed independently: frozen verifier (32 evidence files/24 live inputs), all 32 actual Studio tests (zero failures/skips), and strict TypeScript. Test fixtures exercised real temporary disk/API servers and closed them; the tests are not an integrated frontend/browser or game acceptance run. Source-fact checks cover actual named registries/constants/atlas occupancy, not every catalog sentence or natural balancing.

Two bounded follow-ups were sent to Root:

1. The Courier fact at catalog line 1225 says collect the packet and return it to Haven. The runtime then requires returning to the source beacon to claim (`interact_site`, core lines 1035/1060; SiteHud objective line 3255). Append that step so the reference is complete. Mission cargo correctly remains separate from destructible crafting materials.
2. Search input invokes full main DOM replacement on each edit; main retains only focused identity, not selectionStart/selectionEnd/direction. Editing the middle of a query or an SSE refresh can lose the cursor/selection on the replacement input. This is source-derived and has not been browser-reproduced here; retain selection or update results without replacing the input.

Remaining integration gates: desktop/narrow layout, keyboard focus/caret, related/deep-link navigation, captions/source-reader behavior, and actual two-editor stale phase handling after API restart. There is no CI, GPU, human play, physical phone or publication claim. Existing preview-guard review and its repair addendum remain separate, unchanged evidence.
