---
# ── A lesson that names its environment itself ───────────────────────────────────────────────────────────────
# `environment`: name of a folder of the course that holds a devcontainer.json (here `environnement/`).
# It can be declared in course.md (for every lesson), in a lesson (as here: it replaces the course's) or in
# the :::lab block. Each learner gets their own environment, a small virtual machine built from that folder.
id: environnement-reel
title: "Leçon modèle : ton environnement"
summary: "Un vrai Linux rien que pour toi, avec Git, où le portail vérifie ce que tu as fait."
minutes: 15
environment: environnement
objectives:
  - Démarrer et arrêter ton environnement
  - Faire valider une étape par le serveur
---

<!-- This lesson explains to learners how an environment works: adapt it or drop it in your own course. -->

Pour pratiquer, le portail te prête un **vrai** Linux rien que pour toi : une petite machine virtuelle, déjà équipée
des outils de la leçon. Les commandes y sont exécutées pour de vrai, et rien de ce que tu y fais ne peut abîmer ton
ordinateur ni gêner les autres.

## Comment ça marche

1. Clique sur **Démarrer l'environnement** : le serveur lance une machine virtuelle rien que pour toi.
2. Tape tes commandes dans le terminal, ou utilise les boutons **▶ Lancer** du cours.
3. Clique sur **Vérifier** à chaque étape : c'est le serveur qui regarde dans ton environnement si c'est fait.
4. Clique sur **Arrêter** quand tu as fini (sinon, l'arrêt est automatique après un moment d'inactivité).

```shell run
git init projet
ls -a projet
```

:::warning
Tes fichiers sont **effacés** quand l'environnement s'arrête : n'y garde rien d'important.
:::

:::info Pas d'accès à Internet
Ton environnement n'a pas de réseau : tout ce dont la leçon a besoin y est déjà installé.
:::

## Entraîne-toi

:::lab
# Steps are verified by the SERVER inside the learner's environment, with the checks of catalogue/_checks.yml.
# The commands of the checks come from the catalogue, never from the learner.
engine: real
intro: |
  Ton environnement contient Git, configuré à ton nom. Crée un dépôt et fais un premier commit, pour de vrai.
# Files written in the working folder (/workspace) when the lab starts.
files:
  notes.txt: |
    Mes notes de la leçon.
steps:
  - text: 'Crée un dépôt Git dans un dossier `projet` avec `git init projet`'
    hint: git init projet
    checks:
      - env-file-exists: projet/.git
    solution:
      - git init projet

  - text: 'Dans `projet`, crée `README.md` qui commence par `# Projet`'
    after: [1]
    checks:
      - env-file-contains: [projet/README.md, '^# Projet']
    solution:
      - "echo '# Projet' > projet/README.md"

  - text: 'Fais un commit avec le message `Premier commit`'
    after: [2]
    checks:
      - command-succeeds: 'git -C projet rev-parse HEAD'
      - output-contains: ['git -C projet log -1 --format=%s', '^Premier commit$']
    solution:
      - 'git -C projet add README.md'
      - 'git -C projet commit -m "Premier commit"'
:::

## Vérifie tes acquis

:::quiz
Qui décide qu'une étape du labo est validée ?

- [ ] Ton navigateur, dès que tu tapes la commande
- [x] Le serveur, en regardant l'état de ton environnement
- [ ] Personne : il suffit de cliquer sur « Vérifier »

> Le serveur exécute des vérifications dans ton environnement ; ton navigateur ne peut pas déclarer une étape réussie.
:::

:::quiz
Que deviennent tes fichiers quand l'environnement s'arrête ?

- [ ] Ils sont gardés une semaine
- [ ] Ils sont copiés sur ton ordinateur
- [x] Ils sont effacés

> L'environnement est jetable : à chaque démarrage, tu repars d'un état neuf.
:::

:::quiz
Qu'est-ce que ton environnement ?

- [ ] Un terminal qui imite les commandes dans ton navigateur
- [x] Une petite machine virtuelle rien que pour toi, avec un vrai Linux
- [ ] Le serveur du portail, partagé avec les autres apprenant·e·s

> Chaque personne a sa propre machine virtuelle : les commandes y sont réellement exécutées, à l'écart des autres.
:::
