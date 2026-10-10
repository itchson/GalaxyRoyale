# Independent bounded RoyaleMono wrapper review

No blocking defect was found in the frozen proposal or the inspected production integration. Acceptance here covers native layout/packet parity and the narrow allocation observation. The normal release build, browser presentation, WASM stack behavior and physical devices remain separate gates.

## Pinned candidate and actual reruns

The reviewed candidate manifest is `work/ui-text-allocation-candidate/evidence.json`, SHA256 `7c5c6855e90366b6fe21398fa6bb11aab9f0dc576bc19befa480ed23e3cedaee`. The minimal patch SHA256 is `f2523703a82c02ac24be19daf8d89c18502058419439b3eb80d8be30bb3e1a88`. Independent read-only verification matched all **23 frozen candidate files and seven originally linked native libraries**. No author preparation/build/freeze script was rerun or candidate file rewritten.

The existing frozen executables were run with output captured only in this directory:

- Four private-wrapper oracle tests passed: explicit whitespace/newline/Unicode/long-word cases, 98,040 exhaustive short comparisons, 2,100 seeded prefix comparisons and maximum storage boundaries.
- Two actual public extraction parity probes passed, comparing every emitted pixel, affine transform and batch order across 2,592 text/button/input fixtures, three 512-node cap/clip scenes and the 59,989-quad mixed scene. They include centered buttons, partial clips, Unicode, fonts 1/2/4, row limits and far-camera group transforms. Transform indices remain valid; the existing 60,000-cell cap is preserved.
- Thirteen retained UI tests passed, including cached glyph/letter transforms, short buttons, clipping, malformed geometry, reduced motion and cursor behavior.
- The separate four-scene actual System allocator probe passed and produced JSON semantically identical to the author result. Each original/candidate extraction was warmed 60 times and counted over 120 calls; packet comparisons and scene parsing are outside the counter windows.

These are reruns of the reviewed focused executables, not a new full-workspace or WASM test pass. Their dependency/source snapshots remain historical pins even after root applies new production work.

## Storage and text semantics

The wrapper takes **256 Rust Unicode scalar values**, rather than 256 UTF-8 bytes or grapheme clusters. Its output character array has exactly 256 slots and its line span array exactly 257 slots; each `u16` offset is at most 256.

The stored characters cannot exceed the prefix length: every input word character is copied once, and an inserted normalized separator between distinct words replaces at least one discarded input whitespace character. Long-word chunks flush at the column boundary and do not add an extra separator between chunks. There is no unbounded text-dependent allocation or unbounded append source.

Line storage is also sufficient. Every nonempty full/remainder line consumes at least one of the prefix's characters. Empty paragraph lines are charged to explicit newline positions, with at most one additional trailing empty paragraph. Thus the worst case is exactly **256 newlines → 257 empty spans**. This boundary, plus 256 one-character Unicode lines and inter-word whitespace, is executed by the focused tests. A zero column count is rejected privately; the only production caller returns before wrapping when computed rows or columns are zero, matching the previous `chunks(0)` precondition.

The old behavior is retained: only `\n` creates paragraph boundaries; `\r`, tabs and other Unicode whitespace are normalized within a paragraph; whitespace-only nonempty paragraphs are omitted; explicit empty paragraphs are retained; leading/trailing newlines and 256-character truncation are preserved. Non-ASCII characters remain whole scalar values. NUL/unsupported glyph and combining-character rendering are not newly expanded capabilities; the existing cached glyph lookup still decides which cells are drawn.

Centering now uses `WrappedLine.len()`, which is the length of a **`[char]` slice** and therefore equals the old `String.chars().count()`. It is not UTF-8 byte length. Font choice, letter advance, parent/local affine composition, row selection, clipping and shared work/submission limits are unchanged in the inspected runtime diff.

## Inspected production scope

The actual `crates/game_web/src/ui_scene.rs` integration contains the proposed private fixed-storage wrapper, the character-slice length centering change, and a `#[cfg(test)]` module referencing `ui_wrap_tests.rs`. The separate durable file retains the exact original allocating wrapper as an oracle and the four focused specialist tests. The Unicode fixtures were inspected through explicit UTF-8 reading and contain `é😀Ω日本語` and the intended escaped whitespace values intact. Root's initially failed Windows-decoding import was corrected before the passing tests; no malformed fixture is accepted here.

The production UI diff adds no public API, save field, dependency, manifest, cache behavior or exported fixture. The test-only oracle may allocate while testing; it is excluded from ordinary compilation by `cfg(test)`. Formatting-only changes to the helper/test file do not change their control flow. Other simultaneously integrated game/controller work is outside this UI allocation acceptance.

## Allocation scope and limits

| Native warmed scene | Original allocation / reallocation / requested bytes | Candidate |
| --- | --- | --- |
| Mixed 59,989-quad UI | 5 / 4 / 592 | 0 / 0 / 0 |
| Unicode multiline text | 15 / 4 / 399 | 0 / 0 / 0 |
| Button text | 8 / 8 / 392 | 1 / 2 / 56 |
| Console input | 11 / 11 / 706 | 3 / 4 / 290 |

Requested bytes sum allocation request sizes and each reallocation's target size. They are not live memory, high-water memory or saved memory per application frame. Native `WrappedLines` object size is 2,072 bytes and the separate raw input array 1,024 bytes; the executable printed those sizes, not measured compiler stack usage. This fixes one private text-layout allocation source while leaving button/input preparation, deserialization, other UI paths, cold caches, JS/DOM and GPU allocations outside the zero-allocation observation.

No timing improvement, browser allocator result, phone frame rate or long-run stability is inferred. A later normal build should validate the same production helper and ordinary export exclusion, then inspect centered/Unicode/newline/scroll-clipped UI through the actual renderer at fractional scales. The frozen desktop stress report remains unchanged and is not evidence for a wrapper timing gain.
