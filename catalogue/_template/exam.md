---
# ── Front matter d'un examen de validation ───────────────────────────────────
title: "Examen de validation — Parcours modèle"   # obligatoire
draw: 3            # obligatoire : nombre de questions tirées au sort (le pool doit en contenir au moins autant)
pass_mark: 80      # obligatoire : % de bonnes réponses pour réussir (1 à 100)
minutes: 5         # obligatoire : minutes (1 à 240), chronométrées par le serveur
shuffle: true      # facultatif (true par défaut) : mélange les questions ET les réponses
---

<!-- Introduction : les règles et le public visé. Markdown + directives comme dans une leçon.
     Pas de bloc :::lab dans un examen. -->

Cet examen s'adresse aux personnes qui connaissent déjà le sujet. Il valide **tout le parcours** sans passer par les labos.

<!-- Le pool : un bloc :::quiz par question (même format que dans les leçons). Idéal : 3 fois le tirage.
     Couvre toutes les leçons, mélange les difficultés, ne recopie pas les quiz de leçon. -->

:::quiz
Que fait `git restore --staged fichier` ?

- [ ] Il supprime le fichier du disque
- [x] Il retire le fichier de l'index sans toucher à son contenu
- [ ] Il annule le dernier commit

> `--staged` agit sur l'index (la zone de préparation du prochain commit), pas sur le dossier de travail.
:::

:::quiz
Dans quel ordre se déroule le cycle de base ?

- [ ] commit, add, modifier
- [ ] add, modifier, commit
- [x] modifier, add, commit

> On modifie, on place dans l'index avec `git add`, puis on enregistre avec `git commit`.
:::

:::quiz
Quelle commande liste les branches locales ?

- [x] `git branch`
- [ ] `git log --branches-only`
- [ ] `git status --branches`

> `git branch` sans argument liste les branches ; la branche courante est précédée d'une étoile.
:::

:::quiz
Un `git push` est rejeté (non-fast-forward). Que faire ?

- [ ] `git push --force` pour passer en force
- [x] `git pull`, régler les conflits éventuels, puis `git push`
- [ ] Supprimer le dépôt local et recloner

> Le serveur a des commits que tu n'as pas : intègre-les d'abord. Forcer écraserait le travail des autres.
:::

:::quiz
Que contient le dossier caché `.git` ?

- [ ] Uniquement la configuration de l'éditeur
- [x] L'historique du projet (commits, branches, configuration du dépôt)
- [ ] Les fichiers du projet, compressés

> Supprimer `.git` fait perdre l'historique local ; les fichiers visibles restent mais ne sont plus versionnés.
:::

:::quiz
`git diff --staged` compare…

- [ ] le dossier de travail et l'index
- [x] l'index et le dernier commit
- [ ] ta branche et le serveur

> Sans option, `git diff` regarde ce qui n'est pas encore indexé ; `--staged` montre ce qui sera commité.
:::
