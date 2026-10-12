"""Bounded local reference analysis. Estimates are not a listened transcription."""
from pathlib import Path
import csv
import hashlib
import json
import platform
import subprocess

import numpy as np
import scipy
from scipy.io import wavfile
from scipy import signal, ndimage
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

OUT = Path(__file__).resolve().parent
SOURCE = Path(r"C:\Users\mitch\Downloads\Galaxy Royale (3).mp3")
WAV = OUT / "reference-analysis-mono.wav"
probe = json.loads(subprocess.check_output(["ffprobe", "-v", "error", "-show_format", "-show_streams", "-of", "json", str(SOURCE)], text=True))
sample_rate, pcm = wavfile.read(WAV)
y = pcm.astype(np.float64) / 32768
nfft, hop = 8192, 256
freq, times, z = signal.stft(y, fs=sample_rate, window="hann", nperseg=nfft, noverlap=nfft-hop, boundary="zeros", padded=True)
mag = np.abs(z)
power = mag ** 2
onset_fft, onset_hop = 1024, 128
onset_freq, onset_times, onset_z = signal.stft(y, fs=sample_rate, window="hann", nperseg=onset_fft, noverlap=onset_fft-onset_hop, boundary="zeros", padded=True)
onset_mag = np.abs(onset_z)
# Soft median harmonic/percussive separation, applied only to measured spectra.
time_median = ndimage.median_filter(mag, size=(1, 17))
freq_median = ndimage.median_filter(mag, size=(31, 1))
harmonic_mask = time_median ** 2 / (time_median ** 2 + freq_median ** 2 + 1e-18)
harmonic = mag * harmonic_mask
percussive = mag * (1-harmonic_mask)

def peaks(values, min_bpm=55, max_bpm=200):
    values = values - np.mean(values)
    corr = signal.correlate(values, values, mode="full", method="fft")[len(values)-1:]
    corr /= np.maximum(1, np.arange(len(values), 0, -1))
    corr /= max(corr[0], 1e-18)
    lo = max(1, int(60 * sample_rate / onset_hop / max_bpm))
    hi = min(len(corr)-1, int(60 * sample_rate / onset_hop / min_bpm))
    indices, _ = signal.find_peaks(corr[lo:hi])
    ranked = sorted(indices+lo, key=lambda i: corr[i], reverse=True)[:8]
    return [{"bpm": round(60*sample_rate/onset_hop/i, 3), "autocorrelation": round(float(corr[i]), 5), "lag_frames": int(i)} for i in ranked]

band_defs = [(30,180),(180,2000),(2000,9000),(30,10000)]
onsets, tempo = [], []
for low, high in band_defs:
    mask = (onset_freq >= low) & (onset_freq < high)
    # Positive flux on compressed magnitude, summed by frequency band.
    compressed = np.log1p(onset_mag[mask] * 120)
    flux = np.maximum(0, np.diff(compressed, axis=1, prepend=compressed[:,:1])).mean(axis=0)
    floor = ndimage.median_filter(flux, size=87)
    flux = np.maximum(0, flux - .6*floor)
    onsets.append(flux)
    tempo.append({"band_hz": [low, high], "whole": peaks(flux), "windows": [
        {"start": a, "end": b, "candidates": peaks(flux[(onset_times>=a)&(onset_times<b)])[:4]}
        for a,b in [(0,10),(10,20),(20,30),(30,42.65)]
    ]})
onsets = np.array(onsets)
normalized_onset = onsets / np.maximum(np.quantile(onsets,.95,axis=1)[:,None],1e-12)
combined = normalized_onset.mean(axis=0)
onset_indices, _ = signal.find_peaks(combined, height=np.quantile(combined,.62), prominence=.12, distance=int(.08*sample_rate/onset_hop))

# Fold harmonic peak power into pitch classes. Avoid DC, drums and low bass bins.
midi = np.zeros_like(freq)
positive = freq>0
midi[positive] = 69+12*np.log2(freq[positive]/440)
valid = (freq>=90)&(freq<=4200)
nearest = np.rint(midi).astype(int)
closeness = np.exp(-.5*((midi-nearest)/.18)**2)
chroma = np.zeros((12,len(times)))
whiten = np.maximum(0, harmonic-ndimage.median_filter(harmonic, size=(19,1)))
for pc in range(12):
    bins = valid & (nearest%12==pc)
    chroma[pc] = np.sum(np.sqrt(whiten[bins]) * closeness[bins,None],axis=0)
chroma /= np.maximum(chroma.sum(axis=0),1e-12)
names = ["C","C#","D","D#","E","F","F#","G","G#","A","A#","B"]
major = np.array([6.35,2.23,3.48,2.33,4.38,4.09,2.52,5.19,2.39,3.66,2.29,2.88])
minor = np.array([6.33,2.68,3.52,5.38,2.60,3.53,2.54,4.75,3.98,2.69,3.34,3.17])

def key_candidates(vector):
    scores = []
    for pc in range(12):
        for label, profile in [("major",major),("minor",minor)]:
            score = np.corrcoef(vector,np.roll(profile,pc))[0,1]
            scores.append({"key": names[pc]+" "+label, "profile_correlation":round(float(score),4)})
    return sorted(scores,key=lambda c:c["profile_correlation"],reverse=True)[:6]

def chord_candidates(vector):
    values=[]
    for root in range(12):
        for label, offsets in [("major",[0,4,7]),("minor",[0,3,7]),("sus2",[0,2,7]),("sus4",[0,5,7])]:
            notes=[(root+i)%12 for i in offsets]
            score=float(vector[notes].sum())
            values.append({"chord":names[root]+" "+label,"chroma_share":round(score,4)})
    return sorted(values,key=lambda v:v["chroma_share"],reverse=True)[:4]

# Fundamental salience: interpolate harmonic peaks at partials and penalize
# octave aliases. This remains ambiguous for mixed chords and bright chip voices.
notes = np.arange(36,96)
salience = np.zeros((len(notes),len(times)))
spectral = whiten / np.maximum(ndimage.uniform_filter1d(harmonic, size=71, axis=0),1e-7)
spectral=np.clip(spectral,0,8)
for row,note in enumerate(notes):
    f=440*2**((note-69)/12)
    for h in range(1,7):
        bin_idx=int(round(f*h* nfft/sample_rate))
        if bin_idx>=len(freq): continue
        radius=max(1,int(bin_idx*.006))
        local=spectral[max(0,bin_idx-radius):min(len(freq),bin_idx+radius+1)].max(axis=0)
        salience[row] += local / h
    salience[row] *= np.clip(harmonic[max(1,int(round(f*nfft/sample_rate)))]/.0003,0,1)

def note_name(note):
    return names[note%12]+str(note//12-1)

def salient(a,b,lo=36,hi=95,count=6):
    mask=(times>=a)&(times<b)
    v=salience[:,mask].mean(axis=1)
    ranked=sorted((i for i,n in enumerate(notes) if lo<=n<=hi),key=lambda i:v[i],reverse=True)[:count]
    return [{"midi":int(notes[i]),"note":note_name(int(notes[i])),"salience":round(float(v[i]),3)} for i in ranked]

sections=[]
for a in np.arange(0,42,2):
    b=min(float(a+2),len(y)/sample_rate)
    mask=(times>=a)&(times<b)
    frame=power[:,mask].mean(axis=1)
    centroid=float((freq*frame).sum()/max(frame.sum(),1e-15))
    samples=y[int(a*sample_rate):int(b*sample_rate)]
    vector=chroma[:,mask].mean(axis=1)
    sections.append({"start":float(a),"end":b,"rms_dbfs":round(float(20*np.log10(np.sqrt(np.mean(samples*samples))+1e-12)),2),
        "spectral_centroid_hz":round(centroid,1),"keys":key_candidates(vector)[:3],"chords":chord_candidates(vector),
        "bass_candidates":salient(a,b,36,59,4),"upper_candidates":salient(a,b,60,95,6),
        "chroma":{n:round(float(v),4) for n,v in zip(names,vector)}})

events=[]
for a in np.arange(0,42.65,.125):
    b=min(a+.125,len(y)/sample_rate)
    candidates=salient(a,b,60,95,3)
    events.append({"start":round(float(a),3),"end":round(float(b),3),"candidates":candidates})

report={
    "source":{"path":str(SOURCE),"bytes":SOURCE.stat().st_size,"sha256":hashlib.sha256(SOURCE.read_bytes()).hexdigest(),"ffprobe":probe},
    "analysis":{"python":platform.python_version(),"numpy":np.__version__,"scipy":scipy.__version__,"matplotlib":matplotlib.__version__,
        "script_sha256":hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),"wav_sha256":hashlib.sha256(WAV.read_bytes()).hexdigest(),
        "sample_rate":sample_rate,"samples":len(y),"decoded_duration":len(y)/sample_rate,"harmonic_fft":nfft,"harmonic_hop":hop,"onset_fft":onset_fft,"onset_hop":onset_hop,
        "peak":float(np.abs(y).max()),"rms":float(np.sqrt(np.mean(y*y))),
        "direct_listening":False,"transcription_accepted":False,"warning":"All key/chord/note/timbre statements are machine-analysis candidates. No listening or perceived-match proof."},
    "tempo":tempo,"combined_tempo":peaks(combined),"onset_times_seconds":[round(float(onset_times[i]),4) for i in onset_indices],
    "global_chroma":{n:round(float(v),4) for n,v in zip(names,chroma.mean(axis=1))},"key_candidates":key_candidates(chroma.mean(axis=1)),
    "sections_two_second":sections,"upper_pitch_windows":events}
(OUT/"measured-analysis.json").write_text(json.dumps(report,indent=2),encoding="utf-8")
with (OUT/"onsets.csv").open("w",encoding="utf-8",newline="") as file:
    w=csv.writer(file);w.writerow(["time_seconds","combined_flux","bass_flux","mid_flux","high_flux"])
    w.writerows(zip(onset_times,combined,onsets[0],onsets[1],onsets[2]))

fig,axes=plt.subplots(4,1,figsize=(17,12),sharex=True)
sample_hop=100
axes[0].plot(np.arange(0,len(y),sample_hop)/sample_rate,y[::sample_hop],linewidth=.4)
axes[0].set(ylabel="Mono PCM",title="Galaxy Royale (3) — measured reference, no listened transcription")
axes[1].imshow(20*np.log10(mag+1e-6),origin="lower",aspect="auto",extent=[times[0],times[-1],freq[0],freq[-1]],vmin=-65,vmax=-20,cmap="magma")
axes[1].set(ylim=(0,2500),ylabel="Frequency / Hz")
axes[2].imshow(chroma,origin="lower",aspect="auto",extent=[times[0],times[-1],-.5,11.5],cmap="viridis")
axes[2].set(yticks=np.arange(12),yticklabels=names,ylabel="Harmonic chroma")
axes[3].plot(onset_times,combined,linewidth=.75,label="Multiband positive flux")
axes[3].scatter(onset_times[onset_indices],combined[onset_indices],s=7,c="red")
axes[3].set(xlabel="Decoded time / seconds",ylabel="Onset strength",xlim=(0,len(y)/sample_rate));axes[3].legend()
fig.tight_layout();fig.savefig(OUT/"reference-analysis.png",dpi=150);plt.close(fig)
print(json.dumps({k:report[k] for k in ["analysis","combined_tempo","global_chroma","key_candidates"]},indent=2))
print("Two-second leading chord / bass / upper candidates:")
for s in sections:
    print(s["start"],s["rms_dbfs"],s["chords"][:2],s["bass_candidates"][:2],s["upper_candidates"][:3])
