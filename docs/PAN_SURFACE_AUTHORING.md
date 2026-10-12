# PAN HIKARI semantic surface authoring

The first normal/luminance study reuses the reviewed PAN HIKARI art and its existing material-role/signal masks. `scripts/asset-raicho-material-proof.py` generates an **authoring proposal**, not a new collision body, renderer feature or shipped texture. Root owns native `Surface` integration and actual WebGPU/WebGL2 acceptance.

Generate into an explicit owned proof directory, then check those actual files:

```powershell
python scripts/asset-raicho-material-proof.py --output C:/absolute/owned/proof
python scripts/asset-raicho-material-tests.py --proof C:/absolute/owned/proof
```

Existing Python/Pillow authoring dependencies are sufficient. The script writes only the requested directory. It never modifies the source atlas, launches a service, calls an image model or edits the core ABI.

## Exact source and material conventions

The existing32-color96×96 source carries seven disjoint roles: metal, recess, glass, main accent, reactor, exhaust and fastener. PAN's full authoring surface has2648 occupied cells; its actual retained48 collision hull has662 and sparse96 fine overlay has1205. Source colors/alpha and these masks are exact. The source atlas raw SHA256 is `319aa1fd8cf1112ef33addd06739063c72b92ecca5ca460afcc05aada77666be`.

The authored height field comes from semantic roles: shallow recesses, beveled plated roofs/accents, a rounded canopy, flush reactor/exhaust windows and raised fasteners. It does **not** infer height from painted light/shadow or color luminance. Height is authored in coarse actor-local units. Fine samples are spaced0.5 such units; the corresponding coarse height averages each exact2×2 block and samples at spacing1. Normals follow local **+X right, +Y down, +Z toward viewer**. This first field uses the front hemisphere; it must feed the core's full-hemisphere codec without inventing a second packing implementation.

`pan-material-proposal.json` has exact dense per-index arrays for `coarse48` (2304 entries) and `fine96` (9216 entries). Every populated entry is `[normal_x, normal_y, normal_z, emission_mask_u8]`; `null` means absent optional data/legacy `Surface::default()`, not a new transparency rule. The separate `full96_authoring_only` is useful for source-art inspection; **do not append it with the retained body/fine overlay**, which would duplicate occupied pixels.

Emission bytes are **linear multipliers on the existing object `PixelMaterial.emission`**, not standalone energy, RGB alpha or one point light per pixel. This deliberately authored role mask is0 on non-emissive hull cells and255 on the31 existing explicit fine signal cells. Coarse mask0 prevents broad duplicate glow. A generic normal-only map without an authored suppression mask should use255 to preserve its object's existing emission. Here the semantic hull mask intentionally suppresses it. Native `ACTOR_MATERIAL.emission` currently remains0; the sidecar alone cannot turn on glow. Root must choose a restrained signal-role object emission and gate exhaust by real throttle/state. No palette or alpha change is required.

`pan-height.png`, `pan-normal.png` and `pan-emissive.png` are exact96-grid **data visualizations**, with source occupancy as hard alpha. Normal RGB displays encoded unit-vector components; it is not the engine's oct8 packed payload. Masks/normals are data and must not receive sRGB correction. Source RGB can still follow the renderer's existing color transfer. JSON float normals are the authoring authority; PNG visualization quantization is not claimed identical to native oct8.

## Native integration boundary

The proposal described below is now registered for the PAN starter. The real Rust converter produces the source-bound packed map; the native adapter caches 14,036 bytes of coarse/fine word payload and reconstructs the same rendering-only map for matching co-op starters. Native tests use actual projectile damage and body import. Root inspected healthy and turned hulls in both desktop backends. These stills establish clean appearance, not a controlled normal-versus-flat directional comparison, recent-hit guest parity, RTC visuals or performance. See [PIXEL_SURFACES.md](PIXEL_SURFACES.md) and [the actual acceptance record](../artifacts/pixel-surface-20261012/STATUS.md).

Use the portable core's validated `Surface::new(normal, emission_mask)` and optional `PixelBody::set_surfaces` API. Core owns the enabled bit, oct8 quantization, transform slot packing and hemisphere rules. The compact format reuses the existing16-byte instance's transform word (low7 bits remain slots0…127), so the authoring tool does not manually encode that payload. `PixelBody` occupancy remains the only admission/collision authority.

Fine metadata maps exact source index `y*96+x` to live coarse `[x//2,y//2]`, as `raicho_fine::append` does. Destroyed coarse cells suppress both their base quad and every owned fine normal/emission quad; changing a mask cannot fill a physical hole. Metadata follows the full composed affine, including scale, shear/reflection and nested rig animation. Root/Engine's inverse-transpose convention and native oracle own that calculation. Co-op transfer pages/body signatures still contain RGBA: matching PAN starters reconstruct the same registered native map. Native reduced-motion host/guest tests cover that path; real RTC visuals and recent-hit timing parity remain separate gates.

An optional dense4-byte core field for the48+96 pair has a fixed payload of **46080 bytes** before headers/parsing caches. This is allocation arithmetic, **not measured frame time or memory**. The proposal adds no geometry/transforms/physics bodies or automatic light sources, and must retain the existing1867-cell/two-transform PAN rendering quota. A production adapter should discard full authoring/review-only arrays and parse/cache the relevant fields once.

## Proof scope

The authoring checks cover actual source occupancy/alpha, finite varied unit normals, independent height after extreme palette changes, rejected malformed role/mask data, sparse signals and the existing coarse ownership convention. A deliberate authoring fixture removes coarse `[23,16]` and its four fine pixels. It is **not an observed projectile hit or native frame**.

`authoring-contact.png` is a labeled CPU study of the full96 source: flat-normal colored light, authored-normal colored light, an opposite light and a subtle proposed emission gain0.2. Lighting uses one declared point-light/diffuse reference plus existing style of color transfer; it does not execute WGSL or claim exact future shader parity. The source silhouette stays crisp, no blur/outside bloom is added, and only selected signals brighten. Display review uses explicitly3× nearest source samples, not measured runtime zoom/DPR.

Native packing/field ownership/damage checks and both-backend healthy/turned appearance now have actual evidence. Controlled directional depth, browser hole appearance and any claimed state-gated glow still require stronger visual acceptance. The first PAN treatment does not establish asteroid normals, every asset, all seeds, online guest parity, device performance or whole-game completion.
