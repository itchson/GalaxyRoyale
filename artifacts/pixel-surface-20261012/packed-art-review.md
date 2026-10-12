# Real Rust-packed PAN map and supplied ring-after review

Read-only Art review of Root **817f99ba95daea4a9cad299383425c75fe535ae8**, portable codec **253dde5456754b063e32ebc203534662f0bfddad**, and the unchanged semantic proposal from Art **211f4c4eb4276ed9bcd482043c2f1258477abc3b**. No source edit, manual surface re-encoding, native build/test, listener or browser action occurred in this review. Root owns live extraction and actual normal-shader renderer acceptance.

## Actual packed map

Root's real encoder log records `pack_raicho_surfaces` reading the actual semantic proposal and calling **`engine_core::Surface::new`** for every populated cell. It writes **33702 bytes**, containing **662 coarse /1205 fine** ordered sparse records. I read that encoder source, actual log and actual committed map; I did not replace them with hand-computed encoded words.

`audit_decode.py` only **decodes existing native-produced words**. Its binary32 operation sequence is matched to the pinned `Surface::normal` source and cross-checked against six frozen actual native fixture cases: decoded vectors match **exactly**, including the slot127 word variants. This supports the decoder used for the map audit, not another native execution claim. The native fixture's36 affine oracles were not newly executed by Art.

Both fields contain precisely the semantic proposal's occupied indices, in the same order, with no duplicates or extra outside indices. Every actual word is enabled and keeps all low7 transform bits zero. Masks are exact: coarse all0; fine exactly31 original signals255, all other populated fine cells0. Actual local normal-vector differences from authored f32 values are:

| Field | Maximum | Mean |
| --- | ---: | ---: |
| Coarse48 | 0.0100252075 | 0.0048837457 |
| Fine96 | 0.0147592073 | 0.0052059498 |

All vectors remain finite, unit length within binary32 tolerance and front-hemisphere, below the portable codec's0.017 vector-error bound. Combined runtime fields have **676 distinct decoded normals**; coarse spatial averaging adds different slopes beyond the full96 authoring field's392 normals. Atlas raw SHA256 matches `319aa1fd8cf1112ef33addd06739063c72b92ecca5ca460afcc05aada77666be`, and an independent checksum over those exact bytes matches native `source_atlas_fnv64` **0229814dc9e11390**. Packed asset SHA256 is **903be7ec67fd236c9e2ec788a95da852779e22f05ad97dd2a066f1f9e77ea01c**.

No source/asset correspondence blocker was found. The art treatment remains appropriately bounded: semantic curved glass and beveled roofs, sparse existing signals, no new alpha/silhouette/color art or self-generated pixel lights. Multiplier0 deliberately suppresses non-emissive hull self-illumination;255 preserves the owning signal material. It does not independently create energy. A normal-only map without suppression uses255. Current original actor material emission0 means the map alone cannot produce glow; state-gated signal material remains a real draw-adapter responsibility.

## Root shader source inspection, not GPU acceptance

Pinned817f99 WGSL reads octX/octetY with exact8-bit masks, preserves low7 transform-table indexing and folds the full octahedral hemisphere. Surface data receives no source-RGB transfer. It transforms normals through the whole composed basis using the agreed inverse-transpose/geometric-mean-height embedding, with a bounded degenerate-basis fallback. The new normal affects only the existing dynamic-light energy; the base painted color/ambient term and hard alpha/quad geometry stay unchanged. Emission is the direct linear object-emission×mask product. Unmapped defaults keep multiplier1 and legacy radial-light behavior. The eight-light loop and instance/transform layouts remain bounded.

I did not run Root's Naga checks or render this shader; Root's reported actual validation remains separately attributed. Native live-mask/rig extraction, actor-state glow, actual WebGPU/WebGL2 appearance/cost and online guest-map parity are **not established by this map/source review**. The authored CPU contact is useful art direction, not a screenshot of817f99's shader.

## Actual supplied ring-after WebGL2 views

I inspected untouched Root `after-a-webgl2.jpg`, `after-b-webgl2.jpg` and `after-c-webgl2.jpg`, plus actual console STATS text and `after-build.json`. These show the **303659ec252f4ab7bded3c137ecec5ca3ca33edb normal build**, sourceHash `2c4884f89e2542dda06ed873934220b1c2154b94e2c618fb2dbeb7773a970dad`, packageHash `0b5409f807ac36d2c47901d8dbeba8ab84eea104d4074236b8660088b5b0d3bd`, builtAt `2026-10-12T10:09:36.943Z`. The STATS records confirm each prescribed position, zoom1.60, world400×225 and UI640×360. These are **before the new normal material shader**, not its visual proof.

Scoped still art acceptance:

- A shows the intended steeper upward-to-right whole-group ring/body tilt, consistent with the native−23.203° expectation. B/C are near-horizontal, consistent with+0.564°/−0.492°; do not call the latter two strongly different orientations.
- Each entire planet/ring group is visible and readable left of the craft. Back-ring/body/front-ring occlusion and the curved material shadow remain coherent; I see no new detached rim, rectangular dark cut, silhouette clipping or foreground/HUD obstruction of the reviewed group.
- A/B have genuine visible baseline comparisons. C now visibly contains its selected target, but its earlier `before-c-webgl2.jpg` lacked a visible planet. I therefore accept **C after-presence/composition only**, not a paired C orientation improvement claim.

The warm shared storm material and cache reuse remain explicit limits. The same near-horizontal B/C storm motif persists; the separate albedo design is still unimplemented and is not the current priority. The supplied pictures contain unrelated moving foreground actors/player-state changes, so whole-frame pixel equality is not claimed. One instantaneous console FPS/p95 display is not performance validation. No after-WebGPU, reduced-quality comparison, motion/camera traversal, physical device, all-seed art approval or whole-game visual completion follows from these stills.

`manifest.json` freezes exact Root/Core source inputs, actual native output/log/oracle, semantic proposal, readonly decode audit and supplied before/after images/stats/build identities. `verify.py` checks the raw bytes. Authoring provenance is preserved separately; no new asset, material policy, generated art claim or albedo implementation was introduced in this review.
