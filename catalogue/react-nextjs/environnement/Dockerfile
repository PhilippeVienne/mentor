# Environnement réel du cours « React et Next.js » : Node.js 24 (LTS) avec React 19, Next.js 16, TypeScript 6, tsx,
# Vitest + Testing Library + jsdom (pas de navigateur dans le conteneur : jsdom reconstruit la page en mémoire),
# Tailwind CSS 4 (CLI), Kysely, nano et un serveur SSH pour VS Code (Remote-SSH).
# Règles (vérifiées à la compilation du catalogue, voir catalogue/README.md §15) :
#   - image de base de la liste blanche, avec un tag explicite et épinglé (version de Node.js + distribution) ;
#   - pas de sudo, pas de bit setuid, pas de VOLUME ;
#   - la DERNIÈRE instruction USER est un utilisateur non-root d'uid 1000.
# Le réseau n'est disponible QU'AU BUILD : tout ce dont les leçons ont besoin est installé ici (npm install).
FROM node:24.21.0-bookworm-slim

# L'image Node.js fournit déjà l'utilisateur « node » (uid 1000) : on le remplace par « apprenant ».
# python3 est requis par le pont de VS Code dans le navigateur.
RUN apt-get update \
    && apt-get install -y --no-install-recommends bash curl less nano openssh-server procps python3 systemd-sysv \
    && rm -rf /var/lib/apt/lists/* \
    && userdel -r node \
    && useradd --create-home --uid 1000 --user-group --shell /bin/bash apprenant

# Les outils sont installés une seule fois dans /opt/outils ; chaque labo y relie un lien « node_modules » (lecture seule) avec la commande `lier-outils`.
COPY outils/ /opt/outils/
COPY --chmod=755 lier-outils.sh /usr/local/bin/lier-outils
RUN cd /opt/outils \
    && npm install --no-audit --no-fund --no-package-lock \
    && npm cache clean --force \
    && chmod -R a+rX /opt/outils \
    && chmod a+rx /opt/outils/classes-generees /opt/outils/controler /opt/outils/contient \
    && ln -s /opt/outils/classes-generees /usr/local/bin/classes-generees \
    && ln -s /opt/outils/controler /usr/local/bin/controler \
    && ln -s /opt/outils/contient /usr/local/bin/contient

# Les exercices sont lisibles par tout le monde ; chaque labo en copie un dans /workspace (qui est effacé à l'arrêt).
COPY exercices /opt/exercices
COPY bienvenue.sh /etc/profile.d/mentor-bienvenue.sh
RUN chmod -R a+rX /opt/exercices

# Hors ligne : npm ne doit pas essayer de joindre le réseau ; Next.js ne doit pas envoyer de télémétrie.
ENV PATH="/opt/outils/node_modules/.bin:${PATH}" \
    NPM_CONFIG_OFFLINE=true \
    NPM_CONFIG_UPDATE_NOTIFIER=false \
    NPM_CONFIG_FUND=false \
    NPM_CONFIG_AUDIT=false \
    NEXT_TELEMETRY_DISABLED=1 \
    NODE_NO_WARNINGS=1

# Le dossier de travail appartient à l'apprenant·e dans l'image : la plateforme n'y monte rien.
WORKDIR /workspace
RUN chown apprenant:apprenant /workspace
USER apprenant
