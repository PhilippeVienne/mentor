#!/bin/sh
# Lancé au BUILD de l'image : copie les images de images.txt depuis Docker Hub vers le stockage du miroir.
set -eu
cd /opt/mentor
mkdir -p miroir
# Même configuration que le miroir, mais en écriture le temps du remplissage.
sed 's/enabled: true/enabled: false/' miroir.yml > /tmp/miroir-ecriture.yml
docker-registry serve /tmp/miroir-ecriture.yml &
registry=$!
i=0
until curl -fs http://127.0.0.1:5001/v2/ >/dev/null; do
    i=$((i + 1))
    [ "$i" -lt 30 ] || { echo "Le registry local n'a pas démarré." >&2; exit 1; }
    sleep 1
done
grep -v '^[[:space:]]*\(#\|$\)' images.txt | while read -r source served; do
    skopeo copy --dest-tls-verify=false "docker://docker.io/library/$source" "docker://127.0.0.1:5001/library/$served"
done
kill "$registry"
wait "$registry" 2>/dev/null || true
rm -f /tmp/miroir-ecriture.yml
chmod -R a+rX miroir
