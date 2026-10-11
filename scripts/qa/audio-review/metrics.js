export function pcmMetrics(buffer, from = 0, to = buffer.duration) {
  const begin = Math.max(0, Math.floor(from * buffer.sampleRate));
  const end = Math.min(buffer.length, Math.floor(to * buffer.sampleRate));
  let peak = 0,
    squares = 0,
    over_unity = 0,
    nonfinite = 0,
    samples = 0;
  for (let channel = 0; channel < buffer.numberOfChannels; channel++) {
    const values = buffer.getChannelData(channel);
    for (let i = begin; i < end; i++) {
      const value = values[i];
      samples++;
      if (!Number.isFinite(value)) {
        nonfinite++;
        continue;
      }
      peak = Math.max(peak, Math.abs(value));
      squares += value * value;
      if (Math.abs(value) >= 1) over_unity++;
    }
  }
  const rms = Math.sqrt(squares / Math.max(1, samples));
  return {
    samples,
    peak,
    rms,
    peak_dbfs: peak > 0 ? 20 * Math.log10(peak) : null,
    rms_dbfs: rms > 0 ? 20 * Math.log10(rms) : null,
    over_unity,
    nonfinite,
  };
}

// Hann-windowed radix-2 FFT, eight actual PCM windows in the steady interval.
// The ratio is spectral power above1kHz, not an A-weighted loudness estimate.
export function highFrequencyRatio(buffer, from = 1, to = 3) {
  const size = 4096,
    values = buffer.getChannelData(0);
  const begin = Math.max(0, Math.floor(from * buffer.sampleRate));
  const end = Math.min(values.length, Math.floor(to * buffer.sampleRate));
  if (end - begin < size) return null;
  let total = 0,
    high = 0;
  for (let window = 0; window < 8; window++) {
    const offset = begin + Math.floor(((end - begin - size) * window) / 7);
    const real = new Float64Array(size),
      imag = new Float64Array(size);
    for (let i = 0; i < size; i++)
      real[i] =
        values[offset + i] *
        (0.5 - 0.5 * Math.cos((2 * Math.PI * i) / (size - 1)));
    for (let i = 1, j = 0; i < size; i++) {
      let bit = size >> 1;
      while (j & bit) {
        j ^= bit;
        bit >>= 1;
      }
      j ^= bit;
      if (i < j) [real[i], real[j]] = [real[j], real[i]];
    }
    for (let length = 2; length <= size; length <<= 1) {
      const angle = (-2 * Math.PI) / length;
      for (let start = 0; start < size; start += length) {
        let wr = 1,
          wi = 0;
        for (let j = 0; j < length / 2; j++) {
          const a = start + j,
            b = a + length / 2;
          const tr = wr * real[b] - wi * imag[b],
            ti = wr * imag[b] + wi * real[b];
          real[b] = real[a] - tr;
          imag[b] = imag[a] - ti;
          real[a] += tr;
          imag[a] += ti;
          const next = wr * Math.cos(angle) - wi * Math.sin(angle);
          wi = wr * Math.sin(angle) + wi * Math.cos(angle);
          wr = next;
        }
      }
    }
    for (let i = 1; i <= size / 2; i++) {
      const power = real[i] ** 2 + imag[i] ** 2;
      total += power;
      if ((i * buffer.sampleRate) / size >= 1000) high += power;
    }
  }
  return total > 0 ? high / total : 0;
}

// IEEE float32 WAV preserves the rendered samples, including any >1 peak.
export function wav(buffer) {
  const channels = buffer.numberOfChannels,
    count = buffer.length * channels;
  const output = new ArrayBuffer(44 + count * 4),
    view = new DataView(output);
  const text = (at, value) =>
    [...value].forEach((c, i) => view.setUint8(at + i, c.charCodeAt(0)));
  text(0, "RIFF");
  view.setUint32(4, output.byteLength - 8, true);
  text(8, "WAVE");
  text(12, "fmt ");
  view.setUint32(16, 16, true);
  view.setUint16(20, 3, true);
  view.setUint16(22, channels, true);
  view.setUint32(24, buffer.sampleRate, true);
  view.setUint32(28, buffer.sampleRate * channels * 4, true);
  view.setUint16(32, channels * 4, true);
  view.setUint16(34, 32, true);
  text(36, "data");
  view.setUint32(40, count * 4, true);
  const data = Array.from({ length: channels }, (_, c) =>
    buffer.getChannelData(c),
  );
  for (let i = 0; i < buffer.length; i++)
    for (let c = 0; c < channels; c++)
      view.setFloat32(44 + (i * channels + c) * 4, data[c][i], true);
  return output;
}
