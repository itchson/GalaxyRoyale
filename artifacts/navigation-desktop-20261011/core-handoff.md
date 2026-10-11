# Connected Coil, held thrust, remote belts and sanctuary physics

Production source is frozen after 214/214 native `game_core` tests and all-target strict Clippy. `native-tests.log`, `clippy.log` and `frozen-hashes.json` pin this handoff. No distribution rebuild or browser action was performed by this agent.

## Public integration contract

- `InputFrame.thrust: Option<[f32; 2]>`: a transient, unshaken world cursor while desktop left thrust is held; omitted legacy inputs default to `None`. Finite inputs within ±1,000,000 are accepted. A real joystick takes priority. Otherwise held thrust turns towards the cursor and requests full forward speed even at the cursor; within four world units it preserves heading. Held right-fire aim overrides heading and thrust follows the actual ship nose. The latest held target becomes the persistent destination, which resumes normal braking after release. UI/blur cancellation must clear held input and call the existing `cancel_course()`.
- `regions::site_belt(seed: u32, site: &Site) -> Option<ZoneBelt>` with serialized `ZoneBelt { inner: f32, outer: f32, material_tier: u8, recommended_level: u32 }`. The site is regenerated and validated before use. Every eligible canonical site has a belt at `radius + 240 .. radius + 520`; the entire belt must fit ±1,000,000 and remain outside Haven's 1,400-unit tutorial corridor. The root-owned map uses this exact API through `pub mod universe_map`.
- No profile/schema field, transaction, ability, boss promotion, or reward-rate changes. New asteroid provenance and orbit data are transient ECS components regenerated with streamed chunks.

## Connected whole-body articulation

Ordinary Coil heads and all five boss generations retain their existing art, cell scales, module counts and health. Birth anchors are actual occupied cells on each authored central spine. Segment radius comes from occupied-cell geometry with the existing conservative corner padding. Each front anchor overlaps its parent's rear anchor; bounded angle follow articulates the whole collision-bearing module. The solver runs after head integration and arena/sanctuary confinement. Render extraction re-solves the interpolated joints rather than independently interpolating module positions into gaps. Alpha-one extraction matches the collision pose.

Destroying a module reconnects surviving modules; bind anchors do not restore erased cells. Current damaged masks are passed into existing rig extraction. Changing Coil champion head art now rebinds its rear joint once to that actual chassis. A sanctuary/arena boundary first searches 64 legal bends; impossible externally positioned geometry may detach to preserve protection. This safety fallback is explicit, rather than making a segment invulnerable or drawing collision-free substitute art.

The unchanged unculled authored occupancy/rig-role costs for G1–G5 are recorded in the manifest. Those are asset upper bounds before camera culling/destruction, not new GPU measurements. Joint overlap does not create extra connector quads. Contact damage remains the existing conservative object-circle path; projectile hits still use real occupied cells.

## Belts and physical shields

Haven-born rocks retain T1 material/basic-point drops and no XP. Remote ring provenance retains its canonical centre and birth tier after later displacement. Site-distance tiers use the existing 1,600-unit danger progression: T1 below 3,000, T2 from 3,000, T3 from 4,600, T4 from 6,200, T5 from 7,800. Recommended ship levels are 1/3/5/10/12. Remote rings retain outer-rock two ship XP and existing independent 25% material/basic-category drop rolls; they grant no signature. Outer-rock rewards remain unchanged.

Streamed ring chunks consider 12 seeded candidates; ordinary outer chunks keep three. The existing 112-rock global cap, loaded-chunk bounds, arena exclusion and full-body protected-site exclusion remain. Belt velocity is seeded tangential motion at 6–12 world units/s. Radial spring/damping and tangential correction are capped at 12 world units/s²; shot/contact momentum stays live instead of resetting position to an orbit.

All asteroids, including outer and diagnostic rocks without birth metadata, sweep against Haven's 240 radius and canonical site shields expanded by the actor's conservative body radius. Earliest quadratic surface contact reflects inward momentum with restitution 0.7 and 0.05-unit clearance. Each redirected corridor refreshes its canonical-site query. The query has at most 512 neighbourhood samples and the solver at most four contacts; an untested fifth corridor discards that tick's remaining travel. Normal finite gameplay impulses are much shorter. Post-collision separation gets a second penetration/velocity correction. Corrected previous positions avoid rendering a line through a shield. Shields produce no damage, loot, XP, event or light.

## Actual native evidence and limits

- Six articulation tests: 96 near-world birth/direction combinations, both practical world edges, 900 actual turning boss ticks with five interpolation alphas, destroyed modules/masks, 16 sanctuary-edge directions, and 360 actual Coil-family champion arena ticks. Far-world whole-object f32 translations allow up to 0.125-unit joint rounding; attachment cells are at least 0.85 units wide and occupied-cell overlap is checked.
- Three held-input tests exercise actual ticks, full thrust at arrival, smooth turn, real fired bullet direction, right-aim priority, release braking, cancellation, joystick priority and rejected input parity.
- Eight asteroid tests include 32 fast 20,000-unit/s Home/remote crossings; grazing/start-inside/post-impulse interpolation; a real occupied-cell bullet imparting inward momentum followed by actual game-tick shield reflection; separate 3,600-tick Home and remote orbital runs; deterministic canonical streamed rings; and a real seeded reflected-path encounter with a second site absent from the original query corridor.
- Remote reward tier proof uses real streamed origins and real occupied-cell projectile defeats after a declared controlled reposition to a different global tier. It proves immutable birth accounting, not a naturally travelled drift route. QA isolation compares the real profile after the projectile defeat. Existing tutorial-belt real-defeat tests still pass.
- All existing core migration, cargo, ability, boss queue, campaign, cadence, Mote, particle, cosmic and map tests remain green. No physical mobile, normal GPU/GL, human balance, soak, or performance claim is made here. More compact chains may change encounter contact pressure; real player tuning and browser framing remain root's next gates.

Reproduce native verification with `cargo test -p game_core --locked` and `cargo clippy -p game_core --all-targets --locked -- -D warnings`. The handoff's tests were debug native builds on the connected Windows workstation; these commands do not rebuild distribution assets.
