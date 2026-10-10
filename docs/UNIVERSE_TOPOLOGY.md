# Optional circular arena topology

`engine_core::CircularTopology` is a pure, opt-in primitive for a future small
circular battle universe. It is not enabled in solo exploration and introduces
no arena rules, networking, ownership, or multiplayer runtime. The existing solo
world retains its practical ±1,000,000 coordinate bounds.

Future dock radius upgrades and owned duel arenas can share a dock/arena model,
while battle royale should instantiate its own smaller circle and match state.
For N equally spaced player origins on a spawn ring of radius S, use angles
`2πi/N`; adjacent origins are `2S sin(π/N)` apart. A permitted dock radius needs
to remain below half that spacing after reserving the intended gap, and each
zone must fit within the enclosing arena. Spawn geometry, radius upgrade caps,
ownership, elimination, and the last arena/player victory rule are future game
policies, separate from this boundary primitive.

Create a circle with `CircularTopology::new(center, radius)`, then pass the
previous and already integrated positions to `wrap_segment(previous, current)`.
The start must be inside the closed disk. Stopping exactly on its boundary does
not wrap. An outward crossing at `q` resumes at `2 * center - q`, consuming the
remaining displacement in the same world direction. Velocity, heading, and
object scale remain unchanged. The original outward radial velocity becomes
inward at the opposite boundary because its radial normal changes sign.

For a radius-10 circle, moving from `(9, 0)` to `(12, 0)` crosses at `(10, 0)`,
resumes at `(-10, 0)`, and finishes at `(-8, 0)`. The result's `rebase_delta` is
`(-20, 0)`. Large overshoots may cross multiple times; the full remaining travel
is preserved. Work is bounded to 64 crossings, after which the primitive returns
an explicit error rather than dropping travel. A caller may subdivide a long
step and retry without partially applying a failed result.

Apply one result's delta consistently to the previous and current transforms,
camera and previous camera, and any click-to-move destination in that same local
chart. `rebase_point`, `rebase_transform`, and `rebase_affine` provide these
translation-only operations. Do not apply the delta to velocity vectors or
headings. Rebased previous positions can temporarily sit outside the disk;
they are interpolation history in the new chart, not fresh simulation starts.
Rebasing camera history and the moving object's history together preserves their
relative interpolation; it does not by itself make fixed arena scenery seamless.

Collision sweeps must eventually be split at each crossing. A line from the
rebased previous position to the final endpoint does not describe the original
physical route through the arena. Neighbor queries, projectiles, pickups, static
scenery, edge ghost rendering, click target interpretation across the seam, and
crossing event ordering need explicit arena integration before this topology can
be enabled in gameplay. They are not supplied by this primitive.

This is an arcade teleport rule, not a mathematically globally smooth circular
surface. Exact tangential motion starting on the boundary cannot become inward
after the antipodal teleport, so that degenerate step is explicitly rejected.
An arena controller must avoid that exact state or apply its own documented
boundary policy. Circle intersections use f64 internally; state and rendering
remain f32. A final boundary rounding case is nudged conservatively inward by a
few f32 ULPs. Nonfinite inputs, outside starts, excessive crossings, and outputs
whose circle cannot be represented at their coordinate magnitude return errors.
Prefer a center near the origin and an arena radius large relative to f32 spacing.

Tests cover the closed boundary, immediate outward and inward movement,
multi-crossing overshoot, diagonal crossings with an offset center, preserved
velocity/heading/scale, matching camera/destination rebasing, and explicit
degenerate/error cases. Another 500 seeded oblique cases compare a full swept
step with 32 smaller integration steps. Run `cargo test -p engine_core topology`.
