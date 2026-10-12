# Galaxy Royale title composition reference

The user supplied their original `Galaxy Royale (3).mp3` on2026-10-13 and explicitly requested recreation in the dynamic synthesizer, rather than runtime recording playback. The source remains an offline reference and is not shipped or decoded by the game.

The current native note-score draft uses the existing AudioContext, music/master controls, bounded synth voices and shared tempo/harmonic clock. See `docs/TITLE_SCORE.md`, `apps/game/src/title-theme.ts`, `music-clock.ts` and `music-score.ts` for implementation and provenance. Title/Continue, corporate/area arrangements and120/166 BPM chase/release ramps use the same transport.

The authored draft is not an exact transcription or final listening acceptance. Actual native OfflineAudioContext exports are development audition files only. The user requested instrument sound and mix refinement after the first audition. Keep source-reference analysis, rendered source pins, mix revisions and listening feedback separate from shipped assets and verified scheduler behavior.
