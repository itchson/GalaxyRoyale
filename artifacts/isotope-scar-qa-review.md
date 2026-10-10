# Independent Isotope Scar art and shared-budget QA

**Accepted for bounded integration review; no concrete blocking defect found.** Frozen manifest is `ffe62aa60b1b7b698dd73b117f7ecec127ebdf55a21f984275dac901b0da6a34`. This reviewer changed only `work/radiation-background-qa/`, inspected the pinned original atlas/authoring/provenance/runtime and current region/engine contract, viewed the retained offline preview, and ran the focused native checks below. The published5739067 Mote release, production/API, candidate and previews remain unchanged. No browser/GPU/device, human play, natural seeded visibility or publication acceptance is claimed.

## Exact art and cache

All source, evidence, compiled dependency and read-only-context hashes match the frozen manifest. Authoritative atlas SHA256 is `5c3d293f47d62d4639207aea9d71c577f6c913cdeb5f9ecad76117febfec73ce`. Source is explicit deterministic crater/fault/polygon/shading geometry; no model, imported bitmap or network asset appears. The runtime consumes JSON indices and caches five PixelBodies in a LazyLock, expanding them once. It does not load the PNG preview, clone bodies per frame, introduce an entity/collider/RNG hook or read a Game/save/profile.

Atlas version1,64 palette entries, transparent index0, opaque nonzero entries, index bounds and exact dimensions/row lengths validate.20 unique opaque colors are defined and19 are used; padded palette slots repeat the existing dark color. Independently recomputed counts match provenance:

| Body | Dimensions | Occupied cells |
| --- | --- | --- |
| Surface |96×96 |4,265 |
| Reduced surface |48×48 |1,084 |
| Back ions |112×48 |262 |
| Front ions |96×40 |188 |
| Fragment |24×24 |139 |

The 2×2 reduced surface favors seam/rim colors and uses doubled local scale. Reduced density drops the back ions and retains the selected smaller surface/front/fragment. The group uses4,854quads/four transforms full and1,411/three reduced. Nonzero cells always consume a quad; transparency is never used to claim reduced geometry.

## Independently executed actual-engine checks

Four new public-API tests compile against the pinned actual engine and unedited candidate module. They passed **4/4**, six existing candidate tests filtered, in0.01seconds:

- With declared existing layer packets, maximum gas plus346 full/1,189 reduced earlier quads leaves exactly the required group budget. Extraction appends the whole selected group and leaves zero. One additional earlier quad rejects the whole group with zero emitted transforms/quads, unchanged remaining budget and byte-equivalent preexisting frame packets.
- A frame already holding127 transforms splits at the actual portable128 boundary; all4,854 appended cells have valid batch-local transform indices, four group transforms and the correct shared-budget decrement.
- Invalid scale/rotation/time/camera/viewport/origin and offscreen culling preserve previously emitted contents and caller budget exactly.
- Reduced Motion produces equal complete position/color/transform packets at times0 and100,000 in both quality modes. No simulation clock or saved setting is mutated by the helper.

The actual frozen proof executable was run with an **owned output path**. Its twelve full/reduced × time0/18 × zoom0.6/1/1.6 FrameBuilder packets are byte-identical to the accepted frozen proof: SHA256 `1a93d335adb147fbf4f277a0bdac27adc6c18a05ee511c0162f0ca1f673294ec`. All occupied counts, four/three transforms and remaining346/1,189 agree. The independently compiled test executable SHA256 is `423eeec083d73c369898034ba93b66b714ce182a7255801a0255bfe12bf01039`.

```powershell
./work/radiation-background-qa/run.ps1
```

The first owned compile used an engine constant that is not publicly exported; this test-only import was replaced with the inspected portable128 contract before the successful run. It was not a candidate defect. No candidate build script or authoring/freeze script was rerun. `execution.log`, actual packets and `evidence.json` preserve the final focused results. Gameplay's six authored tests/Clippy and Engine's separate CameraView/transform review remain distinct evidence.

## Source and integration assessment

The helper validates finite inputs, positive bounded viewport and scale range before appending, computes whole-group occupancy before any output, and decrements the caller's existing budget once. Bounded parent/child inputs retain finite transforms; the source's±999,000 native corner fixture and Engine's independent far/reflected projection review do not replace browser precision acceptance. Material response/emission remain zero. Color shimmer and drift are purely cosmetic; negative time freezes atzero and Reduced Motion explicitly selectszero.

Current region extraction allocates its existing6,000/3,000 budget, emits at most800/400 analytic gas pixels, then visits layer0 fields before layer1 planets. The candidate's suggested replacement targets the existing layer1 factor0.22 branch, preserves immutable seed/hash/selection and supplies the unencoded origin. Its helper encodes camera×0.78 once; passing already encoded `position` would double parallax. Caller must pass remaining budget by mutable reference without a second decrement. The generic old planet/rings must not also be drawn for this replacement.

The boundary probes use declared preexisting packets/maximum gas reservations; they do not claim an integrated natural region render. Earlier far nebula/supernova content may consume more than346 full/1,189 reduced quads, omitting the entire Scar. That is documented existing ordering, not a guaranteed showcase. Root must verify actual seeded occurrence/priority and either accept omission or explicitly design a reservation policy without silently increasing the region/global budget. The helper introduces no separate background budget or save field.

## Tiny preview and pending acceptance

The retained software preview shows a distinct broken purple planet, coherent branching lime fault masses and detached fragment. At scale0.68/zoom0.6 the roughly31-pixel planet remains identifiable from its silhouette and fault mass; individual crater rims, sparse ions and fine mineral detail become subpixel. Reduced art brightens/broadens coherent seams while keeping the same broad footprint. This is a read of offline sRGB polygon-blended engine packets on a dark flat background, not observed WebGPU/WebGL2 contrast amid nebulae, enemies or UI.

Natural seeded visibility/admission, source integration, normal dual-WASM/cache/source proof, renderer color/aliasing at changing zoom, threat contrast and visual hierarchy, frame-time/first-use parsing cost, human readability and physical-phone visual/performance acceptance remain Pending. There is no environmental radiation/heat/DOT, physics, reward or persistence change in this candidate. No finished visual-quality or device-performance claim is inferred from these native extraction checks.
