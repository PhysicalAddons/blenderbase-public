import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const host = process.env.TAURI_DEV_HOST;
// Optional override for Vite's dependency cache location. Useful when the
// repository lives inside a synced folder (Dropbox, OneDrive), where the sync
// client can lock the cache mid-write and break the dev server.
const cacheDir = process.env.VITE_CACHE_DIR;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],
  ...(cacheDir ? { cacheDir } : {}),

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
