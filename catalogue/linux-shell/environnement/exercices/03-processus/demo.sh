#!/bin/bash
# Faux service de l'équipe : écrit une ligne dans son journal toutes les 5 secondes.
# Quand il reçoit SIGTERM (kill sans option), il note un arrêt propre puis se termine.
JOURNAL="${1:-service.log}"
trap 'echo "$(date "+%F %T") INFO arrêt propre du service demo" >> "$JOURNAL"; exit 0' TERM
while true; do
    echo "$(date "+%F %T") INFO le service demo répond" >> "$JOURNAL"
    sleep 5 &
    wait $!
done
