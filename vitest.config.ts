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
  },
});
