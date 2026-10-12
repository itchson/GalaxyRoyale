# Append-only timing and bend clarification

Earlier frozen prototypes preserve the unchanged official Basic Pitch JS time
converter and explicitly make no accepted source-event timing claim. The last
output-row mapping by itself does **not** justify shifting or stretching song
timestamps:688 frames map7.94636s and3670 map42.39177s, but padded/windowed
inference output extent is not the physical source note clock.

Engine reviewer `/root/engine_fx` actually ran a separate43-second, sample-exact
fixture with14 separated440/880Hz tones around early/late hop boundaries and
the tail, using the same pinned official JS CPU model/API. Both decoder
variants recovered14 pitches with no extras. Official onset errors were
−27.913..+1.556ms, median−9.335ms, fitted slope−0.131ms/source-second. Naive
frame*256/22050 timing drifts+5.057ms/source-second. Keep the official window
correction; no global−0.28s offset or song time stretch was inferred/applied.
The late note's end error was−44.784ms. These are isolated-tone fixture results,
not accepted real mixed-song note boundaries.

Exact observed peer analysis is copied as `inputs/peer-calibration-analysis.json`,
SHA2568056e38474ec348decfd323974131d2dc0cde13f8f74de01207794ea44428308.
The author read/verified that file, without rerunning or relabelling the peer's
test as an author test. Root separately confirmed the same interpretation.

The known constant tones expose a second important limit: every note's raw
contour displacement is predominantly+1 bin, despite zero true pitch bend.
The official coordinate calibration of3 bins/semitone is correct, but blindly
using those estimates gives these fixtures roughly+33.3 cents of false tuning.
No global−1-bin correction was invented/applied. Fixed-pitch and raw-bend
prototype WAVs remain distinct unaccepted controls. Mixed-source pitch curves
need independent spectral/ear/stem validation before a production patch.

Full WAVs have the exact42.68-second physical source extent. That fact alone
does not accept their estimated note cadence, pickup, rests, tail or seamless
loop. The original8s and420-note frozen reports/outputs are unchanged.
