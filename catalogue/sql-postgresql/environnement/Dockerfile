# Environnement réel du parcours SQL et PostgreSQL : serveur PostgreSQL 15 (paquet Debian) lancé SANS root par
# l'apprenant·e, client psql, pg_dump et pg_restore.
# Règles (vérifiées à la compilation du catalogue, voir catalogue/README.md §15) :
#   - image de base de la liste blanche, avec un tag explicite et épinglé ;
#   - pas de sudo, pas de bit setuid, pas de VOLUME ;
#   - la DERNIÈRE instruction USER est un utilisateur non-root d'uid 1000.
# Le réseau n'est disponible QU'AU BUILD : tout est installé ici. Au démarrage, le dossier personnel et le dossier
# de travail sont vides : le serveur est donc initialisé par /opt/exercices/demarrer.sh (commande du labo).
FROM debian:bookworm-slim

# Pas de « cluster » PostgreSQL créé par le paquet : on en crée un, sans root, dans le dossier personnel.
RUN mkdir -p /etc/postgresql-common \
    && printf 'create_main_cluster = false\n' > /etc/postgresql-common/createcluster.conf \
    && apt-get update \
    && apt-get install -y --no-install-recommends bash less nano openssh-server procps python3 tzdata \
        postgresql-15 postgresql-client-15 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 1000 --user-group --shell /bin/bash apprenant

# Les commandes psql, pg_dump, pg_restore, createdb, dropdb et pg_isready de /usr/local/bin (prioritaires dans le
# PATH) se connectent d'office au serveur du labo (socket /tmp/pgsock, base « asso »).
COPY pg-wrapper.sh /usr/local/lib/mentor/pg-wrapper.sh
RUN chmod 755 /usr/local/lib/mentor/pg-wrapper.sh \
    && for c in psql pg_dump pg_restore createdb dropdb pg_isready; do ln -s /usr/local/lib/mentor/pg-wrapper.sh "/usr/local/bin/$c"; done

# Les exercices sont lisibles par tout le monde ; le labo les charge dans la base.
COPY exercices /opt/exercices
RUN chmod 755 /opt/exercices/demarrer.sh
COPY bienvenue.sh /etc/profile.d/mentor-bienvenue.sh

WORKDIR /workspace
USER apprenant
