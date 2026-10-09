# Environnement réel du cours « Angular » : Angular 20 (core, compiler-cli, forms, router, Material, CDK), TypeScript,
# keycloak-angular / keycloak-js, Vitest + jsdom. Node.js 24 (LTS) sur Debian, nano et un serveur SSH pour VS Code.
# Pas de navigateur dans le conteneur : `ngc` (le compilateur d'Angular, mode strict) vérifie les gabarits et la commande
# `tester` joue les tests *.spec.ts avec Vitest dans jsdom (voir outils/tester).
# Règles (vérifiées à la compilation du catalogue, voir catalogue/README.md §15) :
#   - image de base de la liste blanche, avec un tag épinglé ;
#   - pas de sudo, pas de bit setuid, pas de VOLUME ;
#   - la DERNIÈRE instruction USER est un utilisateur non-root d'uid 1000 (l'utilisateur `node` de l'image).
# Le réseau n'est disponible QU'AU BUILD : tout ce dont les leçons ont besoin (Angular, node_modules) est installé ici.
FROM node:24.18.1-bookworm-slim

# python3 : requis par le pont de VS Code dans le navigateur. openssh-server : VS Code (Remote-SSH).
RUN apt-get update \
    && apt-get install -y --no-install-recommends bash curl less nano openssh-server procps python3 systemd-sysv tmux \
    && rm -rf /var/lib/apt/lists/*

# Les outils : node_modules (275 Mo) est installé une fois ici, en lecture seule. Chaque labo crée dans son projet un lien
# symbolique `node_modules -> /opt/angular/node_modules` avec la commande `preparer` (le dossier de travail est en mémoire :
# on n'y copie pas 275 Mo).
COPY outils /opt/angular
RUN cd /opt/angular \
    && npm install --no-audit --no-fund --no-package-lock \
    && npm cache clean --force \
    && chmod a+rx /opt/angular/tester /opt/angular/ng /opt/angular/preparer /opt/angular/contient /opt/angular/controler-ecrit /opt/angular/verifier-start \
    && ln -s /opt/angular/tester /usr/local/bin/tester \
    && ln -s /opt/angular/contient /usr/local/bin/contient \
    && ln -s /opt/angular/controler-ecrit /usr/local/bin/controler-ecrit \
    && ln -s /opt/angular/verifier-start /usr/local/bin/verifier-start \
    && ln -s /opt/angular/ng /usr/local/bin/ng \
    && ln -s /opt/angular/node_modules/.bin/ngc /usr/local/bin/ngc \
    && ln -s /opt/angular/node_modules/.bin/tsc /usr/local/bin/tsc

# Les projets d'exercice (un par leçon), lisibles par tout le monde : chaque labo en copie un dans /workspace.
COPY exercices /opt/exercices

# Leçon 7 : on construit UNE FOIS la version de production du petit projet (réseau et mémoire du build), pour que le labo
# travaille sur de vrais fichiers `dist/` sans relancer `ng build` dans le conteneur (512 Mo ne suffisent pas).
RUN cd /opt/exercices/07-tests-et-build \
    && ln -s /opt/angular/node_modules node_modules \
    && NG_CLI_ANALYTICS=false /opt/angular/node_modules/.bin/ng build --configuration production \
    && rm -rf node_modules .angular

COPY bienvenue.sh /etc/profile.d/mentor-bienvenue.sh
RUN chmod -R a+rX /opt/angular /opt/exercices

# Le dossier de travail appartient à l'apprenant·e dans l'image : la plateforme n'y monte rien.
WORKDIR /workspace
RUN chown node:node /workspace
USER node
