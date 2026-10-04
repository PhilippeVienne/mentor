---
titre: "SQL et PostgreSQL"
icone: "🗄️"
resume: "Requêtes, jointures, index, migrations : la base de données des projets de l'équipe."
prerequis: []
publie: true
couleur: "#336791"
banniere: images/banniere.svg
environnement: environnement
---

PostgreSQL est la base de données de la plupart des projets (Django, [MiniShop](https://gitlab.example.org/equipe/minishop), Planning). Ce parcours apprend à l'interroger et à faire évoluer son schéma **en pratiquant dans un vrai environnement** : chaque labo te prête un conteneur Linux avec un serveur PostgreSQL 15 déjà démarré, et c'est le serveur du portail qui vérifie l'état de ta base.

## À qui s'adresse-t-il ?

Aux développeur·se·s et aux administrateur·rice·s qui doivent comprendre les données d'un projet. Aucune connaissance préalable n'est nécessaire : la première leçon explique ce qu'est une base de données et comment envoyer une requête.

## Plan

1. Base de données, tables, clés primaires et clés étrangères (tu crées le schéma)
2. `SELECT`, filtres, tris, agrégats
3. Jointures
4. Transactions et contraintes
5. Index et lecture d'un plan de requête (`EXPLAIN`)
6. Migrations de schéma et sauvegardes (`pg_dump`)

## Ce que tu sauras faire

- Créer un schéma et écrire les requêtes courantes dans un vrai PostgreSQL
- Lire le schéma d'un projet de l'équipe et le modifier avec une migration
- Lire un plan `EXPLAIN` et sauvegarder une base avec `pg_dump`

**Prérequis :** aucun (un terminal et ses commandes de base aident, sans être indispensables). **Durée estimée :** environ 3 h, labos compris.
