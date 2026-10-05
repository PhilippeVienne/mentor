---
title: "Examen de validation — Docker advanced"
draw: 12
pass_mark: 80
minutes: 20
shuffle: true
---

Cet examen s'adresse aux personnes qui maîtrisent déjà Docker au quotidien (Dockerfile, volumes, réseaux, Compose, bonnes pratiques de production) et veulent **valider le parcours sans refaire les labos** ni les étapes des leçons. Si tu découvres le sujet, commence plutôt par les leçons : elles t'apportent aussi de l'XP.

## Comment ça se passe

- **12 questions** sont tirées au hasard dans un pool de **45 questions** couvrant toutes les leçons du parcours, puis **mélangées** (les réponses aussi).
- Tu as **20 minutes** et une seule tentative en cours à la fois.
- Il faut **au moins 80 %** de bonnes réponses pour réussir et valider le parcours.
- En cas d'échec, un court délai t'est demandé avant de pouvoir réessayer : le pool est tiré à nouveau, donc les questions changent.
- Une fois l'examen rendu, tu obtiens la **correction détaillée** de chaque question, avec l'explication.

## Le pool de questions

:::quiz
Dans un Dockerfile, à quel moment s'exécute l'instruction `RUN pip install -r requirements.txt` ?

- [x] Pendant la construction de l'image (`docker build`)
- [ ] Au démarrage de chaque conteneur
- [ ] Uniquement lors du premier `docker run`
- [ ] À chaque `docker exec`

> `RUN` ajoute une couche à l'image au moment du build. C'est `CMD` ou `ENTRYPOINT` qui définit ce qui est lancé au démarrage du conteneur.
:::

:::quiz
Tu modifies `app.py` et reconstruis l'image : l'étape `RUN pip install` est ré-exécutée alors que `requirements.txt` n'a pas changé. Voici le Dockerfile :

    FROM python:3.13-slim
    WORKDIR /app
    COPY . .
    RUN pip install -r requirements.txt
    CMD ["python", "app.py"]

Quelle est la cause ?

- [ ] `pip` ne sait pas réutiliser le cache de couches de Docker
- [ ] `WORKDIR` réinitialise tout le cache de couches à chaque construction
- [ ] `CMD` doit être placé avant `RUN` pour que les couches soient réutilisées
- [x] `COPY . .` précède `RUN pip install` : modifier le code invalide le cache

> Une couche modifiée invalide toutes les suivantes. En copiant d'abord `requirements.txt`, puis en installant les dépendances, et en copiant le reste du code ensuite, l'installation n'est refaite que si les dépendances changent.
:::

:::quiz
Que fait `EXPOSE 5000` dans un Dockerfile ?

- [ ] Il publie le port 5000 sur toutes les interfaces de la machine hôte
- [ ] Il ouvre le port 5000 dans le pare-feu de la machine hôte
- [x] Il documente le port d'écoute ; il ne publie rien sur la machine
- [ ] Il empêche tout autre conteneur d'utiliser ce port sur le réseau

> `EXPOSE` est une indication pour les personnes qui utiliseront l'image. L'accès depuis l'hôte demande `-p` à l'exécution (ou `ports:` dans Compose).
:::

:::quiz
Dans la commande `docker build -t demo-app .`, que représente le point final ?

- [ ] Le nom de l'image qui sera produite par la construction
- [x] Le contexte de build : les fichiers envoyés à Docker pour la construction
- [ ] Le port à publier quand le conteneur sera lancé plus tard
- [ ] Le dossier de destination dans lequel l'image sera enregistrée

> Le contexte détermine ce que `COPY` peut atteindre. `-t` donne le nom (et éventuellement le tag) de l'image.
:::

:::quiz
Quelle forme de `CMD` est recommandée pour lancer une application ?

- [ ] La forme *shell* : `CMD python app.py`, toujours préférable
- [x] La forme *exec* en liste JSON : `CMD ["python", "app.py"]`
- [ ] Aucune : on ne définit jamais de `CMD`
- [ ] `CMD` doit contenir un `RUN` pour fonctionner

> Avec la forme exec, le processus de l'application est directement le processus principal et reçoit proprement les signaux d'arrêt. La forme shell passe par `/bin/sh -c` qui peut les intercepter.
:::

:::quiz
Quelle est la différence entre `ENV` et `ARG` dans un Dockerfile ?

- [x] `ARG` n'existe qu'au build ; `ENV` reste dans le conteneur à l'exécution
- [ ] `ENV` n'existe que pendant le build ; `ARG` reste visible dans le conteneur
- [ ] Les deux sont strictement équivalents, `ARG` n'est qu'un ancien nom
- [ ] `ARG` ne peut pas recevoir de valeur, contrairement à `ENV`

> `ARG` se passe avec `--build-arg` et sert à paramétrer la construction. `ENV` fixe une variable visible par les processus du conteneur (et surchargeable avec `-e`).
:::

:::quiz
À quoi sert `WORKDIR /app` ?

- [ ] À copier le contenu du dossier `/app` de ta machine vers l'image
- [ ] À monter `/app` comme volume partagé avec ta machine
- [ ] À changer l'utilisateur qui exécute les instructions suivantes
- [x] À fixer le dossier de travail des instructions suivantes et du conteneur

> `WORKDIR` évite de multiplier les `cd`. Les `COPY` avec un chemin relatif et les `RUN` suivants s'exécutent dans ce dossier.
:::

:::quiz
Tu construis ton image avec `docker build -t demo-app:1.0 .`. Quel est le nom complet de l'image obtenue ?

- [ ] `demo-app` avec le tag `latest`
- [ ] `1.0` avec le tag `demo-app`
- [x] `demo-app` avec le tag `1.0`
- [ ] `demo-app:1.0` est le nom du contexte

> Le format est `nom:tag`. Sans tag, Docker utilise `latest`, qui n'est pas forcément la version la plus récente mais simplement le tag par défaut.
:::

:::quiz
Pour que l'image de ton application Flask soit joignable sur `localhost:5000`, que faut-il faire en plus du Dockerfile ?

- [ ] Ajouter une seconde instruction `EXPOSE` pour le port 5000 dans le Dockerfile
- [ ] Ajouter l'option `-d` à la commande `docker build` de l'image
- [x] Publier le port au lancement : `docker run -p 5000:5000 demo-app`
- [ ] Rien : `EXPOSE 5000` suffit à rendre le port accessible

> L'instruction `EXPOSE` ne sert qu'à documenter. C'est la publication de port, au lancement ou dans Compose, qui rend le service atteignable depuis la machine.
:::

:::quiz
Tu supprimes un conteneur PostgreSQL lancé sans aucun volume. Que deviennent ses données ?

- [ ] Elles sont conservées dans l'image `postgres` pour le prochain conteneur
- [x] Elles sont perdues : elles vivaient dans la couche du conteneur
- [ ] Elles sont copiées automatiquement dans un dossier de ta machine
- [ ] Elles restent dans un volume anonyme, qui n'est jamais supprimé

> Sans volume, tout ce qui est écrit dans le conteneur disparaît avec lui. Pour une base de données, on monte un volume sur le dossier de données (par exemple `/var/lib/postgresql/data`).
:::

:::quiz
Que se passe-t-il avec `docker run -v donnees:/app/data alpine` si le dossier `./donnees` existe sur ta machine ?

- [x] Il crée ou réutilise un volume nommé `donnees`, pas ton dossier
- [ ] Il monte ton dossier `./donnees` dans le conteneur, car il existe déjà
- [ ] Il copie le contenu de `./donnees` dans l'image au démarrage
- [ ] Il échoue, car le nom `donnees` est ambigu entre dossier et volume

> Un nom simple (sans `/` initial) désigne un volume nommé géré par Docker. Pour monter un dossier de ta machine, il faut un chemin absolu, par exemple `$(pwd)/donnees`.
:::

:::quiz
Quel est l'effet de `:ro` dans `-v $(pwd):/usr/share/nginx/html:ro` ?

- [ ] Le dossier est chiffré (*read-only encryption*)
- [ ] Le dossier est monté en lecture et écriture, avec sauvegarde automatique
- [ ] Le dossier est supprimé à l'arrêt du conteneur
- [x] Le conteneur peut lire les fichiers montés, mais pas les modifier

> `ro` signifie *read-only*. C'est une bonne protection quand un conteneur n'a besoin que de lire (page statique, configuration).
:::

:::quiz
Tu supprimes un conteneur qui utilisait le volume nommé `pgdata` avec `docker rm`. Le volume existe-t-il encore ?

- [ ] Non, il est supprimé automatiquement avec son dernier conteneur
- [ ] Oui, mais il est vidé automatiquement à la suppression du conteneur
- [ ] Non, sauf si le conteneur était déjà arrêté au moment du `docker rm`
- [x] Oui : il faut le supprimer avec `docker volume rm pgdata`

> Les volumes nommés sont indépendants des conteneurs. Ils survivent à `docker rm` et ne se suppriment qu'à la demande (impossible tant qu'un conteneur l'utilise).
:::

:::quiz
Pourquoi un bind mount (`-v $(pwd):/app`) est-il très pratique en développement ?

- [ ] Il accélère le démarrage du conteneur grâce au cache de l'hôte
- [ ] Il sauvegarde automatiquement le projet sur GitLab à chaque modification
- [x] Tes modifications sont visibles dans le conteneur sans reconstruire l'image
- [ ] Il rend le conteneur plus sécurisé en isolant le dossier de l'hôte

> Le bind mount partage un dossier réel de l'hôte avec le conteneur. Attention : en production on préfère des volumes nommés, car les chemins de l'hôte et leurs droits varient d'une machine à l'autre.
:::

:::quiz
Que se passe-t-il si deux conteneurs montent le même volume nommé ?

- [ ] Docker refuse le second montage tant que le premier conteneur tourne
- [x] Ils partagent les mêmes fichiers : ce que l'un écrit, l'autre le lit
- [ ] Chaque conteneur reçoit sa propre copie indépendante du volume
- [ ] Le second conteneur écrase le volume et supprime le premier

> C'est un moyen simple d'échanger des fichiers entre conteneurs. Attention toutefois aux écritures concurrentes sur les mêmes fichiers.
:::

:::quiz
Tu veux vérifier si un volume nommé `donnees` existe. Quelle commande utilises-tu ?

- [x] `docker volume ls`
- [ ] `docker ps -v`
- [ ] `docker images --volumes`
- [ ] `docker inspect --all`

> `docker volume ls` liste les volumes, et `docker volume inspect donnees` en détaille un. `docker ps` ne parle que des conteneurs.
:::

:::quiz
Tu veux conserver les données d'une base PostgreSQL entre deux versions de l'image. Quelle approche convient ?

- [x] Monter un volume nommé sur `/var/lib/postgresql/data` et le réutiliser
- [ ] Écrire les données de la base dans l'image au moment du `docker build`
- [ ] Lancer le conteneur avec l'option `--rm` pour conserver ses fichiers
- [ ] Commiter les fichiers de la base dans Git avant chaque mise à jour

> Les données doivent vivre en dehors de l'image et du conteneur, qui sont jetables. Un volume nommé est conçu pour cela.
:::

:::quiz
Un conteneur nginx est lancé avec un bind mount vers ton dossier, mais `curl` renvoie une page « 403 Forbidden ». Quelle est la cause plausible ?

- [ ] Le port 80 du conteneur n'est pas publié sur la machine hôte
- [ ] L'image nginx est trop ancienne pour servir des fichiers montés
- [ ] Le conteneur n'a pas de réseau pour répondre aux requêtes
- [x] Le dossier monté ne contient pas de fichier `index.html`

> nginx sert le contenu de `/usr/share/nginx/html`. Si le dossier monté est vide ou n'a pas d'`index.html`, il refuse de lister son contenu et répond 403.
:::

:::quiz
Ton application Python tourne dans un conteneur et doit joindre PostgreSQL dans un autre conteneur nommé `db`, sur un même réseau Docker. Quelle adresse de base de données utilises-tu ?

- [ ] `localhost:5432`, car la base est sur la même machine
- [ ] `127.0.0.1:5432`, l'adresse locale de la machine hôte
- [x] `db:5432`, le nom du conteneur sert de nom d'hôte
- [ ] L'adresse IP de ta machine, vue depuis le réseau local

> Dans un conteneur, `localhost` désigne le conteneur lui-même, pas ta machine ni ses voisins. Sur un réseau défini par l'utilisateur, Docker résout le nom des conteneurs.
:::

:::quiz
Peut-on joindre un conteneur par son nom sur le réseau `bridge` par défaut ?

- [ ] Oui, toujours : tous les conteneurs partagent un DNS commun
- [x] Non : le nom ne se résout que sur un réseau créé par l'utilisateur
- [ ] Oui, mais seulement si les deux conteneurs ont une option `--name`
- [ ] Non, il faut obligatoirement utiliser une adresse IP fixe

> Sur le réseau `bridge` par défaut, on est réduit aux adresses IP, qui changent. Créer un réseau (`docker network create`) donne un DNS interne.
:::

:::quiz
Faut-il publier le port 5432 de la base pour que l'application, sur le même réseau Docker, s'y connecte ?

- [ ] Oui, avec `-p 5432:5432`, sans quoi l'application ne peut pas se connecter
- [x] Non : sur un même réseau, le port est accessible sans `-p`
- [ ] Oui, avec `EXPOSE 5432` dans le Dockerfile de l'application
- [ ] Non, mais il faut obligatoirement un volume partagé entre eux

> La publication de ports ne sert que pour l'accès depuis la machine hôte. Ne pas la faire pour une base de données est même une bonne pratique de sécurité.
:::

:::quiz
Comment rattaches-tu un conteneur au réseau `demo-net` au lancement ?

- [x] `docker run --network demo-net ...`
- [ ] `docker run -n demo-net ...`
- [ ] `docker run -v demo-net ...`
- [ ] `docker run --net-host demo-net ...`

> L'option est `--network` (ou `--net`). Le réseau doit exister (`docker network create demo-net`), sinon Docker répond qu'il est introuvable.
:::

:::quiz
`docker exec web ping db` répond « bad address 'db' ». Les deux conteneurs tournent. Voici leurs réseaux :

    web : demo-net
    db  : bridge

Quelle est la cause ?

- [ ] Le conteneur `db` est arrêté, donc son nom n'est plus résolu
- [ ] `ping` n'est pas installé dans l'image du conteneur `web`
- [ ] Il manque `-p 5432:5432` sur `web` pour joindre la base
- [x] Ils sont sur deux réseaux différents : `web` ne résout pas `db`

> Chaque réseau Docker est isolé. Pour que `web` joigne `db`, rattache les deux au même réseau (`docker network connect` ou `--network` au lancement).
:::

:::quiz
Quelle est une raison de ne pas publier le port de ta base de données avec `-p` ?

- [ ] Elle fonctionnerait plus lentement à cause de la redirection de port
- [ ] Docker refuse de publier les ports des bases de données connues
- [x] Elle n'est joignable que depuis le réseau Docker : moins de surface d'attaque
- [ ] Elle consommerait davantage de mémoire sur la machine hôte

> Publier un port l'expose à tout ce qui atteint la machine. Si seule l'application a besoin de la base, le réseau Docker suffit.
:::

:::quiz
Quelle commande permet de voir quels conteneurs sont rattachés au réseau `demo-net` ?

- [ ] `docker network ls demo-net`
- [ ] `docker ps --network`
- [x] `docker network inspect demo-net`
- [ ] `docker inspect --all`

> `inspect` détaille un réseau : sous-réseau, conteneurs connectés et leurs adresses. `docker network ls` ne liste que les réseaux.
:::

:::quiz
`docker exec web ping db` répond « executable file not found in $PATH ». Que signifie ce message ?

- [ ] Le conteneur `db` n'existe pas ou n'a jamais été créé sur ce réseau
- [x] L'image de `web` n'a pas la commande `ping` : ce n'est pas le réseau
- [ ] Le réseau `demo-net` est mal configuré : son DNS interne est désactivé
- [ ] Docker interdit `ping` dans les conteneurs pour des raisons de sécurité

> Beaucoup d'images minimales n'embarquent pas `ping`. On peut tester autrement (par exemple avec `curl` ou `python -c`) ou installer l'outil pour le diagnostic.
:::

:::quiz
Combien de réseaux existent par défaut dans une installation Docker toute neuve ?

- [x] Trois : `bridge`, `host` et `none`
- [ ] Un seul : `bridge`
- [ ] Aucun, il faut en créer au moins un
- [ ] Un par conteneur

> `docker network ls` les montre. `bridge` est utilisé par défaut, `host` partage la pile réseau de l'hôte et `none` désactive le réseau.
:::

:::quiz
Quel est l'intérêt principal de décrire une application dans un fichier `compose.yml` ?

- [ ] Il rend les conteneurs plus rapides à démarrer que `docker run`
- [ ] Il remplace Docker : plus besoin du démon pour exécuter les conteneurs
- [ ] Il sert uniquement à construire des images à partir de Dockerfiles
- [x] L'environnement est décrit dans un fichier versionné et lancé en une commande

> Compose décrit plusieurs services, leurs réseaux et leurs volumes. Chaque personne de l'équipe obtient le même environnement avec `docker compose up -d`.
:::

:::quiz
Quelle est la différence entre `docker compose down` et `docker compose down -v` ?

- [ ] Le second supprime aussi les images téléchargées pour les services
- [ ] Le premier arrête seulement les conteneurs, sans les supprimer
- [ ] Aucune : `-v` signifie *verbose* et ne change que l'affichage
- [x] Le second supprime aussi les volumes déclarés, donc les données

> `down` supprime conteneurs et réseau, mais conserve les volumes. `-v` supprime aussi les volumes : à manier avec précaution pour les bases de données.
:::

:::quiz
Ton service `web` déclare `depends_on: [db]`. Au démarrage, `web` échoue car la base n'est pas encore prête à accepter des connexions. Pourquoi ?

- [ ] `depends_on` n'est plus pris en compte par Compose depuis la version 2
- [ ] Il faut ajouter `restart: never` au service `web` pour qu'il patiente
- [x] `depends_on` règle l'ordre de démarrage, pas la disponibilité : il faut un `healthcheck`
- [ ] Compose démarre tous les services en même temps, quoi qu'on écrive dans le fichier

> Docker sait que le conteneur `db` est démarré, pas que PostgreSQL accepte des connexions. Un `healthcheck` sur la base, combiné à `condition: service_healthy`, attend qu'elle réponde.
:::

:::quiz
Après `docker compose up -d`, `docker compose ps` montre `db` en état « Exited (1) » alors que `web` tourne. Quelle commande donne la cause ?

- [ ] `docker compose down`
- [x] `docker compose logs db`
- [ ] `docker compose ps --all`
- [ ] `docker compose restart web`

> Les logs du service en échec expliquent la plupart des problèmes (par exemple, un mot de passe PostgreSQL manquant). On corrige ensuite le fichier et on relance.
:::

:::quiz
Tu ajoutes une variable d'environnement au service `db` dans `compose.yml`, puis tu relances `docker compose up -d`. Que fait Compose ?

- [x] Il recrée seulement `db` (configuration modifiée) et laisse les autres
- [ ] Il ne fait rien tant qu'on n'a pas lancé d'abord `docker compose down`
- [ ] Il recrée tous les conteneurs du projet à chaque `up`, quoi qu'on change
- [ ] Il met à jour la variable dans le conteneur existant sans le recréer

> Compose compare la configuration souhaitée à l'existant et ne recrée que ce qui a changé.
:::

:::quiz
Dans un `compose.yml`, comment publies-tu le port 80 du service `web` sur le port 8080 de la machine ?

- [x] Avec une entrée `"8080:80"` sous `ports:` dans le service `web`
- [ ] Avec une entrée `"8080:80"` sous `expose:` dans le service `web`
- [ ] Avec une entrée `"80:8080"` sous `ports:` dans le service `web`
- [ ] Avec une entrée `8080` sous `network:` dans le service `web`

> Le format est identique à celui de `docker run -p` : port de la machine d'abord, port du conteneur ensuite. Les guillemets évitent une interprétation incorrecte par YAML.
:::

:::quiz
Un projet nommé `projet` déclare le volume `dbdata` dans son `compose.yml`. Quel est le nom du volume créé par Compose ?

- [ ] `dbdata`
- [ ] `volume_dbdata`
- [ ] `projet-dbdata-1`
- [x] `projet_dbdata`

> Compose préfixe les ressources par le nom du projet (par défaut, celui du dossier). Le conteneur `db` s'appellerait lui `projet-db-1`.
:::

:::quiz
Les projets de l'équipe utilisent souvent `docker-compose.yml` en local et `compose-infra.yaml` en production. Où place-t-on les valeurs sensibles comme le secret d'une base ?

- [ ] Directement dans le fichier `compose.yml`, qui est versionné avec le code
- [ ] Dans le `Dockerfile`, avec une instruction `ENV` à la construction
- [x] Dans des variables d'environnement fournies à l'exécution (`.env` non versionné)
- [ ] Dans le nom du service, séparé du reste par un tiret

> Le fichier Compose décrit l'architecture ; les secrets se passent par l'environnement, à travers l'interpolation `${VARIABLE}`. Un secret versionné est un secret exposé.
:::

:::quiz
Pourquoi les services d'un même `compose.yml` se joignent-ils par leur nom (par exemple `db`) ?

- [ ] Parce que Docker modifie le fichier `/etc/hosts` de ta machine hôte
- [x] Parce que Compose crée un réseau du projet avec un DNS interne
- [ ] Parce que tous les conteneurs utilisent `--network host` par défaut
- [ ] Parce que les noms de services sont enregistrés sur le Docker Hub

> C'est le même mécanisme que pour un réseau créé à la main, mais Compose le fait pour toi : le nom du service est le nom d'hôte.
:::

:::quiz
Quel est l'intérêt d'un Dockerfile multi-étapes (*multi-stage*) ?

- [ ] Il permet de construire plusieurs images en parallèle sans utiliser le cache
- [x] L'image finale ne garde que le résultat, sans les outils de compilation
- [ ] Il lance plusieurs conteneurs différents à partir d'un seul Dockerfile
- [ ] Il supprime l'obligation d'écrire `FROM` au début de chaque étape

> Une première étape compile (avec Node.js, un compilateur…) et une seconde, souvent plus légère comme `nginx:alpine`, ne récupère que le produit final avec `COPY --from=`. L'image est plus petite et expose moins de logiciels.
:::

:::quiz
Un Dockerfile contient `COPY .env .` puis `RUN rm .env`. Le secret est-il protégé ?

- [x] Non : il reste lisible dans la couche créée par `COPY`
- [ ] Oui : `RUN rm` le supprime de l'image
- [ ] Oui, car `.env` est un fichier caché
- [ ] Oui, si l'image est privée, quelle que soit la personne qui la récupère

> Chaque instruction crée une couche immuable ; supprimer le fichier dans une couche suivante ne l'enlève pas de la précédente. On fournit les secrets à l'exécution (variables d'environnement, secrets).
:::

:::quiz
Pourquoi préfère-t-on `FROM python:3.13-slim` à `FROM python:latest` ?

- [ ] Parce que le tag `latest` n'existe pas pour les images Python
- [ ] Parce que `slim` démarre plus vite que `latest` sur la même machine
- [ ] Parce que `latest` est une version payante du Docker Hub
- [x] Pour avoir des builds reproductibles et une image plus petite

> Le tag `latest` évolue : le même Dockerfile peut produire des images différentes d'un mois à l'autre, avec de nouvelles versions majeures. Un tag précis fige la base.
:::

:::quiz
Pourquoi ajoute-t-on une instruction `USER` avec un compte non privilégié dans un Dockerfile ?

- [ ] Pour que l'image soit plus légère à télécharger et à stocker
- [ ] Pour accélérer le démarrage du conteneur et du processus
- [x] Pour limiter les dégâts si l'application est compromise
- [ ] Pour cacher le code de l'application aux autres personnes

> Par défaut, un processus de conteneur est root dans son espace de noms. Réduire les privilèges est une bonne pratique de défense en profondeur.
:::

:::quiz
À quoi sert un fichier `.dockerignore` ?

- [ ] À empêcher Docker de démarrer certains conteneurs sur la machine
- [ ] À lister les images à ne jamais supprimer avec `docker rmi`
- [x] À exclure des fichiers (`.git`, `.env`…) du contexte de build
- [ ] À masquer des ports aux autres conteneurs du réseau

> Le contexte est envoyé au démon : l'alléger accélère le build et évite que des secrets ou des dossiers volumineux se retrouvent dans l'image par un `COPY . .`.
:::

:::quiz
Tu as construit `site:leger`. Quelles commandes permettent de la publier sur le registry GitLab du projet `equipe/dev/site` ?

- [ ] `docker push site:leger` seul suffit, Docker devine le registry à utiliser
- [x] `docker tag` vers `registry.gitlab.example.org/…`, `docker login`, puis `docker push`
- [ ] `git push` envoie aussi les images Docker construites localement
- [ ] `docker upload site:leger` puis `docker publish` sur GitLab

> Le nom de l'image doit contenir l'adresse du registry et le chemin du projet. L'authentification (`docker login`) est nécessaire avant le `push`.
:::

:::quiz
`docker push` répond « denied: requested access to the resource is denied ». Quelle est une cause fréquente ?

- [x] Tu n'es pas connecté·e au registry, ou le nom ne correspond pas au projet
- [ ] L'image est trop volumineuse pour être publiée sur le registry
- [ ] Ta version de Docker est trop ancienne pour parler au registry
- [ ] Le conteneur de l'image est encore en cours d'exécution

> Ce message signale un refus d'accès : il manque un `docker login`, ou le chemin de l'image ne correspond pas à un dépôt sur lequel tu as le droit d'écrire.
:::

:::quiz
Dans un `.gitlab-ci.yml`, le job de build utilise `image: docker:latest` et `services: [docker:dind]`. À quoi sert le service `docker:dind` ?

- [ ] À héberger le registry dans lequel les images sont publiées
- [ ] À lancer les tests unitaires du projet dans un conteneur dédié
- [ ] À copier le dépôt sur le serveur de production après le build
- [x] À fournir un démon Docker au job pour `docker build` et `docker push`

> Le job tourne lui-même dans un conteneur : `dind` (*Docker in Docker*) lui donne un démon auquel se connecter. Les variables `CI_REGISTRY_IMAGE` et `CI_REGISTRY_USER` fournissent le nom d'image et l'authentification.
:::

:::quiz
Laquelle de ces pratiques réduit le plus la taille d'une image Python ?

- [ ] Ajouter plusieurs `RUN apt-get install` supplémentaires pour tout prévoir
- [ ] Utiliser `latest` pour toujours avoir la dernière version disponible
- [ ] Copier tout le dépôt, y compris `.git`, avec `COPY . .` pour aller vite
- [x] Une base `slim` ou `alpine`, et ne copier que le nécessaire (`.dockerignore`)

> La taille vient de l'image de base et de tout ce qu'on y ajoute. Une base légère, un contexte filtré et des builds multi-étapes sont les leviers principaux.
:::
