# Environnement réel du parcours « Sauvegardes et stockage objet » :
#   - un serveur MinIO (S3) lancé SANS root sur localhost, avec des identifiants FACTICES de labo, et son client mc ;
#   - python3 + boto3, l'AWS CLI, PostgreSQL 17 (paquet Debian) lancé sans root, pg_dump et pg_restore ;
#   - des versions simplifiées de backup.py, restore.py et test_connection_s3.py, dans l'esprit du dépôt
#     equipe/dev/backups3 (écrites pour ce parcours, sans aucun secret).
# Règles (vérifiées à la compilation du catalogue, voir catalogue/README.md §15) :
#   - image de base de la liste blanche, avec un tag explicite et épinglé ;
#   - pas de sudo, pas de bit setuid, pas de VOLUME ;
#   - la DERNIÈRE instruction USER est un utilisateur non-root d'uid 1000.
# Le réseau n'est disponible QU'AU BUILD : tout est installé ici. Au démarrage, le dossier personnel et le dossier de
# travail sont vides : les services sont lancés par les commandes du labo (demarrer-minio, demarrer-postgres).

# --- Étape 1 : compilation de MinIO et de mc ---------------------------------------------------------------------
# Les binaires officiels (dl.min.io) ne sont plus distribués : le projet communautaire est archivé et le serveur
# répond « 410 Gone ». On compile donc les versions publiées (étiquettes Git épinglées) à partir des sources, que le
# proxy de modules Go sert avec vérification de somme (go.sum / sumdb). Cette étape n'est pas conservée dans l'image.
# Une seule retouche, pour tenir dans les 512 Mo du conteneur : MinIO réserve au démarrage des files d'attente
# internes (réplication, transition de cycle de vie, notifications) de 50 000 à 100 000 entrées qui coûtent environ
# 110 Mo de mémoire ; on les ramène à 500-1 000 entrées. Cela ne change rien au comportement pour un labo (un seul
# utilisateur, quelques objets). Chaque retouche est vérifiée : le build échoue si la ligne visée a changé.
FROM debian:trixie-slim AS compilation
ARG MINIO_VERSION=RELEASE.2025-04-22T22-12-26Z
ARG MC_VERSION=RELEASE.2025-04-16T18-13-26Z
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates git golang-go \
    && rm -rf /var/lib/apt/lists/*
ENV GOBIN=/sortie CGO_ENABLED=0 GOTOOLCHAIN=auto
RUN go install -ldflags "-s -w" "github.com/minio/mc@${MC_VERSION}"
RUN set -eux; \
    dir="$(go mod download -json "github.com/minio/minio@${MINIO_VERSION}" | sed -n 's/.*"Dir": "\(.*\)",/\1/p')"; \
    cp -r "$dir" /src; chmod -R u+w /src; cd /src; \
    sed -i 's/make(chan transitionTask, 100000)/make(chan transitionTask, 1000)/' cmd/bucket-lifecycle.go; \
    sed -i 's/mrfOpsQueueSize = 100000/mrfOpsQueueSize = 1000/' cmd/mrf.go; \
    sed -i 's/maxConcurrentAsyncSend = 50000/maxConcurrentAsyncSend = 500/' internal/event/targetlist.go; \
    sed -i 's/make(chan ReplicationWorkerOperation, 100000)/make(chan ReplicationWorkerOperation, 1000)/; s/make(chan MRFReplicateEntry, 100000)/make(chan MRFReplicateEntry, 1000)/' cmd/bucket-replication.go; \
    grep -q 'transitionTask, 1000)' cmd/bucket-lifecycle.go; \
    grep -q 'mrfOpsQueueSize = 1000$' cmd/mrf.go; \
    grep -q 'maxConcurrentAsyncSend = 500$' internal/event/targetlist.go; \
    grep -q 'MRFReplicateEntry, 1000)' cmd/bucket-replication.go; \
    go build -trimpath -ldflags "-s -w" -o /sortie/minio .

# --- Étape 2 : l'image de l'apprenant·e --------------------------------------------------------------------------
FROM debian:trixie-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends awscli bash ca-certificates curl less nano openssh-server procps systemd-sysv \
        python3 python3-boto3 tzdata postgresql postgresql-client \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 1000 --user-group --shell /bin/bash apprenant

# Serveur MinIO ; le client mc est appelé par le lanceur (qui règle l'alias « labo ») : voir plus bas.
COPY --from=compilation /sortie/minio /usr/local/bin/minio
COPY --from=compilation /sortie/mc /opt/minio/mc
RUN chmod 755 /usr/local/bin/minio /opt/minio/mc

# Outils du labo : lisibles par tout le monde. Le « lanceur » est appelé sous plusieurs noms (liens symboliques de
# /usr/local/bin, seul dossier garanti dans le PATH des vérifications du serveur) et règle l'environnement avant de
# lancer le vrai programme : identifiants factices, point d'accès, base PostgreSQL du labo.
COPY outils /opt/outils
COPY backups3 /opt/backups3
RUN chmod 755 /opt/outils/* /opt/backups3/*.py \
    && for c in aws mc psql pg_dump pg_restore createdb dropdb pg_isready backup.py restore.py test_connection_s3.py; do \
         ln -s /opt/outils/lanceur "/usr/local/bin/$c"; done \
    && ln -s /opt/outils/demarrer-minio /usr/local/bin/demarrer-minio \
    && ln -s /opt/outils/demarrer-postgres /usr/local/bin/demarrer-postgres \
    && ln -s /opt/outils/preparer-restauration /usr/local/bin/preparer-restauration \
    && ln -s /opt/outils/controle-date-ancienne /usr/local/bin/controle-date-ancienne

# Les exercices sont copiés dans /workspace par les labos (commandes).
COPY exercices /opt/exercices
RUN chmod -R a+rX /opt/exercices
COPY bienvenue.sh /etc/profile.d/mentor-bienvenue.sh
RUN echo '. /opt/outils/mentor-labo.sh' >> /etc/bash.bashrc

# Le dossier de travail appartient à l'apprenant·e dans l'image : la plateforme n'y monte rien.
WORKDIR /workspace
RUN chown apprenant:apprenant /workspace
USER apprenant
