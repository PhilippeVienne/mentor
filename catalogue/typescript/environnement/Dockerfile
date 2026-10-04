# Environnement réel du parcours TypeScript : Node.js 24 (LTS), TypeScript 6, tsx, ESLint + typescript-eslint, Prettier, nano
# et un serveur SSH pour VS Code (Remote-SSH).
# Règles (vérifiées à la compilation du catalogue, voir catalogue/README.md §15) :
#   - image de base de la liste blanche, avec un tag explicite et épinglé (version de Node.js + distribution) ;
#   - pas de sudo, pas de bit setuid, pas de VOLUME ;
#   - la DERNIÈRE instruction USER est un utilisateur non-root d'uid 1000.
# Le réseau n'est disponible QU'AU BUILD : tout ce dont les leçons ont besoin est installé ici (npm ci).
FROM node:24.21.0-bookworm-slim

# L'image Node.js fournit déjà l'utilisateur « node » (uid 1000) : on le remplace par « apprenant ».
# python3 est requis par le pont de VS Code dans le navigateur.
RUN apt-get update \
    && apt-get install -y --no-install-recommends bash less nano openssh-server procps python3 \
    && rm -rf /var/lib/apt/lists/* \
    && userdel -r node \
    && useradd --create-home --uid 1000 --user-group --shell /bin/bash apprenant

# Les outils sont installés une seule fois dans /opt/outils ; chaque labo y relie un lien « node_modules » avec la commande `lier-outils` (lecture seule).
COPY outils/package.json /opt/outils/package.json
RUN cd /opt/outils \
    && npm install --no-audit --no-fund --no-package-lock --ignore-scripts \
    && npm cache clean --force \
    && chmod -R a+rX /opt/outils

# Les exercices sont lisibles par tout le monde ; chaque labo en copie un dans /workspace (qui est effacé à l'arrêt).
COPY exercices /opt/exercices
COPY bienvenue.sh /etc/profile.d/mentor-bienvenue.sh
COPY --chmod=755 lier-outils.sh /usr/local/bin/lier-outils

# Outil de vérification des labos (`verifier-ts`) et ses contrôles d'origine : en lecture seule pour l'apprenant·e.
# Il juge une COPIE du travail où les fichiers d'origine (/opt/exercices) sont remis en place et où `strict` est imposé.
COPY verif /opt/verif
COPY --chmod=755 verifier-ts /usr/local/bin/verifier-ts
RUN chmod -R a+rX /opt/verif /opt/exercices

# Hors ligne : npm ne doit pas essayer de joindre le réseau (mises à jour, audit, publicité).
ENV PATH="/opt/outils/node_modules/.bin:${PATH}" \
    NPM_CONFIG_OFFLINE=true \
    NPM_CONFIG_UPDATE_NOTIFIER=false \
    NPM_CONFIG_FUND=false \
    NPM_CONFIG_AUDIT=false \
    NODE_NO_WARNINGS=1

WORKDIR /workspace
USER apprenant
