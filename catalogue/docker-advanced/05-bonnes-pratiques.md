---
id: bonnes-pratiques
title: Prêt pour la production
summary: 'Images légères (multi-stage), registry GitLab et réflexes de sécurité.'
minutes: 20
objectives:
  - "Réduire la taille d'une image avec un build multi-étapes"
  - "Étiqueter et publier une image sur un registry GitLab"
  - "Appliquer les réflexes de sécurité : utilisateur non-root, .dockerignore, versions épinglées, secrets hors de l'image"
---

Une image qui marche sur ton poste n'est pas forcément prête pour la production : trop lourde, trop de secrets, aucune traçabilité. Voici les pratiques qui font la différence.

## 1. Les builds multi-étapes

Pour *construire* un site, il faut Node.js et ses dépendances (≈ 1 Go). Pour le *servir*, un simple nginx suffit (≈ 40 Mo). Avec un Dockerfile multi-étapes, la première étape construit, la seconde ne garde que le résultat.

![L'étape de build (≈ 1,1 Go) est jetée ; seule l'image finale, légère, est publiée sur le registry](images/multistage.svg)

```dockerfile file=Dockerfile
FROM node:20 AS build
WORKDIR /app
COPY package.json .
COPY index.html .
RUN npm run build

FROM nginx:alpine
COPY --from=build /app/index.html /usr/share/nginx/html/index.html
```

## 2. Publier sur un registry

Dans l'équipe, la CI GitLab construit l'image à chaque push et la publie dans le registry du projet ; sur le serveur, un outil comme *Watchtower* (repérable aux labels `com.centurylinklabs.watchtower.enable` des fichiers `compose-infra.yaml`) détecte la nouvelle image et met à jour le conteneur. À la main, ça donne :

```shell run
docker tag site:leger registry.example.org/equipe/site:master
docker login registry.gitlab.example.org
docker push registry.example.org/equipe/site:master
```

```mermaid
flowchart LR
    A[git push] --> B[GitLab CI<br/>job docker]
    B -->|docker build| C[Image]
    C -->|docker push| D[(registry.gitlab.example.org)]
    D -->|pull| E[Serveur<br/>compose-infra.yaml<br/>+ Watchtower]
```

Extrait (simplifié) de la CI de Vitrine : le vrai job ajoute `--cache-from` et un `-f` vers son Dockerfile.

```yaml
docker:
  stage: build
  image: docker:latest
  before_script:
    - docker login -u "$CI_REGISTRY_USER" -p "$CI_REGISTRY_PASSWORD" $CI_REGISTRY
  script:
    - docker build --tag $CI_REGISTRY_IMAGE:$DOCKER_TAG .
    - docker push $CI_REGISTRY_IMAGE:$DOCKER_TAG
  services:
    - docker:dind
```

## 3. Les réflexes de sécurité et de propreté

:::cards
### Utilisateur non-root

Crée un utilisateur dédié puis ajoute `USER app` (ex. `RUN useradd -r app`) : si l'application est compromise, l'attaquant n'est pas root dans le conteneur. L'image Dockerfile de Vitrine fait exactement ça (utilisateur `django`).

### .dockerignore

Exclus `.git`, `node_modules`, `.env`… du contexte de build : plus rapide, et aucun secret dans l'image.

### Versions épinglées

`python:3.13-slim` plutôt que `python:latest` : des builds reproductibles. Même `nginx:alpine` évolue avec le temps : épingle la version précise (`nginx:1.27-alpine`) pour un déploiement critique.

### Secrets hors de l'image

Jamais de mot de passe dans le Dockerfile : variables d'environnement ou secrets au runtime.
:::

:::danger Les couches sont lisibles
Les couches d'une image sont **lisibles par tous ceux qui l'ont**. Un `COPY .env` suivi d'un `RUN rm .env` laisse le secret dans la couche précédente ! Un secret publié par erreur doit être considéré comme compromis : révoque-le.
:::

## À retenir

- Multi-étapes : on livre le résultat, pas les outils de build.
- Tag + login + push, de préférence automatisés par la CI.
- Pas de root, pas de secret dans l'image, des versions précises.

## Entraîne-toi

:::lab
intro: |
  Compare une image « naïve » et une image multi-étapes, puis publie la plus légère sur le registry (simulé).
files:
  package.json: |
    { "name": "site", "scripts": { "build": "echo build" } }
  index.html: |
    <h1>Site du club</h1>
  Dockerfile: |
    FROM node:20 AS build
    WORKDIR /app
    COPY package.json .
    COPY index.html .
    RUN npm run build

    FROM nginx:alpine
    COPY --from=build /app/index.html /usr/share/nginx/html/index.html
  Dockerfile.single: |
    FROM node:20
    WORKDIR /app
    COPY . .
    RUN npm run build
    CMD ["npx", "serve", "."]
steps:
  - text: "Construis l'image naïve : `docker build -t site:lourd -f Dockerfile.single .`"
    hint: "`-f` désigne le fichier de recette à utiliser (ici `Dockerfile.single`), `-t` donne le nom et l'étiquette `lourd`."
    checks:
      - image-present: 'site:lourd'
    solution:
      - 'docker build -t site:lourd -f Dockerfile.single .'
  - text: "Construis l'image multi-étapes : `docker build -t site:leger .`"
    hint: "Cette fois le fichier par défaut (`Dockerfile`, multi-étapes) suffit : seul le tag change."
    checks:
      - image-present: 'site:leger'
    solution:
      - 'docker build -t site:leger .'
  - text: 'Compare les tailles avec `docker images`'
    hint: "Compare la colonne SIZE des deux étiquettes `site`."
    after: [1, 2]
    checks:
      - command: ^docker images
    solution:
      - docker images
  - text: "Étiquette l'image légère pour le registry GitLab : `docker tag`"
    hint: "`docker tag SOURCE DESTINATION` : la destination commence par le nom du registry."
    checks:
      - image-present: 'registry.example.org/equipe/site:master'
    solution:
      - 'docker tag site:leger registry.example.org/equipe/site:master'
  - text: 'Connecte-toi : `docker login registry.gitlab.example.org`'
    hint: "Le nom du registry est celui qui commence l'étiquette de l'étape précédente."
    checks:
      - registry-logged-in: registry.gitlab.example.org
    solution:
      - docker login registry.gitlab.example.org
  - text: "Publie l'image : `docker push`"
    hint: "Pousse l'étiquette complète créée à l'étape 4."
    checks:
      - image-pushed: 'registry.example.org/equipe/site:master'
    solution:
      - 'docker push registry.example.org/equipe/site:master'
:::

## Vérifie tes acquis

:::quiz
Quel est l'intérêt d'un build multi-étapes ?

- [ ] Lancer plusieurs conteneurs en parallèle depuis un seul Dockerfile
- [x] Ne garder dans l'image finale que le nécessaire : image plus légère et plus sûre
- [ ] Chiffrer les couches de l'image
- [ ] Éviter d'écrire un Dockerfile

> Les outils de build (compilateurs, node_modules…) restent dans l'étape intermédiaire et ne sont pas livrés.
:::

:::quiz
Tu ajoutes COPY .env puis RUN rm .env dans un Dockerfile. Le secret est-il protégé ?

- [ ] Oui, il est supprimé
- [x] Non : il reste lisible dans la couche créée par COPY
- [ ] Oui, si on utilise --rm
- [ ] Oui, grâce à .dockerignore

> Chaque couche est conservée dans l'image. Ne copie jamais de secret : fournis-le au démarrage.
:::

:::quiz
Pourquoi préférer python:3.13-slim à python:latest ?

- [x] Pour avoir des builds reproductibles et une image plus petite
- [ ] Parce que latest n'existe pas
- [ ] Pour utiliser moins de CPU
- [ ] Parce que slim est plus récent

> latest change au fil du temps : ton build d'aujourd'hui pourrait casser demain.
:::

:::quiz
Où la CI de l'équipe publie-t-elle les images construites ?

- [ ] Sur le Docker Hub public
- [x] Dans le registry GitLab du projet (registry.example.org/equipe/…)
- [ ] Dans un dossier du serveur, copié à la main à chaque déploiement
- [ ] Dans le dépôt Git, à côté du code

> Chaque projet GitLab dispose de son registry d'images, utilisé par les jobs et par le déploiement.
:::
