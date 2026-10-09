# Environnement réel du cours Go : Go 1.25 (chaîne d'outils officielle), nano et un serveur SSH pour VS Code.
# Règles (vérifiées à la compilation du catalogue, voir catalogue/README.md §15) :
#   - image de base de la liste blanche (pas d'image « golang »), avec un tag explicite et épinglé ;
#   - pas de sudo, pas de bit setuid, pas de VOLUME ;
#   - la DERNIÈRE instruction USER est un utilisateur non-root d'uid 1000.
# Le réseau n'est disponible QU'AU BUILD : Go et les modules des leçons (gorilla/websocket, golang-jwt) sont donc
# téléchargés ici ; à l'exécution, GOPROXY=off et les modules sont lus dans le cache /opt/gomodcache.
FROM debian:bookworm-slim

ARG GO_VERSION=1.25.14
ARG GO_SHA256_AMD64=a21ae5633a269bcd7e90cf767e48225633795e99d831742cbf3397064fee7712
ARG GO_SHA256_ARM64=9bf234ea70ffec9347fdf6b22ce4add51717d3386a38a441e8c8743fceb5eaee

RUN apt-get update \
    && apt-get install -y --no-install-recommends bash ca-certificates curl git less nano openssh-server procps python3 systemd-sysv tmux \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 1000 --user-group --shell /bin/bash apprenant

# Chaîne d'outils Go : archive officielle, somme SHA-256 vérifiée (publiée sur https://go.dev/dl/).
RUN set -eux; \
    arch="$(dpkg --print-architecture)"; \
    case "$arch" in \
      amd64) sha="$GO_SHA256_AMD64" ;; \
      arm64) sha="$GO_SHA256_ARM64" ;; \
      *) echo "architecture non prévue : $arch" >&2; exit 1 ;; \
    esac; \
    curl -fsSL -o /tmp/go.tgz "https://go.dev/dl/go${GO_VERSION}.linux-${arch}.tar.gz"; \
    echo "${sha}  /tmp/go.tgz" | sha256sum -c -; \
    tar -C /usr/local -xzf /tmp/go.tgz; \
    rm /tmp/go.tgz; \
    ln -s /usr/local/go/bin/go /usr/local/bin/go; \
    ln -s /usr/local/go/bin/gofmt /usr/local/bin/gofmt
# Le serveur impose un PATH fixe (/usr/local/bin, /usr/bin…) aux commandes qu'il lance : d'où les liens ci-dessus.

# À l'exécution : pas de réseau (GOPROXY=off, GOSUMDB=off), pas de téléchargement de chaîne d'outils (GOTOOLCHAIN=local),
# modules lus dans un cache en lecture seule pour tout le monde. Le cache de compilation (GOCACHE) est placé dans le
# dossier de travail (/workspace/.gocache) : il a plus de place que le HOME (en tmpfs de 64 Mo dans l'outil de rejeu) et
# est effacé avec lui à l'arrêt.
ENV GOROOT=/usr/local/go \
    PATH=/usr/local/go/bin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
    GOMODCACHE=/opt/gomodcache \
    GOCACHE=/workspace/.gocache \
    GOPROXY=off \
    GOSUMDB=off \
    GOTOOLCHAIN=local \
    GOTELEMETRY=off \
    GOFLAGS=-mod=mod \
    CGO_ENABLED=0

# Les exercices sont lisibles par tout le monde ; chaque labo en copie un dans /workspace (qui est effacé à l'arrêt).
COPY exercices /opt/exercices
COPY bienvenue.sh /etc/profile.d/mentor-bienvenue.sh
# Outil de vérification des labos : exécute les tests d'origine (pris dans /opt/exercices) et non ceux du dossier de
# travail, que l'apprenant·e peut modifier. Propriété de root, non modifiable.
COPY verifier-go /usr/local/bin/verifier-go
RUN chown root:root /usr/local/bin/verifier-go && chmod 0755 /usr/local/bin/verifier-go

# Précharge, AU BUILD (avec le réseau), les modules dont les exercices ont besoin, et écrit leurs go.sum.
RUN set -eux; \
    export GOPROXY=https://proxy.golang.org GOSUMDB=sum.golang.org GOFLAGS=-mod=mod HOME=/root GOCACHE=/tmp/gocache-build; \
    for d in /opt/exercices/*/; do \
      cd "$d"; \
      if grep -q '^require' go.mod; then go mod tidy; go mod download; fi; \
    done; \
    chmod -R a+rX /opt/gomodcache /opt/exercices; \
    rm -rf /tmp/gocache-build /root/.cache

# Le dossier de travail appartient à l'apprenant·e dans l'image : la plateforme n'y monte rien.
WORKDIR /workspace
RUN chown apprenant:apprenant /workspace
USER apprenant
