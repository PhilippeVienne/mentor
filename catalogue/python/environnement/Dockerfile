# Environnement réel du parcours Python : Python 3.13, pytest, nano et un serveur SSH pour VS Code (Remote-SSH).
# Règles (vérifiées à la compilation du catalogue, voir catalogue/README.md §15) :
#   - image de base de la liste blanche, avec un tag explicite et épinglé (version de Python + distribution) ;
#   - pas de sudo, pas de bit setuid, pas de VOLUME ;
#   - la DERNIÈRE instruction USER est un utilisateur non-root d'uid 1000.
# Le réseau n'est disponible QU'AU BUILD : tout ce dont les leçons ont besoin est installé ici (pytest).
FROM python:3.13.7-slim-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends bash less nano openssh-server procps tmux \
    && rm -rf /var/lib/apt/lists/* \
    && pip install --no-cache-dir pytest==8.3.5 \
    && useradd --create-home --uid 1000 --user-group --shell /bin/bash apprenant

# Les exercices sont lisibles par tout le monde ; chaque labo en copie un dans /workspace (qui est effacé à l'arrêt).
COPY exercices /opt/exercices
COPY bienvenue.sh /etc/profile.d/mentor-bienvenue.sh

WORKDIR /workspace
USER apprenant
