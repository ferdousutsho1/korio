import { writable } from "svelte/store";
import { browser } from "$app/environment";

export type ThemeMode = "light" | "dark";
const STORAGE_KEY = "korio.theme";

function initial(): ThemeMode {
  if (!browser) return "light";
  return localStorage.getItem(STORAGE_KEY) === "dark" ? "dark" : "light";
}

export const theme = writable<ThemeMode>(initial());

export function applyTheme(mode: ThemeMode) {
  if (!browser) return;
  document.documentElement.setAttribute("data-theme", mode);
  localStorage.setItem(STORAGE_KEY, mode);
}

export function toggleTheme() {
  theme.update((m) => {
    const next: ThemeMode = m === "light" ? "dark" : "light";
    applyTheme(next);
    return next;
  });
}

/** Call once from the root layout's onMount (browser only). */
export function initTheme() {
  const mode = initial();
  theme.set(mode);
  applyTheme(mode);
}
