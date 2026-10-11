# Final asteroid cluster revision

Current source: **`ebcaf46e41354ac3a18725d36afb3adea4e832a3`**, parent `59e6603f43204b15d42f88f25c9777591e0dab26`, branch `codex/asteroid-diversity`. The follow-up changes only `asteroid_art.rs` and `asteroid_diversity_tests.rs`; the tracked checkout is clean. Integrate after the original four-file commit, which Root already cherry-picked as `f7b9b31`.

The previous sheet's crossing cyan/gold mineral seams and long metal ribs were visually rejected. This revision removes those paths completely and authors four separated irregular mineral nodules with small shaded facets. Existing shape planes, connected filled occupancy, seed/material/tier selection, actor scale/radius/HP, speed, collision/destruction, reward and save paths remain unchanged from the original implementation. The old source/contact/logs remain immutable **historical evidence**, not the final art recommendation.

`native-software-contact.png` is the final viewed 32-case contact, SHA256 `7f3487e507c3d394627ba70154f279aed68027fd76aaebf85d730a7fd646f50c`. It is rendered from actual native FrameBuilder packets with the unchanged half-open flat software coverage helper at display ×3. Root inspected it and reported that the artificial crossing lines are gone and compact deposits fit better. This is source/contact review, not browser, shader, lighting, small-zoom or human gameplay acceptance.

Executed checks on this revision:

- Actual-cell regression: **1,024** mineral components across **256** generated bodies. Each body has four separated components, each 6–40 occupied cells and at most 9×9 local cells; a long skinny streak fails the extent check.
- `cargo test -p game_core asteroid --locked -- --nocapture`: **18 passed**, 356 filtered, zero failures.
- `cargo test -p game_core --locked`: **373 passed**, one pre-existing ignored cosmic export, zero failures.
- `cargo clippy -p game_core --all-targets --locked -- -D warnings`: passed. Both owned files pass rustfmt check; diff check passed.
- Separate packet export: one actual native test passed, 373 filtered. All **32** paired old/new packets have identical seed/family/tier, occupied local positions, counts and transforms. The old packet corpus fails the compact-deposit bound in **32/32** cases; the new corpus passes **32/32**. `packet-comparison.json` retains every case; the unchanged-geometry comparison uses real exported packets rather than a fabricated image.

Original measured diversity/radius and conservative budgets remain: sampled 48 unique silhouettes per family, 1,139–2,098 occupied cells; actual sampled maximum radius 20.291021, theoretical fully occupied max-scale grid radius 31.1127 below the unchanged 32-unit belt reservation. Once-at-birth 64×64 generation and a fixed 960-byte palette cache; no per-frame art cache. Production generation asserts ≤3,000 occupied cells. Ordinary streamed cap112 gives a conservative foreground asteroid-only ceiling336,000 quads/112 body transforms before viewport culling, outside region6,000/3,000 cosmetic quotas. PixelBody payloads total1,892,352 bytes at112, excluding ECS/allocator/temporaries. Cold/warm CPU and GPU cost were not measured.

No game/dist build, browser/server, Studio write, publication or new dependency was performed. Root owns integrated GPU/GL and natural play acceptance. `WIKI_DRAFT.md` is a proposed human-readable addition only; it does not modify canonical documentation or Studio. `frozen-hashes.json` and `verify.py` pin the final source and raw evidence without re-executing writers.
