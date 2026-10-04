# Environnement réel du parcours « HTML, CSS et JavaScript » : Node.js 24 (LTS), jsdom, nano.
# Pas de navigateur dans le conteneur : jsdom reconstruit le DOM d'une page (HTML + CSS + JavaScript) en mémoire,
# et l'outil `verifier-page` (dossier outils/) en donne un diagnostic. Les exercices se testent avec `node --test`.
# Règles (vérifiées à la compilation du catalogue, voir catalogue/README.md §15) :
#   - image de base de la liste blanche, avec un tag épinglé ;
#   - pas de sudo, pas de bit setuid, pas de VOLUME ;
#   - la DERNIÈRE instruction USER est un utilisateur non-root d'uid 1000 (l'utilisateur `node` de l'image).
# Le réseau n'est disponible QU'AU BUILD : tout ce dont les leçons ont besoin (jsdom) est installé ici.
FROM node:24.18.1-bookworm-slim

# python3 : requis par le pont de VS Code dans le navigateur. openssh-server : VS Code (Remote-SSH).
RUN apt-get update \
    && apt-get install -y --no-install-recommends bash less nano openssh-server procps python3 tmux \
    && rm -rf /var/lib/apt/lists/*

# Outils de vérification (en lecture seule pour tout le monde) : jsdom et acorn (analyseur JavaScript) sont installés
# dans /opt/outils/node_modules. `verifier-web` juge une COPIE du travail où les fichiers d'origine (/opt/exercices)
# sont remis en place (voir verifier-web.js) ; ses contrôles d'origine sont dans /opt/outils/controles.
COPY outils /opt/outils
RUN cd /opt/outils \
    && npm install --no-audit --no-fund --no-package-lock jsdom@26.1.0 acorn@8.16.0 \
    && chmod -R a+rX /opt/outils \
    && chmod a+rx /opt/outils/verifier-page /opt/outils/servir /opt/outils/verifier-web \
    && ln -s /opt/outils/verifier-page /usr/local/bin/verifier-page \
    && ln -s /opt/outils/verifier-web /usr/local/bin/verifier-web \
    && ln -s /opt/outils/servir /usr/local/bin/servir \
    && npm cache clean --force

# Les exercices et le petit paquet local de la dernière leçon : lisibles par tout le monde.
# Chaque labo copie ce dont il a besoin dans /workspace (qui est effacé à l'arrêt).
COPY exercices /opt/exercices
COPY paquets /opt/paquets
COPY bienvenue.sh /etc/profile.d/mentor-bienvenue.sh
RUN chmod -R a+rX /opt/exercices /opt/paquets

WORKDIR /workspace
USER node
