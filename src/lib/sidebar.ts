import { writable } from "svelte/store";
import { browser } from "$app/environment";

const KEY = "korio.sidebar.hidden";
/** Nav ids the user can never hide. */
export const ALWAYS = ["dashboard", "settings"];

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
