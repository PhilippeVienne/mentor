import { defineConfig } from "vitest/config";

// Réglages de Vitest pour le labo : jsdom joue le rôle du navigateur, et le cache va dans /tmp
// (le dossier node_modules est en lecture seule).
export default defineConfig({
  cacheDir: "/tmp/vitest-cache",
  resolve: { alias: { "@": new URL("./src", import.meta.url).pathname } },
  oxc: { jsx: { runtime: "automatic" } },
  test: {
    environment: "jsdom",
    setupFiles: ["./vitest.setup.ts"],
    include: ["**/*.test.{ts,tsx}"],
    exclude: ["node_modules/**", ".next/**"],
  },
});
