---
id: modeles-securite
title: "Les analyses de sécurité de GitLab"
summary: "SAST, Dependency Scanning, Secret Detection et Code Quality : les ajouter en quatre lignes et les régler."
minutes: 40
objectives:
  - Dire ce que détecte chacune des quatre analyses
  - Ajouter les modèles de sécurité à un nouveau projet
  - Exclure des chemins ou adapter les `rules` d'un job de modèle
  - Éviter le pipeline en double
---

Écrire un job de test, c'est du travail. Écrire un scanner de vulnérabilités, c'est un métier. Heureusement, GitLab fournit des **modèles** (*templates*) prêts à l'emploi : on les inclut, et leurs jobs apparaissent dans le pipeline.

Quelques mots, d'abord. Une **vulnérabilité** est une faiblesse du code (ou d'une bibliothèque) qu'une personne malveillante pourrait exploiter. Une **dépendance** est une bibliothèque dont ton projet a besoin. Un **fichier de verrouillage** (`package-lock.json`, `Pipfile.lock`…) liste les versions exactes de ces dépendances. Un **faux positif** est une alerte qui n'en est pas une.

## Les quatre analyses

:::cards
### SAST

*Static Application Security Testing* : analyse statique, c'est-à-dire sans exécuter le programme, de **ton code** à la recherche de motifs dangereux (injection SQL, usage risqué d'une fonction…). Jobs vus dans l'équipe : `semgrep-sast`, `kubesec-sast` pour les fichiers Kubernetes, `gitlab-advanced-sast`.

### Dependency Scanning

Compare les **dépendances** (Pipfile.lock, package-lock.json…) à une base de vulnérabilités connues. Job vu dans l'équipe : `gemnasium-dependency_scanning`.

### Secret Detection

Cherche des **secrets** (clés d'API, mots de passe, jetons) commités par erreur. Job : `secret_detection`.

### Code Quality

Mesure la **qualité** du code (complexité, duplications) et la signale dans la merge request. Job : `code_quality`.
:::

## Les ajouter à un projet

C'est tout l'intérêt : quatre lignes. Voici le début du `.gitlab-ci.yml` de Vitrine :

```yaml
include:
  - template: Jobs/SAST.gitlab-ci.yml
  - template: Jobs/Code-Quality.gitlab-ci.yml
  - template: Jobs/Dependency-Scanning.gitlab-ci.yml
  - template: Jobs/Secret-Detection.gitlab-ci.yml
```

`include` fusionne d'autres fichiers YAML dans le pipeline ; `template:` désigne un modèle fourni par GitLab, identifié par son nom de fichier. Rien n'est copié dans ton dépôt : GitLab ajoute les jobs du modèle au moment de créer le pipeline.

Le projet d'Adhésion utilise les mêmes modèles, avec une écriture sans le préfixe `Jobs/` (`template: SAST.gitlab-ci.yml`). Les deux fonctionnent ; sur un nouveau projet, reprends l'écriture de Vitrine, plus récente.

:::warning Les jobs des modèles sont rangés dans le stage `test`
Un **stage** est une étape du pipeline, déclarée dans la liste `stages` (leçon 2). Les jobs de ces modèles n'indiquent pas de `stage` : ils vont donc dans le stage `test`. Si ton fichier déclare ses propres `stages` **sans** `test`, GitLab refuse le pipeline avec une erreur du genre « chosen stage test does not exist ». Ajoute `test` à ta liste.
:::

## Régler les analyses

Les modèles sont configurés par des **variables** (un nom suivi d'une valeur, sous `variables:`). Vitrine écarte les dossiers qui ne sont pas du code maison, et active le SAST avancé :

```yaml
variables:
  GITLAB_ADVANCED_SAST_ENABLED: 'true'
  SAST_EXCLUDED_PATHS: coverage/**, doc/**, docker/**, node_modules/**, test/**, venv/**
  DS_EXCLUDED_PATHS: coverage/**, doc/**, docker/**, node_modules/**, test/**, venv/**
```

- `GITLAB_ADVANCED_SAST_ENABLED: 'true'` active la version avancée du SAST (le `'true'` est entouré de guillemets pour que ce soit un texte).
- `SAST_EXCLUDED_PATHS` liste les chemins que le SAST doit ignorer, séparés par des virgules. `**` remplace n'importe quelle suite de dossiers : `node_modules/**` désigne tout ce qui se trouve sous `node_modules` (les bibliothèques téléchargées, qui ne sont pas ton code).
- `DS_EXCLUDED_PATHS` fait la même chose pour le Dependency Scanning (*DS*).

Adhésion, de son côté, exclut `bash`, `helmchart` et `static` avec `SAST_EXCLUDED_PATHS`.

Pour changer **quand** un job tourne, tu le redéclares par son nom : GitLab fusionne ta définition avec celle du modèle. Ici, `semgrep-sast` ne tourne dans une merge request que si des fichiers `.js` ou `.py` changent :

```yaml
semgrep-sast:
  rules:
    - if: '$CI_PIPELINE_SOURCE == "merge_request_event"'
      changes:
        - "**/*.js"
        - "**/*.py"
    - if: '$CI_COMMIT_BRANCH && $CI_OPEN_MERGE_REQUESTS'
      when: never
    - if: '$CI_COMMIT_BRANCH'
      changes:
        - "**/*.js"
        - "**/*.py"
```

Lecture, règle par règle (GitLab applique **la première règle qui correspond** ; si aucune ne correspond, le job n'est pas créé) :

1. `$CI_PIPELINE_SOURCE == "merge_request_event"` est vrai dans un pipeline de merge request. `changes:` ajoute une condition : le job n'est créé que si, dans cette modification, un fichier correspond à l'un des motifs. `**/*.py` veut dire « n'importe quel fichier `.py`, dans n'importe quel dossier ».
2. `$CI_COMMIT_BRANCH && $CI_OPEN_MERGE_REQUESTS` est vrai quand le pipeline est celui d'une branche (`CI_COMMIT_BRANCH` n'est pas vide) **et** qu'une merge request est déjà ouverte pour elle (`CI_OPEN_MERGE_REQUESTS` n'est pas vide). `when: never` signifie « ne jamais créer le job » : cette règle l'exclut.
3. `$CI_COMMIT_BRANCH` seul est vrai pour tout pipeline de branche, mais seulement avec des fichiers `.js` ou `.py` modifiés.

La deuxième règle évite le **pipeline en double** : sans elle, un push sur une branche qui a une merge request ouverte lancerait un pipeline de branche **et** un pipeline de merge request, avec les mêmes jobs. Avec elle, seul le pipeline de merge request tourne.

:::warning Un scanner qui réussit ne prouve pas l'absence de problème
Ces outils ont des faux positifs et des angles morts. Un résultat vert n'est pas un certificat de sécurité ; un résultat rouge se lit, il ne se contourne pas en excluant le dossier qui gêne.
:::

:::info Où lire les résultats
Les alertes apparaissent dans la merge request et dans le rapport de vulnérabilités du projet. Selon l'offre GitLab, certaines vues sont limitées : en cas de doute, demande à l'équipe Infra.
:::

## Entraîne-toi

:::info Les modèles ne sont pas téléchargés dans ce labo
Le conteneur n'a pas de réseau et les vrais modèles de GitLab ne s'y trouvent pas. `verifier-ci` connaît seulement le **nom** de quelques jobs de ces modèles (`semgrep-sast`, `secret_detection`, `code_quality`, `gemnasium-dependency_scanning`…) et sait qu'ils vont dans le stage `test`. Il peut donc simuler tes `rules` sur ces jobs, mais il ne voit pas leur contenu réel (et il ignore les jobs que GitLab ajoute selon les langages détectés). Le résultat réel s'observe seulement dans GitLab.
:::

:::lab
engine: real
intro: |
  Ton dossier de travail contient un `.gitlab-ci.yml` à deux jobs (`build` et `deploiement`, dans les stages `build` et `deploy`), un dossier `app` avec un fichier Python et un `README.md`. Modifie le fichier avec `nano .gitlab-ci.yml` (Ctrl+O puis Entrée pour enregistrer, Ctrl+X pour quitter) ou l'éditeur de VS Code. Pour simuler un pipeline de merge request qui modifie `app/main.py` : `verifier-ci --contexte CI_PIPELINE_SOURCE=merge_request_event --modifie app/main.py .gitlab-ci.yml`.
commands:
  - cp -R /opt/exercices/05-securite/. .
steps:
  - text: 'Inclus les quatre modèles de sécurité comme Vitrine : `Jobs/SAST.gitlab-ci.yml`, `Jobs/Code-Quality.gitlab-ci.yml`, `Jobs/Dependency-Scanning.gitlab-ci.yml` et `Jobs/Secret-Detection.gitlab-ci.yml`, sous une clé `include:` en tête de fichier'
    hint: 'Copie le bloc `include:` de la leçon au tout début du fichier. Ensuite, lance `verifier-ci .gitlab-ci.yml` : il va se plaindre d''un stage, c''est l''étape suivante.'
    checks:
      - command-succeeds: "verifier-ci .gitlab-ci.yml --include-template Jobs/SAST.gitlab-ci.yml --include-template Jobs/Code-Quality.gitlab-ci.yml --include-template Jobs/Dependency-Scanning.gitlab-ci.yml --include-template Jobs/Secret-Detection.gitlab-ci.yml"
    solution:
      - write:
          .gitlab-ci.yml: |
            include:
              - template: Jobs/SAST.gitlab-ci.yml
              - template: Jobs/Code-Quality.gitlab-ci.yml
              - template: Jobs/Dependency-Scanning.gitlab-ci.yml
              - template: Jobs/Secret-Detection.gitlab-ci.yml

            stages:
              - build
              - deploy

            build:
              stage: build
              image: node:24-alpine
              script:
                - npm ci
                - npm run build

            deploiement:
              stage: deploy
              image: alpine:3.20
              script:
                - echo "Déploiement"
  - text: 'Corrige l''erreur signalée par `verifier-ci` : les jobs des modèles vont dans le stage `test`, qui n''est pas déclaré. Ajoute-le à `stages`, entre `build` et `deploy`. Vérifie avec `verifier-ci --montrer .gitlab-ci.yml` : on doit voir `semgrep-sast` et `secret_detection`'
    hint: 'La liste devient `build`, `test`, `deploy`, dans cet ordre (les analyses tournent après le build et avant le déploiement).'
    after: [1]
    checks:
      - command-succeeds: verifier-ci .gitlab-ci.yml
      - output-contains: ['verifier-ci --montrer .gitlab-ci.yml', '^stages : .*\btest\b']
      - output-contains: ['verifier-ci --montrer .gitlab-ci.yml', '\[test\] semgrep-sast']
      - output-contains: ['verifier-ci --montrer .gitlab-ci.yml', '\[test\] secret_detection']
    solution:
      - write:
          .gitlab-ci.yml: |
            include:
              - template: Jobs/SAST.gitlab-ci.yml
              - template: Jobs/Code-Quality.gitlab-ci.yml
              - template: Jobs/Dependency-Scanning.gitlab-ci.yml
              - template: Jobs/Secret-Detection.gitlab-ci.yml

            stages:
              - build
              - test
              - deploy

            build:
              stage: build
              image: node:24-alpine
              script:
                - npm ci
                - npm run build

            deploiement:
              stage: deploy
              image: alpine:3.20
              script:
                - echo "Déploiement"
  - text: 'Écarte les dossiers qui ne sont pas du code maison, comme Vitrine : ajoute une section `variables:` avec `SAST_EXCLUDED_PATHS` et `DS_EXCLUDED_PATHS`, toutes deux égales à `node_modules/**, test/**`'
    hint: 'Une section `variables:` au premier niveau du fichier, avec deux lignes `NOM: valeur`. La valeur est une liste de chemins séparés par des virgules.'
    after: [2]
    checks:
      - command-succeeds: verifier-ci .gitlab-ci.yml
      - command-succeeds: "verifier-ci .gitlab-ci.yml --variable-globale 'SAST_EXCLUDED_PATHS=node_modules/**, test/**' --variable-globale 'DS_EXCLUDED_PATHS=node_modules/**, test/**'"
    solution:
      - write:
          .gitlab-ci.yml: |
            include:
              - template: Jobs/SAST.gitlab-ci.yml
              - template: Jobs/Code-Quality.gitlab-ci.yml
              - template: Jobs/Dependency-Scanning.gitlab-ci.yml
              - template: Jobs/Secret-Detection.gitlab-ci.yml

            stages:
              - build
              - test
              - deploy

            variables:
              SAST_EXCLUDED_PATHS: node_modules/**, test/**
              DS_EXCLUDED_PATHS: node_modules/**, test/**

            build:
              stage: build
              image: node:24-alpine
              script:
                - npm ci
                - npm run build

            deploiement:
              stage: deploy
              image: alpine:3.20
              script:
                - echo "Déploiement"
  - text: 'Redéclare `semgrep-sast` avec des `rules` : dans un pipeline de merge request (`merge_request_event`), il ne doit exister que si un fichier `.py` change. Teste avec `--modifie app/main.py` (le job doit apparaître), puis avec `--modifie README.md` (il doit être absent)'
    hint: 'Ajoute un bloc `semgrep-sast:` avec `rules:`, une règle `- if: ''$CI_PIPELINE_SOURCE == "merge_request_event"''` et dessous `changes:` suivi de `- "**/*.py"`.'
    after: [3]
    checks:
      - command-succeeds: verifier-ci .gitlab-ci.yml
      - command-succeeds: verifier-ci --contexte CI_PIPELINE_SOURCE=merge_request_event --modifie app/main.py .gitlab-ci.yml | grep -q '^+ \[test\] semgrep-sast'
      - command-succeeds: verifier-ci --contexte CI_PIPELINE_SOURCE=merge_request_event --modifie README.md .gitlab-ci.yml | grep -q '^- semgrep-sast'
    solution:
      - write:
          .gitlab-ci.yml: |
            include:
              - template: Jobs/SAST.gitlab-ci.yml
              - template: Jobs/Code-Quality.gitlab-ci.yml
              - template: Jobs/Dependency-Scanning.gitlab-ci.yml
              - template: Jobs/Secret-Detection.gitlab-ci.yml

            stages:
              - build
              - test
              - deploy

            variables:
              SAST_EXCLUDED_PATHS: node_modules/**, test/**
              DS_EXCLUDED_PATHS: node_modules/**, test/**

            semgrep-sast:
              rules:
                - if: '$CI_PIPELINE_SOURCE == "merge_request_event"'
                  changes:
                    - "**/*.py"

            build:
              stage: build
              image: node:24-alpine
              script:
                - npm ci
                - npm run build

            deploiement:
              stage: deploy
              image: alpine:3.20
              script:
                - echo "Déploiement"
  - text: 'Complète les règles de `semgrep-sast` pour les pipelines de **branche** (`CI_PIPELINE_SOURCE` vaut alors `push`) : exclus d''abord le cas d''une branche qui a déjà une merge request ouverte (`$CI_COMMIT_BRANCH && $CI_OPEN_MERGE_REQUESTS`, avec `when: never`), puis lance le job pour une branche (`$CI_COMMIT_BRANCH`) si un fichier `.py` change'
    hint: 'Copie les deux dernières règles du bloc `semgrep-sast` de la leçon, en gardant seulement `**/*.py`. Pour tester : `--contexte CI_PIPELINE_SOURCE=push --contexte CI_COMMIT_BRANCH=ma-branche --modifie app/main.py`, avec puis sans `--contexte CI_OPEN_MERGE_REQUESTS=1`.'
    after: [4]
    checks:
      - command-succeeds: verifier-ci .gitlab-ci.yml
      - command-succeeds: verifier-ci --contexte CI_PIPELINE_SOURCE=push --contexte CI_COMMIT_BRANCH=ma-branche --modifie app/main.py .gitlab-ci.yml | grep -q '^+ \[test\] semgrep-sast'
      - command-succeeds: verifier-ci --contexte CI_PIPELINE_SOURCE=push --contexte CI_COMMIT_BRANCH=ma-branche --contexte CI_OPEN_MERGE_REQUESTS=1 --modifie app/main.py .gitlab-ci.yml | grep -q '^- semgrep-sast'
      - command-succeeds: verifier-ci --contexte CI_PIPELINE_SOURCE=merge_request_event --modifie app/main.py .gitlab-ci.yml | grep -q '^+ \[test\] semgrep-sast'
    solution:
      - write:
          .gitlab-ci.yml: |-
            include:
              - template: Jobs/SAST.gitlab-ci.yml
              - template: Jobs/Code-Quality.gitlab-ci.yml
              - template: Jobs/Dependency-Scanning.gitlab-ci.yml
              - template: Jobs/Secret-Detection.gitlab-ci.yml

            stages:
              - build
              - test
              - deploy

            variables:
              SAST_EXCLUDED_PATHS: node_modules/**, test/**
              DS_EXCLUDED_PATHS: node_modules/**, test/**

            semgrep-sast:
              rules:
                - if: '$CI_PIPELINE_SOURCE == "merge_request_event"'
                  changes:
                    - "**/*.py"
                - if: '$CI_COMMIT_BRANCH && $CI_OPEN_MERGE_REQUESTS'
                  when: never
                - if: '$CI_COMMIT_BRANCH'
                  changes:
                    - "**/*.py"

            build:
              stage: build
              image: node:24-alpine
              script:
                - npm ci
                - npm run build

            deploiement:
              stage: deploy
              image: alpine:3.20
              script:
                - echo "Déploiement"
:::

## Vérifie tes acquis

:::quiz
Quelle analyse repère une clé d'API commitée par erreur ?

- [ ] SAST
- [ ] Dependency Scanning
- [x] Secret Detection

> Secret Detection cherche des secrets dans le code et l'historique ; le SAST cherche des motifs de code dangereux.
:::

:::quiz
Une faille connue vient d'être publiée sur une bibliothèque listée dans ton `package-lock.json`. Quel job la signale ?

- [x] `gemnasium-dependency_scanning`
- [ ] `code_quality`
- [ ] `kubesec-sast`

> Le Dependency Scanning compare les versions de tes dépendances aux vulnérabilités publiées.
:::

:::quiz
À quoi sert la règle `- if: '$CI_COMMIT_BRANCH && $CI_OPEN_MERGE_REQUESTS'` avec `when: never` ?

- [ ] À forcer le job à tourner sur toutes les branches
- [ ] À bloquer la fusion de la merge request
- [x] À éviter un second pipeline pour une branche qui a déjà une merge request ouverte

> Sans elle, un push donnerait un pipeline de branche et un pipeline de merge request identiques.
:::

:::quiz
Tu déclares `stages: [build, deploy]` puis tu inclus `Jobs/SAST.gitlab-ci.yml`. Que se passe-t-il ?

- [ ] Le SAST s'exécute avant le stage `build`
- [ ] Les jobs du modèle sont ignorés sans message
- [x] GitLab refuse le pipeline : les jobs du modèle vont dans le stage `test`, absent de la liste

> Sans `stage` précisé, un job est rangé dans `test` : il faut déclarer ce stage.
:::
