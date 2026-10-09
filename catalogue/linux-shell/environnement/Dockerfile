# Environnement réel du cours « Linux et shell » : un vrai Debian avec les outils des leçons
# (coreutils, grep, find, less, nano, client SSH, ps/kill) et des fichiers d'entraînement dans /opt/exercices.
# Règles (vérifiées à la compilation du catalogue, voir catalogue/README.md §15) :
#   - image de base de la liste blanche, avec un tag explicite ;
#   - pas de sudo, pas de bit setuid, pas de VOLUME ;
#   - la DERNIÈRE instruction USER est un utilisateur non-root d'uid 1000.
# Le réseau n'existe QU'AU BUILD : tout ce dont les leçons ont besoin est installé ici.
# python3 : requis par la couche VS Code web du portail.
FROM debian:trixie-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends bash coreutils findutils grep less nano \
       openssh-client curl openssh-server systemd-sysv procps python3 tree \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 1000 --user-group --shell /bin/bash apprenant

# Les exercices sont lisibles par tout le monde ; chaque labo en copie un dans /workspace (effacé à l'arrêt).
COPY exercices /opt/exercices
COPY bienvenue.sh /etc/profile.d/mentor-bienvenue.sh

# Le dossier de travail appartient à l'apprenant·e dans l'image : la plateforme n'y monte rien.
WORKDIR /workspace
RUN chown apprenant:apprenant /workspace
USER apprenant
