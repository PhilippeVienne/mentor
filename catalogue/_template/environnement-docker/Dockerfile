# Environnement « Docker » du parcours modèle : un vrai moteur Docker, sur Debian.
# Règles (voir catalogue/README.md, « Environments ») :
#   - image de base avec un tag explicite ;
#   - pas de sudo, pas de bit setuid, pas de VOLUME ;
#   - la DERNIÈRE instruction USER est un utilisateur non-root d'uid 1000.
# Le démon Docker n'est PAS lancé par cette image : `customizations.mentor.dockerInDocker` (devcontainer.json)
# demande à la plateforme de le démarrer dans la microVM, et l'apprenant·e y accède par le groupe `docker`.
# Le réseau n'existe QU'AU BUILD : les images des leçons sont copiées ici dans un registry local (miroir de
# Docker Hub, en lecture seule), que `docker run` et `docker pull` interrogent comme le vrai. Voir images.txt.
# openssh-server et python3 : pour VS Code (Remote-SSH et VS Code dans le navigateur).
FROM debian:trixie-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends bash ca-certificates curl docker-registry less nano openssh-server procps python3 skopeo tmux \
    && install -m 0755 -d /etc/apt/keyrings \
    && curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc \
    && echo "deb [signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian trixie stable" \
        > /etc/apt/sources.list.d/docker.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends containerd.io docker-ce docker-ce-cli docker-compose-plugin \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 1000 --user-group --groups docker --shell /bin/bash apprenant

# Le moteur cherche d'abord les images de Docker Hub dans le miroir local.
COPY daemon.json /etc/docker/daemon.json
COPY miroir.yml images.txt /opt/mentor/
COPY remplir-miroir.sh /opt/mentor/remplir-miroir.sh
RUN sh /opt/mentor/remplir-miroir.sh

COPY mentor-docker /usr/local/bin/mentor-docker
RUN chmod 0755 /usr/local/bin/mentor-docker

WORKDIR /workspace
USER apprenant
