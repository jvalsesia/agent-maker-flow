/// <reference types="vitest/config" />
import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig(({ mode }) => {
  // Load non-prefixed env (e.g. BACKEND_URL) for Node-side config like the dev
  // proxy. Client code still calls the relative "/api/v1" path; this only sets
  // where the dev server forwards it (nginx does the same job in production).
  const env = loadEnv(mode, ".", "");
  const backendUrl = env.BACKEND_URL || "http://localhost:8080";

  return {
    plugins: [react()],
    build: {
      rollupOptions: {
        output: {
          // Split the stable, always-eager vendor libs into their own chunks so
          // app-code deploys don't bust their cache. React Flow is intentionally
          // left out — it's only imported by the lazy FlowsPage route, so it
          // already ships as that route's chunk and must not be pulled eager.
          manualChunks(id) {
            if (!id.includes("node_modules")) return;
            if (id.includes("@clerk")) return "clerk";
            if (/[\\/]node_modules[\\/](react|react-dom|react-router|react-router-dom|scheduler)[\\/]/.test(id))
              return "react-vendor";
            if (id.includes("@tanstack")) return "tanstack";
          },
        },
      },
    },
    server: {
      port: 5173,
      proxy: {
        "/api": {
          target: backendUrl,
          changeOrigin: true,
        },
      },
    },
    test: {
      globals: true,
      environment: "jsdom",
      setupFiles: ["./src/test/setup.ts"],
      css: false,
    },
  };
});
