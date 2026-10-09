# Environnement réel du cours « Docker advanced » : un vrai moteur Docker, sur Debian.
# Règles (voir catalogue/README.md §15) :
#   - image de base de la liste blanche, avec un tag explicite ;
#   - pas de sudo, pas de bit setuid, pas de VOLUME ;
#   - la DERNIÈRE instruction USER est un utilisateur non-root d'uid 1000.
# Le démon Docker n'est PAS lancé par cette image : `customizations.mentor.dockerInDocker` (devcontainer.json)
# demande à la plateforme de le démarrer dans la microVM, et l'apprenant·e y accède par le groupe `docker`.
# Le réseau n'existe QU'AU BUILD. Tout ce que les leçons téléchargeraient est donc préparé ici :
#   - les images des leçons, copiées dans un registry local (miroir de Docker Hub, en lecture seule), que
#     `docker run`, `docker pull` et `docker build` interrogent comme le vrai. Voir images.txt ;
#   - les paquets Python de la leçon 1 (Flask), dans /opt/mentor/wheels. Voir paquets-python.txt ;
#   - un second registry, en écriture, pour publier une image à la leçon 5. Voir registre.yml.
# python3 : requis par la couche VS Code web du portail. python3-pip et apache2-utils (htpasswd) ne servent
# qu'au build ; docker-buildx-plugin donne à `docker build` son affichage habituel (étapes « CACHED »).
FROM debian:trixie-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends apache2-utils bash ca-certificates curl docker-registry less nano openssh-server procps python3 python3-pip skopeo systemd-sysv tmux \
    && install -m 0755 -d /etc/apt/keyrings \
    && curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc \
    && echo "deb [signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian trixie stable" \
        > /etc/apt/sources.list.d/docker.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends containerd.io docker-buildx-plugin docker-ce docker-ce-cli docker-compose-plugin \
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

# Le paquet docker-registry installe un service qui démarrerait tout seul sur le port 5000 : il est neutralisé,
# les registries de l'environnement sont lancés par `mentor-docker`, sous le compte de l'apprenant·e.
RUN ln -s /dev/null /etc/systemd/system/docker-registry.service

# Le moteur cherche d'abord les images de Docker Hub dans le miroir local.
COPY daemon.json /etc/docker/daemon.json
COPY miroir.yml images.txt /opt/mentor/
COPY remplir-miroir.sh /opt/mentor/remplir-miroir.sh
RUN sh /opt/mentor/remplir-miroir.sh

# Les paquets Python de la leçon 1, pour l'image python:3.13-slim (CPython 3.13, Linux x86_64).
COPY paquets-python.txt /opt/mentor/paquets-python.txt
RUN pip download --no-cache-dir --no-deps --only-binary=:all: --python-version 3.13 --implementation cp \
        --platform manylinux2014_x86_64 --platform manylinux_2_28_x86_64 \
        --dest /opt/mentor/wheels -r /opt/mentor/paquets-python.txt \
    && chmod -R a+rX /opt/mentor/wheels

# Le registry de la leçon 5 : identifiant « apprenant », mot de passe « mentor » (empreinte bcrypt).
COPY registre.yml /opt/mentor/registre.yml
RUN htpasswd -Bbn apprenant mentor > /opt/mentor/registre.htpasswd \
    && chmod 0644 /opt/mentor/registre.htpasswd

COPY mentor-docker /usr/local/bin/mentor-docker
RUN chmod 0755 /usr/local/bin/mentor-docker
COPY bienvenue.sh /etc/profile.d/mentor-bienvenue.sh

# Le dossier de travail appartient à l'apprenant·e dans l'image : la plateforme n'y monte rien.
WORKDIR /workspace
RUN chown apprenant:apprenant /workspace
USER apprenant
