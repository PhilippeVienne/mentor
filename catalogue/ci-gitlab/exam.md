---
title: "Examen de validation — CI/CD avec GitLab"
draw: 5
pass_mark: 80
minutes: 10
shuffle: true
---

Cet examen valide les bases de la CI/CD dans l'équipe : lecture d'un `.gitlab-ci.yml`, build d'image, secrets, analyses de sécurité, Renovate et débogage.

:::quiz
Un pipeline contient les stages `build`, `test` et `deploy`. Un job du stage `test` échoue. Que devient le stage `deploy` ?

- [x] Il ne démarre pas
- [ ] Il s'exécute quand même, en parallèle
- [ ] Il s'exécute, mais en mode simulation

> Un stage ne démarre que si les jobs du stage précédent ont réussi (sauf `allow_failure`).
:::
:::quiz
Dans une liste `rules`, plusieurs règles correspondent au pipeline. Laquelle est appliquée ?

- [ ] La dernière de la liste
- [ ] Toutes, combinées
- [x] La première qui correspond

> GitLab lit les règles de haut en bas et s'arrête à la première qui correspond.
:::
:::quiz
Que fait `when: never` dans une règle `rules` ?

- [ ] Le job est créé mais ne démarre jamais
- [x] Le job n'est pas créé quand la condition de la règle est vraie
- [ ] Le job est supprimé définitivement du fichier

> Avec `when: never`, le job est exclu du pipeline pour ce cas.
:::
:::quiz
Quelle variable prédéfinie donne le nom de la branche dans un pipeline de branche ?

- [ ] `CI_PIPELINE_SOURCE`
- [ ] `CI_REGISTRY_IMAGE`
- [x] `CI_COMMIT_BRANCH`

> `CI_PIPELINE_SOURCE` indique l'origine du pipeline, `CI_REGISTRY_IMAGE` le nom de l'image.
:::
:::quiz
Que contient `$CI_REGISTRY_IMAGE` ?

- [x] Le nom du registry du projet, sans tag
- [ ] Le mot de passe permettant de se connecter au registry du projet
- [ ] L'identifiant du dernier commit poussé dans le dépôt

> On y ajoute `:tag` pour désigner une image précise.
:::
:::quiz
Dans le job de build d'Adhésion, à quoi sert `DOCKER_HOST: tcp://docker:2375` ?

- [ ] À ouvrir un port du runner vers Internet pour que le registry y accède
- [x] À dire au client Docker où joindre le démon du service `docker:dind`
- [ ] À choisir la version de Docker utilisée pour construire l'image

> Le service `docker:dind` est joignable sous le nom d'hôte `docker`.
:::
:::quiz
Quel tag donne `$CI_COMMIT_REF_SLUG` pour la branche `fix/Page-Contact` ?

- [ ] `fix/Page-Contact`
- [ ] `fix_page_contact`
- [x] `fix-page-contact`

> Le slug est en minuscules, avec des `-` à la place des caractères non autorisés.
:::
:::quiz
Où stocker le secret d'un client Keycloak utilisé par un job de CI ?

- [x] Dans les variables CI/CD du projet, masquées
- [ ] Dans le `Dockerfile`, pour qu'il soit dans l'image
- [ ] Dans le fichier `README.md`

> Les variables de projet ne sont pas versionnées et peuvent être masquées dans les logs.
:::
:::quiz
Quel est l'effet de l'option « Masquée » (*Masked*) d'une variable ?

- [ ] Elle n'est disponible que sur les branches protégées
- [x] Sa valeur est remplacée dans les logs du job
- [ ] Elle est chiffrée dans la base de données uniquement

> « Protégée » contrôle où elle est disponible ; « Masquée » contrôle l'affichage dans les logs.
:::
:::quiz
Un mot de passe vient d'être poussé par erreur dans un commit de ta branche. Que fais-tu en priorité ?

- [ ] Je supprime le fichier dans un nouveau commit
- [ ] Je réécris l'historique et je ne dis rien
- [x] Je le révoque, j'en crée un autre et je préviens l'équipe Infra

> Le secret a pu être copié : il faut le considérer comme compromis.
:::
:::quiz
Quelle analyse compare les dépendances d'un projet à une base de vulnérabilités connues ?

- [ ] Code Quality
- [x] Dependency Scanning, qui lit les fichiers de verrouillage
- [ ] Secret Detection

> Code Quality mesure la maintenabilité, Secret Detection cherche des secrets.
:::
:::quiz
Que détecte le SAST ?

- [x] Des motifs dangereux dans le code source du projet
- [ ] Des versions obsolètes de bibliothèques
- [ ] Des tentatives d'intrusion sur le serveur en production

> SAST signifie *Static Application Security Testing* : une analyse du code, sans l'exécuter.
:::
:::quiz
Dans le `.gitlab-ci.yml` de Vitrine, quelle variable limite les dossiers analysés par le SAST ?

- [ ] `DOCKER_DRIVER`
- [ ] `GITLAB_ADVANCED_SAST_ENABLED`
- [x] `SAST_EXCLUDED_PATHS`

> `GITLAB_ADVANCED_SAST_ENABLED` active le SAST avancé, elle n'exclut aucun chemin.
:::
:::quiz
Pourquoi ajouter `- if: '$CI_COMMIT_BRANCH && $CI_OPEN_MERGE_REQUESTS'` avec `when: never` ?

- [ ] Pour ne lancer le job que sur la branche principale du projet
- [x] Pour éviter un pipeline en double avec une merge request
- [ ] Pour interdire aux personnes d'ouvrir des merge requests

> Seul le pipeline de merge request tourne alors, pas celui de la branche.
:::
:::quiz
Que fait Renovate sur un dépôt de l'équipe ?

- [x] Il ouvre des merge requests pour mettre à jour les dépendances
- [ ] Il lance les tests à la place de la CI
- [ ] Il déploie la dernière version en production

> Les merge requests du robot passent par la CI puis par une relecture humaine.
:::
:::quiz
Un job se termine avec `exit code 127` et `npm: not found`. Quelle est la piste la plus probable ?

- [ ] Le registry est saturé
- [ ] La branche n'est pas protégée
- [x] L'image du job ne contient pas Node et npm

> 127 signifie « commande introuvable » dans le shell.
:::
:::quiz
Tu cherches pourquoi un pipeline est rouge. Par quoi commences-tu ?

- [x] Par la première erreur du log du premier job rouge
- [ ] Par modifier le `.gitlab-ci.yml` et pousser pour voir
- [ ] Par relancer tous les jobs plusieurs fois

> Le message final d'un job ne donne que le code de sortie ; la cause est plus haut dans le log.
:::

:::quiz
Quel problème l'intégration continue cherche-t-elle d'abord à éviter ?

- [ ] Que le code soit publié sans relecture, pour aller plus vite
- [ ] Qu'un serveur de production tombe en panne un jour férié
- [x] Découvrir tard qu'une modification casse le projet
- [ ] Écrire le moindre fichier de configuration

> La CI vérifie chaque modification dès qu'elle est poussée, pour que l'erreur soit repérée tout de suite.
:::

:::quiz
Chaque version qui passe tous les contrôles est mise en production automatiquement. De quoi s'agit-il ?

- [x] De déploiement continu
- [ ] De livraison continue
- [ ] D'intégration continue seulement
- [ ] D'une revue de code obligatoire

> En livraison continue, la version est prête mais la mise en ligne reste manuelle ; en déploiement continu, elle est automatique.
:::

:::quiz
Dans le job d'Adhésion, que fait l'option `--fail` de la commande `curl` ?

- [ ] Elle affiche un message d'erreur même quand tout va bien
- [x] Elle fait échouer la commande si le serveur répond par une erreur HTTP
- [ ] Elle relance automatiquement la requête jusqu'à ce que le serveur réponde

> Sans `--fail`, `curl` réussit même quand le serveur répond par une page d'erreur : le job resterait vert à tort.
:::

:::quiz
Un projet déclare `stages: [build, deploy]` puis inclut `Jobs/Secret-Detection.gitlab-ci.yml`. Quelle est la conséquence ?

- [ ] Le job du modèle tourne avant tous les autres stages, sans rien changer d'autre
- [x] GitLab refuse le pipeline : le job du modèle va dans le stage `test`, absent de `stages`
- [ ] Le modèle est ignoré sans aucun message, puisque le stage `deploy` existe déjà

> Un job sans `stage` est rangé dans `test` : si ce stage est absent de `stages`, le pipeline est invalide.
:::

:::quiz
Un secret est affiché en base64 dans le log d'un job alors que la variable est « Masquée ». Pourquoi n'a-t-il pas été caché ?

- [ ] Le masquage ne fonctionne que dans les pipelines de merge request
- [ ] La variable n'était pas protégée, et seules les variables protégées sont masquées
- [x] GitLab remplace la valeur exacte, pas sa version transformée par un encodage

> Le masquage cherche la valeur telle quelle ; une version encodée n'est pas reconnue. D'où la règle : n'affiche jamais un secret.
:::
