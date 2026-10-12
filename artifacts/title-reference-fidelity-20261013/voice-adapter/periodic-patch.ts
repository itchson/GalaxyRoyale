/** Work-only source-parameter adapter; no PCM samples, inference or song-match claim. */
export type StereoPartial = {
  harmonic: number;
  real: readonly [number, number];
  imaginary: readonly [number, number];
};
export type PeriodicPatch = {
  frequencyHz: number;
  phaseOrigin: "event-first-sample-zero";
  partials: readonly StereoPartial[];
};
export type ChannelWave = { real: Float32Array; imaginary: Float32Array };
export type StereoWave = {
  frequencyHz: number;
  channels: readonly [ChannelWave, ChannelWave];
  disableNormalization: true;
};

/** NumPy's Re(c*exp(i*h*phase)) is WebAudio's real*cos + imag*sin.
 * Therefore its imaginary coefficient has the opposite sign. Preserve fitted
 * amplitude: WebAudio's default per-wave normalization would destroy the mix.
 * The caller owns stereo routing, envelopes and the shared beat transport.
 */
export function periodicParameters(
  patch: PeriodicPatch,
  sampleRate: number,
  maxHarmonic = 128,
): StereoWave {
  if (
    !Number.isFinite(sampleRate) || sampleRate < 8000 ||
    !Number.isFinite(patch.frequencyHz) || patch.frequencyHz <= 0 ||
    patch.frequencyHz >= sampleRate / 2 ||
    patch.phaseOrigin !== "event-first-sample-zero" ||
    !Number.isInteger(maxHarmonic) || maxHarmonic < 1 || maxHarmonic > 2048 ||
    !Array.isArray(patch.partials) || !patch.partials.length ||
    patch.partials.length > maxHarmonic
  ) throw new RangeError("Invalid source harmonic patch");
  const seen = new Set<number>();
  let last = 0;
  for (const partial of patch.partials) {
    if (
      !Number.isInteger(partial.harmonic) || partial.harmonic < 1 ||
      partial.harmonic > maxHarmonic || seen.has(partial.harmonic) ||
      partial.harmonic * patch.frequencyHz >= sampleRate / 2 ||
      partial.real.length !== 2 || partial.imaginary.length !== 2 ||
      [...partial.real, ...partial.imaginary].some(
        value => !Number.isFinite(value) || Math.abs(value) > 8,
      )
    ) throw new RangeError("Invalid or aliased stereo partial");
    seen.add(partial.harmonic);
    last = Math.max(last, partial.harmonic);
  }
  const channels = [0, 1].map(channel => {
    const real = new Float32Array(last + 1);
    const imaginary = new Float32Array(last + 1);
    for (const partial of patch.partials) {
      real[partial.harmonic] = partial.real[channel]!;
      imaginary[partial.harmonic] = -partial.imaginary[channel]!;
    }
    return { real, imaginary };
  }) as [ChannelWave, ChannelWave];
  return { frequencyHz: patch.frequencyHz, channels, disableNormalization: true };
}
