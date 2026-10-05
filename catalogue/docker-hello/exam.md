---
title: "Examen de validation — Docker hello world"
draw: 10
pass_mark: 80
minutes: 15
shuffle: true
---

Cet examen s'adresse aux personnes qui maîtrisent déjà les bases de Docker (images, conteneurs, ports, cycle de vie, logs) et veulent **valider le parcours sans refaire les labos** ni les étapes des leçons. Si tu découvres le sujet, commence plutôt par les leçons : elles t'apportent aussi de l'XP.

## Comment ça se passe

- **10 questions** sont tirées au hasard dans un pool de **36 questions** couvrant toutes les leçons du parcours, puis **mélangées** (les réponses aussi).
- Tu as **15 minutes** et une seule tentative en cours à la fois.
- Il faut **au moins 80 %** de bonnes réponses pour réussir et valider le parcours.
- En cas d'échec, un court délai t'est demandé avant de pouvoir réessayer : le pool est tiré à nouveau, donc les questions changent.
- Une fois l'examen rendu, tu obtiens la **correction détaillée** de chaque question, avec l'explication.

## Le pool de questions

:::quiz
Tu lances trois fois la commande `docker run -d nginx`. Combien d'images et de conteneurs obtiens-tu ?

- [x] Une image et trois conteneurs
- [ ] Trois images et trois conteneurs
- [ ] Une image et un seul conteneur
- [ ] Trois images et un seul conteneur

> L'image `nginx` est téléchargée une seule fois et sert de modèle : chaque `docker run` crée un nouveau conteneur à partir d'elle (avec un nom aléatoire si tu n'en donnes pas).
:::

:::quiz
À la première exécution de `docker run hello-world`, Docker affiche « Unable to find image 'hello-world:latest' locally » puis « Pulling from library/hello-world ». Que se passe-t-il ?

- [ ] La commande échoue : il faut d'abord télécharger l'image à la main avec `docker pull`
- [ ] L'image est reconstruite à partir des fichiers de ton dossier courant
- [ ] Le conteneur démarre sans image, avec un système de fichiers vide
- [x] L'image est téléchargée depuis le Docker Hub, puis le conteneur démarre

> Quand l'image n'existe pas en local, `docker run` fait un `docker pull` implicite. Un second lancement réutilise l'image déjà présente et démarre immédiatement.
:::

:::quiz
Quelle différence principale existe entre un conteneur et une machine virtuelle ?

- [ ] Le conteneur embarque son propre noyau Linux, plus léger que celui d'une machine virtuelle
- [ ] Le conteneur ne peut pas accéder au réseau de la machine hôte par défaut
- [x] Le conteneur partage le noyau de l'hôte, sans système d'exploitation complet
- [ ] La machine virtuelle ne sait pas utiliser d'images pré-construites

> Un conteneur n'isole que des processus et des fichiers, grâce aux fonctions du noyau de l'hôte. Il démarre donc en une seconde et pèse quelques Mo, contre plusieurs Go et une minute pour une VM.
:::

:::quiz
Dans un conteneur lancé depuis `nginx`, tu modifies un fichier. Que devient l'image `nginx` ?

- [ ] Elle est mise à jour et tous les futurs conteneurs verront la modification
- [x] Elle ne change pas : la modification vit dans la couche d'écriture propre au conteneur
- [ ] Elle est supprimée puis recréée automatiquement
- [ ] Elle est envoyée sur le Docker Hub

> Une image est en lecture seule. Chaque conteneur ajoute au-dessus sa propre couche inscriptible, qui disparaît avec lui. Pour conserver un changement dans une image, il faut en construire une nouvelle.
:::

:::quiz
Dans `docker images`, tu vois deux lignes : `nginx   latest` et `nginx   alpine`. Que représente la colonne TAG ?

- [ ] Le nom du conteneur qui utilise actuellement cette image
- [x] Une variante ou une version de la même image (ex. `alpine`)
- [ ] Le port sur lequel l'application de l'image écoute par défaut
- [ ] La date de création de l'image sur le Docker Hub

> Le tag distingue des versions d'un même dépôt d'images. `latest` est le tag utilisé si tu n'en précises aucun, et `alpine` désigne une variante construite sur la distribution Alpine, plus légère.
:::

:::quiz
Pourquoi dit-on que Docker aide à éviter le « ça marche chez moi » ?

- [x] Parce que l'application voyage avec ses dépendances dans une image
- [ ] Parce que Docker installe les mêmes logiciels sur toutes les machines de l'école
- [ ] Parce que Docker corrige automatiquement les bugs du code de l'application
- [ ] Parce que Docker remplace le système d'exploitation de la machine hôte

> Avec une image, la version de Python, les bibliothèques et les fichiers de configuration sont figés et partagés : on exécute le même environnement en local, en CI et en production.
:::

:::quiz
Que devient un conteneur lorsque son processus principal se termine ?

- [ ] Il est supprimé automatiquement dès que son processus se termine
- [ ] Il redémarre tout seul tant que Docker est actif sur la machine
- [ ] Il reste en cours d'exécution, mais sans aucun processus actif
- [x] Il s'arrête, mais existe encore (état « exited ») jusqu'à sa suppression

> Un conteneur vit aussi longtemps que son processus principal. Une commande ponctuelle comme `echo` le termine aussitôt ; il reste visible avec `docker ps -a` jusqu'à `docker rm` (ou l'option `--rm` au lancement).
:::

:::quiz
Parmi ces adresses, laquelle est typiquement celle d'une image hébergée dans le registry GitLab d'un projet de l'équipe ?

- [ ] `docker.io/library/mon-projet:latest`
- [ ] `https://gitlab.example.org/equipe/dev/mon-projet.git`
- [x] `registry.example.org/equipe/mon-projet:master`
- [ ] `equipe/mon-projet/Dockerfile`

> Le nom d'une image commence par l'adresse du registry, suivie du chemin du projet et d'un tag. L'URL en `.git` est celle du dépôt de code, pas celle d'une image.
:::

:::quiz
Que fait `docker --version` ?

- [ ] Elle liste les versions des images disponibles
- [ ] Elle met Docker à jour
- [x] Elle affiche la version du client Docker installé sur ta machine
- [ ] Elle affiche la version de l'application dans le conteneur

> C'est le premier test après l'installation. Elle n'interagit ni avec les images ni avec les conteneurs.
:::

:::quiz
Une application écoute sur le port 3000 à l'intérieur de son conteneur et tu veux y accéder par `localhost:9000`. Quelle option ajoutes-tu à `docker run` ?

- [ ] `-p 3000:9000`
- [x] `-p 9000:3000`
- [ ] `-p 9000`
- [ ] `-v 9000:3000`

> Le format est `port-de-la-machine:port-du-conteneur`. Dans `-p 3000:9000`, l'ordre est inversé, et `-v` sert à monter des volumes, pas à publier des ports.
:::

:::quiz
Quel est l'effet de l'option `-d` dans `docker run -d nginx` ?

- [x] Il tourne en arrière-plan et le terminal est rendu immédiatement
- [ ] Le conteneur est supprimé automatiquement dès qu'il s'arrête de lui-même
- [ ] Le conteneur démarre en mode débogage, avec des logs détaillés
- [ ] Le conteneur est téléchargé mais pas démarré avant le prochain run

> `-d` (*detach*) détache le conteneur du terminal : Docker affiche son identifiant et rend la main. Pour la suppression automatique, c'est `--rm`.
:::

:::quiz
Pourquoi `docker run alpine echo bonjour` n'affiche-t-il le message qu'une fois, puis s'arrête-t-il ?

- [ ] Parce qu'Alpine ne supporte qu'un seul processus à la fois
- [ ] Parce qu'il manque l'option `-d` pour garder le conteneur actif
- [ ] Parce que Docker supprime les conteneurs après chaque affichage
- [x] Parce que son processus principal, `echo`, se termine aussitôt

> La durée de vie d'un conteneur est celle de son processus principal. Un serveur web reste actif car son processus ne se termine pas.
:::

:::quiz
Ton conteneur nginx est lancé (`docker ps` le montre « Up »), mais `curl localhost:8080` répond « Failed to connect ». Voici la ligne de `docker ps` :

    CONTAINER ID   IMAGE   STATUS         PORTS    NAMES
    c3c6b0e9dce7   nginx   Up 5 seconds   80/tcp   web

Quelle est la cause la plus probable ?

- [ ] nginx n'écoute pas sur le port 80 mais sur un autre port interne
- [ ] Le conteneur est arrêté, donc il ne répond pas aux requêtes
- [ ] Il faut utiliser `127.0.0.1` à la place de `localhost` pour joindre nginx
- [x] Le port 80 n'est pas publié : il manque `-p 8080:80` au lancement

> La colonne PORTS ne montre pas de redirection du type `0.0.0.0:8080->80/tcp`. Le port existe dans le conteneur mais n'est pas atteignable depuis ta machine tant qu'on ne le publie pas.
:::

:::quiz
À quoi sert `docker logs web` ?

- [ ] À afficher la liste de tous les conteneurs de la machine
- [ ] À se connecter dans le conteneur pour y lancer des commandes
- [x] À afficher les sorties écrites par le conteneur `web`
- [ ] À enregistrer le journal des commandes tapées dans ton terminal

> Les logs d'un conteneur sont ses sorties. C'est le premier réflexe quand un conteneur ne fait pas ce qu'on attend, y compris s'il est déjà arrêté.
:::

:::quiz
Tu lances deux fois `docker run -d --name web nginx`. Que se passe-t-il à la seconde exécution ?

- [ ] Le premier conteneur est remplacé par le second
- [x] Docker refuse : le nom `web` est déjà utilisé par un autre conteneur
- [ ] Un second conteneur est créé et nommé automatiquement `web2`
- [ ] Les deux conteneurs partagent le même nom

> Les noms de conteneurs sont uniques, même pour un conteneur arrêté. Il faut le supprimer (`docker rm web`) ou choisir un autre nom.
:::

:::quiz
Dans la colonne PORTS de `docker ps`, tu lis `0.0.0.0:8080->80/tcp`. Que signifie cette ligne ?

- [x] Le port 8080 de la machine est redirigé vers le port 80 du conteneur
- [ ] Le port 80 de la machine est redirigé vers le port 8080 du conteneur
- [ ] Le conteneur utilise les ports 8080 à 80
- [ ] Le port 80 est fermé

> La flèche indique le sens de la redirection : de l'hôte (à gauche) vers le conteneur (à droite). `0.0.0.0` signifie que le port est ouvert sur toutes les interfaces de l'hôte.
:::

:::quiz
En lançant un second conteneur avec `-p 8080:80`, tu obtiens « port is already allocated ». Que faire ?

- [x] Choisir un autre port côté machine, par exemple `-p 8081:80`
- [ ] Choisir un autre port côté conteneur, par exemple `-p 8080:81`
- [ ] Redémarrer Docker pour libérer les ports
- [ ] Ajouter l'option `--force`

> Le conflit concerne le port de l'hôte, déjà pris par un autre conteneur ou un autre programme. Le port interne (80) est propre au conteneur et peut rester identique.
:::

:::quiz
Tu lances `docker run nginx` sans `-d`. Que se passe-t-il dans ton terminal ?

- [ ] Le conteneur ne démarre pas tant que l'option `-d` n'est pas fournie
- [ ] Le conteneur s'exécute une fois puis se supprime tout seul
- [ ] Docker te demande de choisir un nom avant de démarrer
- [x] Le terminal reste attaché au conteneur et affiche ses logs

> Sans `-d`, le processus du conteneur est au premier plan. Un serveur web monopolise donc le terminal jusqu'à son arrêt (par exemple par Ctrl+C).
:::

:::quiz
Quelle commande affiche aussi les conteneurs arrêtés ?

- [ ] `docker ps`
- [ ] `docker images -a`
- [x] `docker ps -a`
- [ ] `docker stop --all`

> `docker ps` seul n'affiche que les conteneurs en cours d'exécution. `-a` (*all*) ajoute ceux qui sont arrêtés, ce qui est indispensable pour comprendre un conteneur qui s'est terminé.
:::

:::quiz
Que fait `docker stop web` par rapport à `docker kill web` ?

- [ ] `stop` supprime définitivement le conteneur ; `kill` le met seulement en pause
- [x] `stop` demande un arrêt propre (SIGTERM), `kill` envoie SIGKILL tout de suite
- [ ] Les deux sont strictement identiques, c'est une question de préférence
- [ ] `stop` arrête tous les conteneurs ; `kill` seulement celui nommé `web`

> Un arrêt propre laisse le processus terminer ce qu'il fait (fermer des fichiers, des connexions). Docker attend 10 secondes par défaut avant de forcer. `kill` ne laisse aucune chance au processus.
:::

:::quiz
Un conteneur `web` est arrêté. Tu lances `docker start web`. Avec quelle configuration redémarre-t-il ?

- [ ] Avec la configuration par défaut de l'image, sans les ports publiés
- [x] Avec celle d'origine : même image, mêmes ports, mêmes variables
- [ ] Avec la configuration du dernier conteneur lancé sur la machine
- [ ] Il ne peut pas redémarrer : il faut le recréer avec `docker run`

> `docker start` relance le même conteneur : ses options (`-p`, `-e`, `--name`…) sont conservées, contrairement à un nouveau `docker run`.
:::

:::quiz
Tu essaies de supprimer un conteneur qui tourne avec `docker rm web`. Que répond Docker ?

- [x] Il refuse : il faut l'arrêter d'abord, ou utiliser `docker rm -f web`
- [ ] Il le supprime sans rien dire, même s'il est en cours d'exécution
- [ ] Il le met en pause puis le supprime au prochain redémarrage
- [ ] Il supprime aussi l'image associée au conteneur sans demander

> `docker rm` protège un conteneur en cours d'exécution. `-f` arrête puis supprime en une seule commande.
:::

:::quiz
Tu essaies de supprimer l'image `nginx` avec `docker rmi nginx`, mais un conteneur arrêté en vient. Que se passe-t-il ?

- [ ] L'image est supprimée, et le conteneur arrêté l'est aussi avec elle
- [ ] L'image est supprimée mais le conteneur reste fonctionnel
- [ ] Docker supprime l'image en ignorant l'existence du conteneur
- [x] Docker refuse tant que ce conteneur existe (sauf avec `-f`)

> Un conteneur dépend de son image, même arrêté. On supprime les conteneurs d'abord, puis les images devenues inutiles.
:::

:::quiz
Quel est l'intérêt de `docker run --rm alpine echo test` ?

- [ ] Il supprime l'image `alpine` une fois la commande exécutée
- [ ] Il redémarre le conteneur à chaque erreur de son processus
- [x] Le conteneur est supprimé automatiquement dès qu'il s'arrête
- [ ] Il supprime les volumes du conteneur ainsi que ceux des autres

> `--rm` convient aux commandes ponctuelles : plus de conteneur arrêté à nettoyer. L'image reste en cache pour le prochain lancement.
:::

:::quiz
Dans `docker ps -a`, tu lis `Exited (1) 2 minutes ago` pour un conteneur. Que signifie `(1)` ?

- [ ] Le nombre de fois où le conteneur a redémarré depuis sa création
- [ ] Le nombre de conteneurs arrêtés sur la machine à ce moment-là
- [x] Le code de sortie : une valeur différente de 0 indique une erreur
- [ ] Le numéro du port utilisé par le conteneur avant son arrêt

> Un code `0` signale une fin normale. Un code non nul signale un échec : on consulte `docker logs` pour en connaître la cause.
:::

:::quiz
Que fait `docker restart web` ?

- [ ] Il recrée un nouveau conteneur à partir de l'image d'origine
- [x] Il arrête puis redémarre `web`, avec sa configuration d'origine
- [ ] Il supprime le conteneur et télécharge à nouveau l'image depuis le Docker Hub
- [ ] Il redémarre le démon Docker, donc tous les conteneurs de la machine

> `restart` équivaut à `stop` puis `start` sur le même conteneur. Les modifications du dossier de travail du conteneur sont conservées, mais pas celles faites dans l'image (elle est immuable).
:::

:::quiz
Pour nettoyer la machine, dans quel ordre doit-on supprimer les éléments ?

- [x] D'abord les conteneurs, puis les images qui n'ont plus de conteneur
- [ ] D'abord les images, puis les conteneurs
- [ ] Peu importe : Docker supprime les dépendances tout seul
- [ ] Seulement les conteneurs : les images se suppriment automatiquement

> Les images en cours d'utilisation par un conteneur ne peuvent pas être supprimées. On retire donc d'abord les conteneurs.
:::

:::quiz
Tu lances `docker run -d --name db postgres` et `docker ps` ne montre rien. Le conteneur s'est arrêté. Quelle est la première étape de diagnostic ?

- [ ] `docker rm db` puis relancer la même commande jusqu'à ce que ça marche
- [ ] `docker rmi postgres` pour télécharger l'image à nouveau depuis le registry
- [ ] Redémarrer la machine hôte pour libérer les ressources du conteneur
- [x] `docker ps -a` pour le code de sortie, puis `docker logs db` pour l'erreur

> Un conteneur arrêté garde ses logs. Pour PostgreSQL, ils indiquent qu'il faut définir `POSTGRES_PASSWORD`.
:::

:::quiz
Comment passes-tu un mot de passe à PostgreSQL au démarrage d'un conteneur ?

- [ ] Avec l'option `-p POSTGRES_PASSWORD=secret` au moment du lancement
- [ ] En modifiant l'image `postgres` directement sur le Docker Hub
- [ ] En créant un fichier du même nom dans ton dossier de travail
- [x] Avec une variable d'environnement : `-e POSTGRES_PASSWORD=secret`

> `-e` (ou `--env`) définit des variables d'environnement dans le conteneur. L'image de PostgreSQL les lit au démarrage pour initialiser la base.
:::

:::quiz
Quelle différence existe entre `docker run` et `docker exec` ?

- [ ] `run` s'utilise pour les images ; `exec` s'utilise uniquement pour les volumes
- [ ] `exec` crée un nouveau conteneur, alors que `run` en supprime un
- [x] `run` crée un nouveau conteneur ; `exec` agit dans un conteneur existant
- [ ] Il n'y a aucune différence, ce sont deux alias de la même commande

> `docker exec -it db sh` ouvre un shell dans `db` sans créer de nouveau conteneur ; il suppose que `db` tourne déjà.
:::

:::quiz
Tu tapes `docker run -it --rm alpine sh`, puis `exit` dans le shell. Qu'advient-il du conteneur ?

- [ ] Il continue de tourner en arrière-plan avec un autre processus
- [x] Il s'arrête, car le shell se termine, puis `--rm` le supprime
- [ ] Il redémarre aussitôt avec un nouveau shell interactif
- [ ] Il est mis en pause jusqu'à la prochaine connexion au terminal

> Le shell était le processus principal. Quand il se termine, le conteneur s'arrête, et `--rm` le supprime.
:::

:::quiz
À quoi servent les options `-it` dans `docker run -it ubuntu bash` ?

- [x] À garder l'entrée ouverte (`-i`) et à allouer un terminal (`-t`)
- [ ] À installer (`-i`) puis à télécharger (`-t`) l'image avant le lancement
- [ ] À lancer deux conteneurs en parallèle avec la même image
- [ ] À rendre le conteneur invisible dans la sortie de `docker ps`

> Sans `-it`, un shell se terminerait aussitôt faute d'entrée. Avec, tu peux taper des commandes comme dans un terminal.
:::

:::quiz
Tu veux vérifier les variables d'environnement d'un conteneur `db` en cours d'exécution. Que lances-tu ?

- [x] `docker exec db printenv`
- [ ] `docker run db printenv`
- [ ] `docker logs db --env`
- [ ] `docker rm db printenv`

> `exec` exécute `printenv` dans le conteneur existant. `docker run db` essaierait de créer un conteneur à partir d'une image qui s'appellerait `db`.
:::

:::quiz
Que renvoie `docker inspect db` ?

- [ ] La liste complète des fichiers présents à l'intérieur du conteneur
- [ ] Les logs du conteneur depuis son démarrage, ligne par ligne
- [ ] Un shell interactif dans le conteneur pour l'explorer
- [x] Des détails sur le conteneur au format JSON (état, réseau, montages)

> `docker inspect` expose la configuration complète : ports publiés, variables d'environnement, volumes, réseaux, code de sortie. Pratique pour un diagnostic précis.
:::

:::quiz
Un conteneur `pg` est arrêté depuis hier (`Exited (1)`). Peux-tu encore lire ses logs ?

- [ ] Non, les logs disparaissent dès que le conteneur s'arrête
- [ ] Oui, mais seulement après avoir redémarré le conteneur avec `docker start pg`
- [x] Oui, avec `docker logs pg`, tant que le conteneur n'est pas supprimé
- [ ] Non, seul `docker inspect` conserve les messages d'erreur

> Les logs sont rattachés au conteneur. Ils survivent à son arrêt et disparaissent avec `docker rm`, d'où l'intérêt de diagnostiquer avant de nettoyer.
:::

:::quiz
La commande `docker run -d --name db -e POSTGRES_PASSWORD=pw postgres` affiche un long identifiant puis rend la main. Que signifie ce comportement ?

- [ ] Le conteneur a échoué : l'identifiant est un code d'erreur
- [x] Le conteneur est démarré en arrière-plan et l'identifiant affiché est le sien
- [ ] L'image vient d'être téléchargée mais le conteneur n'est pas démarré
- [ ] Le conteneur est en pause

> En mode détaché, Docker affiche seulement l'identifiant du conteneur créé. Pour savoir s'il tourne encore, on utilise `docker ps` puis `docker logs db`.
:::
