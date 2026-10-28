#!/usr/bin/env python3
"""Synthesize the MP3 test sounds that tests/SOUNDS.md lists as generated here.

Every sample is computed by this script -- oscillators, envelopes and seeded
noise -- so the output contains no third-party audio, and running it again
gives the same samples. The MP3 bytes depend on the encoder as well, so a
different `lameenc` release may encode them slightly differently; the sound
itself does not change.

    pip install lameenc          # dev-only; not a dependency of the SDK
    python scripts/generate-test-audio.py

It overwrites both files and prints each one's length. A length that changed
has to be copied into the `<msec>` of the sound list beside it, which
SpriteStudio fills in when it imports a sound and nothing else recomputes.
"""
import array
import math
import random
from pathlib import Path

import lameenc

ROOT = Path(__file__).resolve().parent.parent
SR = 44100


def midi_hz(n):
    return 440.0 * 2 ** ((n - 69) / 12)


def tri(phase):
    p = phase % 1.0
    return 4 * p - 1 if p < 0.5 else 3 - 4 * p


def env_adsr(t, dur, a=0.01, d=0.08, s=0.7, r=0.12):
    if t < 0 or t > dur + r:
        return 0.0
    if t < a:
        return t / a
    if t < a + d:
        return 1 - (1 - s) * (t - a) / d
    if t <= dur:
        return s
    return s * (1 - (t - dur) / r)


def render_music(seconds=30.0):
    """A 120 BPM loop: C - Am - F - G pad, bass, arpeggio, kick and hi-hat."""
    n = int(SR * seconds)
    left = [0.0] * n
    right = [0.0] * n
    beat = 60 / 120
    bar = beat * 4
    chords = [[60, 64, 67], [57, 60, 64], [53, 57, 60], [55, 59, 62]]
    rng = random.Random(20261028)

    def add(start, dur, fn, gain, pan, release=0.12):
        i0 = max(0, int(start * SR))
        i1 = min(n, int((start + dur + release) * SR))
        gl = gain * math.cos(pan * math.pi / 2)
        gr = gain * math.sin(pan * math.pi / 2)
        for i in range(i0, i1):
            t = i / SR - start
            v = fn(t) * env_adsr(t, dur, r=release)
            left[i] += v * gl
            right[i] += v * gr

    t = 0.0
    bar_idx = 0
    while t < seconds:
        chord = chords[bar_idx % 4]
        for k, note in enumerate(chord):
            f = midi_hz(note)
            add(t, bar - 0.05, lambda x, f=f: tri(f * x), 0.07, 0.3 + 0.2 * k, release=0.3)
        root = midi_hz(chord[0] - 24)
        for b in (0, 2):
            add(t + b * beat, beat * 0.9, lambda x, f=root: math.sin(2 * math.pi * f * x), 0.22, 0.5)
        arp = chord + [chord[0] + 12]
        for e in range(8):
            f = midi_hz(arp[(e + bar_idx) % 4] + 12)
            add(t + e * beat / 2, beat / 2 * 0.8,
                lambda x, f=f: math.sin(2 * math.pi * f * x) + 0.25 * math.sin(4 * math.pi * f * x),
                0.06, 0.2 if e % 2 == 0 else 0.8, release=0.08)
        for b in range(4):
            add(t + b * beat, 0.12,
                lambda x: math.sin(2 * math.pi * (55 + 90 * math.exp(-x * 30)) * x) * math.exp(-x * 18),
                0.35, 0.5, release=0.02)
            noise = [rng.uniform(-1, 1) for _ in range(int(0.05 * SR))]
            add(t + b * beat + beat / 2, 0.04,
                lambda x, nz=noise: nz[min(int(x * SR), len(nz) - 1)] * math.exp(-x * 60),
                0.08, 0.6, release=0.01)
        t += bar
        bar_idx += 1

    for i in range(n):  # fade in 0.5 s, fade out 2 s
        x = i / SR
        g = min(1.0, x / 0.5, (seconds - x) / 2.0)
        left[i] *= g
        right[i] *= g
    return left, right


def render_chime(seconds=0.8):
    """A two-note rising chime with bell-like partials, the length of a UI sound."""
    n = int(SR * seconds)
    left = [0.0] * n
    right = [0.0] * n
    notes = [(0.0, midi_hz(88), 0.35), (0.12, midi_hz(95), 0.5)]
    partials = [(1.0, 1.0), (2.76, 0.35), (5.4, 0.12)]
    for start, f, pan in notes:
        gl = math.cos(pan * math.pi / 2)
        gr = math.sin(pan * math.pi / 2)
        for i in range(int(start * SR), n):
            x = i / SR - start
            attack = min(1.0, x / 0.004)
            v = sum(a * math.sin(2 * math.pi * f * m * x) * math.exp(-x * (6 + 4 * m))
                    for m, a in partials)
            v *= 0.3 * attack
            left[i] += v * gl
            right[i] += v * gr
    for i in range(n):  # short fade-out so the tail ends at silence
        g = min(1.0, (seconds - i / SR) / 0.05)
        left[i] *= g
        right[i] *= g
    return left, right


def to_pcm(left, right):
    peak = max(max(abs(v) for v in left), max(abs(v) for v in right)) or 1.0
    scale = 0.89 / peak * 32767  # about -1 dBFS
    pcm = array.array('h')
    for a, b in zip(left, right):
        pcm.append(int(max(-32768, min(32767, a * scale))))
        pcm.append(int(max(-32768, min(32767, b * scale))))
    return pcm.tobytes()


def encode(pcm, kbps):
    enc = lameenc.Encoder()
    enc.set_bit_rate(kbps)
    enc.set_in_sample_rate(SR)
    enc.set_channels(2)
    enc.set_quality(2)
    return enc.encode(pcm) + enc.flush()


def mp3_msec(data):
    """Length in milliseconds, counted from the MPEG-1 Layer III frames this script writes."""
    rates = [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320]
    i = frames = 0
    while i + 4 <= len(data):
        if data[i] == 0xFF and (data[i + 1] & 0xFE) == 0xFA:
            br = rates[data[i + 2] >> 4] * 1000
            if br:
                frames += 1
                i += 144 * br // SR + ((data[i + 2] >> 1) & 1)
                continue
        i += 1
    return frames * 1152 * 1000 // SR


TARGETS = [
    ('tests/overall/sound/test.mp3', render_music, 128),
    ('tests/allPartsV7/sound/se_sys_49.mp3', render_chime, 160),
]


def main():
    for rel, render, kbps in TARGETS:
        data = encode(to_pcm(*render()), kbps)
        (ROOT / rel).write_bytes(data)
        print(f'{rel}: {mp3_msec(data)} ms')


if __name__ == '__main__':
    main()
