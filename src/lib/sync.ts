import { get, type Writable } from "svelte/store";
import { browser } from "$app/environment";

export interface Bus {
  emit: (channel: string, payload: unknown) => void;
  listen: (channel: string, cb: (payload: unknown) => void) => void;
}
interface SyncMsg { origin: string; state: unknown; }

/** Pure, testable sync engine. Each window has a unique `originId`. */
export function createSync(bus: Bus, originId: string) {
  function register<T>(channel: string, store: Writable<T>) {
    // Reference-equality echo-guard. A simple `applying` boolean does NOT work:
    // Svelte's writable defers subscriber callbacks via a global queue, so the flag
    // is reset before the deferred callback runs, re-broadcasting applied state in a
    // loop. Instead we record the exact object we applied; the (deferred) subscribe
    // callback skips that same reference but still broadcasts genuinely-new objects.
    let last: T | undefined;
    let primed = false;
    bus.listen(channel, (payload) => {
      const msg = payload as SyncMsg;
      if (!msg || msg.origin === originId) return; // ignore our own echo
      last = msg.state as T;                        // mark so our own subscribe skips it
      store.set(msg.state as T);
    });
    bus.listen(channel + ":req", (payload) => {
      const msg = payload as { origin: string };
      if (!msg || msg.origin === originId) return; // don't answer our own request
      bus.emit(channel, { origin: originId, state: get(store) });
    });
    store.subscribe((state) => {
      if (!primed) { primed = true; last = state; return; } // skip the initial value
      if (state === last) return;                  // this set came from an applied foreign value
      last = state;
      bus.emit(channel, { origin: originId, state });
    });
  }
  function request(channel: string) { bus.emit(channel + ":req", { origin: originId }); }
  return { register, request };
}

// ---- Tauri wiring (no-op without Tauri, so vitest/browser preview are safe) ----
let IS_MAIN = true;
/** True in the main app window; false in widget windows. Defaults true until initSync resolves. */
export function isMainWindow(): boolean { return IS_MAIN; }

let started = false;
/** Call once from the root layout (browser only). Wires the shared stores across windows. */
export async function initSync() {
  if (!browser || started) return;
  started = true;
  let bus: Bus;
  let originId: string;
  try {
    const { emit, listen } = await import("@tauri-apps/api/event");
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    const label = getCurrentWindow().label;
    IS_MAIN = label === "main";
    originId = `${label}-${Math.random().toString(36).slice(2)}`;
    bus = {
      emit: (c, p) => { void emit(c, p); },
      listen: (c, cb) => { void listen(c, (e) => cb(e.payload)); },
    };
  } catch {
    return; // not in Tauri (browser preview / tests) — nothing to sync
  }
  const { stopwatch, timer } = await import("$lib/tools");
  const { pomodoro, config } = await import("$lib/pomodoro");
  const sync = createSync(bus, originId);
  const channels: [string, Writable<unknown>][] = [
    ["sync:stopwatch", stopwatch as unknown as Writable<unknown>],
    ["sync:timer", timer as unknown as Writable<unknown>],
    ["sync:pomodoro", pomodoro as unknown as Writable<unknown>],
    ["sync:pomodoro-cfg", config as unknown as Writable<unknown>],
  ];
  for (const [ch, store] of channels) sync.register(ch, store);
  // Ask peers for current state on open (widgets adopt the main window's running state).
  for (const [ch] of channels) sync.request(ch);
}
