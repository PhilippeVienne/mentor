## Règle 3-2-1 et objectifs

| Notion | Sens |
| --- | --- |
| 3-2-1 | 3 copies, 2 supports, 1 hors site |
| RPO | Données qu'on accepte de perdre (fréquence de sauvegarde) |
| RTO | Temps accepté pour redémarrer (facilité de restauration) |

## Outils de l'équipe

| Besoin | Dépôt | Cible |
| --- | --- | --- |
| Fichier ou volume vers S3 | `backups3` | S3 (`boto3`) |
| Fichier ou volume vers Swift | `backup-files-swift` | Swift |
| PostgreSQL | `docker-postgres-backup` | Dossier `/backup`, MinIO via restic |
| MySQL | `mysql-backup-container` | Swift |

## backups3

```bash
python backup.py -v -c BUCKET /data/ backup.tgz
python restore.py -v -c BUCKET backup.tgz /data/
python restore.py -v -c --date "AAAA-MM-JJ HH:MM:SS" BUCKET backup.tgz /data/
```

Dans les labos, `python` est omis : `backup.py` et `restore.py` sont directement dans le PATH.

Variables : `URL`, `ACCESSKEY`, `SECRETKEY` (secret Kubernetes). Dossier : `-c` obligatoire. Date en UTC.

## MinIO : bucket, politique, utilisateur (`mc` récent)

```bash
mc mb labo/B
mc admin policy create labo POLITIQUE politique.json
mc admin user add labo ACCES SECRET
mc admin policy attach labo POLITIQUE --user ACCES
```

`mc admin policy add` est l'ancienne syntaxe, encore utilisée par `cluster-configuration`.

## Dump PostgreSQL et cron

```bash
pg_dump -Fc -f asso.dump
pg_dump | gzip | aws s3 cp - s3://B/asso.sql.gz
createdb asso_restauration && pg_restore -d asso_restauration asso.dump
```

Cron : `minute heure jour-du-mois mois jour-de-la-semaine`, par exemple `0 0 * * *` (minuit) ou `30 2 * * *` (2 h 30).

## Versionnement et cycle de vie

```bash
aws --endpoint-url=URL s3api put-bucket-versioning --bucket B --versioning-configuration Status=Enabled
aws --endpoint-url=URL s3api list-object-versions --bucket B
aws --endpoint-url=URL s3api put-bucket-lifecycle-configuration --bucket B --lifecycle-configuration file://lifecycle_policy.json
```

## Avant de restaurer

Restaurer d'abord dans un volume de test · vérifier le contenu · chronométrer (RTO réel) · ne jamais écraser la production sans copie de l'état actuel.
