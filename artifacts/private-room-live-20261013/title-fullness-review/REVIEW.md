# V3 title fullness audit

**Present V3 for audition before changing it again.** The latest “still fuller” reaction belongs to V2; Root confirmed V3 has not received human feedback. V3 already makes substantive instrument changes. No source-backed defect or new listening evidence justifies another runtime refinement now.

At Root `5e4b629e91f3334582ffa30d5e65af98675f8c39`, all five current audio source files match the retained V3 native capture's exact input byte hashes. Canonical Git and CRLF working hashes are recorded separately. All seven retained WAV hashes match their report, and all seven report no nonfinite/over-unity samples and zero cleanup voices/callbacks. This audit verified existing evidence; it did not render, play or certify perceived sound.

| Current title instrument | V3 body and sustain |
| --- | --- |
| Lead (`audio.ts:1104`) | Two low-pass square pulses at +3/−4 cents and opposite ±0.28 pans, plus centered nominal triangle body; three sources per note, attacks 9–14ms and hold to 55% of the finite note gate. |
| Pad (`:981`) | Four chord tones, each with a triangle and darker detuned filtered saw in the same register; eight sources, opposing modest pans, 180/230ms attacks, sustained inner envelopes and one shared finite outer gate. |
| Bass (`:1056`) | Centered triangle, sine body and filtered saw, all at the authored fundamental; three finite sources, 8–14ms attacks and 55% holds. |
| Arp / bubble / drums (`:1026`) | Held triangle arp, sine pitch-bend bubble with body at its target pitch, restrained alternating pans; finite kick/noise percussion. |

The retained 48-second title report measures peak **0.157576**, RMS **0.024680**, correlation **0.988069**, and maximum **7 music voices**. Side RMS is 22.19dB below mid RMS; mono-mid RMS loss versus stereo RMS is 0.02615dB. This supports a conservative stereo spread with little measured whole-clip downmix loss. It does not prove subjective richness, local absence of cancellation, stereo-speaker quality or reference fidelity.

Source limits remain deliberate: at 169.275BPM the pad gates for 15 of 16 steps (1.329s), fades during its final 100ms, then leaves an 88.6ms gap before the next bar's attack. Bass gates are 124ms and arps 115ms; the lead retains short articulated notes. The shared music route includes a 5600Hz low-pass and quiet 280ms echo, without a title reverb/chorus network. These explain the present articulation and texture; they are hypotheses to revisit if specific V3 feedback asks for more sustained body, not defects inferred from V2 feedback.

The existing 32-music/16-SFX voice caps and shared ownership remain in place. V3 increases sources inside existing voices, so voice count alone is not a source budget. The prior independently recorded full-loop evidence sampled at most 40 music sources / 129 stereo nodes and nine sources in one voice for this unchanged authored score. No fresh resource tests were run here; future note/layer additions would require that bound to be revisited. Current pad gate retiming replaces every layer's stop and preserves the shared outer envelope; mono fallback sums the same finite layers.

**Recommended additional musical implementation: none before V3 audition.** If V3 still needs work, obtain the precise remaining issue (sustain/body/width/articulation) and change one corresponding contour or layer allocation with a fresh native comparison. Current measurements do not justify a master-gain increase.

`audit-evidence.json` / `audit.log` retain actual hash checks and arithmetic from existing measurements. `node work/title-fullness-review/audit.mjs` exited 0. No runtime edits, tests, listeners, browser, new PCM render, listening, gameplay/mobile or exact-song acceptance. Root networking helpers were untouched.
