---
id: workflow
title: "Workflow d'équipe et bonnes pratiques"
summary: 'Branches de fonctionnalité, Merge Requests et réflexes de pro.'
minutes: 15
objectives:
  - Appliquer le flux « feature branch + Merge Request »
  - "Appliquer les bons réflexes : commits atomiques, pull avant push"
  - "Écrire un `.gitignore` pour ne pas suivre secrets et fichiers générés"
  - Savoir comment contribuer à un projet de l'équipe
---

Savoir utiliser les commandes, c'est bien. Savoir *comment s'organiser à plusieurs*, c'est ce qui fait qu'un projet associatif tient dans la durée. Voici le flux utilisé par la plupart des équipes.

## Le flux « feature branch + Merge Request »

![Les six étapes d'une fonctionnalité : branche, commits, publication, merge request, fusion, nettoyage](images/workflow-mr.svg)

1. `main` reste toujours stable et déployable : on n'y commite jamais directement.
2. Pour chaque tâche : `git switch -c ma-fonctionnalite` depuis un `main` à jour.
3. Tu commites régulièrement, puis tu publies : `git push -u origin ma-fonctionnalite`.
4. Sur GitLab, tu ouvres une **Merge Request** (MR) : l'équipe relit, commente, la CI teste.
5. Une fois approuvée, la MR est fusionnée dans `main`. Tu fais `git switch main && git pull` et tu supprimes ta branche.

```mermaid
gitGraph
   commit id: "v1"
   branch fix-typo
   commit id: "Corrige la faute"
   checkout main
   branch ajout-recherche
   commit id: "Barre de recherche"
   commit id: "Tests"
   checkout main
   merge fix-typo
   merge ajout-recherche
   commit id: "v2" tag: "release"
```

:::info Merge Request ou Pull Request ?
C'est la même chose : **Merge Request** sur GitLab, **Pull Request** sur GitHub. Ce n'est pas une fonction de Git lui-même, mais du service qui héberge ton dépôt.
:::

## Les bons réflexes

:::cards
### Commits atomiques

Un commit = un changement logique, qui fonctionne. Plus facile à relire, à annuler, à comprendre.

### .gitignore

Liste les fichiers à ne jamais suivre : `node_modules/`, `.env`, fichiers compilés, clés d'API…

### Pull avant push

Récupère le travail des autres avant de publier le tien : moins de rejets, moins de conflits.

### Jamais de secrets

Un mot de passe commité reste dans l'historique, même supprimé ensuite. Il faut le **révoquer** immédiatement.
:::

## Un fichier .gitignore typique

```text
# dépendances
node_modules/

# secrets
.env

# fichiers générés
dist/
*.log
```

:::danger Un secret commité est un secret compromis
Même si tu le supprimes au commit suivant, il reste dans l'historique. Change le mot de passe ou la clé tout de suite, *puis* nettoie.
:::

:::info Contribuer dans l'équipe
Les projets sont sur [GitLab](https://gitlab.example.org/equipe). Aucun prérequis technique : l'équipe forme les nouvelles recrues aux outils et technologies utilisés.
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  Mini-projet final : tu corriges une faute de frappe via une branche, tu la publies, et la « Merge Request » est acceptée par l'équipe.
commands:
  - 'rm -rf ~/.gitlab-sim && git init -q --bare ~/.gitlab-sim/formation-git.git'
  - git init -q
  - 'echo "# Annuaire des assos" > README.md'
  - 'echo "Bienvenu sur le site." >> README.md'
  - "git add . && git commit -q -m \"Initialise l'annuaire\""
  - 'git remote add origin git@gitlab.example.org:equipe/formation-git.git'
  - git push -q -u origin main
steps:
  - text: 'Crée la branche `fix-typo`'
    hint: git switch -c fix-typo
    checks:
      - command-succeeds: 'git rev-parse --verify -q fix-typo'
    solution:
      - git switch -c fix-typo
  - text: 'Corrige « Bienvenu » en « Bienvenue » dans `README.md` et commite'
    hint: "Avec nano README.md, ajoute le « e » manquant, enregistre, puis lance git commit -am \"Corrige la faute de frappe\""
    after: [1]
    checks:
      - command-succeeds: 'test "$(git rev-list --count main..fix-typo)" -ge 1 && git show fix-typo:README.md | grep -q Bienvenue'
    solution:
      - "sed -i 's/Bienvenu /Bienvenue /' README.md"
      - 'git commit -am "Corrige la faute de frappe"'
  - text: 'Publie ta branche : `git push -u origin fix-typo`'
    hint: git push -u origin fix-typo
    after: [2]
    checks:
      - command-succeeds: 'test "$(git rev-parse fix-typo)" = "$(git --git-dir="$HOME/.gitlab-sim/formation-git.git" rev-parse fix-typo)"'
    solution:
      - git push -u origin fix-typo
  - text: 'La MR est acceptée ! Lance `mr-acceptee fix-typo` pour simuler l''équipe, puis reviens sur `main` et mets-le à jour avec `git pull`'
    hint: mr-acceptee fix-typo puis git switch main puis git pull
    after: [3]
    checks:
      - output-contains: ['git branch --show-current', '^main$']
      - command-succeeds: 'test "$(git rev-parse main)" = "$(git rev-parse fix-typo)"'
    solution:
      - mr-acceptee fix-typo
      - git switch main
      - git pull
  - text: 'Nettoie : supprime la branche locale avec `git branch -d fix-typo`'
    hint: git branch -d fix-typo
    after: [4]
    checks:
      - command-fails: 'git rev-parse --verify -q fix-typo'
    solution:
      - git branch -d fix-typo
:::

## Vérifie tes acquis

:::quiz
À quoi sert un fichier .gitignore ?

- [ ] À lister les contributeurs
- [x] À indiquer à Git les fichiers à ne pas suivre (secrets, dépendances, fichiers générés)
- [ ] À configurer GitLab
- [ ] À accélérer Git

> Il évite de polluer le dépôt et, surtout, de publier des secrets par erreur.
:::

:::quiz
Tu as commité un mot de passe puis supprimé le fichier au commit suivant. Est-ce réglé ?

- [ ] Oui, il n'existe plus
- [x] Non : il reste dans l'historique, il faut le révoquer/changer
- [ ] Oui si on fait git gc
- [ ] Oui, si on supprime la branche qui le contenait

> Quiconque a accès à l'historique peut le retrouver. Considère-le comme compromis.
:::

:::quiz
Qu'est-ce qu'une Merge Request ?

- [ ] Une commande Git
- [x] Une demande de fusion sur GitLab pour relecture avant d'intégrer une branche
- [ ] Un type de conflit
- [ ] Un backup

> C'est une fonctionnalité de GitLab (Pull Request sur GitHub), pas de Git lui-même.
:::

:::quiz
Peut-on utiliser git push --force sur la branche main partagée ?

- [ ] Oui, c'est la méthode normale
- [x] Non, ça peut écraser le travail des autres : à éviter
- [ ] Oui, tant que personne n'a encore ouvert de merge request
- [ ] Oui, si tu préviens l'équipe après coup

> --force réécrit l'historique distant : tes collègues perdent des commits.
:::
