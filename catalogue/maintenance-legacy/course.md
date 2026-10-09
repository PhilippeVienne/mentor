---
title: "Maintenir du code hérité"
icon: "🛠️"
summary: "Monter de version Django, React ou Material-UI sans tout casser."
requires: [django]
published: true
color: "#78716C"
banner: images/banniere.svg
environment: environnement
---

Plusieurs projets de l'équipe reposent sur des versions anciennes (Django 3.1 dans [PlanningAPI](https://gitlab.example.org/equipe/dev/planning/planning-api) et l'API d'Adhésion, React 16 et `react-scripts` dans [Planning](https://gitlab.example.org/equipe/dev/planning/planning-js)). Ce cours apprend à les mettre à jour sereinement, **en pratiquant dans un vrai terminal Linux** : chaque leçon te prête un conteneur (une petite machine jetable) avec Python 3.10 et quatre versions de Django installées côte à côte (3.1, 3.2, 4.2 et 5.2), et c'est le serveur qui vérifie ton travail en lançant les tests.

## À qui s'adresse-t-il ?

Aux développeur·se·s qui reprennent un projet qu'ils n'ont pas écrit. Aucune expérience de la maintenance n'est nécessaire : chaque leçon explique à quoi sert l'étape et définit les termes avant de les utiliser. Il faut simplement savoir lire un peu de Python (cours *Django*) : les outils (Git, Docker, pip, Node.js, Keycloak…) sont expliqués quand ils apparaissent. La partie React se travaille sur des fichiers fournis, sans Node.js : tu prépares la migration, le build se fait ensuite sur ton poste.

## Plan

1. Lire les notes de version et évaluer le risque (30 min)
2. Écrire des tests avant de toucher au code (35 min)
3. Monter de version Django pas à pas (40 min)
4. Quitter `react-scripts` et mettre à jour React (35 min)
5. Renovate : automatiser les mises à jour (30 min)
6. Documenter et passer la main (30 min)

## Ce que tu sauras faire

- Planifier une montée de version par paliers à partir des fichiers de dépendances
- Sécuriser une base de code avec des tests de caractérisation
- Corriger les API retirées de Django et migrer un front de `react-scripts` vers Vite
- Régler Renovate pour avancer sans se noyer, et documenter pour la personne suivante

**Prérequis :** cours *Django*. **Durée estimée :** environ 3 h 20, labos compris.
