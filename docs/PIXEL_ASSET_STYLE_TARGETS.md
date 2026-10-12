# Pixel asset targets for the next world pass

This is a planning review dated 2026-10-11 for the broader art backlog,
task-ef6ed92a. It changes no artwork, renderer, collision body or gameplay.
The three priorities below are proposed work, not completed quality acceptance.
The engine's separate cosmic refinement remains READY and unapplied at this
review boundary. Root owns integration and subsequent normal-browser review.

## The accepted benchmark and the inspected baseline

The benchmark is deliberate color clustering and material structure, rather
than a universal source resolution. The user-approved DORA reference and the
current independently authored DORA/DORO/TALO 192×256 cells show distinct faces,
hair volumes, planar mechanical armor, fine cables and localized catchlights.
The current 64×64 controls use readable central silhouettes, open apertures,
beveled metal, dark recesses and small explicit indicators. Fine detail supports
the silhouette instead of competing with it.

| Class | Current source and actual scope |
| --- | --- |
| KIKAI | [Detailed pack](../assets/ui/kikai/detail/atlas.json), [source/provenance](../assets/concepts/kikai/detail/generation.json), [authoring overview](../assets/ui/kikai/detail/authoring-overview.png) and [portrait contract](KIKAI_PROFILE_ART.md). DORA/DORO/TALO submit 34,594/28,524/24,262 cells in three layers, with 63/61/60 palette entries. The fixed guide is APR-1337, DORA-PAN-RENKO-1337. |
| Controls | [Ten 64-cell objects](../assets/ui/flight-icons/atlas.json), [compact software samples](../assets/ui/flight-icons/compact-readability.png) and [mask/budget contract](FLIGHT_ICON_ART.md). Shared palette has 21 entries; total occupancy is 14,709. These are newly authored geometry and clusters, not enlarged historical 32-cell icons. |
| Corporate starter RAICHO | [identity.rs](../crates/game_core/src/identity.rs), `STARTERS`/`starter_body`: original 48×48 PAN HIKARI, RENKO HIBANA and HUGO MAMORI silhouettes, with cockpit glass, rails, reactor windows, vents and fasteners. The palette has 12 authored roles. These already contain meaningful detail. |
| Haven | [dock_facilities.rs](../crates/game_core/src/dock_facilities.rs) uses 44×44 floor/roof parts at 0.55 world scale; [pixel_assets.rs](../crates/game_core/src/pixel_assets.rs) gives the dock a strictly overhead treatment on its existing 64×64 mask. [Native metrics](../artifacts/haven-topdown/metrics.json) report 7,428 cells/9 transforms: 1,454 dock cells and 5,974 facility cells. [World overview](../artifacts/haven-topdown/world-overview.png) is actual native extraction rendered with flat software colors. |
| Asteroids | [pixel_assets.rs](../crates/game_core/src/pixel_assets.rs): four 40×40 families with four deterministic surface fields each. Procedural basalt, ice and silicate use eight opaque shades per family; the fourth family retains the converted source palette. Tests enforce connected intact material, no enclosed transparent cavities, stable family radii and real damage holes. [Historical native sheet](../artifacts/presentation-refinement-20261011/native-art-contact-sheet.png) shows their surface repetition; its older facilities/icons are superseded. |
| Space | [cosmic_background.rs](../crates/game_core/src/cosmic_background.rs): sparse layered stars/gas and cached two-part spiral, torn nebula and nova motifs. [Native proof](../artifacts/cosmic-background-proof.md) records 978/694/473 full motif cells and 439/324/228 reduced cells for the declared fixtures. The [Isotope Scar](../assets/pixels/isotope-scar-provenance.json) and [shooting-star source](../assets/pixels/shooting-star/provenance.json) already provide more structured local material references. |

The source review inspected the portrait/control authoring images, the Haven
native overview, the historical asteroid native sheet, the authored
shooting-star preview and the historical galaxy WebGPU capture. Those images
have different scopes. Authoring previews are not native packets; native flat
software previews are not GPU screenshots. The older galaxy capture is useful
for dark-space contrast, not evidence of the current HUD or a newly installed
cosmic refinement. No new browser, human play or device test ran for this doc.

## 1. Make RAICHO and Haven surfaces belong to the same material family

The current hulls and Haven are readable, strictly top-down objects, but their
large simple roof/wing regions and short repeated vent patterns are visibly
less articulated than the accepted portraits and controls. Increasing every
plane's cell density would multiply cost without necessarily improving its
readability. The next pass should add authored fine clusters where the player
looks: canopy rims, manufacturing seams, recessed sockets, cooling assemblies,
bay latches and instrument lamps.

Start with the three corporate starter RAICHO and their Haven accents. Preserve
their distinct overhead silhouettes: PAN's slender prow/nacelles, RENKO's
lancer/weapon rails, HUGO's broad shield frame; preserve the four readable
fabrication, hangar, archive and arena shapes. Haven remains a horizontal roof
and deck assembly. No front facades, upright towers or isometric diamonds.
Main manufacturer owns hull/Haven accents; sub manufacturer owns crystal-horn
roles on KIKAI portraits, not an extra hull ability or an accidental skin tint.

Use the existing collision body as the first layer and author a separate finer
surface layer, rather than silently replacing gameplay geometry. An entire
opaque 2× finer roof would cost roughly four times as many cells. A few bounded
fine assemblies can provide the desired detail while keeping the coarse metal
planes efficient. Any future silhouette/collision change needs its own
gameplay review and damage-to-presentation mapping.

| Asset | Proposed grid and palette | Proposed per-instance limit | Explicit relative presentation |
| --- | --- | --- | --- |
| Starter RAICHO | Keep the 48×48 collision layer; author a 96×96 fine surface layer at half that layer's cell scale. At most 32 packed colors, including transparency, with metal/recess/glass/main-accent/exhaust/signal roles. | At most 1,800 added occupied cells and 4 transforms per whole hull. Current starter counts 662/688/1,040 make the largest proposed whole hull at most 2,840 cells. | Canopy instruments and narrow reactor/exhaust indicators; no moving damage hull. Thrust effects follow actual thrust state. Reduced Motion freezes cosmetic sweep while retaining actual thrust/damage information. |
| Four facilities | Keep 44×44 floor/roof masks and their centers; author selected 88×88 fine assemblies at 0.275 world scale. At most 32 colors per compatible pack, shared metal ramp with distinct coral/cyan/green/gold functional accents. | At most 600 added cells and one added transform per facility. | Local clamps, bay guide lamps, archive scan lens or arena score markers. Animate lamps/presentation parts without moving interaction centers or collision footprints. |
| Central dock | Keep the existing 64×64 occupied mask/open aperture; author a 128×128 fine berth/signal layer at half the base cell scale. Same Haven metal ramp. | At most 1,200 added cells and 2 added transforms. Combined Haven target is at most 11,028 cells/15 transforms, before unrelated world entities. | Short sequenced berth lights and slow local instrument phases. No added elevation perspective or filled docking aperture. |

These are authoring ceilings, not reservations of free frame capacity. Reused
instances and every layer still cost cells. If the normal scene cannot admit
the full object, revise the authored detail or scene policy explicitly; do not
partially draw the fine layer, restore destroyed cells or increase caps by
default. Keep facility centers and HIT_RADIUS=24, actor world scales, orbit
rules and reward semantics unchanged in this art pass.

## 2. Give asteroid geology distinct structure without born damage holes

The native sheet shows meaningful basalt/ice/silicate differences, but repeated
four-crater layouts, similar diagonal bands and field variants dominated by
palette shifts make nearby rocks look reused. This is a high-impact gap because
rocks are a frequent close-range interaction. More random bright speckles or
larger grids would add noise and dense-instance cost.

Keep the 40×40 damage/collision grid and its current family silhouettes in the
first pass. Develop a small explicit geology vocabulary: broad basalt fracture
plates with compact vesicles; directional ice cleavage and embedded dark
minerals; layered silicate folds and off-center impact basins. Choose motif
positions/orientation from immutable birth hashes. Surface-field variation
should move the geology features, not merely lighten an identical texture.

Proposed limits are 12–20 colors per family, at most 32 including transparency,
and the existing 500–1,600 occupied-cell envelope/one transform per rock.
Use contiguous light/mid/dark planes, opaque crater floors and narrow inset
seams. Crystal/mineral accents occupy an explicit minority of cells; avoid
making rocks resemble pickups or charging enemy emitters. Intact rocks remain
one four-neighbor connected body without enclosed transparent cavities.
Only actual damage removes interior cells. Preserve family radii, immutable
material/reward births, shield reflection and belt confinement.

Color animation is optional, restrained and selected by material role; it must
not imply an unimplemented hazard or resource. Reduced Motion freezes it.
Impact dust remains the existing bounded actual-impact particle path, not a
new per-rock per-tick allocation. A later higher-resolution render layer would
need a demonstrated damage mapping and crowd budget before admission.

## 3. Enrich cosmic forms while retaining darkness and combat contrast

The current galaxies, nebulae and novas have useful negative space and seeded
parallax, but small two/three-color analytic patterns provide less internal
structure than the Scar or the authored shooting-star nucleus. The next pass
should add fine clusters and a few coherent volumes, not raise global sky
brightness or fill every empty patch. Preserve the existing gas transitions
and actual birth-space placement rather than rerolling forms from the camera's
current biome.

Use an authored sparse spiral core/arms, torn gas lanes with visible voids and
an asymmetric broken nova shell. A proposed finer grid may be 192×192 for the
galaxy arms, 160×96 for the nebula and 128×128 for the shell, each at half the
existing part's local cell scale. This is a ceiling on source extent, not a
requirement to fill the area or enlarge the world footprint. Start with the
engine's separate READY proposal and compare real occupied counts before
choosing grids.

Target 12–24 restrained color/alpha entries per authored motif pack, capped at
32 including transparency, and at most 2,200 occupied cells/4 transforms for
one full motif. A separately authored reduced variant should stay at most
1,100 cells and preserve the large voids and core identity. Continuous streamed
gas tint is an existing analytic path, not falsely described as a fixed atlas
palette. Keep the custom background cap **8,000 full/4,000 reduced**, star
reserve **1,800/900**, and gas limit **800/400**. Stars/stellar systems and
motifs share the custom cosmic pool; the star reserve is not extra capacity.
The gas helper is authored in cosmic_background.rs but its 800/400 limit is
charged inside the separate **regional 6,000/3,000** budget. It is not charged
inside the custom cosmic 8,000/4,000 pool. Preserve the Scar's existing atomic
regional group; do not silently take its budget or change motif probability.

Use dark diffuse material, localized low-alpha ridges and a small brightest
core region. Normal cosmic motifs remain unlit unless a separately reviewed
material needs emission. The shooting star already uses bounded emissions
and 64/32 analytic tail cells; preserve its current cycle, projection and
seeded origin. Relative rotations/breathing belong to their own parent groups,
not world lights or gameplay particles. Reduced Motion freezes cosmetic time;
cosmetic density explicitly selects the reduced asset. Neither changes combat,
save fields, hazards or game RNG.

## How consistency is judged

- **Silhouette before detail:** identify each corporate hull, facility function,
  rock material and cosmic family in a monochrome/low-color view at its actual
  gameplay size. UI motifs must remain distinct in the existing 14/28/42-pixel
  review sizes. These are review procedures, not claimed human passes.
- **Shared material language:** ivory highlights, steel midplanes, graphite
  recesses, controlled glass and small lamps describe manufactured surfaces;
  rocks use coherent local facets rather than the metal UI ramp. Keep brightest
  tactical shots/telegraphs above diffuse scenery. One consistent upper-left
  local plane treatment may support readability, without turning overhead art
  into a facade or pretending to follow a world light.
- **Explicit semantic masks:** independently test actual source landmarks for
  metal, glass, corporation accents and emitters. Recoloring must preserve skin,
  gold irises and immutable secondary material. The repaired KIKAI forehead
  leak demonstrates why a broad hue/y threshold is insufficient. Keep masks
  occupied, bounded, disjoint where required, and emitter subsets explicit.
- **Detail without sampling claims:** retain source cells and compose fractional
  affine transforms into the intended layer. No blanket minimum quad size,
  automatic low-resolution bitmap or integer-fit floor. Physical raster size
  and zoom can still leave fine cells unsampled; inspect settled/moving normal
  WebGPU and WebGL2 at zoom 0.6/1.0/1.6 before calling detail readable.
- **Animation with stable truth:** preserve damage occupancy, collision and
  interaction geometry. Test actual host state and Reduced Motion, not metadata
  alone. Decorative motion must not mask a dangerous shot, an open boss core,
  ship identity or authoritative hull/shield information.

## Required handoff and acceptance boundary

Use [ASSET_PIPELINE.md](ASSET_PIPELINE.md): explicit source grids/crops, exact
binary-alpha imports by default, all components and transparent apertures
retained, stable palette indices and compact runtime JSON. If a generated
raster reference is helpful, retain the actual prompt/tool/source hash and
review it before authoring cells. Declare measured resize/alpha/quantization
loss; an arbitrary generated sheet is not a perfect cell atlas. PNG encoding
may differ by compression library, so canonical encoded provenance requires
full independent decoded-RGBA equality before canonical bytes are retained.
A changed color, alpha or dimension must fail. Do not discard disconnected
lights or refill artist-authored holes silently.

Each pass must hand off original sources, deterministic regeneration/tests,
role/topology/count evidence, finite all-corner transformed bounds and a native
extracted contact sheet. Compose full affine matrices for nested nonuniform,
reflected or sheared parts. Preflight every occupied child/instance atomically;
clipped alpha-zero submissions still charge work. The 128 transform batch
boundary is a valid split, not permission to omit later parts. UI remains
bounded by 60,000 submitted cells/120,000 work; for example, all ten controls
plus the current dense interface and DORA can exceed 60,000. Correct whole-
portrait rejection is not a reason to widen caps or trim accepted art silently.

Root's normal dual-backend captures then assess actual material blending,
foreground contrast, animation and menu/world interactions on the final source
and cache identity. Moving/crowded human play, subjective art approval,
physical Pixel6a/iPhone12 input/performance and sustained soak remain separate
pending gates. This document performs no new generation or acceptance test and
does not claim that every existing asset meets the portrait benchmark.

## Inspection pins

These SHA-256 values identify the read-only source/asset boundary used for this
plan, not a new deployed build. Future runtime work should pin its own source
and preserve historical negative evidence.

| Input | SHA-256 |
| --- | --- |
| `assets/ui/kikai/detail/atlas.json` | `31fa6a1b04d83f3d5fad377562e73c6ad35530b028626a1136c5b56afd9ea661` |
| `assets/ui/flight-icons/atlas.json` | `e86980bbb99f3c1423900283f78e7bba2aac4b90d02b4a0855291c2e5ff8ee73` |
| `crates/game_core/src/identity.rs` | `6e3eb3c7e57afe4215dbd7c6b3ae22221e6fe5eb43d312833c6b5c97f807c395` |
| `crates/game_core/src/dock_facilities.rs` | `f783a1602f6e01574f60018c11111ed2f34a2286edfdc7721839f6d11b63d7e5` |
| `crates/game_core/src/pixel_assets.rs` | `d7e946fd729ee61d3bca9980f087ed14cbc83cb77d2a9aa7d3c4c5a952920eba` |
| `crates/game_core/src/cosmic_background.rs` | `52d290a2f9d69a3e905638b73d87220f8fba8afa87ac965a7d8f0dd57897fad2` |
| `crates/game_core/src/shooting_star.rs` | `ab0b7f420528d5e3bf01cdc2d3bd90c999059ff56be40a370e4688f8fbe47a87` |
