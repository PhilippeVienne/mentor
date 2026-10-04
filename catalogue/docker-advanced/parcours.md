---
titre: "Docker advanced"
icone: "🚢"
resume: "Dockerfile, volumes, réseaux, Compose et bonnes pratiques de production, comme sur l'infra de l'équipe."
moteur: docker
prerequis: [docker-hello]
publie: true
couleur: "#1D63ED"
banniere: images/banniere.svg
---

Tu sais lancer des conteneurs : place à la **construction d'applications complètes**. Ce parcours reprend les pratiques réellement utilisées sur l'infra de l'équipe : images construites par la CI GitLab, publiées sur `registry.gitlab.example.org`, déployées avec un `compose-infra.yaml`.

## À qui s'adresse ce parcours ?

À celles et ceux qui ont terminé **Docker hello world** (prérequis) et veulent passer de l'utilisation à la création : packager leur propre application, la faire dialoguer avec une base et préparer sa mise en production.

## Ce que tu sauras faire à la fin

- Écrire un **Dockerfile** et tirer parti du cache de build.
- **Persister** des données (volumes) et développer en direct (bind mounts).
- Connecter plusieurs conteneurs avec un **réseau** et décrire toute l'application avec **Docker Compose**.
- Produire une image **légère et sûre** (multi-étapes, non-root) et la publier sur un registry.

:::info Durée
Environ **1 h 30** pour 5 leçons. Prépare-toi à écrire quelques fichiers (Dockerfile, `compose.yml`) : le labo propose un éditeur intégré et des boutons « Créer ce fichier ».
:::
