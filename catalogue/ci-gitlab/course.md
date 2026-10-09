---
title: "CI/CD avec GitLab"
icon: "🚦"
summary: "Pipelines, jobs, images, analyses de sécurité (SAST, secrets, dépendances) : écris et répare de vrais pipelines dans un terminal Linux."
requires: [git-basics]
published: true
color: "#FC6D26"
banner: images/banniere.svg
environment: environnement
---

Presque tous les dépôts de l'équipe ont un `.gitlab-ci.yml` : construction de l'image Docker, tests, analyses de sécurité, déploiement. Exemple : la CI de l'[API d'Adhésion](https://gitlab.example.org/equipe/adhesion/api). Ce cours t'apprend à les lire, à les écrire et à les réparer, **en pratiquant dans un vrai terminal Linux** : chaque labo te prête un conteneur jetable, et c'est le serveur qui vérifie ton travail.

:::info Pas de vrai runner GitLab dans les labos
Un vrai pipeline demande un serveur GitLab et un runner, absents du conteneur (qui n'a d'ailleurs pas de réseau). Les labos utilisent donc `verifier-ci` et `verifier-renovate`, deux petits outils de l'équipe qui lisent tes fichiers et en contrôlent la **structure** comme GitLab le ferait, et qui peuvent simuler les `rules`. Ils n'exécutent aucun job. Ils t'évitent des allers-retours, mais le dernier mot reste à GitLab : valide toujours un vrai pipeline dans l'éditeur de pipeline du projet.
:::

## À qui s'adresse-t-il ?

À celles et ceux qui savent utiliser Git et doivent lire, corriger ou écrire un pipeline. Tu n'as besoin d'aucune connaissance préalable en CI : on définit chaque mot au moment où il apparaît, et on explique chaque ligne de code.

## Plan

| # | Leçon | Durée |
| --- | --- | --- |
| 1 | Qu'est-ce que la CI ? (intégration, livraison et déploiement continus, runners, premier pipeline) | 25 min |
| 2 | Anatomie d'un `.gitlab-ci.yml` (stages, jobs, `rules`, `extends`, `include`) | 35 min |
| 3 | Construire et publier une image Docker | 35 min |
| 4 | Variables et secrets | 35 min |
| 5 | Les modèles de sécurité : SAST, Dependency Scanning, Secret Detection, Code Quality | 40 min |
| 6 | Mises à jour automatiques avec Renovate | 30 min |
| 7 | Déboguer un pipeline en échec | 30 min |

## Ce que tu sauras faire

- Lire et modifier le pipeline d'un projet
- Écrire un job de build d'image et ne jamais laisser un secret dans le dépôt
- Ajouter les analyses de sécurité à un nouveau projet
- Configurer Renovate et diagnostiquer un pipeline en échec à partir de son message

**Prérequis :** cours *Git*. **Durée estimée :** environ 3 h 50.
