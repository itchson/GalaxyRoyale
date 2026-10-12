# Ordinary co-op Wake native budget audit

This audit uses exact integrated runtime `332ddd5ec0370e13ad346ea4ea988f35b61ca3e7` and docs checkpoint `60e16acaba7ceda12afd02bf497f6aadd34da885`. Native/game sources have no diff between those refs. A Git archive and two required Git test fixtures were extracted only under this work directory. The original `codex/private-room-negative-qa` branch remains at `1b14bff9e6da0a1a249f1bdc42ab49e59e74e52c`; current v2 `7c188d4198901ead15e4bffa2b0063b9c95f7674` remains tracked-clean. Neither tracked Root nor isolated production sources changed. No moving trail candidate was read or measured.

The repository's `core.autocrlf=true` Git archive exports text with CRLF; canonical Git blobs use LF. The verifier separately pins canonical blobs/SHA-256, exact exported archive bytes, and exact actual compiled snapshot bytes. It accepts only verified CRLF-to-LF equality to those blobs when literal equality differs. Binary data is compared literally. The one work-only test append is pinned separately, and every archived compiled input is checked.

## Source and resource invariants

`update_co_op_wakes` visits the fixed eight participant-origin slots. Ordinary samples age and expire in each Pilot's own vector; the stamping path enforces capacities 12/16/24 and TTL 2.0/2.6/3.4 seconds. Evolution five forms at most 23 short links per pilot, rejecting lengths above 160, Haven overlap, and canonical sanctuary overlap. Eight maximum configured trails therefore hold 192 samples and 184 candidate links. Ordinary Wake adds no projectiles. Damage queries real Motion/Actor entities, accepts only enemies/bosses, and applies sanctuary exclusion before damage; boss multipliers and slot-ordered kill attribution remain authoritative. Lights/events share existing bounded 32/128 buffers. These checks preserve simulation independence from render backend.

The target query scans all Motion/Actor entities per active pilot, including nonhostile entities before its kind predicate. The ordinary spawn constant of 48 enemies does not prove the entire Motion/Actor query has only 48 rows: bosses/subobjects/world entities can add rows. The configured 48-target probe below is not a global worst case.

At this checkpoint `Snapshot` carries owned Wake equipment/evolution claims but no ordinary per-point trail geometry. Clearing all 192 configured samples without changing time/membership/world leaves the actual serialized snapshot and published occupied/storage counts identical. The native publication path is bounded independently: at most eight participants, 180 projectiles, 512 body entities, dimensions 1..512 per axis, and total body storage at most 1,048,576 cells. It admits scalar and aggregate bounds before body signatures/clones; an error retains the prior immutable cache. Publication acquisition replaces one packed immutable view, with checked generations and membership invalidation. Entity pages are 64 rows and body pages 1,024 occupied cells; borrowed packed bodies are available to native viewers. These are packing/capacity guarantees, not a transport bandwidth or GPU budget.

## Actual native execution

Commands compiled only `game_core` in a separate work-owned target directory, Rust 1.97.0 / x86_64-pc-windows-msvc, release optimization, thin LTO, `--offline --locked`, one test thread. This was a native test build, not the normal game/WASM build. The probe was appended only to the archived native test module; all other archived compiled bytes were preserved. Its test global allocator delegates to System and counts allocation/reallocation calls plus requested sizes on the measured thread. Requested bytes include repeated allocations/reallocations and are not retained memory, peak RSS, leak size, or total process activity. Instrumentation and Windows CPU scheduling affect times.

Actual checks: **14 Wake tests pass** (13 unchanged tests plus the work-only probe), **7 publication tests pass**. The existing eight-pilot test executed 300 real authoritative co-op ticks with real submitted inputs, checked sample/light/event bounds and zero Wake-owned projectiles, and preserved Profile. The existing differential test executed 25 configured rank/evolution builds against solo Wake stages. Raw commands/results are in the logs; no browser, GPU, RTC, listener or physical pass is inferred.

The isolated stage probe arranges eight ordinary evolution-five/rank-five builds. It inserts 24 finite samples per Pilot with 3.4-second TTL, freezes stamping through a long timer and zero motion, and chooses real sanctuary-free target positions. Each of five batches warms 20 stages and measures 120 actual Wake stages at dt=1/60. Assertions prove the 192 samples and 184 candidate links remain present throughout each batch; those are deliberately configured maximum trails, not earned progression. Miss targets have high health and are outside all point/link reach. Five conditions measure 3,000 stages per run; the probe ran once alone and once when the full focused Wake suite included it.

The first successful standalone run (`native-probe.log`) reports these batch medians:

| Configured Wake stage | Native time / stage | Allocation or realloc calls / stage | Requested bytes / stage |
|---|---:|---:|---:|
| 8 pilots, zero samples, zero enemies | 0.912 µs | 0 | 0 |
| 8 pilots, 192 samples, zero enemies | 120.343 µs | 1,872 | 179,904 |
| 8 pilots, 192 samples, 48 local misses | 114.997 µs | 1,872 | 179,904 |
| 8 pilots, zero samples, 48 remote misses | 181.678 µs | 3,840 | 359,424 |
| 8 pilots, 192 samples, 48 remote misses | 294.396 µs | 5,712 | 539,328 |

The second actual run inside `native-wake-tests.log` has identical allocation/byte counts but different timings (full/remote median 364.065 µs). This variation is retained and precludes treating the table as stable frame timing or an FPS claim. The isolated stage excludes AI, movement integration, chunk streaming, rendering, serialization and transport.

The same work probe separately measured 600 packed acquisitions over five batches on 8 pilots + 48 small hostile bodies: median **50.148 µs/acquisition**, 295 allocation/reallocation calls and 105,144 requested bytes per acquisition. Actual counts were 56 entities, 19,200 storage cells, 6,064 occupied cells, 1 entity page and 56 body pages; capture JSON was 5,248 bytes and all body-page JSON totaled 115,216 bytes. JSON serialization/page construction lengths were computed outside the acquisition timing. These bytes are not network usage and omit entity-page bytes.

The existing larger eight-pilot publication fixture also actually ran: 120 entities, 181,047 occupied cells, 477,184 storage cells, 2 entity pages and 233 body pages. Its 32 warm acquisitions reported p50 **1,359.900 µs**, p95 **1,484.600 µs**, header 2,998 bytes, entity-page JSON 43,434 bytes and initial full body-page JSON 3,282,949 bytes. Its fixture/body workload differs from the small probe, so their acquisition times should not be equated. It remains CPU/native publication evidence, not whole-tick or browser performance.

## One concrete bottleneck and bounded follow-up

The target predicate calls `safe_zone_at(target)` before asking whether any Wake point/link reaches that target. Remote targets outside the host site cache invoke `regions::protected` → `sites_near`, materializing a nine-site Vec and formatted site ID Strings. Eight pilots × 48 remote misses repeat that work **384 times per stage even when every Wake is empty**. The real empty-remote fixture's 3,840 allocations and 359,424 requested bytes versus zero for empty/no-target stages confirm this cost. Full-remote minus full-local yields the same 3,840 / 359,424 delta. Links independently reconstruct their sanctuary neighborhoods once per link, explaining the remaining nonzero full/no-target baseline; no second rewrite is proposed here.

The smallest follow-up is to evaluate the already bounded point/link geometry predicate before the target sanctuary predicate, while retaining sanctuary rejection before damage. This would avoid remote-site work for definite geometry misses without introducing a cache or changing authoritativeness. Reuse this exact empty-remote/full-remote probe to measure allocation counts and native time, then rerun the unchanged solo differential, sanctuary/link-crossing, hostile/boss/body, eight-pilot boundedness and publication cases. The expected removed allocation delta is a source-backed hypothesis, not an implemented or measured optimization result. No runtime change was applied in this audit.

## Preserved failures and limits

The first compilation failed because the initial archive omitted two `include_str!` fixtures and the work-only query binding needed `mut`. Exact archived fixtures were added; the original probe and compiler log are retained. The next run measured all Wake conditions, then failed a probe assertion that searched for the text `wake` anywhere in JSON: the valid owned equipment claim contains that text. The assertion was replaced by the actual before/after serialized snapshot equality check; the failed run and original probe version remain intact. Neither failure is presented as a production regression.

The initial pin verifier also rejected Cargo.lock because it incorrectly expected literal canonical LF bytes from the configured CRLF Git archive. A subsequent pin count assumed directory entries were compiled files; the actual count is 469 verified files. Both error logs are retained. Final verification explicitly distinguishes canonical and compiled/export bytes; no measured source was changed to resolve those pinning assumptions.

This audit does not claim eight real connected clients, Internet/TURN, native guest rendering, actual game FPS, a memory-leak test, exact maximum-world cost, title DSP/listening, physical/mobile acceptance or proof of a later trail-publication candidate. Root owns integration and any optimization authoring/acceptance. All native processes ended; none remains running.
