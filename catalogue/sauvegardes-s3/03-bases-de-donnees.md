---
id: bases-de-donnees
titre: "Sauvegarder PostgreSQL et MySQL"
resume: "Faire un dump logique d'une base, l'envoyer vers un stockage objet sans exposer de mot de passe, et comprendre comment le planifier."
duree: 35
objectifs:
  - Expliquer ce que produit `pg_dump` et pourquoi on ne copie pas les fichiers d'une base en marche
  - Lire la configuration de l'image `docker-postgres-backup`, y compris sa commande `docker run` et sa fréquence cron
  - Repérer les défauts du script MySQL de l'équipe
  - Sauvegarder une vraie base PostgreSQL vers un bucket S3, de deux façons
---

Copier le dossier de données d'une base pendant qu'elle tourne donne une copie incohérente : un fichier peut être copié au milieu d'une écriture. Pour une base, on utilise un **dump logique** (en français, un « export ») : un fichier produit par la base elle-même, qui décrit ses tables et son contenu, cohérent à l'instant de la commande.

## Les deux outils

PostgreSQL et MySQL sont deux **SGBD** (systèmes de gestion de base de données), des logiciels qui rangent des données dans des tables et répondent à des requêtes écrites en **SQL**. Chacun fournit son outil de dump :

| Base | Outil | Résultat |
| --- | --- | --- |
| PostgreSQL | `pg_dump` | un fichier de commandes SQL de recréation (`.sql`), ou un fichier binaire compressé (format *custom*, option `-Fc`) |
| MySQL | `mysqldump` | un fichier `.sql` équivalent |

Pour restaurer, on rejoue ce fichier : avec `psql` (le client en ligne de commande de PostgreSQL, qui envoie du SQL à la base) ou `mysql` pour un fichier `.sql`, avec `pg_restore` pour un fichier `-Fc`.

## Comment envoyer le dump vers S3

Tu connais déjà `mc` et `aws` (leçon 2). Il y a deux façons d'envoyer un dump :

- **via un fichier** : `pg_dump` écrit un fichier local, puis un outil l'envoie ;
- **en flux** : la sortie de `pg_dump` est branchée directement sur l'outil d'envoi avec une **redirection** `|` (le « tube » : la sortie de la commande de gauche devient l'entrée de la commande de droite). Aucun fichier intermédiaire n'est écrit.

```bash
pg_dump -Fc -f asso.dump
pg_dump | gzip | aws s3 cp - s3://sauvegardes-bd/asso.sql.gz
```

- `pg_dump -Fc -f asso.dump` : `-Fc` choisit le format *custom* (compressé), `-f asso.dump` nomme le fichier de sortie. Dans le labo, la connexion à la base `asso` est déjà réglée pour toi. Hors labo, on ajoute `-h` (machine du serveur), `-U` (utilisateur) et `-d` (base).
- `pg_dump | gzip | aws s3 cp - s3://sauvegardes-bd/asso.sql.gz` : sans `-f`, `pg_dump` écrit son SQL sur sa sortie ; `gzip` le compresse ; `aws s3 cp -` lit ce flux (le `-` signifie « l'entrée standard ») et en fait l'objet `asso.sql.gz` du bucket `sauvegardes-bd`.

Le mot de passe de la base ne se tape jamais dans la commande : utilise la variable d'environnement `PGPASSWORD` ou un fichier `~/.pgpass`. Une **variable d'environnement** est un réglage que le système donne aux programmes qu'il lance ; elle n'apparaît pas dans la ligne de commande.

## L'image docker-postgres-backup

L'équipe maintient une image Docker (un modèle de conteneur prêt à l'emploi) qui lance `pg_dump` périodiquement. Son script lit des variables d'environnement.

| Variable | Rôle |
| --- | --- |
| `POSTGRES_HOST`, `POSTGRES_PORT` | Où se trouve la base |
| `POSTGRES_USER`, `POSTGRES_PASSWORD` | Compte utilisé |
| `POSTGRES_DB` | Base à sauvegarder (le script refuse de démarrer si elle est vide) |
| `CRON_TIME` | Fréquence, sous forme d'expression cron (voir plus bas) |
| `MAX_BACKUPS` | Nombre de sauvegardes locales conservées |
| `MINIO_HOST`, `MINIO_HOST_URL`, `MINIO_ACCESS_KEY`, `MINIO_SECRET_KEY`, `MINIO_BUCKET` | Envoi vers MinIO |

### La fréquence : le format cron

**Cron** est le planificateur de tâches des systèmes Linux. Sa syntaxe tient en cinq champs séparés par des espaces :

```text
minute  heure  jour-du-mois  mois  jour-de-la-semaine
```

Une `*` veut dire « toutes les valeurs ». La valeur par défaut de `CRON_TIME` est `0 0 * * *` : minute `0`, heure `0`, et `*` pour le reste, donc « chaque jour à minuit ». Autre exemple : `30 2 * * *` se lit « chaque jour à 2 h 30 ». Les modèles Kubernetes de l'équipe utilisent aussi des raccourcis comme `@daily` (une fois par jour).

### Lancer l'image avec docker run

```bash
docker run -d \
  --name docker-postgres-backup \
  --env POSTGRES_HOST=db \
  --env POSTGRES_PORT=5432 \
  --env POSTGRES_USER=sauvegarde \
  --env POSTGRES_PASSWORD=mot-de-passe-factice \
  --env POSTGRES_DB=portail \
  --volume sauvegardes:/backup \
  docker-postgres-backup
```

Ligne à ligne :

- `docker run` : crée et démarre un conteneur à partir d'une image. L'image à utiliser est le **dernier mot** de la commande, `docker-postgres-backup` ; le `\` en fin de ligne dit au terminal que la commande continue sur la ligne suivante.
- `-d` (*detach*) : lance le conteneur en arrière-plan, au lieu de bloquer ton terminal.
- `--name docker-postgres-backup` : donne un nom au conteneur, pour le retrouver ensuite (`docker exec`, `docker logs`).
- `--env NOM=valeur` : définit une variable d'environnement **dans** le conteneur. Ici, elles disent à l'image où est la base (`db`, port `5432`, le port habituel de PostgreSQL) et avec quel compte. Le mot de passe est factice.
- `--volume sauvegardes:/backup` : branche le volume Docker nommé `sauvegardes` (un espace de stockage géré par Docker, qui survit au conteneur) sur le dossier `/backup` du conteneur. Les sauvegardes y sont écrites et ne disparaissent pas si le conteneur est supprimé.

Chaque exécution écrit `/backup/AAAA.MM.JJ.HHMMSS.sql`. Si `pg_dump` échoue, le fichier partiel est supprimé. Quand `MINIO_HOST` est défini, le script envoie ensuite le contenu de `/backup` avec **restic**, un outil de sauvegarde. Il **chiffre** les données (il les rend illisibles sans la clé de l'outil) et les **déduplique** (il ne stocke qu'une seule fois les morceaux identiques d'une sauvegarde à l'autre, ce qui économise de la place).

:::info Un README partiellement dépassé
Le README de ce dépôt cite aussi des variables `PG_*` et l'image `jmcarbo/...` : il vient du projet d'origine. Fie-toi aux variables `POSTGRES_*` lues par `run.sh`.
:::

## Restaurer

```bash
docker exec docker-postgres-backup ls /backup
docker exec docker-postgres-backup /restore.sh /backup/2024.01.15.000000.sql
```

`docker exec NOM COMMANDE` lance une commande dans un conteneur déjà démarré. La première liste les sauvegardes, la seconde rejoue le fichier choisi (ici une date fictive) avec `psql`.

## MySQL vers Swift

**MySQL** est un autre SGBD très répandu. L'image `mysql-backup-container` fait un `mysqldump -A` (`-A` = toutes les bases), puis l'envoie avec le client `swift` (celui du stockage objet OpenStack, vu dans la leçon 2) dans le bucket `OS_BUCKET_NAME`. Le nom du fichier se termine par la date, par exemple `-2024-01-15_01h00.dump.sql`. Le chart Helm planifie le tout avec `schedule: 0 1 * * *`, soit chaque jour à 1 h.

:::warning Un mot de passe sur la ligne de commande
Ce script passe le mot de passe avec `-p${MYSQL_PASSWORD}`. Sur une machine partagée, il est visible dans la liste des processus. Préfère un fichier d'options ou la variable `MYSQL_PWD`, et signale-le à l'équipe si tu modifies l'image.
:::

## Entraîne-toi

Dans le labo, une vraie base PostgreSQL `asso` (deux associations, cinq adhérent·e·s, données inventées) tourne dans ton conteneur, ainsi qu'un MinIO local. Tu sauvegardes la base des deux façons vues plus haut, puis tu écris la ligne cron qui automatiserait la sauvegarde. `docker run` n'est pas exécuté ici (le conteneur n'a pas de Docker) : seule la partie `pg_dump` et S3 est pratiquée.

:::labo
moteur: reel
intro: |
  La base PostgreSQL `asso` et un MinIO local sont démarrés pour toi, avec des identifiants factices. `pg_dump`, `pg_restore`, `psql`, `aws` et `mc` sont déjà connectés à ces services (inutile de donner `-h` ni `-U`). Pour voir la base, essaie `psql -c "SELECT * FROM adherents;"`. Le script `backup.py`, version simplifiée de celui de `backups3`, envoie un fichier vers un bucket : `backup.py BUCKET FICHIER OBJET`. Les étapes sont vérifiées sur les fichiers et sur le contenu du bucket.
commandes:
  - demarrer-minio
  - demarrer-postgres
etapes:
  - texte: 'Fais un dump de la base `asso` au format *custom* dans le fichier `asso.dump`'
    indice: 'pg_dump -Fc -f asso.dump'
    verif:
      - fichier-existe-dans-env: asso.dump
      - commande-reussit: pg_restore -l asso.dump
    solution:
      - pg_dump -Fc -f asso.dump
  - texte: 'Crée le bucket `sauvegardes-bd` avec `mc mb`'
    indice: 'mc mb labo/sauvegardes-bd'
    verif:
      - commande-reussit: mc ls labo/sauvegardes-bd
    solution:
      - mc mb labo/sauvegardes-bd
  - texte: 'Envoie `asso.dump` dans le bucket `sauvegardes-bd`, sous le même nom, avec `backup.py`'
    indice: 'backup.py -v sauvegardes-bd asso.dump asso.dump'
    apres: [1, 2]
    verif:
      - commande-reussit: mc stat labo/sauvegardes-bd/asso.dump
    solution:
      - backup.py -v sauvegardes-bd asso.dump asso.dump
  - texte: 'Sans fichier intermédiaire, envoie un dump SQL compressé dans l''objet `asso.sql.gz` du même bucket, avec `pg_dump | gzip | aws s3 cp -`'
    indice: 'pg_dump | gzip | aws s3 cp - s3://sauvegardes-bd/asso.sql.gz'
    apres: [2]
    verif:
      - sortie-contient:
          - aws s3api head-object --bucket sauvegardes-bd --key asso.sql.gz --query ContentLength --output text
          - '^[1-9][0-9]*$'
      - commande-reussit: aws s3 cp s3://sauvegardes-bd/asso.sql.gz - | gzip -dc | grep -q "CREATE TABLE"
    solution:
      - pg_dump | gzip | aws s3 cp - s3://sauvegardes-bd/asso.sql.gz
  - texte: 'Écris dans `crontab.txt` la ligne cron qui lance `/usr/local/bin/sauvegarder.sh` chaque jour à 2 h 30 (cinq champs cron, puis la commande)'
    indice: 'Minute 30, heure 2, puis trois étoiles : echo ''30 2 * * * /usr/local/bin/sauvegarder.sh'' > crontab.txt'
    verif:
      - fichier-contient-dans-env: [crontab.txt, '^30 2 \* \* \* /usr/local/bin/sauvegarder\.sh\s*$']
    solution:
      - echo '30 2 * * * /usr/local/bin/sauvegarder.sh' > crontab.txt
:::

## Vérifie tes acquis

:::quiz
Pourquoi ne copie-t-on pas simplement les fichiers d'une base PostgreSQL en marche ?

- [ ] Ils sont toujours chiffrés
- [ ] Ils sont trop gros pour être copiés
- [x] La copie risque d'être incohérente pendant que la base écrit
- [ ] PostgreSQL l'interdit

> Un dump logique est produit par la base elle-même et cohérent à un instant donné.
:::

:::quiz
Que fait `docker-postgres-backup` si `pg_dump` échoue ?

- [x] Il supprime le fichier de sauvegarde incomplet
- [ ] Il envoie quand même le fichier vers MinIO
- [ ] Il redémarre la base
- [ ] Il double `MAX_BACKUPS`

> Le script journalise « Backup failed » puis supprime le fichier partiel.
:::

:::quiz
Que signifie l'expression cron `0 0 * * *` de `CRON_TIME` ?

- [ ] Toutes les minutes
- [ ] Une fois par semaine, le dimanche
- [x] Chaque jour à minuit
- [ ] Au démarrage du conteneur seulement

> Les cinq champs sont minute, heure, jour du mois, mois et jour de la semaine : minute 0, heure 0, le reste en `*`.
:::

:::quiz
Quel défaut présente la ligne `mysqldump ... -p${MYSQL_PASSWORD}` ?

- [ ] Elle sauvegarde une seule base
- [x] Le mot de passe apparaît dans les arguments du processus
- [ ] Elle écrase la base existante
- [ ] Elle ne fonctionne qu'en root (administrateur)

> Les arguments d'un processus sont visibles par les autres utilisateur·rice·s de la machine.
:::
