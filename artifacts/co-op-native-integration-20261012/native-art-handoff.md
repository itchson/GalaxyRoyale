# Native co-op presentation art

Frozen source: `35d2532f41c2d880b556925bc1c54544c598314a`, parent
`eb011de79f11d2afe6124ed1f7e92cf2e9fee3bc`, branch
`codex/co-op-presentation-art`. Only `co_op_art.rs`, focused
`co_op_art_tests.rs`, and the authorized `pub mod co_op_art;` registration
changed. Tracked checkout is clean. No push, server, browser or normal build.

Original deterministic native authoring follows the accepted sports-league
title's cut platinum/graphite planes, cyan instruments and small gold catches.
Invite has an outward athletic beacon and split orbital wings; Join has inward
docking arms and a recessed receiver. Broad central motifs remain distinct at
the proposed compact scale. Fine bevels, segmented instruments and fasteners
are actually authored cells, not an enlarged earlier bitmap or raster overlay.
No generated concept image or model claim applies to this slice.

## Root routing contract

Suggested semantic kinds `co-op-invite`, `co-op-join`, `co-op-roster` map to
`co_op_art::Kind::{Invite, Join, Roster(u8)}`. Root validates a finite integral
roster value in `0..=8`; invalid `Roster` values return None/zero. Count is the
controller's **actual admitted population including the local pilot**, never
pending channel readiness. These anonymous seats contain no invitation,
identity, readiness or host-role data. Accessibility labels and count text
remain shell-owned. The art does not create or resize a hitbox.

`art(kind) -> Option<&'static Art>` exposes a cached immutable `body` and
`Art::color(cell, original, Style)` if Root chooses an existing fitting route.
The complete helper is:

```rust
co_op_art::append(frame, &co_op_art::Input {
    kind,
    rect: measured_rect,
    clip: inherited_local_clip,
    parent: group_affine,
    style: co_op_art::Style {
        hovered, busy, disabled, time, reduced_motion,
    },
}, &mut remaining_pixels, &mut remaining_work);
```

Symbols have 64×64 source grids, proposed measured art rectangles 16×16
logical. Roster has a 128×24 source grid, proposed 64×12 logical rectangle.
Continuous uniform fit keeps the complete source field; no detail LOD or cell
size floor. Clip is local UI space before parent affine, with full cell corners
masked. Parent reflections/shear compose exactly. Nonfinite, singular, overflow
and invalid bounds reject before frame/quota mutation.

| Object | Occupied cells | Transforms |
| --- | ---: | ---: |
| Invite | 1,334 | 1 |
| Join | 1,590 | 1 |
| Roster (each count 0..8) | 1,672 | 1 |
| All three once | 4,596 | 3 |

All nine roster states have identical occupied indices; count changes only
known admitted pilot colors. Neutral material colors stay fixed during hover
or busy motion. A small explicit cyan signal mask has a restrained brightness
pulse; Reduced Motion, idle and disabled states are time-invariant. Disabled
art is neutral/muted. No moving silhouette or automatic connection inference.

Both pixel and work budgets admit the whole object before appending; every
occupied cell is charged, including clipped alpha-zero submissions. Existing
60k/120k caps are unchanged. Each object uses one unlit transform; normal
FrameBuilder batching still applies. Lazy cache creation and FrameBuilder
buffer growth can allocate, but append does not clone a body or allocate role
data per frame. The nine small roster caches are finite static presentation
states, not peer/world caches.

## Actual checks and evidence

`cargo test --locked -p game_web co_op_art:: -- --nocapture`:
**8 passed, 1 explicitly ignored exporter**, retained `final-native.log`.
Checks cover handpicked alloy/bevel/latch/negative-space landmarks, eight-seat
colors/constant indices, cached identity, bounded masks and Reduced Motion,
full cells at fractional widths 64/16/7.125/0.8, reflected/sheared parents,
partial whole-cell clip, invalid inputs and either-quota atomic rejection.
An early draft roster had 1,880 cells and failed its 1,800 target check; its
empty lower casing was shortened, yielding the final 1,672-cell rail.

`cargo clippy --locked -p game_web --all-targets -- -D warnings` passed,
retained `final-clippy.log`. `git diff --check` passed before commit.

The explicitly requested exporter was run once after the final refinement:

```powershell
$env:GALAXY_CO_OP_ART_OUTPUT = '<owned work/co-op-presentation-art folder>'
cargo test --locked -p game_web co_op_art::tests::export_co_op_native_art_packet -- --ignored --exact --nocapture
```

Actual result: **1 passed**, `final-export.log`; `native-packet.json` contains
15,880 cells / 10 transforms for the contact composition. The retained Python
plot decodes and counts the actual packet, and renders its native corners and
source RGBA; it does not simulate the display shader. The final PNG was opened
and inspected with `view_image`. It shows source-size symbols, proposed
fractional fits, and roster states 0/1/4/8. Source detail remains in the packet,
but a small display cannot sample every fine cell; this is not a claim that
all source details remain physically visible at 16 logical units.

- `native-software-contact.png`: **flat RGBA software**, not GPU/DOM evidence.
- `small-symbol-software.png`: downsampled software sample, not a device test.
- `render_contact.py`: reproducible plotting code; Pillow is an existing work
  dependency and is not added to the shipped game.
- `manifest.json`: raw evidence and Git/checkout source fingerprints.

Root still owns actual semantic routing, accessible labels, secret-safe code
fields, session lifecycle and final normal WebGPU/WebGL2 captures. No real
network admission, eight-browser session, phone, performance, listening or
human readability acceptance is claimed by this art handoff.
