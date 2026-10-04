#!/bin/sh
# Simule l'acceptation d'une Merge Request : la branche publiée est fusionnée dans `main` du « GitLab » local.
set -eu
branche="${1:-}"
case "$branche" in ""|*[!A-Za-z0-9._/-]*) echo "Usage : mr-acceptee <branche>" >&2; exit 2 ;; esac
remote="$HOME/.gitlab-sim/formation-git.git"
git --git-dir="$remote" rev-parse --verify -q "refs/heads/$branche" >/dev/null \
    || { echo "La branche « $branche » n'est pas publiée sur le serveur : fais d'abord git push -u origin $branche." >&2; exit 1; }
git --git-dir="$remote" update-ref refs/heads/main "refs/heads/$branche"
echo "La Merge Request « $branche » a été acceptée : main avance sur le serveur."
