# Fuller title instruments — third native audition

The user asked for instrument sound and mix after the initial draft, then said the revised version still needed fuller instruments. This pass layers finite detuned lead tones, richer sustained chord pads and centered bass harmonics; it preserves the existing provisional authored notes and shared title/Haven/combat tempo transport. It is not an accepted transcription of the original song.

Seven actual browser OfflineAudioContext fixtures passed sample finiteness, over-unity and disposal checks. The 48-second title render has peak 0.157576, RMS 0.024680, stereo correlation 0.988069 and measured mono-downmix RMS loss 0.0261 dB. This measures channel energy; it does not establish perceived tonal balance or listening comfort. The latest title RMS is -1.012 dB relative to V2; fuller layering is not a loudness increase.

The two audition MP3s are development exports of this engine's new native float PCM, made with FFmpeg/libmp3lame at 192 kbps. They are not the supplied reference song and are not shipped in the game. Raw float WAV files remain in the pinned capture directory in work/. The report pins those WAV bytes and native source/helper modules.

The title-transition fixture follows title (0–6s), PAN Haven (6–11s), exploration (11–15s), chase (15–20s), release (20–26s), and Haven (26–32s). Its actual score states record continuous beat progression and 169.275→120→166→120 BPM ramps. Listening acceptance, exact melody/form, output-device comfort and physical-device performance remain open.

The later independent QA amendment corrects an absent-native-panner feature guard in the isolated adapter. Its original regression and exact helper source pins are retained under `independent-review/helper-fix/`. The V3 render used the capable pre-amendment helper pinned in its native report; no later render is implied. Root's final working-source verification passed 638 shell cases with three declared skips and typecheck. Runtime source and the F4 compiled package remained identical; this helper/documentation pass is not a new game build.
