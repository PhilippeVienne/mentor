// Configuration de Vitest pour les exercices : les tests tournent dans jsdom (un faux navigateur en mémoire).
import os from 'node:os';
import path from 'node:path';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  // Le dossier d'outils est en lecture seule : le cache de Vite va dans le dossier temporaire.
  cacheDir: path.join(os.tmpdir(), 'vite-demo'),
  test: {
    environment: 'jsdom',
    globals: true,
    include: ['**/*.spec.js'],
    exclude: ['**/node_modules/**'],
    setupFiles: ['/opt/angular/vitest.setup.mjs'],
    reporters: ['verbose'],
    pool: 'forks',
    poolOptions: { forks: { singleFork: true } },
  },
});
