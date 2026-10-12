/** Authored synthesis draft from measured pitch classes, not a transcription.
 * The reference MP3 is never fetched, decoded or shipped by this module.
 */
export type TitleNote = {
  step: number;
  note: number;
  steps: number;
  level: number;
  voice: "chip" | "bubble";
};
export type TitleTheme = {
  enabled: boolean;
  bpm: number;
  loopSteps: number;
  notes: readonly TitleNote[];
};
// F#/B opening, F#–A–D turn, E/G/B replies. Notes, octaves, gates and the
// 32-bar form are provisional authored choices; direct listening remains open.
const PHRASES: readonly (readonly [number, number, number][])[] = [
  [
    [0, 78, 2],
    [3, 71, 2],
    [6, 83, 3],
    [12, 76, 1],
    [14, 79, 1],
    [16, 78, 2],
    [19, 81, 2],
    [22, 74, 3],
    [28, 71, 2],
    [32, 83, 2],
    [35, 81, 1],
    [38, 78, 2],
    [42, 76, 2],
    [46, 74, 1],
    [48, 76, 2],
    [52, 79, 2],
    [55, 83, 2],
    [60, 78, 3],
  ],
  [
    [0, 71, 3],
    [4, 74, 2],
    [8, 78, 3],
    [14, 81, 1],
    [16, 83, 2],
    [20, 81, 2],
    [23, 78, 3],
    [28, 74, 2],
    [32, 79, 3],
    [36, 76, 2],
    [40, 71, 3],
    [46, 74, 1],
    [48, 78, 2],
    [51, 74, 2],
    [56, 71, 4],
    [62, 78, 1],
  ],
  [
    [0, 78, 2],
    [3, 71, 2],
    [6, 83, 3],
    [12, 76, 1],
    [14, 79, 1],
    [16, 78, 2],
    [18, 81, 1],
    [22, 86, 3],
    [28, 83, 2],
    [32, 83, 2],
    [36, 86, 2],
    [40, 81, 2],
    [43, 78, 2],
    [46, 76, 1],
    [48, 79, 2],
    [52, 83, 2],
    [56, 78, 2],
    [60, 74, 2],
  ],
  [
    [0, 76, 3],
    [4, 79, 2],
    [8, 83, 3],
    [14, 81, 1],
    [16, 79, 2],
    [20, 76, 2],
    [24, 74, 3],
    [30, 71, 1],
    [32, 78, 3],
    [36, 74, 2],
    [40, 71, 3],
    [46, 66, 1],
    [48, 70, 2],
    [52, 78, 2],
    [56, 70, 2],
    [60, 78, 2],
  ],
];
const notes: TitleNote[] = [];
for (let phrase = 0; phrase < 8; phrase++) {
  for (const [step, note, steps] of PHRASES[phrase % PHRASES.length]!)
    notes.push({
      step: phrase * 64 + step,
      note,
      steps,
      level: 0.78,
      voice: step % 16 === 14 ? "bubble" : "chip",
    });
}
export const TITLE_THEME: TitleTheme = {
  enabled: true,
  bpm: 169.275,
  loopSteps: 512,
  notes,
};
export const TITLE_SCORE_PROVENANCE = {
  sourceSha256:
    "95fcb149bb55da752c4d93c18fcc1913a23b70ccafecb52d8db5548dc64755d6",
  sourceDurationSeconds: 42.68,
  measuredKeyCandidate: "B minor",
  measuredTempoCandidate: 169.275,
  status: "reference-inspired composition draft",
  exactTranscription: false,
  directListeningAccepted: false,
} as const;
export function titleNotesAtStep(
  theme: TitleTheme,
  step: number,
): readonly TitleNote[] {
  const local = step % theme.loopSteps;
  return theme.enabled ? theme.notes.filter((note) => note.step === local) : [];
}
