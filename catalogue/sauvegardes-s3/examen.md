---
titre: "Examen de validation — Sauvegardes et stockage objet"
tirage: 8
seuil: 80
duree: 12
melange: true
---

Cet examen valide les bases des sauvegardes dans l'équipe : règle 3-2-1, stockage objet, dumps de bases, volumes Kubernetes, rétention et restauration. Il s'adresse aux personnes qui ont déjà sauvegardé et restauré des données : les termes techniques ne sont pas redéfinis ici, ils le sont dans les leçons.

:::quiz
Quelle est la différence essentielle entre réplication et sauvegarde ?

- [ ] La réplication ne fonctionne qu'avec des bases de données relationnelles
- [x] La réplication propage aussi les erreurs, pas la sauvegarde datée
- [ ] Il n'y en a aucune
- [ ] La sauvegarde ne concerne que les fichiers, jamais les bases

> Une suppression accidentelle est répliquée aussitôt ; une sauvegarde datée permet de revenir en arrière.
:::

:::quiz
Un service exige de ne pas perdre plus de 4 heures de données. Quelle conséquence sur les sauvegardes ?

- [ ] Une sauvegarde hebdomadaire est suffisante si elle est testée
- [ ] Une copie hors site devient inutile
- [ ] Il faut doubler la durée de rétention du bucket
- [x] Une sauvegarde au moins toutes les 4 h

> Le RPO fixe l'écart maximal entre deux sauvegardes.
:::

:::quiz
Pourquoi une copie sur un second dossier du même disque est-elle insuffisante pour la règle 3-2-1 ?

- [x] Le disque est un point de défaillance commun
- [ ] Les dossiers sont toujours trop petits
- [ ] Elle n'est pas versionnée
- [ ] Elle n'est pas compressée automatiquement

> Le deuxième support doit être indépendant du premier.
:::

:::quiz
Quel élément identifie un objet dans un bucket ?

- [ ] Son adresse IP dans le cluster
- [ ] Le nom du pod qui l'a envoyé
- [x] Sa clé, c'est-à-dire son nom
- [ ] Le numéro du volume persistant qui l'héberge

> Chaque objet est repéré par sa clé ; les « dossiers » ne sont que des préfixes.
:::

:::quiz
Pourquoi l'équipe a-t-il choisi MinIO ?

- [ ] Parce qu'il n'a pas besoin de volume persistant
- [ ] Parce qu'il est le seul S3 qui fonctionne sur Kubernetes
- [ ] Parce qu'il chiffre tout sans aucune configuration
- [x] Pour éviter le coût, trop élevé, d'un S3 commercial

> La documentation du dépôt `cluster-configuration` donne ce raisonnement : pas de réimplémentation de S3.
:::

:::quiz
Une politique de bucket autorise `s3:PutObject` et `s3:GetObject` sur un seul bucket. Quel principe applique-t-on ?

- [ ] La haute disponibilité
- [x] Le principe du moindre privilège
- [ ] Le chiffrement de bout en bout
- [ ] La déduplication des données envoyées

> Chaque service n'a que les droits dont il a besoin.
:::

:::quiz
Quelle commande liste les versions d'un bucket avec leur date ?

- [ ] `aws s3api put-bucket-versioning --bucket B`
- [ ] `aws s3 ls --versions --bucket B`
- [x] `aws s3api list-object-versions --bucket B`
- [ ] `mc mb labo/B --region=ovh-gra5`

> `list-object-versions` affiche les `VersionId` et les `LastModified`, utiles pour restaurer.
:::

:::quiz
Que produit `pg_dump` sur une base PostgreSQL en marche ?

- [x] Un export cohérent à l'instant de la commande
- [ ] Les fichiers bruts du dossier de données, copiés à chaud
- [ ] Seulement les index de la base
- [ ] Uniquement les tables modifiées depuis hier

> Le dump est produit par la base, ce qui garantit la cohérence.
:::

:::quiz
Dans `docker-postgres-backup`, que contient `MAX_BACKUPS` ?

- [ ] La taille maximale d'une sauvegarde en Mio
- [x] Le nombre de sauvegardes locales gardées
- [ ] Le nombre de tentatives avant abandon
- [ ] La date de la plus ancienne sauvegarde conservée

> Quand la limite est atteinte, la plus ancienne sauvegarde locale est supprimée.
:::

:::quiz
Tu lis `mysqldump -A -u U -pMOTDEPASSE`. Quel est le problème ?

- [ ] L'option `-u` est interdite par MySQL
- [ ] Le dump est chiffré deux fois
- [ ] `-A` supprime toutes les bases au lieu de les exporter
- [x] Le mot de passe est visible dans la liste des processus

> Utilise un fichier d'options ou une variable d'environnement.
:::

:::quiz
Que fait l'option `-A` de `mysqldump` dans le script de l'équipe ?

- [x] Elle exporte toutes les bases
- [ ] Elle sauvegarde seulement la base nommée dans la commande
- [ ] Elle active la compression
- [ ] Elle supprime l'ancien dump avant d'en créer un nouveau

> `-A` signifie `--all-databases`.
:::

:::quiz
Quel argument de `backup.py` est obligatoire pour sauvegarder un dossier ?

- [ ] `-q`
- [ ] `-v`, qui affiche les messages d'information
- [x] `-c`
- [ ] `--date`, suivi d'une date en UTC

> Un dossier sans compression est refusé par le script.
:::

:::quiz
Où `backup.py` crée-t-il sa copie de travail avant l'envoi ?

- [ ] Dans `/etc`
- [ ] Directement dans le bucket S3, sans copie locale
- [ ] Dans le volume monté en lecture seule du CronJob
- [x] Dans `/tmp`

> Il copie dans `/tmp`, ce qui demande autant de place que les données.
:::

:::quiz
Pourquoi le volume est-il monté avec `readOnly: true` dans le modèle de CronJob ?

- [ ] Pour accélérer le démarrage du CronJob
- [x] Pour protéger les données lues par la sauvegarde
- [ ] Pour économiser des clés d'accès S3
- [ ] Pour activer le versionnement du bucket sans commande

> La sauvegarde lit seulement.
:::

:::quiz
Où doivent se trouver `ACCESSKEY` et `SECRETKEY` dans un déploiement Kubernetes ?

- [ ] Écrites en clair dans le YAML du CronJob
- [ ] Dans le Dockerfile de l'image
- [x] Dans un secret Kubernetes, créé à part
- [ ] Dans le README du dépôt, pour les retrouver

> Ainsi aucun secret n'est versionné dans Git.
:::

:::quiz
Le CronJob écrit toujours l'objet `backup.tgz`. Qu'est-ce qui permet de garder un historique ?

- [x] Le versionnement activé du bucket
- [ ] `restartPolicy: OnFailure` du CronJob
- [ ] Le nom du pod qui l'a envoyé
- [ ] Le paramètre `-q` (mode silencieux)

> Chaque envoi crée une version sans perdre les précédentes.
:::

:::quiz
Dans la politique `lifecycle_policy.json`, que supprime la règle au bout de 365 jours ?

- [ ] La version courante de chaque objet
- [x] Les versions devenues non courantes
- [ ] Le bucket entier, objets compris
- [ ] Les secrets Kubernetes associés

> `NoncurrentVersionExpiration` ne supprime que les anciennes versions.
:::

:::quiz
Quelle commande applique une règle de cycle de vie depuis un fichier JSON ?

- [ ] `aws s3api list-buckets --bucket B`
- [ ] `restore.py -c B f /data/`
- [ ] `mc admin user add labo A S`
- [x] `aws s3api put-bucket-lifecycle-configuration`

> On lui passe le fichier avec `--lifecycle-configuration file://…`.
:::

:::quiz
Tu veux restaurer la version d'un objet du 15 janvier à 02:00:00 UTC. Que fais-tu ?

- [x] Tu ajoutes l'option `--date` à `restore.py`
- [ ] Tu relances `backup.py` avec la date voulue
- [ ] Tu supprimes la version courante puis tu attends
- [ ] Tu règles le fuseau de ton poste sur UTC

> La date doit correspondre exactement à la version, en UTC.
:::

:::quiz
Que se passe-t-il si aucune version ne correspond exactement à `--date` ?

- [ ] Le script restaure la version la plus proche
- [ ] Le script crée une version vide du bucket
- [x] Le script s'arrête avec une erreur explicite
- [ ] Le script restaure la version la plus ancienne

> Il faut recopier la date de `list-object-versions`.
:::

:::quiz
Quelle est la meilleure première étape pour tester une restauration ?

- [ ] Écraser le volume de production pour gagner du temps
- [ ] Supprimer l'archive après téléchargement
- [ ] Désactiver le CronJob quelques jours, par précaution
- [x] Restaurer d'abord dans un volume de test

> Un test ne doit pas risquer les données de production.
:::

:::quiz
Pourquoi chronométrer un test de restauration ?

- [ ] Pour connaître le RPO de chaque bucket du cluster
- [x] Pour connaître le RTO réel de la procédure
- [ ] Pour calculer le coût mensuel de MinIO
- [ ] Pour réduire la taille de l'archive sauvegardée

> La durée mesurée est la seule valeur fiable du RTO.
:::

:::quiz
Tu dois donner à un service le droit d'écrire dans un seul bucket avec un `mc` récent. Quel enchaînement est correct ?

- [ ] `mc admin policy add` puis `mc cp`, sans utilisateur
- [ ] `mc mb`, puis donner les droits administrateur à l'utilisateur
- [x] `mc admin policy create`, `mc admin user add`, puis `mc admin policy attach`
- [ ] `mc admin user add` seul, la politique étant créée toute seule

> `mc admin policy add` est l'ancienne syntaxe : on crée la politique, on crée l'utilisateur, puis on les relie avec `attach`.
:::

:::quiz
Que signifie l'expression cron `30 2 * * *` ?

- [x] Chaque jour à 2 h 30 du matin
- [ ] Toutes les 30 minutes pendant 2 heures
- [ ] Le 30 de chaque mois à 2 h
- [ ] Le deuxième jour de la semaine à 30 h

> Les champs sont minute, heure, jour du mois, mois et jour de la semaine : minute 30, heure 2, le reste en `*`.
:::

:::quiz
Tu viens de restaurer un dump dans une base temporaire. Quelle vérification apporte le plus de preuves ?

- [ ] Constater que `pg_restore` n'a rien affiché
- [x] Compter les lignes d'une table clé et les comparer à l'original
- [ ] Vérifier que le fichier du dump n'est pas vide
- [ ] Relire le nom de la base créée

> Seul un contrôle du contenu prouve que les données sont revenues ; l'absence de message ne prouve rien.
:::

:::quiz
Que fait `pg_dump | gzip | aws s3 cp - s3://B/dump.sql.gz` ?

- [ ] Il copie le dossier de données de la base vers S3
- [ ] Il restaure un dump compressé dans la base
- [ ] Il crée un fichier `dump.sql.gz` sur le disque local, puis le supprime
- [x] Il envoie un dump compressé vers S3, sans fichier intermédiaire

> Chaque commande lit la sortie de la précédente grâce au tube `|` ; le `-` de `aws s3 cp` désigne l'entrée standard.
:::

:::quiz
Que signifie `NoncurrentDays: 30` dans une règle de cycle de vie ?

- [x] Une version remplacée est supprimée 30 jours après avoir été remplacée
- [ ] La version courante est supprimée après 30 jours
- [ ] Le bucket est vidé tous les 30 jours à minuit
- [ ] Les objets non courants sont rendus illisibles au bout de 30 jours

> `Noncurrent` désigne les versions qui ne sont plus la dernière ; le compte commence quand une nouvelle version les remplace.
:::

:::quiz
Un test de restauration mesuré donne 3 heures alors que le RTO attendu est 1 heure. Que fais-tu ?

- [ ] Tu notes 1 heure, puisque c'est l'objectif
- [ ] Tu ignores le résultat, un test ne compte pas
- [x] Tu consignes les 3 heures et tu améliores la procédure
- [ ] Tu supprimes la sauvegarde la plus ancienne

> Le RTO réel est la durée mesurée ; l'écart avec l'objectif justifie d'automatiser ou de documenter la restauration.
:::
