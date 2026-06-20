import { writable, get } from "svelte/store";
import { browser } from "$app/environment";

export type Phase = "focus" | "short" | "long";
export interface PomodoroConfig { focusMin: number; shortMin: number; longMin: number; longEvery: number; }
export interface PomodoroState {
  phase: Phase; running: boolean; endsAt: number; remaining: number;
  completedFocus: number; transitions: number;
}

const CFG_KEY = "korio.pomodoro.cfg";
const COUNT_KEY = "korio.pomodoro.count";
const DEFAULT_CFG: PomodoroConfig = { focusMin: 25, shortMin: 5, longMin: 15, longEvery: 4 };

function todayStr(): string { return new Date().toDateString(); }
function clampMin(v: unknown, dflt: number): number {
  return typeof v === "number" && v >= 1 && v <= 999 ? Math.round(v) : dflt;
}
function loadCfg(): PomodoroConfig {
  if (!browser) return DEFAULT_CFG;
  try {
    const raw = JSON.parse(localStorage.getItem(CFG_KEY) || "{}");
    return {
      focusMin: clampMin(raw.focusMin, DEFAULT_CFG.focusMin),
      shortMin: clampMin(raw.shortMin, DEFAULT_CFG.shortMin),
      longMin: clampMin(raw.longMin, DEFAULT_CFG.longMin),
      longEvery: clampMin(raw.longEvery, DEFAULT_CFG.longEvery),
    };
  } catch { return DEFAULT_CFG; }
}
function loadCount(): number {
  if (!browser) return 0;
  try {
    const raw = JSON.parse(localStorage.getItem(COUNT_KEY) || "{}");
    return raw.date === todayStr() && typeof raw.count === "number" ? raw.count : 0;
  } catch { return 0; }
}
function saveCount(count: number) {
  if (browser) localStorage.setItem(COUNT_KEY, JSON.stringify({ date: todayStr(), count }));
}

export const config = writable<PomodoroConfig>(loadCfg());

export function phaseDuration(cfg: PomodoroConfig, phase: Phase): number {
  const min = phase === "focus" ? cfg.focusMin : phase === "short" ? cfg.shortMin : cfg.longMin;
  return min * 60_000;
}

/** Pure: phase following a completed `phase`. `completedFocus` = count AFTER incrementing a finished focus. */
export function nextPhase(phase: Phase, completedFocus: number, longEvery: number): Phase {
  if (phase === "focus") return completedFocus % Math.max(1, longEvery) === 0 ? "long" : "short";
  return "focus";
}

const initialCfg = loadCfg();
export const pomodoro = writable<PomodoroState>({
  phase: "focus", running: false, endsAt: 0,
  remaining: phaseDuration(initialCfg, "focus"),
  completedFocus: loadCount(), transitions: 0,
});

export function setConfig(cfg: PomodoroConfig) {
  config.set(cfg);
  if (browser) localStorage.setItem(CFG_KEY, JSON.stringify(cfg));
  pomodoro.update((s) => s.running ? s : { ...s, remaining: phaseDuration(cfg, s.phase) });
}

export function pomoRemaining(s: PomodoroState, now: number): number {
  return s.running ? Math.max(0, s.endsAt - now) : s.remaining;
}

export function pomoStartPause() {
  pomodoro.update((s) => s.running
    ? { ...s, running: false, remaining: Math.max(0, s.endsAt - Date.now()) }
    : { ...s, running: true, endsAt: Date.now() + s.remaining });
}
export function pomoReset() {
  const cfg = get(config);
  pomodoro.update((s) => ({ ...s, phase: "focus", running: false, endsAt: 0, remaining: phaseDuration(cfg, "focus") }));
}
export function pomoSkip() {
  const cfg = get(config);
  pomodoro.update((s) => {
    const np = nextPhase(s.phase, s.completedFocus, cfg.longEvery);
    const dur = phaseDuration(cfg, np);
    return { ...s, phase: np, running: false, endsAt: 0, remaining: dur, transitions: s.transitions + 1 };
  });
}

function advance() {
  const cfg = get(config);
  const s = get(pomodoro);
  const finishedFocus = s.phase === "focus";
  const completedFocus = s.completedFocus + (finishedFocus ? 1 : 0);
  const np = nextPhase(s.phase, completedFocus, cfg.longEvery);
  const dur = phaseDuration(cfg, np);
  if (finishedFocus) saveCount(completedFocus);
  pomodoro.set({
    phase: np, running: true, endsAt: Date.now() + dur, remaining: dur,
    completedFocus, transitions: s.transitions + 1,
  });
}

// ticker owns the completion transition; components never write the store
if (browser) {
  setInterval(() => {
    const s = get(pomodoro);
    if (s.running && Date.now() >= s.endsAt) advance();
  }, 250);
}
