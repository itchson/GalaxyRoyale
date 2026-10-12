/** Score data checks; no claim of an accepted transcription or audible match. */
import assert from "node:assert/strict";
import { test } from "node:test";
import { readFile } from "node:fs/promises";
import {
  TITLE_THEME,
  TITLE_SCORE_PROVENANCE,
  titleNotesAtStep,
} from "../../apps/game/src/title-theme.ts";

test("title is explicitly a provisional authored score with no recording/player path", async () => {
  assert(TITLE_THEME.enabled);
  assert.equal(TITLE_SCORE_PROVENANCE.exactTranscription, false);
  assert.equal(TITLE_SCORE_PROVENANCE.directListeningAccepted, false);
  assert.match(TITLE_SCORE_PROVENANCE.sourceSha256, /^[0-9a-f]{64}$/);
  const source = await readFile(
    new URL("../../apps/game/src/title-theme.ts", import.meta.url),
    "utf8",
  );
  const engine = await readFile(
    new URL("../../apps/game/src/audio.ts", import.meta.url),
    "utf8",
  );
  assert(
    !/fetch\(|decodeAudioData\(|TitleTrack|\.mp3[\"']/.test(source + engine),
  );
  assert.equal("url" in TITLE_THEME, false);
});

test("authored notes fit finite loop boundaries, include rests and repeat without note debt", () => {
  let occupied = 0;
  for (let step = 0; step < TITLE_THEME.loopSteps; step++) {
    const notes = titleNotesAtStep(TITLE_THEME, step);
    assert(notes.length <= 2);
    occupied += notes.length > 0 ? 1 : 0;
    assert.deepEqual(
      titleNotesAtStep(TITLE_THEME, step + TITLE_THEME.loopSteps),
      notes,
    );
    for (const note of notes) {
      assert(Number.isInteger(note.note) && note.note >= 24 && note.note <= 96);
      assert(Number.isFinite(note.steps) && note.steps > 0);
      assert(note.step + note.steps <= TITLE_THEME.loopSteps);
      assert(note.level > 0 && note.level <= 1);
    }
  }
  assert(
    occupied > 64 && occupied < TITLE_THEME.loopSteps / 2,
    "rhythmic rests remain authored",
  );
  assert.equal(
    titleNotesAtStep({ ...TITLE_THEME, enabled: false }, 0).length,
    0,
  );
});
