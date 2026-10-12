import assert from "node:assert/strict";
import { test } from "node:test";
import {
  pcmMetrics,
  stereoMetrics,
  highFrequencyRatio,
  wav,
} from "./metrics.js";
const buffer = (values, sampleRate = 48000) => ({
  numberOfChannels: values.length,
  length: values[0].length,
  sampleRate,
  duration: values[0].length / sampleRate,
  getChannelData: (channel) => values[channel],
});

test("stereo metrics distinguish mono-compatible, opposite-phase and silent analytical channels", () => {
  const v = new Float32Array([-0.5, 0.5]),
    opposite = new Float32Array([0.5, -0.5]);
  const mono = stereoMetrics(buffer([v, v]));
  assert.equal(mono.midRms, 0.5);
  assert.equal(mono.sideRms, 0);
  assert.equal(mono.correlation, 1);
  const anti = stereoMetrics(buffer([v, opposite]));
  assert.equal(anti.midRms, 0);
  assert.equal(anti.sideRms, 0.5);
  assert.equal(anti.correlation, -1);
  assert.equal(
    stereoMetrics(buffer([new Float32Array(2), new Float32Array(2)]))
      .correlation,
    null,
  );
  assert.equal(stereoMetrics(buffer([v])), null);
});
test("PCM metrics count actual nonfinite and >1 values without silently clamping", () => {
  const b = buffer([new Float32Array([0, 1.25, -0.5, Number.NaN])], 4);
  const m = pcmMetrics(b);
  assert.equal(m.samples, 4);
  assert.equal(m.peak, 1.25);
  assert.equal(m.over_unity, 1);
  assert.equal(m.nonfinite, 1);
  assert.equal(m.rms, Math.sqrt((1.25 ** 2 + 0.5 ** 2) / 4));
});
test("PCM windows include channels and preserve the requested time interval", () => {
  const b = buffer(
    [
      new Float32Array([0, 0.25, 0.5, 1]),
      new Float32Array([0, -0.25, -0.5, -1]),
    ],
    4,
  );
  const m = pcmMetrics(b, 0.25, 0.75);
  assert.equal(m.samples, 4);
  assert.equal(m.peak, 0.5);
  assert.equal(m.over_unity, 0);
  assert.equal(m.nonfinite, 0);
});
test("Hann FFT distinguishes known low/high analytical tones (math-only fixture)", () => {
  const sine = (hz) =>
    buffer([
      Float32Array.from(
        { length: 48000 },
        (_, i) => 0.5 * Math.sin((2 * Math.PI * hz * i) / 48000),
      ),
    ]);
  assert(highFrequencyRatio(sine(100), 0, 1) < 0.000001);
  assert(highFrequencyRatio(sine(4000), 0, 1) > 0.999999);
  assert.equal(highFrequencyRatio(buffer([new Float32Array(48000)]), 0, 1), 0);
});
test("float32 WAV preserves exact interleaved PCM including values above unity", () => {
  const b = buffer([
    new Float32Array([0.25, 1.25]),
    new Float32Array([-0.5, -1.5]),
  ]);
  const bytes = wav(b),
    view = new DataView(bytes);
  assert.equal(view.getUint16(20, true), 3);
  assert.equal(view.getUint16(22, true), 2);
  assert.equal(view.getUint16(34, true), 32);
  assert.equal(view.getUint32(40, true), 16);
  assert.deepEqual(
    [0, 1, 2, 3].map((i) => view.getFloat32(44 + i * 4, true)),
    [0.25, -0.5, 1.25, -1.5],
  );
});
