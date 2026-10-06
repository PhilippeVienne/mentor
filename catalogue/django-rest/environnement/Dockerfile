# Environnement réel du parcours « API REST avec Django REST framework » : Python 3.13, Django, DRF, django-filter,
# pytest-django, PyJWT et cryptography, et un serveur SSH pour VS Code (Remote-SSH).
# Règles (vérifiées à la compilation du catalogue, voir catalogue/README.md §15) :
#   - image de base de la liste blanche, avec un tag explicite et épinglé (version de Python + distribution) ;
#   - pas de sudo, pas de bit setuid, pas de VOLUME ;
#   - la DERNIÈRE instruction USER est un utilisateur non-root d'uid 1000.
# Le réseau n'est disponible QU'AU BUILD : tout ce dont les leçons ont besoin est installé ici (versions épinglées).
FROM python:3.13.7-slim-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends bash curl less nano openssh-server procps systemd-sysv tmux \
    && rm -rf /var/lib/apt/lists/* \
    && pip install --no-cache-dir \
        Django==6.1.1 \
        djangorestframework==3.18.1 \
        django-filter==26.1 \
        pytest==9.1.1 \
        pytest-django==4.14.0 \
        PyJWT==2.15.1 \
        cryptography==50.0.2 \
        PyYAML==6.0.3 \
        uritemplate==4.2.0 \
        inflection==0.5.1 \
        tzdata==2026.2 \
    && useradd --create-home --uid 1000 --user-group --shell /bin/bash apprenant

# Les exercices sont lisibles par tout le monde ; chaque labo en copie un dans /workspace (qui est effacé à l'arrêt).
COPY exercices /opt/exercices
# Outils de vérification des labos (lecture seule, hors de /workspace) : ils rejouent les tests d'origine sur le travail
# de l'apprenant·e au lieu de faire confiance à ses fichiers de test.
COPY outils /opt/outils
RUN chmod 755 /opt/outils/* && chmod a-w /opt/outils
COPY bienvenue.sh /etc/profile.d/mentor-bienvenue.sh

# Le dossier de travail appartient à l'apprenant·e dans l'image : la plateforme n'y monte rien.
WORKDIR /workspace
RUN chown apprenant:apprenant /workspace
USER apprenant
