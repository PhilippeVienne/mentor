# Environnement réel des parcours AWS : l'AWS CLI officielle et MiniStack, un émulateur LOCAL et libre d'AWS
# (https://github.com/ministackorg/ministack, licence MIT, pur Python), qui répond sur http://127.0.0.1:4566.
#   - aucun compte AWS, aucun accès à Internet, rien de facturé : tout ce que crée l'apprenant·e vit dans la mémoire
#     de l'émulateur et disparaît à l'arrêt de l'environnement ;
#   - MiniStack tourne SANS root, comme l'apprenant·e (uid 1000), lancé par « demarrer-aws » (postStartCommand) ;
#   - il est lancé avec AUTH=true : les politiques IAM sont réellement évaluées (un compte limité reçoit AccessDenied) ;
#   - les fonctions Lambda Python s'exécutent dans un sous-processus local (pas de Docker dans cet environnement).
# Règles (voir catalogue/README.md, « Environments ») :
#   - image de base de la liste blanche, avec un tag explicite ;
#   - pas de sudo, pas de bit setuid, pas de VOLUME ;
#   - la DERNIÈRE instruction USER est un utilisateur non-root d'uid 1000.
# Le réseau n'existe QU'AU BUILD : tout est installé ici. Au démarrage, le dossier personnel et le dossier de travail
# sont vides : ce dont un labo a besoin y est créé par ses propres commandes.
# python3 : requis par MiniStack, par les fonctions Lambda des labos et par la couche VS Code web du portail.
FROM debian:trixie-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends awscli bash ca-certificates curl jq less nano openssh-server procps systemd-sysv \
        python3 python3-venv tzdata unzip zip \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 1000 --user-group --shell /bin/bash apprenant \
    && /usr/bin/aws --version | grep -q '^aws-cli/2\.'

# MiniStack et boto3 dans un environnement Python isolé. Les versions ET les sommes SHA-256 de chaque paquet sont
# épinglées dans requirements.txt : le build échoue si un fichier téléchargé ne correspond pas.
COPY requirements.txt /opt/ministack/requirements.txt
RUN python3 -m venv /opt/ministack/venv \
    && /opt/ministack/venv/bin/pip install --no-cache-dir --disable-pip-version-check --require-hashes \
        --only-binary=:all: -r /opt/ministack/requirements.txt \
    && /opt/ministack/venv/bin/python -c "import ministack, boto3"

# Outils du labo. Le « lanceur » est appelé sous les noms aws et python3 (liens symboliques de /usr/local/bin, seul
# dossier garanti dans le PATH des vérifications du serveur) : il règle le point d'accès local, la région et les
# identifiants factices, puis lance le vrai programme. Les vérifications ne reçoivent que PATH, HOME, USER et LANG.
COPY outils /opt/outils
RUN chmod 755 /opt/outils/lanceur /opt/outils/demarrer-aws \
    && ln -s /opt/outils/lanceur /usr/local/bin/aws \
    && ln -s /opt/outils/lanceur /usr/local/bin/python3 \
    && ln -s /opt/outils/demarrer-aws /usr/local/bin/demarrer-aws \
    && echo '. /opt/outils/mentor-aws.sh' >> /etc/bash.bashrc
COPY bienvenue.sh /etc/profile.d/mentor-bienvenue.sh

# Le dossier de travail appartient à l'apprenant·e dans l'image : la plateforme n'y monte rien.
WORKDIR /workspace
RUN chown apprenant:apprenant /workspace
USER apprenant
