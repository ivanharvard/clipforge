import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  base: "./",
  plugins: [react()],
  publicDir: "../crates/clipforge-app/icons/src",
  build: {
    outDir: "../dist/web",
    emptyOutDir: true,
  },
  test: {
    setupFiles: ["./vitest.setup.ts"],
  },
});
