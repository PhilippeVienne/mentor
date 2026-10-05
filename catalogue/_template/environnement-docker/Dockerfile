# Environnement réel « Docker » (Docker-in-Docker) du parcours modèle.
#
# Il ne démarre QUE si le serveur dispose du runtime Sysbox (ENVIRONMENTS_DIND_RUNTIME=sysbox-runc) : sans lui, le
# portail refuse de le lancer (aucun repli vers --privileged). Le moteur Docker de l'apprenant·e est « rootless » :
# il tourne avec l'uid 1000, jamais en root. Voir doc/environnements.md §4.3 (recette à faire sur l'hôte cible).
FROM debian:trixie-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends bash ca-certificates curl git gnupg iproute2 procps slirp4netns tmux \
    && install -m 0755 -d /etc/apt/keyrings \
    && curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc \
    && echo "deb [signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian trixie stable" \
        > /etc/apt/sources.list.d/docker.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends docker-ce docker-ce-cli docker-ce-rootless-extras docker-compose-plugin \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 1000 --user-group --shell /bin/bash apprenant

COPY mentor-docker /usr/local/bin/mentor-docker
RUN chmod 0755 /usr/local/bin/mentor-docker

WORKDIR /workspace
USER apprenant
