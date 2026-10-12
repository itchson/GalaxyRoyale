/** Shared B-minor draft harmony; arrangements retain the same phrase phase. */
export const SCORE_CHORDS = [
  [47, 50, 54, 57], // Bm7
  [43, 47, 50, 52], // G6
  [50, 54, 57, 61], // Dmaj7
  [45, 49, 52, 55], // A7
  [47, 50, 54, 57],
  [43, 47, 50, 52],
  [40, 43, 47, 50], // Em7
  [42, 49, 54, 58], // F# dominant color; provisional cadence
] as const;
export const SCORE_BASS = [
  0,
  null,
  null,
  0,
  0,
  null,
  7,
  null,
  0,
  null,
  null,
  12,
  7,
  null,
  null,
  7,
] as const;
export const AMBIENT_BPM = 120;
export const COMBAT_BPM = 166;
export const SCORE_LOOKAHEAD_SECONDS = 0.18;
export const SCORE_MAX_STEPS_PER_TICK = 16;
export function chordAtStep(step: number) {
  return SCORE_CHORDS[Math.floor(step / 16) % SCORE_CHORDS.length]!;
}
