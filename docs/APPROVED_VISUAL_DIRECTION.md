# Approved visual direction

The user approved the generated PAN MEISHI image on 2026-10-11 as the desired
**game art style**. It is the visual benchmark for subsequent authored assets,
not a claim that every existing asset already matches it or that the raster
has been imported unchanged into the engine.

The exact reference is
[pan-collectible-reference.png](../artifacts/concepts/meishi-collectible-20261011/pan-collectible-reference.png),
SHA256 `a419b2896140a32d9683a10d55eba453aa7c78c237d1e6763d2a85f199314243`.
[Its provenance](../artifacts/concepts/meishi-collectible-20261011/provenance.json)
records the built-in image_gen tool and separate native adaptation. The local
generated output and retained reference were independently checked byte-equal.
No model/version is inferred from the image. Generated text, slogans, Japanese
writing, statistics and craft variants are illustrative; current game data,
identity and functional copy remain authoritative.

The user subsequently approved the **new in-game spacehub/arena detail and
animation** as a much closer match to the desired quality. Its
[actual PAN WebGPU capture](../artifacts/dock-components-20261011/dock-pan-webgpu-1280.png)
is a second practical benchmark. Preserve the overhead silhouette, layered
mechanical construction and localized lamp animation when improving the
remaining facility and world assets.

The prior native MEISHI card was explicitly rejected as too close to the old
UI despite passing its technical checks. Its earlier screenshots are
historical evidence, not visual approval. The active rework enlarges the
portrait, reduces the RAICHO strip to current hull and level, and replaces thin
outlines with substantial metallic fittings and octagonal sockets. Retained
points, canonical identity, eight earned-only patches and the Close-only
interaction remain. Visual acceptance is assessed on the rebuilt game.

## What the reference establishes

**Recognizable anatomy before decoration.** The DORA portrait has a distinct
face and long layered hair, precise gold irises, hard crystal facets and pale
mechanical armor over a black articulated neck. Fine cables, jaw/ear hardware,
small fasteners and reflected highlights explain construction rather than
adding uniform noise. Preserve the approved feminine DORA silhouette; DORO
uses its separately authored short spiky hair and flat masculine mechanical
torso, and TALO its crew cut and flat neutral torso. Changing colors alone
does not make those different types. Hair follows the main manufacturer and
horns the secondary manufacturer; skin, eyes, brows and neutral mechanical
parts must stay outside their recolor masks.

**Cut and layered metal.** Platinum/ivory edge planes sit above graphite
recesses. Chamfered corners, stepped rails, clipped panel outlines, small
screws and thin dark separation make the depth legible. Shadows and highlights
form deliberate connected clusters: a wide dark recess, a midtone plane and
a narrow bright edge, rather than a single bright outline or random grain.
Portrait armor can use its character perspective; RAICHO and Haven remain
strictly overhead with their existing world footprints.

**Restrained corporate color.** PAN green enamel and crystals provide the
reference's main accent, with warm gold/copper fittings, cyan instruments and
cool pale alloy subordinate to it. RENKO red and HUGO blue variants use
explicit corporate surfaces/signals, not a tint across every material. Keep
neutral alloy, graphite, skin and instruments stable. Separate accent and
emitter roles so hover, status or lamp animation changes local indicators
without flashing the whole object or changing its silhouette.

**Dark negative space.** The near-black deck makes the face, fine rails and
subtle enamel readable. Empty areas are part of the composition, not places
to fill with texture. Eight recessed sockets have cut octagonal wells,
layered metal rims and dark opaque interiors. Runtime MEISHI has exactly
eight slots; an empty socket reveals no unearned patch identity. Portrait,
RAICHO, identity and earned collection belong to one collectible card, whose
only action is Close. This art reference does not authorize navigation tabs
or settings controls inside the card.

**Useful microdetail.** Small rivets, segmented vents, cable junctions,
canopy rims, bevel corners and crystalline highlights reinforce larger
structures. Inspect both true source size and intended display size. A
source can retain all detailed cells while a small display samples fewer of
them; that is a real visibility limit, not proof that each source cell is
individually visible. Strong face features, central icon motifs and object
silhouettes must still read when those finer marks become subtle.

## Translating the style into native assets

Use the existing [asset pipeline](ASSET_PIPELINE.md): retain the original
reference, author or explicitly sample a declared grid, quantify alpha/palette
losses, validate every occupied layer and preserve exact indexed cells through
import. Purely enlarging an older bitmap does not create the required detail.
The current 192×256 portraits, 64×64 controls and 96×96 RAICHO surfaces are
useful class-specific grids, not a universal resolution rule.

Keep source cells as native quad objects with complete affine composition and
continuous fractional fit. A small view must not silently select an enlarged
coarse mask, impose a one-pixel minimum on every source cell or discard a
component. Validate transformed occupied-cell corners, actual clipping,
role-mask independence, disjoint animation layers and exact submitted costs.
Damage holes remain absent cells. Lamp/expression states retain their declared
geometry contract; a model did not generate motion merely because multiple
authored states exist.

Cached objects and explicit whole-element admission keep the presentation
bounded. Count all occupied cells and instances, including submitted cells
with zero alpha, and every relative transform. Preserve the existing UI and
world limits; when a composition does not fit, revise its placement or
authored scope and report the rejection rather than partly drawing a portrait
or lifting a cap without review.

## Quality gate and acceptance boundary

Review the actual source and its decoded native translation side by side:
anatomy and silhouette, metal depth, corporate masks, cutouts, relative parts,
animation states, palette losses and occupied budgets. Native packet tests
must establish what was emitted. Software sheets must be labeled as software;
normal WebGPU/WebGL2 captures must then establish actual display fidelity,
clipping and readability at the intended sizes. Record exact source/build
identities and concrete defects, such as tinted forehead skin or lost panel
structure, instead of describing every result as faithful.

The user's approval settles the **style target**. It does not establish exact
raster/quad parity, complete game-wide reauthoring, subjective moving-combat
readability, audio quality, frame time, physical-phone cost or a 30-minute
soak. Those remain separately evidenced acceptance gates.
