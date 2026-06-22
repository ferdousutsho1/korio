import { browser } from "$app/environment";

/** Read a JSON value from localStorage, or `fallback` if missing/corrupt/SSR. */
export function readJSON<T>(key: string, fallback: T): T {
  if (!browser) return fallback;
  try {
    const raw = localStorage.getItem(key);
    return raw == null ? fallback : (JSON.parse(raw) as T);
  } catch {
    return fallback;
  }
}

/** Write a JSON value to localStorage (no-op on SSR or quota error). */
export function writeJSON(key: string, value: unknown): void {
  if (!browser) return;
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    /* ignore */
  }
}
