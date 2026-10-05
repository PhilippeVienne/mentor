---
id: deboguer-pipeline
title: "Déboguer un pipeline en échec"
summary: "Une méthode en quatre temps pour passer du rouge au vert sans deviner."
minutes: 30
objectives:
  - Retrouver le job et la ligne qui font échouer un pipeline
  - Identifier les causes fréquentes (YAML, `rules`, variables, image)
  - Diagnostiquer une erreur à partir de son message
  - Tester sans polluer l'historique
---

Ta merge request est rouge. La tentation : modifier le `.gitlab-ci.yml`, pousser, attendre, recommencer. Dix commits « fix ci » plus tard, le problème n'est toujours pas compris. Il existe une méthode plus calme.

## Quatre temps

Un **log** est le journal d'un job : tout ce que ses commandes ont affiché, ligne après ligne, consultable dans GitLab en cliquant sur le job.

1. **Où ?** Ouvre le pipeline et repère le premier job rouge. Les jobs des stages suivants n'ont pas tourné : ils ne sont pas coupables.
2. **Quoi ?** Ouvre le log de ce job et remonte jusqu'à la **première** erreur. Le message de la fin (`ERROR: Job failed: exit code 1`) ne dit que le code de sortie, le petit nombre qu'une commande renvoie en finissant (0 = succès, tout autre nombre = échec).
3. **Pourquoi ?** Rapproche l'erreur d'une cause fréquente (tableau ci-dessous).
4. **Corriger et prouver** : reproduis en local quand c'est possible, change une seule chose à la fois.

## Les causes fréquentes

| Symptôme | Cause probable |
| --- | --- |
| Le pipeline n'est même pas créé, message « yaml invalid » | Erreur de syntaxe : indentation, `:` oublié, guillemets, ou `: ` (deux-points et espace) dans une commande non entourée de guillemets |
| Le job que tu attends n'apparaît pas | Aucune de ses `rules` ne correspond (branche, source, `changes`) |
| Le pipeline apparaît en double (branche + merge request) | Règle manquante contre le pipeline en double (voir la leçon sur la sécurité) |
| `docker: command not found` ou `Cannot connect to the Docker daemon` | Mauvaise `image`, ou `docker:dind` et `DOCKER_HOST` manquants |
| `unauthorized` au `docker push` | `docker login` oublié ou mauvais registry |
| Une variable est vide, ou une image a un nom bizarre (`:main`) | Elle est définie comme **protégée** et la branche ne l'est pas, ou le nom est mal orthographié |
| `npm ci` ou `pipenv` échoue | Fichier de verrouillage absent ou désynchronisé, version de l'image différente de celle du poste |

Un log typique, quand l'image ne contient pas l'outil attendu :

```console
$ npm run lint
/bin/sh: eval: line 142: npm: not found
Cleaning up project directory and file based variables
ERROR: Job failed: exit code 127
```

Lecture : `$ npm run lint` est la commande lancée ; `npm: not found` dit que le shell ne la connaît pas. Le code 127 signifie « commande introuvable » : l'`image` du job est la bonne piste, pas le script.

Un autre cas, quand une commande contient un `: ` non protégé. YAML la prend alors pour une paire clé/valeur au lieu d'un texte :

```yaml
script:
  - echo Version: 1.0
```

GitLab répond `yaml invalid` avec la ligne et la colonne. La correction est d'entourer toute la commande de guillemets : `- 'echo Version: 1.0'`.

## Outils pour avancer sans polluer l'historique

- **Valider le YAML** dans l'éditeur de pipeline de GitLab avant de pousser.
- **Relancer un job** depuis l'interface plutôt que de refaire un commit.
- **Lancer la commande en local** dans la même image : `docker run --rm -it node:24-alpine sh` ouvre un shell dans l'image, où tu tapes les commandes du `script`.
- **`allow_failure: true`** : MiniShop l'utilise pour son job `sonarqube-check`, qui peut échouer sans bloquer le pipeline. C'est un choix à documenter, pas un moyen de cacher un job rouge.
- **Pousser sur une branche jetable** pour tester, puis regrouper (*squash*) les commits à la fusion : le squash fusionne tous les commits d'une branche en un seul.

:::warning N'affiche pas tes secrets pour déboguer
Pour voir une variable, `echo "$MON_SECRET"` est tentant. Ne le fais pas : le log est lisible par toutes les personnes ayant accès au projet. Teste plutôt sa présence avec `test -n "$MON_SECRET"`.
:::

## Entraîne-toi

:::info Diagnostiquer, sans runner
Il n'y a pas de runner dans ce labo : tu ne verras pas de vrais logs produits par GitLab. Tu as à la place des logs d'exemple (dossier `logs`) et des pipelines cassés, que `verifier-ci` sait diagnostiquer. Il reconnaît des erreurs de structure (stage inconnu, YAML invalide, variable mal orthographiée, commande absente de l'image) ; il ne reproduit pas les erreurs qui n'arrivent qu'à l'exécution (réseau, droits, tests qui échouent).
:::

:::lab
engine: real
intro: |
  Ton dossier de travail contient des logs d'exemple dans `logs/` et quatre pipelines cassés : `pipeline-image.yml`, `pipeline-yaml.yml`, `pipeline-absent.yml` et `pipeline-variable.yml`. Pour lire un log : `cat logs/job-lint.log`. Pour chercher un mot : `grep -n "not found" logs/job-lint.log`. Pour diagnostiquer un pipeline : `verifier-ci pipeline-image.yml`. Modifie les fichiers avec `nano` (Ctrl+O puis Entrée pour enregistrer, Ctrl+X pour quitter) ou l'éditeur de VS Code.
commands:
  - cp -R /opt/exercices/07-debogage/. .
steps:
  - text: 'Lis `logs/job-lint.log` : quel est le nom de la commande introuvable ? Écris-le dans un fichier `commande.txt`'
    hint: 'Cherche la ligne qui contient `not found` : le nom de la commande se trouve juste avant. Écris-le avec `echo NOM > commande.txt`.'
    checks:
      - env-file-contains: [commande.txt, '^\s*npm\s*$']
    solution:
      - echo npm > commande.txt
  - text: 'Ce log vient du pipeline `pipeline-image.yml` : le code de sortie 127 pointe l''`image`. Lance `verifier-ci pipeline-image.yml`, corrige le fichier (sans retirer le job `lint`) jusqu''à ce que `verifier-ci --strict pipeline-image.yml` réussisse'
    hint: 'Le job lance `npm`, mais l''image `python:3.13` ne contient que Python. Remplace-la par une image qui contient Node, par exemple `node:24-alpine`.'
    checks:
      - command-succeeds: verifier-ci --strict pipeline-image.yml
      - output-contains: ['verifier-ci --montrer pipeline-image.yml', '\[quality\] lint']
    solution:
      - write:
          pipeline-image.yml: |
            stages:
              - quality

            lint:
              stage: quality
              image: node:24-alpine
              script:
                - npm ci
                - npm run lint
  - text: 'Le log `logs/pipeline-invalide.log` annonce « yaml invalid » à la ligne 9 du fichier `pipeline-yaml.yml`. Repère la commande fautive, entoure-la de guillemets, jusqu''à ce que `verifier-ci pipeline-yaml.yml` réussisse'
    hint: 'La ligne 9 est `- echo Version: 1.0` : le `: ` (deux-points suivi d''une espace) est pris pour une clé YAML. Écris `- ''echo Version: 1.0''`.'
    checks:
      - command-succeeds: verifier-ci pipeline-yaml.yml
      - command-succeeds: "verifier-ci pipeline-yaml.yml --job build --cree --script-lance 'echo :: Version: 1.0'"
      - output-contains: ['verifier-ci --montrer pipeline-yaml.yml', '\[build\] build']
    solution:
      - write:
          pipeline-yaml.yml: |
            stages:
              - build

            build:
              stage: build
              image: alpine:3.20
              script:
                - echo "Début"
                - 'echo Version: 1.0'
                - echo "Fin"
  - text: 'Le job `deploiement` de `pipeline-absent.yml` n''apparaît jamais sur la branche `main`. Teste avec `verifier-ci --contexte CI_COMMIT_BRANCH=main pipeline-absent.yml`, corrige la règle : il doit exister sur `main` et seulement sur `main`'
    hint: 'Regarde le nom de branche écrit dans la condition : la branche principale s''appelle `main`, pas `master`.'
    checks:
      - command-succeeds: verifier-ci pipeline-absent.yml
      - command-succeeds: verifier-ci --contexte CI_PIPELINE_SOURCE=push --contexte CI_COMMIT_BRANCH=main pipeline-absent.yml | grep -q '^+ \[deploy\] deploiement'
      - command-succeeds: verifier-ci --contexte CI_PIPELINE_SOURCE=push --contexte CI_COMMIT_BRANCH=feature-x pipeline-absent.yml | grep -q '^- deploiement'
    solution:
      - write:
          pipeline-absent.yml: |
            stages:
              - build
              - deploy

            build:
              stage: build
              image: alpine:3.20
              script:
                - echo "Construction"

            deploiement:
              stage: deploy
              image: alpine:3.20
              script:
                - echo "Déploiement"
              rules:
                - if: '$CI_COMMIT_BRANCH == "main"'
  - text: 'Le log `logs/job-push.log` montre un `docker build -t ":main"` : le début du nom d''image est vide. Dans `pipeline-variable.yml`, `verifier-ci` repère une variable qui n''existe pas. Corrige-la partout jusqu''à ce que `verifier-ci --strict pipeline-variable.yml` réussisse'
    hint: 'La variable prédéfinie s''appelle `CI_REGISTRY_IMAGE` (avec un E à la fin). Une variable inconnue est simplement vide, sans erreur : d''où le nom d''image bancal.'
    checks:
      - command-succeeds: verifier-ci --strict pipeline-variable.yml
      - command-succeeds: "verifier-ci pipeline-variable.yml --job build --cree --script-lance 'docker push :: $CI_REGISTRY_IMAGE:'"
    solution:
      - write:
          pipeline-variable.yml: |-
            stages:
              - build

            build:
              stage: build
              image: docker:27
              services:
                - docker:dind
              variables:
                DOCKER_HOST: tcp://docker:2375
              script:
                - docker login -u "$CI_REGISTRY_USER" -p "$CI_REGISTRY_PASSWORD" "$CI_REGISTRY"
                - docker build -t "$CI_REGISTRY_IMAGE:$CI_COMMIT_REF_SLUG" .
                - docker push "$CI_REGISTRY_IMAGE:$CI_COMMIT_REF_SLUG"
:::

## Vérifie tes acquis

:::quiz
Un pipeline a trois stages et le job du premier stage est rouge. Par où commences-tu ?

- [ ] Par le job du dernier stage
- [ ] Par le message de fin `ERROR: Job failed` du dernier job
- [x] Par le log du job rouge du premier stage, en cherchant la première erreur

> Les stages suivants ne se lancent pas après un échec : la cause est dans le premier job rouge.
:::

:::quiz
Un job attendu dans une merge request est absent du pipeline. Quelle cause est la plus probable ?

- [ ] L'image Docker du job est trop ancienne
- [x] Aucune de ses `rules` ne correspond à ce pipeline
- [ ] Le registry du projet est plein

> Un job dont aucune règle ne correspond n'est pas ajouté au pipeline.
:::

:::quiz
Un log se termine par `exit code 127` avec `npm: not found`. Que vérifies-tu ?

- [ ] Que la branche est protégée
- [x] Que l'`image` du job contient bien Node et npm
- [ ] Que le pipeline est planifié

> Le code 127 veut dire « commande introuvable » : l'outil n'est pas dans l'image.
:::

:::quiz
GitLab refuse un pipeline avec « yaml invalid » à cause de la ligne `- echo Version: 1.0`. Comment la corriges-tu ?

- [ ] En supprimant la commande `echo`
- [x] En entourant toute la commande de guillemets : `- 'echo Version: 1.0'`
- [ ] En ajoutant une tabulation au début de la ligne

> Le `: ` fait croire à YAML qu'il lit une clé ; les guillemets en font un simple texte.
:::
