# Environnement réel du parcours Terraform : le binaire officiel `terraform` (version épinglée, somme SHA-256 vérifiée)
# et un MIROIR LOCAL de fournisseurs (local, random, null, time), pour que `terraform init` fonctionne SANS réseau.
# Règles (vérifiées à la compilation du catalogue, voir catalogue/README.md §15) :
#   - image de base de la liste blanche, avec un tag explicite ;
#   - pas de sudo, pas de bit setuid, pas de VOLUME ;
#   - la DERNIÈRE instruction USER est un utilisateur non-root d'uid 1000.
# Le réseau n'existe QU'AU BUILD : le binaire et les fournisseurs sont téléchargés ici, jamais à l'exécution.
# python3 : requis par la couche VS Code web du portail.
FROM debian:trixie-slim

ARG TERRAFORM_VERSION=1.16.5
ARG TERRAFORM_SHA256=2bc2fcfff033265c9e02ca0351f01794eb122f62a9b2a49a3294b9e49eaab5e4

RUN apt-get update \
    && apt-get install -y --no-install-recommends bash ca-certificates coreutils curl findutils grep less nano \
       openssh-server systemd-sysv procps python3 tree unzip \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 1000 --user-group --shell /bin/bash apprenant

# 1. Le binaire officiel, téléchargé puis vérifié avec sa somme SHA-256 (le build échoue si elle ne correspond pas).
#    Il est rangé hors du PATH sous le nom `terraform-reel` : `terraform` est un petit script (voir plus bas).
RUN curl -fsSL -o /tmp/terraform.zip "https://releases.hashicorp.com/terraform/${TERRAFORM_VERSION}/terraform_${TERRAFORM_VERSION}_linux_amd64.zip" \
    && echo "${TERRAFORM_SHA256}  /tmp/terraform.zip" | sha256sum -c - \
    && mkdir -p /opt/terraform/bin \
    && unzip -q /tmp/terraform.zip terraform -d /opt/terraform/bin \
    && mv /opt/terraform/bin/terraform /opt/terraform/bin/terraform-reel \
    && rm /tmp/terraform.zip

# 2. Le miroir local des fournisseurs : `terraform providers mirror` les télécharge UNE FOIS, ici, au build.
COPY miroir /tmp/miroir
RUN cd /tmp/miroir \
    && /opt/terraform/bin/terraform-reel providers mirror /opt/terraform-providers \
    && rm -rf /tmp/miroir \
    && chmod -R a+rX /opt/terraform-providers

# 3. La configuration du client Terraform et le script `terraform`, qui l'active avant de lancer le vrai binaire.
COPY terraformrc /opt/terraform/terraformrc
COPY terraform /usr/local/bin/terraform
RUN chmod 0755 /usr/local/bin/terraform

# Les exercices sont lisibles par tout le monde ; chaque labo en copie un dans /workspace (effacé à l'arrêt).
COPY exercices /opt/exercices
COPY bienvenue.sh /etc/profile.d/mentor-bienvenue.sh

# Le dossier de travail appartient à l'apprenant·e dans l'image : la plateforme n'y monte rien.
WORKDIR /workspace
RUN chown apprenant:apprenant /workspace
USER apprenant
