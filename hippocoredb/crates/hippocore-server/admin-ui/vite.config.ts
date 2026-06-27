import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "node:path";

// The Rust server embeds the build output with `include_str!` from
// `crates/hippocore-server/src/console/`, so we emit flat, non-hashed
// filenames there instead of the default hashed `dist/assets/*`.
//
// `base: "/console/"` makes the generated index.html reference the assets
// at the same path the server serves them from.
export default defineConfig({
  base: "/console/",
  plugins: [react()],
  build: {
    outDir: resolve(__dirname, "../src/console"),
    emptyOutDir: true,
    assetsDir: ".",
    cssCodeSplit: false,
    rollupOptions: {
      output: {
        entryFileNames: "index.js",
        chunkFileNames: "index.js",
        assetFileNames: "index.[ext]",
        manualChunks: undefined,
      },
    },
  },
});
