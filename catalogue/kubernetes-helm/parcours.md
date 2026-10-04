---
titre: "Kubernetes et Helm"
icone: "☸️"
resume: "Pods, déploiements, services, charts Helm, cluster de dev k3d : écris, valide et diagnostique des fichiers Kubernetes dans un atelier réel, sans cluster."
prerequis: [docker-advanced]
publie: true
couleur: "#326CE5"
banniere: images/banniere.svg
environnement: environnement
---

L'équipe dispose d'un cluster Kubernetes ([`cluster-configuration`](https://gitlab.example.org/equipe/dev/cluster-configuration)) et d'un cluster de test local ([`infra-dev`](https://gitlab.example.org/equipe/dev/infra-dev)) basé sur k3d.

## À qui s'adresse-t-il ?

Aux membres de l'équipe Infra, et aux développeur·se·s curieux·ses. Tu n'as pas besoin d'avoir déjà vu Kubernetes : chaque terme est défini quand il apparaît.

## Un atelier réel, mais sans cluster

Chaque leçon se termine par un labo dans un **vrai terminal Linux** prêté par le portail, avec `kubectl`, `helm`, `kubeconform` et `yamllint`. **Il n'y a pas de cluster Kubernetes dans cet atelier** (ni Docker, ni k3d, ni réseau) : tu écris et corriges des fichiers YAML et un chart Helm, tu les valides **hors ligne** (schémas officiels de Kubernetes, `helm lint`, `helm template`) et tu analyses des sorties de `kubectl` enregistrées dans des fichiers. Rien de ce que tu y fais n'est donc démontré sur un vrai cluster : le comportement réel (pods qui démarrent, sondes, certificats, DNS) est expliqué dans les leçons, pas exécuté. Le serveur vérifie ton travail en lançant lui-même les commandes dans ton conteneur.

## Plan

| # | Leçon | Durée |
| --- | --- | --- |
| 1 | Pods, déploiements et services | 45 min |
| 2 | Configuration, secrets et volumes | 45 min |
| 3 | Ingress, certificats (cert-manager) et DNS (External-DNS) | 45 min |
| 4 | Écrire et utiliser un chart Helm | 55 min |
| 5 | Monter un cluster de dev avec k3d, kubectl et Lens | 45 min |
| 6 | Diagnostiquer un pod qui ne démarre pas | 45 min |

## Ce que tu sauras faire

- Expliquer à quoi servent un pod, un déploiement, un service et un Ingress
- Écrire ces objets en YAML et les valider hors ligne avec `kubeconform`
- Lire le chart Helm d'un projet de l'équipe, le surcharger avec ses valeurs et vérifier le résultat avec `helm template`
- Savoir comment se monte un cluster de dev local et ce qui le distingue de la production
- Diagnostiquer un pod qui ne démarre pas à partir de `get`, `describe` et `logs`

## Ressources utilisées par l'équipe

- [infra-dev](https://gitlab.example.org/equipe/dev/infra-dev) : cluster de test local

**Prérequis :** parcours *Docker avancé*. **Durée estimée :** environ 4 h 40 (6 leçons, labos compris).
