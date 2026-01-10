import { defineConfig } from "vitest/config";
import { resolve } from "path";

export default defineConfig({
  test: {
    include: ["src/test/**/*.test.ts"],
    browser: {
      enabled: true,
      name: "chromium",
      provider: "playwright",
      headless: true,
    },
  },
  optimizeDeps: {
    exclude: ["@bokuweb/zstd-wasm"],
  },
  build: {
    rollupOptions: {
      external: ["@bokuweb/zstd-wasm"],
    },
  },
  server: {
    fs: {
      // Allow serving files from node_modules
      allow: [".."],
    },
  },
  resolve: {
    alias: {
      // Use the web build for browser tests
      "@bokuweb/zstd-wasm": resolve(
        __dirname,
        "node_modules/@bokuweb/zstd-wasm/dist/web/index.web.js"
      ),
    },
  },
});
