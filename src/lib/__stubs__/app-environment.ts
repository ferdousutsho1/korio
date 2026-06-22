// Test stub for SvelteKit's `$app/environment` (vitest runs without the SvelteKit plugin).
// When running under jsdom (browser-like) `window` exists → treat as browser.
export const browser = typeof window !== "undefined";
export const dev = false;
export const building = false;
export const version = "test";
