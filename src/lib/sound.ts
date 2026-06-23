import { readJSON, writeJSON } from "$lib/prefs";

export type SoundId = "chime" | "bell" | "ding" | "beep" | "alarm" | "marimba" | "pulse" | "siren" | "silent";
export type SoundType = "pomodoro" | "timer" | "limit";
export type LoopType = "pomodoro" | "timer";

export const SOUNDS: { id: SoundId; label: string }[] = [
  { id: "chime", label: "Chime" },
  { id: "bell", label: "Bell" },
  { id: "ding", label: "Ding" },
  { id: "beep", label: "Beep" },
  { id: "alarm", label: "Alarm" },
  { id: "marimba", label: "Marimba" },
  { id: "pulse", label: "Pulse" },
  { id: "siren", label: "Siren" },
  { id: "silent", label: "Silent" },
];

const DEFAULTS: Record<SoundType, SoundId> = { pomodoro: "chime", timer: "beep", limit: "alarm" };
const KEY = (t: SoundType) => `korio.sound.${t}`;
const valid = (v: unknown): v is SoundId => SOUNDS.some((s) => s.id === v);

export function getSoundPref(t: SoundType): SoundId {
  const v = readJSON<SoundId | null>(KEY(t), null);
  return valid(v) ? v : DEFAULTS[t];
}

export function setSoundPref(t: SoundType, id: SoundId): void {
  writeJSON(KEY(t), id);
}

const LOOP_KEY = (t: LoopType) => `korio.sound.${t}.loop`;
export function getLoopPref(t: LoopType): boolean {
  return readJSON<boolean>(LOOP_KEY(t), false) === true;
}
export function setLoopPref(t: LoopType, on: boolean): void {
  writeJSON(LOOP_KEY(t), on);
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
      case "alarm":
        tone(880, 0, 0.18, 0.3); tone(740, 0.2, 0.18, 0.3); tone(880, 0.4, 0.22, 0.3); break;
      case "marimba":
        tone(523, 0, 0.18); tone(659, 0.12, 0.18); tone(784, 0.24, 0.24); break;
      case "pulse":
        tone(600, 0, 0.1, 0.3); tone(600, 0.16, 0.1, 0.3); break;
      case "siren":
        tone(700, 0, 0.45, 0.3); tone(550, 0.18, 0.45, 0.3); break;
    }
  } catch { /* audio not available */ }
}

const _loops = new Set<() => void>();

/** Repeat a preset until the returned stop() is called. No-op for "silent". */
export function startLoop(id: SoundId): () => void {
  if (id === "silent") return () => {};
  let stopped = false;
  playSound(id);
  const iv = setInterval(() => { if (!stopped) playSound(id); }, 1300);
  const stop = () => { stopped = true; clearInterval(iv); _loops.delete(stop); };
  _loops.add(stop);
  return stop;
}

/** Stop every active loop (safety net on unmount). */
export function stopAllSounds(): void {
  for (const s of [..._loops]) s();
}
