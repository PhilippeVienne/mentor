# Environnement réel du parcours « Maintenir du code hérité » : Python 3.10 avec QUATRE versions de Django
# installées côte à côte, chacune dans son environnement virtuel (3.1, 3.2 LTS, 4.2 LTS, 5.2 LTS).
# Python 3.10 est la seule version qui les accepte toutes : Django 3.1 va jusqu'à 3.9 (3.10 fonctionne en pratique),
# 3.2 jusqu'à 3.10, 4.2 jusqu'à 3.12 et 5.2 demande au moins 3.10.
# Règles (vérifiées à la compilation du catalogue, voir catalogue/README.md §15) :
#   - image de base de la liste blanche, avec un tag explicite et épinglé ;
#   - pas de sudo, pas de bit setuid, pas de VOLUME ;
#   - la DERNIÈRE instruction USER est un utilisateur non-root d'uid 1000.
# Le réseau n'est disponible QU'AU BUILD : tout est installé ici. Il n'y a ni Node.js ni npm : la partie React/Vite
# du parcours se travaille sur des fichiers fournis (package.json, vite.config.js…), sans lancer de build.
FROM python:3.10.22-slim-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends bash git less nano openssh-server procps tmux \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 1000 --user-group --shell /bin/bash apprenant \
    && pip install --no-cache-dir pytest==9.1.1 coverage==7.16.2

# Un environnement virtuel par palier de Django, chacun avec coverage.
RUN set -e; \
    for v in 3.1.14:django31 3.2.25:django32 4.2.30:django42 5.2.17:django52; do \
        version="${v%%:*}"; nom="${v##*:}"; \
        python -m venv "/opt/venvs/$nom"; \
        "/opt/venvs/$nom/bin/pip" install --no-cache-dir "Django==$version" coverage==7.16.2; \
    done

# Les exercices sont lisibles par tout le monde ; chaque labo en copie un dans /workspace (qui est effacé à l'arrêt).
COPY exercices /opt/exercices
# Outils de vérification des labos (lecture seule, hors de /workspace) : ils analysent le travail de l'apprenant·e
# (AST, mutations, rejeu des tests d'origine) au lieu de chercher un mot dans ses fichiers.
COPY outils /opt/outils
RUN chmod 755 /opt/outils/* && chmod a-w /opt/outils
COPY bienvenue.sh /etc/profile.d/mentor-bienvenue.sh

WORKDIR /workspace
USER apprenant
