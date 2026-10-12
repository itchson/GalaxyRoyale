# Guest callgraph clarification (evidence only)

This addendum preserves the original boxed-world HANDOFF.md, MANIFEST.json and verification.json unchanged, and does not modify c95c008 or any runtime/test source. The strict native checks and source pins retain their original scope.

The wording that guest bridge ownership is source-backed by an immutable &Game signature is incomplete when read as a type-only proof. hecs::World supports mutable component borrowing/query access through a shared World reference. Therefore &Game alone does not establish that every possible function reachable from it is free of ECS mutation.

For the actual frozen bridge, the no-Game-mutation conclusion comes from the inspected callgraph: dispatch_at advances only GuestView.replica; execute Begin reads game.co_op_session() and game.hud().dock; the remaining request arms read/write only GuestView/Replica. The Game HUD/session accessors read profile/state/shared component queries and do not acquire mutable ECS components, alter RNG/timers/world or drain events. The bridge accepts no Profile and does not call Game.tick. This callgraph is unchanged in the canonical Root5e source pins.

The existing four guest bridge tests directly assert Profile equality and ownership/error behavior, not a richer whole-ECS guest fingerprint. The new c95c008 rich state/continuation regression exercises native HOST boxed teardown, not guest browser stepping. Browser viewport/settings/controller actions remain explicitly outside the bridge-only conclusion, exactly as the original handoff states. A full live guest restoration proof still needs those observable boundaries defined separately.

This clarification responds to audio_score's actual bounded independent source review. That review reported no blocking test defect at the time of its message; its final report remains owned by audio_score/Root. This addendum is not an independent acceptance or a new runtime test result.
