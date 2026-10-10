# Biome enemy content

The seeded 4,200-unit region grid controls palettes, scenery and ordinary family spawn weights. The Iron Belt now also has one distinct ordinary enemy: the Riveter. The other five biomes retain their existing family mixes; a full biome roster and radiation hazards remain future content.

An eligible ordinary Bastion born in the Iron Belt has a one-third chance of becoming a Riveter. Family selection still uses the player's biome; variant selection uses the actual spawn location. Its identity and birth grid persist for that live entity when it crosses a border. This adds no enemy roots, changes no density limits and creates no save schema.

The original 40×40 hull contains 856 occupied cells and two destructible barrels. The runtime loads palette indices into `PixelBody`; the preview PNG is not a game texture. Atlas, source and provenance are in `assets/pixels/riveter-*`. The artwork is authored cell geometry, with no generated-image claim.

A charge begins when the hull and warning fit inside the current unshaken world viewport. It lasts 0.9 seconds, locking its aim for the final 0.25 seconds. Amber barrel cells and short world-space rays communicate the shot direction. Three normal hostile rounds fire at approximately 0, 0.12 and 0.24 seconds, followed by 4.14 seconds of recovery. Tick rounding can slightly extend the cycle. Reduced Motion freezes decorative cyan shimmer while retaining live charge cues.

Each round starts at an actual surviving barrel cell, selected nearest the front-row mean with a stable left tie. If one barrel is destroyed, the other supplies the shots. Removing both barrels stops firing. The body uses standard swept-cell projectile hits and material lighting; holes remain holes. No pristine hidden barrel is used after damage.

The Riveter retains ordinary Bastion tier-scaled hull/contact damage, pursuit range, shared projectile cap, loot chance and reward path. A defeat gives one Bastion family kill and five selected-ship XP, can advance an accepted Bastion bounty once, and can contribute to the existing 24-kill first-boss queue. Sanctuary disengagement and projectile crossing protection remain shared. Retreat grants no rewards.

`SCENE biome-riveter` in a `?qa=1` console sets up a real Iron Belt encounter using the existing whole-scene reward isolation. It is a testing scene, not a shortcut to XP, currency or progression. Normal play uses ordinary procedural spawns.

Ten native regressions exercise actual hostile rounds hitting the player, locked triple bursts, surviving muzzle cells, real friendly projectile defeat, once-only rewards, bounty/family accounting, shader extraction after damage, camera gating, reduced motion, cap saturation, retreat and sanctuary protection. Native fixtures use tier-one stats to isolate behavior. Natural distant-biome difficulty, human dodge/readability and physical-phone performance remain pending.
