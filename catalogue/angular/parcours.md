---
titre: "Angular"
icone: "🅰️"
resume: "Composants, services, RxJS, Material : le frontend d'administration d'Adhésion."
prerequis: [typescript]
publie: true
couleur: "#DD0031"
banniere: images/banniere.svg
environnement: environnement
---

Le frontend d'administration d'[Adhésion](https://gitlab.example.org/equipe/adhesion/frontend) (le site web qui sert à gérer les adhérent·e·s des associations étudiantes) est une application Angular avec Angular Material et Keycloak. Ce parcours prépare à y contribuer **en pratiquant** : chaque labo te prête un conteneur Linux avec Angular 20, TypeScript, Angular Material, `keycloak-angular` et un outil de tests, sans navigateur ni accès à Internet. C'est le serveur qui vérifie le **résultat** de ton travail : un projet qui compile en mode strict, des tests qui passent.

## À qui s'adresse-t-il ?

Aux développeur·se·s qui connaissent TypeScript, sans connaître Angular : chaque notion est expliquée au moment où elle apparaît, et on suppose que tu n'as jamais vu Angular.

## Ce que tu sauras faire à la fin

- Ajouter un écran à l'administration d'Adhésion
- Écrire des composants, des services, des formulaires et des appels à une API
- Comprendre le flux d'authentification Keycloak côté navigateur
- Écrire des tests et comprendre comment l'application est construite et déployée

## Le programme

| # | Leçon | Durée |
| --- | --- | --- |
| 1 | Composants, modules et gabarits | 45 min |
| 2 | Services et injection de dépendances | 35 min |
| 3 | Formulaires et validation | 40 min |
| 4 | Appels HTTP et RxJS | 40 min |
| 5 | Angular Material | 40 min |
| 6 | Authentification avec `keycloak-angular` | 35 min |
| 7 | Tests et build de production | 40 min |

**Durée totale : environ 4 h 35** (275 min, labos et quiz compris).

:::info Des labos dans un vrai conteneur
Ces labos utilisent un **environnement réel** : un conteneur Linux jetable, sans droits administrateur et sans accès à Internet, effacé à l'arrêt. Il n'y a pas de navigateur dedans : la commande `tester` compile ton projet avec le compilateur d'Angular (en mode strict) puis joue les tests avec Vitest dans un faux navigateur. Si ton portail ne propose pas encore les environnements réels, tu peux quand même suivre les leçons, et **valider tout le parcours avec l'examen** si tu maîtrises déjà le sujet.
:::

**Prérequis :** parcours *TypeScript*. **Durée estimée :** environ 4 h 35.
