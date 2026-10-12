"""Measured mix differences only; does not prove notes or listening fidelity."""
from pathlib import Path
import hashlib,json
import numpy as np
from scipy.io import wavfile
from scipy.signal import welch

OUT=Path(__file__).resolve().parent
REFERENCE=OUT/'reference-stereo.wav'
DRAFT=OUT.parent/'title-fullness-v4/captures/qa-1791838846520-3f539dcd/title-score.wav'
BANDS=[(30,180),(180,700),(700,2000),(2000,6000),(6000,16000)]
def read(path):
    sr,y=wavfile.read(path)
    if np.issubdtype(y.dtype,np.integer):y=y.astype(np.float64)/2**(np.iinfo(y.dtype).bits-1)
    else:y=y.astype(np.float64)
    if y.ndim==1:y=np.column_stack([y,y])
    return sr,y
def metrics(path,seconds):
    sr,y=read(path);y=y[:round(seconds*sr)]
    mono=y.mean(axis=1)
    f,p=welch(mono,fs=sr,nperseg=8192)
    total=p[(f>=30)&(f<16000)].sum()
    rms=float(np.sqrt(np.mean(y*y)))
    return {'path':str(path),'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),
            'sample_rate':sr,'measured_seconds':len(y)/sr,'peak':float(np.max(np.abs(y))),
            'rms':rms,'rms_dbfs':float(20*np.log10(max(rms,1e-12))),
            'stereo_correlation':float(np.corrcoef(y.T)[0,1]),
            'normalized_mono_band_power':{f'{a}-{b}Hz':float(p[(f>=a)&(f<b)].sum()/total) for a,b in BANDS}}
result={'scope':'Actual decoded reference versus retained native V4 PCM, first42.68s. Bands normalized within30–16000Hz: independent of level. Different arrangement/phase means these are descriptive mix diagnostics, not perceptual or note-fidelity scores.',
        'reference':metrics(REFERENCE,42.68),'draft_v4':metrics(DRAFT,42.68)}
(OUT/'reference-v4-mix-comparison.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
print(json.dumps(result,indent=2))
