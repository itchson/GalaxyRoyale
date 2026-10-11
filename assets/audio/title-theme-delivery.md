# Galaxy Royale title theme delivery

The original theme recording is pending the user's composition. No substitute
recording is shipped and no missing MP3 is requested by the game.

For integration, place the edited title loop in `apps/game/public/audio/`, set
its local URL and decoded loop-in/loop-out seconds in `apps/game/src/title-theme.ts`,
then rebuild normally so the game-only PWA asset manifest includes the file.
Use an original/licensed MP3, OGG or WAV; keep the playable loop at 120 seconds
or less, mono/stereo, below 16 MiB encoded and 64 MiB decoded. Keep the full master
separately. Actual decoded memory and seam audibility must be measured on
delivery; the loader's budget check is not a peak decoder-memory measurement.

The existing AudioContext and Master/Music controls own playback. A title
gesture unlocks it; one loop source fades over 1.2 seconds against the procedural
world score. Hidden tabs stop it; returning to title reuses the decoded loop.
The score low-pass/echo does not process the mastered title track. Verify the
chosen loop points by listening, including MP3 padding, and edit a seamless
loop before acceptance. The code does not claim to make an arbitrary song's
ends musically seamless.
