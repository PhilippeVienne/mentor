#!/bin/sh
# Lance le vrai programme PostgreSQL (psql, pg_dump…) du même nom, connecté par défaut au serveur du labo :
# socket Unix dans /tmp/pgsock, base « asso ». Les options -h et -d (ou PGHOST et PGDATABASE) restent prioritaires.
BIN=$(ls -d /usr/lib/postgresql/*/bin | head -n 1)
: "${PGHOST:=/tmp/pgsock}"
: "${PGDATABASE:=asso}"
export PGHOST PGDATABASE
exec "$BIN/$(basename "$0")" "$@"
