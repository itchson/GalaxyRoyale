# Wake presentation contract refinements, version two

This addendum preserves the frozen `DESIGN.md` SHA-256 `fd85ae1c87bcf7e44697bd2481aadccd6c924c0fe9b1349aa7e6ae455ca313b3` and its original `source-pins.json` SHA-256 `a65652f61c13ae635d13490ba4310f46d6cdf39769add81ff017718c61bc8a9b` unchanged. It resolves three concrete compatibility/lifecycle findings from `engine_fx`'s independent design review. No implementation, native execution, build, browser or RTC acceptance is implied.

The existing recommendation remains one full active-state wake page, at most 192 points/24 per member/32 KiB, using the existing admitted host `body` frame kind and immutable acquisition/revision. Header metadata/count, exact page completeness, shared five-dot native quads, bounded publication/backpressure, first active-state visibility, strict newer-tick authoritative revalidation and separate birth-light scope are unchanged. Root owns the strict JavaScript wire/controller/adapter changes.

## Identical raw and canonical numeric validation

The page reader must validate raw JSON scalar values before conversion to native f32. A serde DTO that deserializes life immediately into f32 is insufficient: a raw number just outside the agreed maximum can round down into range, while a positive raw number can underflow to zero.

Use raw finite f64 values for new wake page life/position DTO fields, then explicitly validate and convert into canonical native presentation f32 values. For life, both boundaries require:

1. Raw scalar is a finite number, strictly positive, and <= the exact f64 representation of `3.4_f32` (3.4000000953674316).
2. Canonical f32 conversion is finite and strictly positive, and remains <= `3.4_f32`.

JavaScript uses the same raw numeric maximum `Math.fround(3.4)` and the positive finite result of `Math.fround(raw)`. Rust performs the equivalent raw f64 check before `raw as f32`, followed by the f32 check. Do not require raw equality to the canonical f32 value: legitimate short native f32 serializers, such as raw `3.4`, should remain legal and canonicalize to the same native value. Null/strings/booleans/missing fields reject under the existing strict contract.

Illustrative conversion outcomes from IEEE-754, not new native serializer executions:

| Raw life | Canonical f32 | Both boundaries |
| --- | --- | --- |
| `1e-46` | `0` | reject underflow |
| `3.4000001` | `3.4000000953674316` | reject raw out-of-range number |
| `3.4` | `3.4000000953674316` | accept |
| `3.4000000953674316` | unchanged | accept |
| `0`, negative, nonfinite | invalid | reject |

Actual implementation tests must export the actual host `co_op_batch` Value-converted point fixtures and exercise the native replica page decoder plus the strict browser boundary. Hand-created wire fixtures or a conversion model do not establish an executed native acceptance gate. Include smallest positive f32, immediately adjacent maximum values, positive raw underflow, short decimal forms and out-of-range round-down cases. JavaScript and Rust must accept/reject the same inputs; compare canonical native values after successful conversion.

## Legal native anchor margin

The new wake coordinate limit is `WORLD_LIMIT + 14`, not the pilot-position limit. Frozen native code clamps pilot/pre-integration origin to the world boundary but admits the actual wake at `origin - forward(current_heading)*14` without clamping the sample. An inward-facing pilot at x=1,000,000 may therefore stamp x=1,000,014. This is a real legal gameplay sample and must not be clamped, moved, inferred or omitted for cosmetic transport.

Validate each raw f64 coordinate as finite and within +/-1,000,014, then require the converted f32 coordinate to be finite and within the same representable f32 bound. `1,000,014` is an exactly representable f32 integer. JavaScript applies the identical raw bound and `Math.fround` post-conversion bound. Only this new wake projection receives the 14-unit margin; existing pilot, projectile, entity and other protocol limits stay unchanged.

Future meaningful native tests should place a real admitted fast pilot at each world edge/corner, face inward, run the real fixed tick and capture its actual admitted sample through the host bridge. Verify a legal outward rear anchor crosses both native and browser validators, remains numerically unchanged, and produces the shared native quads. Test the first raw value outside the margin, nonfinite values, and a raw value that would otherwise round into the f32 margin. Do not substitute a synthetic sparse-motion reconstruction test or change native damage samples to make transport pass.

## Same-tick canonical captured-set immutability

For every `(participant ID, epoch)` surviving from the previous successful committed header into a new header with the same authoritative tick, require its complete captured wake set to be unchanged. This includes stable birth keys, canonical f32 positions and canonical f32 captured life. Compare the validated canonical page data; do not compare raw f64 spellings, serialized decimal text or receiver-aged visual remaining life/deadlines. Equivalent short and Value-converted f32 representations must compare equal. The local clock may age visuals while the underlying captured set stays immutable.

Reject a same-tick legacy/missing/zero descriptor or positive page that changes a surviving member's captured set before moving the visible Scene/cache. The counterexample `populated tick T -> committed zero/legacy tick T -> populated same key at tick T` is closed because its first changed complete snapshot rejects atomically; the old canonical set and expired/preserved deadline remain. Repeated equivalent same-tick captures continue to use `min(old_deadline, receipt_time + captured_life)` and cannot extend or resurrect locally expired visuals.

This rule follows actual native lifecycle: aging, birth, cap removal and respawn reset occur inside the checked fixed `tick_co_op`. Same-tick admission/leave can change membership, but cannot mutate the trails of a surviving exact membership. Removed members/changed epochs may clear their own state; a new membership is evaluated independently. New epoch/slot reuse never inherits the old participant's points or deadline.

A strictly newer committed native tick remains fresh evidence of the exact currently active samples and captured remaining life. It may revalidate an actually alive point after host pause/slow simulation, without replaying sound, damage or a birth activation. Legacy/missing/zero state still clears on that newer complete tick. Receiver-only presentation expiry is not permanent native gameplay death.

Retain the bounded current canonical set alongside its receiver deadlines (at most 192 points) until legitimate replacement/removal. No unbounded event history or new persistent tombstone storage is required. Perform same-tick comparison only after full page/geometry/membership validation and before visible state moves; invalid input cannot advance the comparison baseline or partially clear state. Failed/incomplete body commit and invalid receiver clock continue to preserve existing state.

Future tests must exercise actual same-tick repeated captures; equivalent canonicalized scalar forms; populated->zero, populated->legacy and positive key/position/life mutations; mixed member removal plus unchanged surviving trails; failed commits; expired visuals with retained canonical set; valid next-tick empty clear; next-tick paused-host revalidation; and new epoch reuse. Fixtures that arrange native state directly must not manufacture same-tick surviving-set changes and then label them a production-native capture path.

## Integration note

Root has integrated the gameplay candidate and independently fixed the semantic-equivalent ownership replacement to `hit_owner.replace(owner)` plus redundant test borrows during strict workspace Clippy. A future trail implementation must preserve those Root changes rather than restoring the older frozen spelling. The original `87de1f7` source and original design pins remain unchanged in this worktree; this addendum introduces no source amendment. Implementation remains pending Root authorization and the independent review of this exact versioned text.
