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
    && apt-get install -y --no-install-recommends bash ca-certificates curl docker-registry less nano openssh-server procps python3 skopeo systemd-sysv tmux \
    && install -m 0755 -d /etc/apt/keyrings \
    && curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc \
    && echo "deb [signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian trixie stable" \
        > /etc/apt/sources.list.d/docker.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends containerd.io docker-ce docker-ce-cli docker-compose-plugin \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 1000 --user-group --groups docker --shell /bin/bash apprenant \
    && groupadd --non-unique --gid 1000 vscode \
    && useradd --non-unique --uid 1000 --gid 1000 --groups docker --no-create-home --home-dir /home/apprenant --shell /bin/bash vscode

# `vscode` est un alias de l'apprenant·e (même uid, même dossier personnel) : le plan d'exécution ouvre ses
# sessions sous ce nom, et sans cet alias elles n'auraient pas le groupe `docker`.
# Le noyau des microVM n'a ni nf_tables ni la table iptables `raw` : Docker utilise les iptables historiques et
# renonce aux règles qui demandent cette table.
RUN update-alternatives --set iptables /usr/sbin/iptables-legacy \
    && update-alternatives --set ip6tables /usr/sbin/ip6tables-legacy \
    && mkdir -p /etc/systemd/system/docker.service.d \
    && printf '[Service]\nEnvironment=DOCKER_INSECURE_NO_IPTABLES_RAW=1\n' > /etc/systemd/system/docker.service.d/mentor.conf

# Le moteur cherche d'abord les images de Docker Hub dans le miroir local.
COPY daemon.json /etc/docker/daemon.json
COPY miroir.yml images.txt /opt/mentor/
COPY remplir-miroir.sh /opt/mentor/remplir-miroir.sh
RUN sh /opt/mentor/remplir-miroir.sh

COPY mentor-docker /usr/local/bin/mentor-docker
RUN chmod 0755 /usr/local/bin/mentor-docker

# Le dossier de travail appartient à l'apprenant·e dans l'image : la plateforme n'y monte rien.
WORKDIR /workspace
RUN chown apprenant:apprenant /workspace
USER apprenant
