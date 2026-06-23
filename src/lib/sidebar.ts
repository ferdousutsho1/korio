import { writable } from "svelte/store";
import { browser } from "$app/environment";

const KEY = "korio.sidebar.hidden";
/** Nav ids the user can never hide. */
export const ALWAYS = ["settings"];
/** All hideable nav ids. */
export const HIDEABLE = ["dashboard", "stats", "watchlist", "sites", "tools", "tasks", "notes", "goals"];

function load(): string[] {
  if (!browser) return [];
  try {
    const v = JSON.parse(localStorage.getItem(KEY) || "[]");
    return Array.isArray(v) ? v.filter((x) => typeof x === "string" && !ALWAYS.includes(x)) : [];
  } catch {
    return [];
  }
}

export const hiddenSections = writable<string[]>(load());

export function toggleSection(id: string) {
  if (ALWAYS.includes(id)) return;
  hiddenSections.update((cur) => {
    const next = cur.includes(id) ? cur.filter((x) => x !== id) : [...cur, id];
    if (browser) localStorage.setItem(KEY, JSON.stringify(next));
    return next;
  });
}

export function initSidebar() {
  hiddenSections.set(load());
}

export function hideAllButSettings() {
  if (browser) localStorage.setItem(KEY, JSON.stringify(HIDEABLE));
  hiddenSections.set([...HIDEABLE]);
}

export function showAll() {
  if (browser) localStorage.setItem(KEY, JSON.stringify([]));
  hiddenSections.set([]);
}
