#!/bin/sh
# Relie les outils préinstallés (React, Next.js, TypeScript, Vitest, Testing Library, Tailwind CSS, Kysely) au dossier courant :
# le conteneur n'a pas de réseau, donc pas de « npm install » ; un lien « node_modules » (en lecture seule) suffit.
set -e
if [ ! -e node_modules ]; then
    ln -s /opt/outils/node_modules node_modules
fi
