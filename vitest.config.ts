import { defineConfig } from "vitest/config";
import { fileURLToPath } from "node:url";

export default defineConfig({
  resolve: {
    alias: {
      // SvelteKit's $app/environment isn't available under plain vitest; stub it.
      "$app/environment": fileURLToPath(new URL("./src/lib/__stubs__/app-environment.ts", import.meta.url)),
      // SvelteKit's $lib alias isn't wired under plain vitest; map it to src/lib.
      "$lib": fileURLToPath(new URL("./src/lib", import.meta.url)),
    },
  },
  test: {
    include: ["src/**/*.{test,spec}.ts"],
    environment: "node",
    environmentMatchGlobs: [
      // Tests that use localStorage need a browser-like environment.
      ["src/lib/prefs.test.ts", "happy-dom"],
    ],
    // Polyfill Node 25's partial localStorage (no .clear()) with a full
    // in-memory implementation so localStorage-using tests work in node env.
    setupFiles: [fileURLToPath(new URL("./src/lib/__stubs__/localStorage.ts", import.meta.url))],
  },
});
