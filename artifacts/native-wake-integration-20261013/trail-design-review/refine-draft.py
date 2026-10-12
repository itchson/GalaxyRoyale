from pathlib import Path
p=Path('work/coop-wake-presentation-review/REVIEW-DRAFT.md')
s=p.read_text(encoding='utf-8')
s=s.replace('## Two contract refinements before implementation','## Three contract refinements before implementation')
s=s.replace('Both findings were sent directly to the author and Root. An explicit versioned resolution should be accepted before coding; preserve the frozen proposal input bytes.', '''3. **Include the native anchor halo in this new position bound.** `co_op.rs:836` confines pilot position/previous position to +/-WORLD_LIMIT, but `co_op_wake.rs:32` places the sample at the pre-integration origin minus 14 forward. The actual legal source can therefore stamp x=1,000,014 from an inward-facing pilot at x=1,000,000. The proposed +/-1,000,000 page position validator would reject that native point. Define a cosmetic point bound WORLD_LIMIT+14 (with matching raw/f32 checks) or a reviewed explicit cosmetic omission; never clamp or alter the authoritative wake sample merely to satisfy serialization. This is source-derived compatibility reasoning, not an executed edge fixture or runtime reproducer. Future actual native edge-capture tests should exercise heading on each boundary while the existing pilot/projectile schema bounds stay unchanged.

All three findings were sent directly to the author and Root. An explicit versioned resolution should be accepted before coding; preserve the frozen proposal input bytes.''')
s=s.replace('the two explicit refinements','the three explicit refinements')
p.write_text(s,encoding='utf-8')
