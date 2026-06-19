import { writable } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import { browser } from "$app/environment";

export interface Live { exe: string | null; active_seconds: number; }
export const live = writable<Live>({ exe: null, active_seconds: 0 });

// bumps each time a session is saved, so views can refresh
export const sessionTick = writable(0);

if (browser) {
  listen<Live>("live-status", (e) => live.set(e.payload));
  listen<string>("session-finished", () => sessionTick.update((n) => n + 1));
}
