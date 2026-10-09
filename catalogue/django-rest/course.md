---
title: "API REST avec Django REST framework"
icon: "🔌"
summary: "Sérialiseurs, vues, permissions, documentation : l'API d'Adhésion."
requires: [django]
published: true
color: "#A30000"
banner: images/banniere.svg
environment: environnement
---

L'équipe  développe des applications qui échangent des données par des **API**. L'[API d'Adhésion](https://gitlab.example.org/equipe/adhesion/api) (adhérent·e·s, cartes, adhésions) et [PlanningAPI](https://gitlab.example.org/equipe/dev/planning/planning-api) (modules, participants, créneaux) sont écrites avec Django REST framework, une bibliothèque Python construite au-dessus de Django. Ce cours part de zéro côté API (qu'est-ce qu'une API ? un code de statut ? un jeton ?) et t'amène jusqu'à un point d'accès protégé, filtré, documenté et testé, sur un domaine fictif d'associations et d'événements. **Chaque labo te prête un vrai conteneur Linux** avec Django et un projet de départ : tu écris le code, et c'est le serveur qui vérifie le résultat en lançant les tests.

Les deux projets de l'équipe utilisent des versions anciennes (Django 3.1, DRF 3.12 pour Adhésion et 3.11 pour PlanningAPI) ; les labos tournent avec des versions récentes (Django 6.1, DRF 3.18). Les différences utiles sont signalées dans les leçons.

## À qui s'adresse ce cours ?

À toute personne qui connaît les bases de Django (modèles, vues, `settings.py`), même sans avoir jamais construit ni utilisé d'API. Aucune autre connaissance n'est supposée : les mots nouveaux (API, HTTP, JSON, jeton, schéma…) sont définis au fur et à mesure.

## Le programme

| # | Leçon | Durée |
| --- | --- | --- |
| 1 | À quoi sert une API REST ? (HTTP, JSON, conventions REST) | 35 min |
| 2 | Les sérialiseurs : du modèle au JSON, et retour | 45 min |
| 3 | Vues, viewsets et routeurs | 45 min |
| 4 | Authentification par jeton OIDC et permissions | 50 min |
| 5 | Filtres, pagination, import et export | 45 min |
| 6 | Documenter son API | 35 min |
| 7 | Tester son API | 45 min |

**Durée totale : environ 5 h** (labos et quiz compris).

## Ce que tu sauras faire à la fin

- Expliquer ce qu'est une API REST et lire une requête HTTP
- Ajouter un point d'accès à une API Django REST framework
- Protéger une ressource par rôle
- Lire les parties « API » des projets de l'équipe (sérialiseurs, viewsets, permissions)
- Écrire des tests d'API

:::info Des labos dans un vrai conteneur
Ces labos utilisent un **environnement réel** : un conteneur Linux jetable, sans droits administrateur et sans accès à Internet (tout est déjà installé), effacé à l'arrêt. Si ton portail ne les propose pas encore, tu peux quand même suivre les leçons, et **valider tout le cours avec l'examen** si tu maîtrises déjà le sujet.
:::

**Prérequis :** cours *Django*.
