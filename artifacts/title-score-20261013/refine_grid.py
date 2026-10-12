"""Refine timing from the exported flux; does not establish perceived meter."""
from pathlib import Path
import json
import numpy as np
from scipy import signal, ndimage

OUT=Path(__file__).resolve().parent
data=np.genfromtxt(OUT/'onsets.csv',delimiter=',',names=True)
t=data['time_seconds']
dt=t[1]-t[0]
results=[]
for name in ['combined_flux','bass_flux','mid_flux','high_flux']:
    x=data[name]
    x=x-np.mean(x)
    window=signal.windows.hann(len(x))
    f, p=signal.periodogram(x,fs=1/dt,window=window,nfft=262144)
    mask=(f>=155/60)&(f<=185/60)
    i=np.flatnonzero(mask)[np.argmax(p[mask])]
    one=[]
    for a,b in [(0,42.6),(0,10),(10,20),(20,30),(30,42.6)]:
        xx=x[(t>=a)&(t<b)]
        ff,pp=signal.periodogram(xx,fs=1/dt,window='hann',nfft=262144)
        mm=(ff>=155/60)&(ff<=185/60)
        ii=np.flatnonzero(mm)[np.argmax(pp[mm])]
        one.append({'start':a,'end':b,'bpm':round(float(ff[ii]*60),4),'periodogram_peak':round(float(pp[ii]),7)})
    results.append({'flux':name,'periodogram_windows':one})
    subdivision=[]
    for multiplier in [1,2,4]:
        mm=(f>=155/60*multiplier)&(f<=185/60*multiplier)
        ii=np.flatnonzero(mm)[np.argmax(p[mm])]
        subdivision.append({'pulses_per_candidate_quarter':multiplier,'quarter_bpm_candidate':round(float(f[ii]*60/multiplier),4),'periodogram_peak':round(float(p[ii]),7)})
    results[-1]['whole_clip_subdivisions']=subdivision

source=data['bass_flux']
# Local bass onset peaks are spaced on eighths and quarters; aggregate a periodic
# grid with a 14 ms tolerance rather than labeling any particular hit a downbeat.
smooth=ndimage.gaussian_filter1d(source,sigma=.014/dt)
bpms=np.arange(155,185,.025)
best=[]
for bpm in bpms:
    period=60/bpm
    phases=np.linspace(0,period,160,endpoint=False)
    ticks=phases[:,None]+np.arange(0,t[-1]+period,period)[None,:]
    vals=np.interp(ticks,t,smooth,left=0,right=0)
    valid=ticks<=t[-1]
    scores=(vals*valid).sum(axis=1)/valid.sum(axis=1)
    i=int(np.argmax(scores))
    best.append((float(scores[i]),float(bpm),float(phases[i])))
ranked=sorted(best,reverse=True)
selected=[]
for score,bpm,phase in ranked:
    if all(abs(bpm-v['bpm'])>.35 for v in selected):
        selected.append({'bpm':round(bpm,3),'phase_seconds':round(phase,5),'mean_grid_flux':round(score,5)})
    if len(selected)>=8:break
report={'method':'Hann periodogram and constant-tempo bass-flux grid fit; estimates, no listening','periodogram':results,'bass_grid_candidates':selected,
    'chosen_for_analysis_only':selected[0],'not_playback_tempo_instruction':True}
(OUT/'refined-timing.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
print(json.dumps(report,indent=2))
