# Independent Root surface shader and Nova wire draft review

QA captured twelve stable raw inputs from Root's working draft at base commit `817f99ba95daea4a9cad299383425c75fe535ae8`. Hashes and exact copies are in `source-pins.json` / `source/`; these draft bytes are the review boundary, not an assertion that a later commit or compiled preview contains them. The native packing comparison uses exact `engine_core::Surface` source at `253dde5456754b063e32ebc203534662f0bfddad`.

QA inspected source and Root's actual saved logs. No compiler, native/shell test, WGSL validator, browser, renderer, reference viewing or build was executed by QA.

## Source verdict

No blocker found in the reviewed WGSL/light-height or matched Nova wire draft. Core surface integration, external CPU index updates, actual asset opt-in, coherent normal bundles and actual backend views are still required before claiming the rendered feature.

### Surface shader and uniforms

- The shader masks the transform word with 127 before table lookup and decodes oct X/Y at bits 7/15, emissive at 23 and validity at 31. This matches the frozen native Surface encoder's layout and axes. The word stays an integer; no complete-word f32 conversion occurs. Shader oct unfolding uses sign-not-zero, preserving lower-hemisphere/axis behavior. Normal and mask are flat cell outputs.
- The inverse-transpose calculation is correct for the proposed 3D embedding. Let normalized A have determinant D: the shader's XY adjugate multiplied by `sign(D)/sqrt(abs(D))`, with unchanged normal.z and final normalization, is algebraically equivalent to the CPU inverse transpose of `diag(A,sqrt(abs(det A)))`. It preserves the determinant sign for reflection and the full composed matrix for shear/nonuniform hierarchy; it does not apply camera, projection zoom or screen Y inversion to the normal.
- Matrix normalization by maximum absolute basis component makes the determinant test relative, allowing uniformly tiny matrices without an absolute quad-size floor. Zero magnitude or relative determinant below 1e-12 uses a finite +Z fallback. This fallback deliberately differs from the CPU f64 oracle for extreme anisotropy/near-singular bases; it should be documented/tested rather than called exact CPU/GPU parity for every finite f32 matrix. With finite native basis inputs, decoded unit normals and the guard, the remaining factor stays in a representable finite range.
- Directional light response correctly uses light.position minus the flat cell center, with positive height in world units. Height is clamped-world-radius times 0.35 in the existing spare light.color.w. Eight sources, original camera-relative positions and the 304-byte global uniform layout remain. Unused slots are still dark, and a missing/nonpositive height keeps the old isotropic factor rather than normalizing a zero vector.
- Legacy validity-zero cells initialize mask 1 and directional factor 1. Their original color conversion, ambient formula, attenuation, alpha and hard geometry remain in the source formula. UI stays unlit. This is source preservation with neutral factors; exact device pixel identity is not proven by source inspection.
- The original soft-particle covariance/coverage branch is unchanged. Negative material emission is decoded first, then multiplied by the per-cell mask; the original negative marker still controls radial/coverage alpha independently. No texture, interpolation across cells, bloom pass, light-budget increase or hard-cell footprint clamp was added.

Root's frozen log reports **nine lighting tests passing**, including actual shared WGSL parsing and Naga validation with empty optional capabilities, plus bounded light-height/ABI checks. The exact dev dependency is Naga 30.0.1, already present transitively in the lockfile; the lock diff adds its direct game_web dependency only. It is a dev-only parser/validator addition, not another shipping shading architecture. The saved pass establishes parser/IR validation and uniform tests, not actual WebGL2 translation/pipeline creation, visual normal response or frame costs.

### Matched Nova wire contract

`snapshot()` allows only the added optional key and invokes its validator by own-property presence, so explicit null/undefined is rejected instead of treated as omission. Journal/birth required key counts and allowed keys align with native strict deserialization. The guard checks u32 counters, at most 32 births, strictly increasing nonzero sequence up to highwater, nondecreasing birth tick up to capture tick, lowercase emitter identity, nonzero exact roster epoch, finite bounded position and 0..36 rays. This remains stateless transport validation; committed counter regression, age, cache/dedup and complete-body transaction remain native authority.

The four saved fixture files are actual native-produced legacy/active/expired/max-count headers. Root's test parses them and sends them through real `publication()` and `encode()` boundaries, retaining exact semantic payload data and the 65KiB envelope bound. Negative tests cover null/missing/unknown keys, ownership, nonfinite/wrong-sized vectors, chronology, count and integer bounds; zero rays/empty retained highwater are explicitly valid omissions. These are appropriate boundary tests and honestly say they do not execute native Game/Replica in Node.

The frozen Root log reports **twenty-two scoped shell tests passing**. That is Root execution, not an independent QA rerun. Native implementation had already been independently reviewed at 04ac672+094f7dd. Matching this wire guard removes the new consumer's shape mismatch; it does not make an old cached `galaxy-coop-1` client accept populated metadata or prove actual guest Nova rendering.

## Remaining gates

1. Before mapped assets, every external CPU/capture consumer of `PixelInstance.transform` must use the masked slot. Current legacy-only data cannot exercise that failure path. Core packing/oracle and shader need native reference-vector comparison, including near-singular fallback scope.
2. The first real PAN map must be ingested through native Surface, retain exact surviving RGBA/occupancy/rig metadata and render the expected localized moving-light/emission behavior. Both backend pipelines and saved actual views are required; Naga success alone cannot replace them.
3. Co-op body pages/signatures still carry RGBA without authored surface maps. Local asset rendering does not prove guest map retention. Nova's own actual RTC appearance/useful delivery timing, guest audio and mixed-bundle compatibility remain separate.

The reviewed draft is consistent with the efficient packed foundation. These remaining gates should stay visible on the board, with source, native, shader-validation and actual rendered evidence recorded separately.
