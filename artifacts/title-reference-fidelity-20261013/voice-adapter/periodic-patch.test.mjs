import assert from 'node:assert/strict';
import test from 'node:test';
import { periodicParameters } from './periodic-patch.ts';

const patch = () => ({
  frequencyHz: 440, phaseOrigin: 'event-first-sample-zero',
  partials: [
    { harmonic: 1, real: [.17, -.08], imaginary: [-.12, .23] },
    { harmonic: 4, real: [.04, .01], imaginary: [.03, -.06] },
  ],
});
test('stereo harmonic signs preserve the source local-phase basis and amplitude', () => {
  const input = patch(), result = periodicParameters(input, 48000);
  assert.equal(result.disableNormalization, true);
  for (let channel = 0; channel < 2; channel++) {
    const wave = result.channels[channel];
    assert.equal(wave.real[0], 0); assert.equal(wave.imaginary[0], 0);
    for (let i = 0; i < 257; i++) {
      const phase = i * Math.PI * 2 / 257;
      const source = input.partials.reduce((sum, p) => sum +
        p.real[channel] * Math.cos(p.harmonic * phase) -
        p.imaginary[channel] * Math.sin(p.harmonic * phase), 0);
      const web = Array.from(wave.real).reduce((sum, r, h) => sum +
        r * Math.cos(h * phase) + wave.imaginary[h] * Math.sin(h * phase), 0);
      assert.ok(Math.abs(source - web) < 3e-8);
    }
  }
  assert.notDeepEqual(result.channels[0], result.channels[1]);
  assert.deepEqual(input, patch());
});
test('sparse harmonics have zero gaps and isolated immutable input ownership', () => {
  const input = patch(), first = periodicParameters(input, 48000);
  assert.equal(first.channels[0].real[2], 0);
  first.channels[0].real[1] = 0;
  assert.ok(periodicParameters(input, 48000).channels[0].real[1] > .16);
});
test('rejects aliases and malformed banks instead of silently removing source partials', () => {
  const variants = [
    { ...patch(), frequencyHz: NaN },
    { ...patch(), frequencyHz: 24000 },
    { ...patch(), phaseOrigin: 'absolute-source-time' },
    { ...patch(), partials: [] },
    { ...patch(), partials: [...patch().partials, patch().partials[0]] },
    { ...patch(), partials: [{ harmonic: 60, real: [0, 0], imaginary: [0, 0] }] },
    { ...patch(), partials: [{ harmonic: 1.5, real: [0, 0], imaginary: [0, 0] }] },
    { ...patch(), partials: [{ harmonic: 1, real: [NaN, 0], imaginary: [0, 0] }] },
    { ...patch(), partials: [{ harmonic: 1, real: [0], imaginary: [0, 0] }] },
  ];
  for (const invalid of variants) assert.throws(() => periodicParameters(invalid, 48000), RangeError);
  assert.throws(() => periodicParameters(patch(), 0), RangeError);
  assert.throws(() => periodicParameters(patch(), 48000, 3), RangeError);
  assert.throws(() => periodicParameters(patch(), 48000, 2049), RangeError);
});
