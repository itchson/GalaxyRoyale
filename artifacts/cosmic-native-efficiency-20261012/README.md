# Preserved native CPU audit

This folder preserves the specialist's actual frozen raw evidence. Root ran
the read-only verifier in the original isolated checkout: 86 source pins,
21 evidence pins, nine cold processes, 18 warm cases and 4,608 frames passed.
Root did not rerun the native measurements. HANDOFF.md defines measured scope,
configuration, source byte domains, distributions and limitations.

Test-only instrumentation commit d7327c729f483176e26d342b055a665e9471b11c is
preserved on origin/codex/cosmic-native-efficiency. It is not merged into the
normal game. The original verifier uses the source checkout relative to its
original work/ folder and exact recorded raw source bytes; copying it here
does not make Root's different checkout a measurement source. Reproduction
instructions in the handoff apply to that isolated instrumentation branch.

No GPU, phone, FPS, resident-memory or whole-game timing claim follows from
these native cache/extraction measurements. No shipping refactor was justified
by this one selected deterministic region.
