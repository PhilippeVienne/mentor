# Environnement du parcours modèle : Debian, Git, Python et un serveur SSH pour VS Code (Remote-SSH).
# Le réseau n'existe QU'AU BUILD : tout ce dont les leçons ont besoin est installé ici.
# Règles (voir catalogue/README.md, « Environments ») :
#   - image de base de la liste blanche, avec un tag explicite ;
#   - pas de sudo, pas de bit setuid, pas de VOLUME ;
#   - la DERNIÈRE instruction USER est un utilisateur non-root d'uid 1000 ;
#   - pour VS Code dans le navigateur : python3 présent et une base glibc (sinon `vscodeWeb: false`).
# Le portail ajoute ensuite sa propre couche de durcissement (retrait des bits setuid/setgid, suppression de su).
FROM debian:trixie-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends bash ca-certificates git less nano openssh-server procps python3 tmux \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 1000 --user-group --shell /bin/bash apprenant

# Le dossier personnel est un volume propre à la session : son contenu initial est celui de l'image (/etc/skel
# est copié à la création de l'utilisateur).
COPY bienvenue.sh /etc/profile.d/mentor-bienvenue.sh

WORKDIR /workspace
USER apprenant
