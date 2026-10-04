#!/bin/bash
# Faux service « têtu » : il ignore SIGTERM (kill sans option), le note dans son fichier, et ne s'arrête que de force (kill -9).
trap 'echo "SIGTERM reçu et ignoré" >> tetu-demarre.txt' TERM
echo "tetu est démarré" > tetu-demarre.txt
while true; do
    sleep 1 &
    wait $!
done
