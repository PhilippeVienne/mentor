---
title: "Go"
icon: "🐹"
summary: "Écris, teste et fais tourner tes premiers programmes Go dans un vrai terminal Linux : syntaxe, erreurs, HTTP, WebSocket, JWT, base de données, tests et Docker."
requires: []
published: true
color: "#00ADD8"
banner: images/banniere.svg
environment: environnement
---

Trois projets de l'équipe sont écrits en Go : [`adhesion/mgmt`](https://gitlab.example.org/equipe/adhesion/mgmt) (mode maintenance et WebSocket), [`event-planner-api`](https://gitlab.example.org/equipe/dev/event-planner/event-planner-api) et [`billetterie`](https://gitlab.example.org/equipe/dev/billetterie). Ce parcours t'apprend à les lire, avec un domaine fictif (une ludothèque) pour les exemples, **en pratiquant dans un vrai environnement** : chaque labo te prête un conteneur Linux avec Go 1.25 déjà installé, et c'est le serveur qui vérifie ton travail en lançant les tests.

## À qui s'adresse-t-il ?

À toute personne qui veut comprendre ou faire évoluer un service Go de l'équipe. Aucune connaissance de Go n'est nécessaire : chaque notion est expliquée à sa première apparition. Savoir déjà programmer un peu, dans n'importe quel langage, aide mais n'est pas obligatoire. Une aisance avec un terminal aide aussi, sans être indispensable : la première leçon te donne les réflexes. Tu n'as rien à installer : les labos s'ouvrent dans le portail (la première leçon explique quand même comment installer Go sur ton poste).

## Le programme

| # | Leçon | Durée |
| --- | --- | --- |
| 1 | Premiers pas : syntaxe, types et structures | 45 min |
| 2 | Erreurs, interfaces et paquets | 45 min |
| 3 | Un serveur HTTP qui parle JSON | 50 min |
| 4 | WebSocket : une connexion qui reste ouverte | 45 min |
| 5 | Authentification par JWT | 45 min |
| 6 | Accéder à une base de données | 45 min |
| 7 | Tests et compilation dans une image Docker minimale | 50 min |

**Durée totale : environ 5 h 25** (labos et quiz compris).

:::info Des versions anciennes
Les projets n'utilisent pas la même version de Go : 1.16 pour `mgmt`, 1.14 pour `billetterie`, 1.23 pour `event-planner-api`. Le parcours signale ce qui change d'une version à l'autre.
:::

:::info Des labos dans un vrai conteneur
Ces labos utilisent un **environnement réel** : un conteneur Linux jetable, sans droits administrateur et **sans accès à Internet**, effacé à l'arrêt. Les bibliothèques dont les exercices ont besoin sont déjà installées. Si ton portail ne les propose pas encore, tu peux quand même suivre les leçons, et **valider tout le parcours avec l'examen** si tu maîtrises déjà le sujet.
:::

## Ce que tu sauras faire

- Lire et corriger un service Go de l'équipe
- Écrire une petite API HTTP, la tester et la livrer dans une image Docker légère
- Diffuser des messages par WebSocket et vérifier un jeton JWT

**Prérequis :** aucun.
