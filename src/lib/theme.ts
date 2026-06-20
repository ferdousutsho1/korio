import { writable } from "svelte/store";
import { browser } from "$app/environment";

export type ThemeMode = "light" | "dark";
export type Tint = "paper" | "slate" | "sand" | "mono";
export interface Appearance { mode: ThemeMode; accent: string | null; tint: Tint; }

const KEY = "korio.appearance";
const DEFAULT: Appearance = { mode: "light", accent: null, tint: "paper" };

function load(): Appearance {
  if (!browser) return DEFAULT;
  try {
    const raw = JSON.parse(localStorage.getItem(KEY) || "{}");
    return {
      mode: raw.mode === "dark" ? "dark" : "light",
      accent: typeof raw.accent === "string" ? raw.accent : null,
      tint: ["paper", "slate", "sand", "mono"].includes(raw.tint) ? raw.tint : "paper",
    };
  } catch {
    return DEFAULT;
  }
}

export const appearance = writable<Appearance>(load());
/** Backward-compatible mode store (topbar toggle uses `$theme`). */
export const theme = writable<ThemeMode>(load().mode);

/** Relative-luminance pick of readable text for a hex accent. */
export function contrastColor(hex: string): string {
  const c = hex.replace("#", "");
  const n = c.length === 3 ? c.split("").map((x) => x + x).join("") : c;
  const ch = (i: number) => parseInt(n.substring(i, i + 2), 16) / 255;
  const lin = (v: number) => (v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4));
  const L = 0.2126 * lin(ch(0)) + 0.7152 * lin(ch(2)) + 0.0722 * lin(ch(4));
  return L > 0.179 ? "#1A1714" : "#ffffff";
}

export function applyAppearance(a: Appearance) {
  if (!browser) return;
  const el = document.documentElement;
  el.setAttribute("data-theme", a.mode);
  el.setAttribute("data-tint", a.tint);
  if (a.accent) {
    el.style.setProperty("--accent", a.accent);
    el.style.setProperty("--accent-contrast", contrastColor(a.accent));
  } else {
    el.style.removeProperty("--accent");
    el.style.removeProperty("--accent-contrast");
  }
  localStorage.setItem(KEY, JSON.stringify(a));
}

function update(partial: Partial<Appearance>) {
  appearance.update((cur) => {
    const next = { ...cur, ...partial };
    applyAppearance(next);
    theme.set(next.mode);
    return next;
  });
}

export const setMode = (mode: ThemeMode) => update({ mode });
export const setAccent = (accent: string | null) => update({ accent });
export const setTint = (tint: Tint) => update({ tint });

export function toggleTheme() {
  appearance.update((cur) => {
    const next: Appearance = { ...cur, mode: cur.mode === "light" ? "dark" : "light" };
    applyAppearance(next);
    theme.set(next.mode);
    return next;
  });
}

/** Call once from the root layout's onMount (browser only). */
export function initTheme() {
  const a = load();
  appearance.set(a);
  theme.set(a.mode);
  applyAppearance(a);
}
