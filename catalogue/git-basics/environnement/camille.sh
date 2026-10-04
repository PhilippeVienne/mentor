#!/bin/sh
# Simule Camille, une coéquipière qui pousse un commit sur `main` du « GitLab » local (leçon « Collaborer avec GitLab »).
set -eu
remote="$HOME/.gitlab-sim/formation-git.git"
[ -d "$remote" ] || { echo "Le serveur local n'existe pas encore : lance d'abord le labo de la leçon." >&2; exit 1; }
git --git-dir="$remote" rev-parse --verify -q refs/heads/main >/dev/null \
    || { echo "Publie d'abord ta branche main (git push -u origin main) : Camille travaille à partir d'elle." >&2; exit 1; }
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
git clone -q "$remote" "$tmp/camille"
cd "$tmp/camille"
git config user.name Camille
git config user.email camille@example.org
echo "<h1>Contact</h1>" > contact.html
git add contact.html
git commit -q -m "Ajoute la page de contact"
git push -q origin main
echo "Camille vient de pousser « Ajoute la page de contact » sur origin/main."
