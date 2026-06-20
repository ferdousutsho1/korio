import { writable } from "svelte/store";
import { browser } from "$app/environment";

/** A monotonically-updating clock (ms since epoch), ticked ~every 50ms while the app is open. */
export const nowMs = writable<number>(browser ? Date.now() : 0);
if (browser) setInterval(() => nowMs.set(Date.now()), 50);

// ---- Stopwatch ----
export interface StopwatchState { running: boolean; startedAt: number; elapsed: number; laps: number[]; }
export const stopwatch = writable<StopwatchState>({ running: false, startedAt: 0, elapsed: 0, laps: [] });

export function swStartStop() {
  stopwatch.update((s) => s.running
    ? { ...s, running: false, elapsed: s.elapsed + (Date.now() - s.startedAt) }
    : { ...s, running: true, startedAt: Date.now() });
}
export function swLap(displayMs: number) {
  stopwatch.update((s) => ({ ...s, laps: [displayMs, ...s.laps] }));
}
export function swReset() {
  stopwatch.set({ running: false, startedAt: 0, elapsed: 0, laps: [] });
}
export function swElapsed(s: StopwatchState, now: number): number {
  return s.elapsed + (s.running ? now - s.startedAt : 0);
}

// ---- Countdown timer ----
export interface TimerState { running: boolean; endsAt: number; remaining: number; durationMs: number; done: boolean; }
export const timer = writable<TimerState>({ running: false, endsAt: 0, remaining: 0, durationMs: 0, done: false });

export function timerSet(durationMs: number) {
  timer.set({ running: false, endsAt: 0, remaining: durationMs, durationMs, done: false });
}
export function timerStartPause() {
  timer.update((t) => {
    if (t.remaining <= 0 && !t.running) return t;
    return t.running
      ? { ...t, running: false, remaining: Math.max(0, t.endsAt - Date.now()) }
      : { ...t, running: true, endsAt: Date.now() + t.remaining, done: false };
  });
}
export function timerReset() {
  timer.update((t) => ({ running: false, endsAt: 0, remaining: t.durationMs, durationMs: t.durationMs, done: false }));
}
export function timerRemaining(t: TimerState, now: number): number {
  if (!t.running) return t.remaining;
  return Math.max(0, t.endsAt - now);
}
export function timerTick(now: number) {
  timer.update((t) => {
    if (t.running && now >= t.endsAt) {
      return { ...t, running: false, remaining: 0, done: true };
    }
    return t;
  });
}
