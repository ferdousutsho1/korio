import { readJSON, writeJSON } from "$lib/prefs";

export type SoundId = "chime" | "bell" | "ding" | "beep" | "silent";
export type SoundType = "pomodoro" | "timer";

export const SOUNDS: { id: SoundId; label: string }[] = [
  { id: "chime", label: "Chime" },
  { id: "bell", label: "Bell" },
  { id: "ding", label: "Ding" },
  { id: "beep", label: "Beep" },
  { id: "silent", label: "Silent" },
];

const DEFAULTS: Record<SoundType, SoundId> = { pomodoro: "chime", timer: "beep" };
const KEY = (t: SoundType) => `korio.sound.${t}`;
const valid = (v: unknown): v is SoundId => SOUNDS.some((s) => s.id === v);

export function getSoundPref(t: SoundType): SoundId {
  const v = readJSON<SoundId | null>(KEY(t), null);
  return valid(v) ? v : DEFAULTS[t];
}

export function setSoundPref(t: SoundType, id: SoundId): void {
  writeJSON(KEY(t), id);
}

let _ctx: AudioContext | undefined;
function getCtx(): AudioContext {
  if (!_ctx) {
    const Ctor = window.AudioContext || (window as any).webkitAudioContext;
    _ctx = new Ctor();
  }
  if (_ctx.state === "suspended") _ctx.resume();
  return _ctx;
}

/** Two-tone/oscillator presets. No-op for "silent" or when WebAudio is unavailable. */
export function playSound(id: SoundId): void {
  if (id === "silent") return;
  try {
    const ctx = getCtx();
    const tone = (freq: number, start: number, dur: number, peak = 0.25) => {
      const o = ctx.createOscillator(); const g = ctx.createGain();
      o.frequency.value = freq; o.connect(g); g.connect(ctx.destination);
      const t0 = ctx.currentTime + start;
      g.gain.setValueAtTime(0.0001, t0);
      g.gain.exponentialRampToValueAtTime(peak, t0 + 0.02);
      g.gain.exponentialRampToValueAtTime(0.0001, t0 + dur);
      o.start(t0); o.stop(t0 + dur);
    };
    switch (id) {
      case "chime": tone(660, 0, 0.16); tone(880, 0.18, 0.18); break;
      case "bell": tone(440, 0, 0.5, 0.3); tone(880, 0, 0.5, 0.12); break;
      case "ding": tone(1320, 0, 0.25); break;
      case "beep": tone(880, 0, 0.6, 0.3); break;
    }
  } catch { /* audio not available */ }
}
