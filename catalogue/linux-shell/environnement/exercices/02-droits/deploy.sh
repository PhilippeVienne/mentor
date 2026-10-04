#!/bin/bash
# Déploiement simulé : laisse une trace datée et signée dans deploiement.txt (le serveur la contrôle).
HORODATAGE=$(date +%s)
EMPREINTE=$(printf '%s demo-deploy' "$HORODATAGE" | sha256sum | cut -c1-12)
echo "Déploiement simulé terminé le $HORODATAGE (empreinte $EMPREINTE)" | tee deploiement.txt
