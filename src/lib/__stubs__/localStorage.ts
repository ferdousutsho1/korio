/**
 * In-memory localStorage polyfill for test environments.
 * Node 25 ships a partial localStorage (no .clear()) that vitest sees before
 * happy-dom can inject its own. This setup file replaces it with a complete
 * in-memory implementation that works identically to the browser API.
 */
const store = new Map<string, string>();

const localStorageMock: Storage = {
  get length() { return store.size; },
  key(index: number) { return [...store.keys()][index] ?? null; },
  getItem(key: string) { return store.get(key) ?? null; },
  setItem(key: string, value: string) { store.set(key, value); },
  removeItem(key: string) { store.delete(key); },
  clear() { store.clear(); },
};

Object.defineProperty(globalThis, "localStorage", {
  value: localStorageMock,
  writable: true,
  configurable: true,
});

// Ensure browser detection in prefs.ts sees this as a browser-like env.
// (The $app/environment stub reads `typeof window !== "undefined"`, so we
// also need a minimal `window` shim so the `browser` export becomes true.)
if (typeof window === "undefined") {
  Object.defineProperty(globalThis, "window", {
    value: globalThis,
    writable: true,
    configurable: true,
  });
}
