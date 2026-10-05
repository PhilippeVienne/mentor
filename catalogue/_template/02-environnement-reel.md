---
# ── Leçon avec un environnement RÉEL (un vrai conteneur Linux par apprenant·e) ───────────────────────────────
# `environnement` : nom d'un dossier du parcours qui contient un devcontainer.json (ici `environnement/`).
# On peut aussi le déclarer dans parcours.md (pour toutes les leçons) ou dans le bloc :::labo.
# Le portail doit avoir ENVIRONMENTS_ENABLED=True ; sinon la leçon s'affiche avec un avertissement à la place du labo.
id: environnement-reel
title: "Leçon modèle : un vrai terminal"
summary: "Le même Git, mais dans un vrai conteneur Linux, avec ton propre VS Code si tu veux."
minutes: 15
environment: environnement
objectives:
  - Démarrer et arrêter ton environnement réel
  - Faire valider une étape par le serveur
---

<!-- Une leçon avec environnement réel suit le même fil qu'une autre leçon. Seul le labo change : `moteur: reel`. -->

Jusqu'ici, tu as utilisé un terminal **simulé**. Pour certaines leçons, le portail te prête un **vrai** conteneur
Linux : les commandes sont exécutées pour de vrai, et tu peux même t'y connecter avec ton propre VS Code.

## Comment ça marche

1. Clique sur **Démarrer l'environnement** : le serveur lance un conteneur rien que pour toi.
2. Tape tes commandes dans le terminal, ou utilise les boutons **▶ Lancer** du cours.
3. Clique sur **Vérifier** à chaque étape : c'est le serveur qui regarde dans ton conteneur si c'est fait.
4. Clique sur **Arrêter** quand tu as fini (sinon, l'arrêt est automatique après un moment d'inactivité).

```shell run
git init projet
ls -a projet
```

:::warning
Tes fichiers sont **effacés** quand l'environnement s'arrête. Pour garder ton travail, pousse-le sur GitLab.
:::

## Entraîne-toi

:::lab
# `moteur: reel` : étapes vérifiées par le SERVEUR dans le conteneur (vérifications « moteurs: [reel] » de
# _verifications.yml). Les commandes des vérifications viennent du catalogue, jamais de l'apprenant·e.
engine: real
intro: |
  Ton environnement contient Git, configuré à ton nom. Crée un dépôt et fais un premier commit, pour de vrai.
# Fichiers créés dans le dossier de travail (/workspace) au démarrage de l'environnement.
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
Qui décide qu'une étape d'un labo réel est validée ?

- [ ] Ton navigateur, dès que tu tapes la commande
- [x] Le serveur, en regardant l'état de ton conteneur
- [ ] Personne : il suffit de cliquer sur « Vérifier »

> Le serveur exécute des vérifications dans ton conteneur ; ton navigateur ne peut pas déclarer une étape réussie.
:::

:::quiz
Que deviennent tes fichiers quand l'environnement s'arrête ?

- [ ] Ils sont gardés une semaine
- [ ] Ils sont envoyés sur GitLab
- [x] Ils sont effacés

> L'environnement est jetable : pousse ton travail sur GitLab si tu veux le garder.
:::

:::quiz
Avec quel utilisateur tournent tes commandes dans l'environnement ?

- [x] Un utilisateur ordinaire (non-root), sans `sudo`
- [ ] root, pour pouvoir tout installer
- [ ] L'administrateur du serveur

> Pour la sécurité de tout le monde, aucun environnement ne tourne en root et `sudo` n'existe pas.
:::
