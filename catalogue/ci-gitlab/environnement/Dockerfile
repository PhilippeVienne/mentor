# Environnement réel du parcours « CI/CD avec GitLab » : Python 3.13 et deux petits outils maison, hors ligne,
# `verifier-ci` (structure d'un .gitlab-ci.yml) et `verifier-renovate` (structure d'un renovate.json), plus yamllint.
# Il n'y a PAS de runner GitLab dans le conteneur (il faudrait un serveur GitLab et un démon Docker) : ces outils
# contrôlent la structure des fichiers, ils n'exécutent aucun job.
# Règles (vérifiées à la compilation du catalogue, voir catalogue/README.md §15) :
#   - image de base de la liste blanche, avec un tag explicite et épinglé ;
#   - pas de sudo, pas de bit setuid, pas de VOLUME ;
#   - la DERNIÈRE instruction USER est un utilisateur non-root d'uid 1000.
# Le réseau n'est disponible QU'AU BUILD : tout ce dont les leçons ont besoin est installé ici (PyYAML, yamllint).
FROM python:3.13.7-slim-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends bash curl git jq less nano openssh-server procps systemd-sysv \
    && rm -rf /var/lib/apt/lists/* \
    && pip install --no-cache-dir PyYAML==6.0.2 yamllint==1.35.1 \
    && useradd --create-home --uid 1000 --user-group --shell /bin/bash apprenant

# Les outils et les exercices sont lisibles par tout le monde ; chaque labo copie son exercice dans /workspace
# (qui est effacé à l'arrêt). Les outils sont dans le PATH de l'image ET liés dans /usr/local/bin, car les commandes
# de vérification du serveur ne reçoivent qu'un PATH système fixe.
COPY outils /opt/outils
COPY exercices /opt/exercices
COPY bienvenue.sh /etc/profile.d/mentor-bienvenue.sh
RUN chmod 755 /opt/outils/verifier-ci /opt/outils/verifier-renovate /opt/outils/lint-yaml \
    && ln -s /opt/outils/verifier-ci /usr/local/bin/verifier-ci \
    && ln -s /opt/outils/verifier-renovate /usr/local/bin/verifier-renovate \
    && ln -s /opt/outils/lint-yaml /usr/local/bin/lint-yaml
ENV PATH="/opt/outils:${PATH}"

# Le dossier de travail appartient à l'apprenant·e dans l'image : la plateforme n'y monte rien.
WORKDIR /workspace
RUN chown apprenant:apprenant /workspace
USER apprenant
