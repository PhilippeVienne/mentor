#!/bin/sh
# Relie les outils préinstallés (TypeScript, tsx, ESLint, Prettier, types de Node.js) au dossier courant :
# le conteneur n'a pas de réseau, donc pas de « npm install » ; un lien « node_modules » suffit.
set -e
if [ ! -e node_modules ]; then
    ln -s /opt/outils/node_modules node_modules
fi
