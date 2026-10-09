# Environnement réel du cours « Git basics » : un vrai Debian avec Git, nano et tmux, et un « GitLab » local.
# Règles (vérifiées à la compilation du catalogue, voir catalogue/README.md §15) :
#   - image de base de la liste blanche, avec un tag explicite ;
#   - pas de sudo, pas de bit setuid, pas de VOLUME ;
#   - la DERNIÈRE instruction USER est un utilisateur non-root d'uid 1000.
# Le réseau n'existe QU'AU BUILD : il n'y a donc pas de vrai GitLab. L'URL `git@gitlab.example.org:equipe/…` est redirigée
# (url.<base>.insteadOf) vers un dépôt nu local, créé par les labos dans ~/.gitlab-sim/ : les commandes sont les vraies.
# python3 : requis par la couche VS Code web du portail.
FROM debian:trixie-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends bash ca-certificates curl git less nano openssh-server procps python3 systemd-sysv tmux \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 1000 --user-group --shell /bin/bash apprenant

# Identité par défaut (niveau système) pour les commits de mise en place des labos ; la leçon 1 apprend à définir la
# sienne avec `git config --global` (qui prend le dessus). Branche par défaut `main`, « GitLab » local redirigé.
RUN git config --system user.name "Apprenant·e Mentor" \
    && git config --system user.email "apprenant@mentor" \
    && git config --system init.defaultBranch main \
    && git config --system url."/home/apprenant/.gitlab-sim/".insteadOf "git@gitlab.example.org:equipe/"

# Simulations des coéquipières et de la Merge Request (le serveur local n'a personne d'autre pour agir).
COPY camille.sh /usr/local/bin/camille
COPY mr-acceptee.sh /usr/local/bin/mr-acceptee
RUN chmod 0755 /usr/local/bin/camille /usr/local/bin/mr-acceptee
COPY bienvenue.sh /etc/profile.d/mentor-bienvenue.sh

# Le dossier de travail appartient à l'apprenant·e dans l'image : la plateforme n'y monte rien.
WORKDIR /workspace
RUN chown apprenant:apprenant /workspace
USER apprenant
