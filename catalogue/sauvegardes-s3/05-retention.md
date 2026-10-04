---
id: retention
titre: "Versionnement et politiques de conservation"
resume: "Garder un historique utile sans remplir le stockage : activer le versionnement d'un bucket et lui appliquer une règle de cycle de vie."
duree: 30
objectifs:
  - Activer le versionnement d'un bucket et lister les versions d'un objet
  - Lire et appliquer une politique de cycle de vie
  - Choisir une durée de conservation adaptée
---

Sauvegarder chaque nuit sous le même nom écrase la veille. Si une corruption passe inaperçue pendant trois jours, il n'y a plus de bonne copie. Deux mécanismes du bucket règlent ça : le **versionnement** et le **cycle de vie**.

## Le versionnement

Quand il est activé, écrire un objet qui existe déjà **crée une nouvelle version** au lieu de remplacer l'ancienne. Chaque version a un identifiant propre, le `VersionId`. La dernière est la version *courante*, les autres sont *non courantes*.

```bash
aws s3api put-bucket-versioning \
  --bucket mon-bucket \
  --versioning-configuration Status=Enabled
```

- `aws s3api put-bucket-versioning` : la requête S3 qui règle le versionnement d'un bucket.
- `--bucket mon-bucket` : le bucket concerné.
- `--versioning-configuration Status=Enabled` : le nouveau réglage, `Enabled` (activé). Le `\` en fin de ligne dit au terminal que la commande continue sur la ligne suivante.

Dans les exemples de cette leçon, l'adresse du service est supposée déjà connue de `aws` (dans le labo, elle l'est). Hors labo, ajoute `--endpoint-url=https://s3.exemple.invalid` après `aws`, comme dans la leçon 2.

Pour voir les versions et leurs dates :

```bash
aws s3api list-object-versions --bucket mon-bucket
```

La sortie, au format JSON, contient pour chaque version un `VersionId` et un `LastModified` (la date d'envoi).

## Le cycle de vie

Sans nettoyage, les versions s'accumulent. Une **règle de cycle de vie** supprime automatiquement ce qui est trop ancien. Voici la politique du dépôt `backups3` :

```json
{
    "Rules": [
        {
            "ID": "1-year-expiration",
            "Filter": {},
            "Status": "Enabled",
            "NoncurrentVersionExpiration": {
                "NoncurrentDays": 365
            }
        }
    ]
}
```

- `"Rules"` : la liste des règles ; il y en a une ici.
- `"ID": "1-year-expiration"` : le nom de la règle, libre.
- `"Filter": {}` : la règle s'applique à tous les objets du bucket (un filtre vide ne restreint rien) ;
- `"Status": "Enabled"` : la règle est active.
- `NoncurrentVersionExpiration` : seules les versions **non courantes** sont supprimées ;
- `NoncurrentDays: 365` : une ancienne version est supprimée un an après avoir été remplacée.

On l'applique avec une commande `s3api` sur un fichier local :

```bash
aws s3api put-bucket-lifecycle-configuration \
  --bucket mon-bucket \
  --lifecycle-configuration file://lifecycle_policy.json
```

`--lifecycle-configuration file://lifecycle_policy.json` dit à `aws` de lire la politique dans le fichier `lifecycle_policy.json` du dossier courant.

:::info Deux syntaxes
Le README de `backups3` montre `put-bucket-lifecycle` avec `Prefix`, l'ancienne forme. Le fichier `lifecycle_policy.json` du dépôt utilise `Filter`, la forme actuelle. Vérifie ce que ton S3 accepte.
:::

## Choisir une durée

La **rétention** est la durée pendant laquelle on garde une sauvegarde. Elle se choisit en pesant trois questions.

| Question | Conséquence |
| --- | --- |
| Combien de temps peut-on ne pas voir une erreur ? | Durée minimale de conservation |
| Y a-t-il des données personnelles ? | Durée maximale à justifier (**RGPD**, le règlement européen qui encadre la conservation des données personnelles) |
| Combien de place coûte chaque version ? | Plus la sauvegarde grossit, plus la rétention coûte |

:::warning Le cycle de vie supprime pour de bon
Une règle trop courte ou mal ciblée efface des sauvegardes sans demander confirmation. Teste d'abord sur un bucket vide et relis le `Filter`.
:::

## Entraîne-toi

Dans le labo, un MinIO local joue le S3. Tu crées un bucket, tu actives son versionnement, tu envoies deux fois le fichier `journal.txt` (le contenu change entre les deux), puis tu écris et appliques la règle de cycle de vie du dépôt `backups3`.

Pour envoyer deux versions, tu modifies le fichier entre deux envois :

```bash
backup.py historique journal.txt journal.txt
echo "jour 2" >> journal.txt
backup.py historique journal.txt journal.txt
```

- `backup.py historique journal.txt journal.txt` envoie le fichier local `journal.txt` dans le bucket `historique`, sous le nom d'objet `journal.txt`.
- `echo "jour 2" >> journal.txt` **ajoute** la ligne « jour 2 » à la fin du fichier (`>>` ajoute, alors que `>` remplace tout le contenu).

:::labo
moteur: reel
intro: |
  Un MinIO local est démarré pour toi (identifiants factices), et le fichier `journal.txt` contient une ligne `jour 1`. `aws` est déjà réglé pour parler à ce serveur : inutile de donner l'adresse. Les étapes sont vérifiées sur l'état du bucket : son réglage de versionnement, le nombre de versions de `journal.txt` et la règle de cycle de vie.
fichiers:
  journal.txt: |
    jour 1
commandes:
  - demarrer-minio
etapes:
  - texte: 'Crée le bucket `historique` avec `mc mb`'
    indice: 'mc mb labo/historique'
    verif:
      - commande-reussit: mc ls labo/historique
    solution:
      - mc mb labo/historique
  - texte: 'Active le versionnement du bucket `historique` avec `aws s3api put-bucket-versioning`'
    indice: 'aws s3api put-bucket-versioning --bucket historique --versioning-configuration Status=Enabled'
    apres: [1]
    verif:
      - sortie-contient:
          - aws s3api get-bucket-versioning --bucket historique
          - 'Enabled'
    solution:
      - aws s3api put-bucket-versioning --bucket historique --versioning-configuration Status=Enabled
  - texte: 'Envoie `journal.txt` dans `historique`, ajoute une ligne au fichier, puis envoie-le encore : le bucket doit contenir au moins deux versions de `journal.txt`'
    indice: 'Les trois commandes du cours : backup.py historique journal.txt journal.txt, puis echo "jour 2" >> journal.txt, puis backup.py une seconde fois.'
    apres: [2]
    verif:
      - sortie-contient:
          - aws s3api list-object-versions --bucket historique --prefix journal.txt --query 'length(Versions)'
          - '^([2-9]|[1-9][0-9]+)$'
    solution:
      - backup.py historique journal.txt journal.txt
      - echo "jour 2" >> journal.txt
      - backup.py historique journal.txt journal.txt
  - texte: 'Écris le fichier `lifecycle_policy.json` : une règle active, avec un filtre vide, qui supprime les versions non courantes au bout de 365 jours'
    indice: 'Recopie le JSON de la section « Le cycle de vie » avec nano lifecycle_policy.json.'
    verif:
      - commande-reussit: python3 -m json.tool lifecycle_policy.json
      - fichier-contient-dans-env: [lifecycle_policy.json, 'NoncurrentVersionExpiration']
      - fichier-contient-dans-env: [lifecycle_policy.json, '"NoncurrentDays": *365']
    solution:
      - ecrire:
          lifecycle_policy.json: |
            {
                "Rules": [
                    {
                        "ID": "1-year-expiration",
                        "Filter": {},
                        "Status": "Enabled",
                        "NoncurrentVersionExpiration": {
                            "NoncurrentDays": 365
                        }
                    }
                ]
            }
  - texte: 'Applique cette règle au bucket `historique` avec `aws s3api put-bucket-lifecycle-configuration`'
    indice: 'aws s3api put-bucket-lifecycle-configuration --bucket historique --lifecycle-configuration file://lifecycle_policy.json'
    apres: [1, 4]
    verif:
      - sortie-contient:
          - aws s3api get-bucket-lifecycle-configuration --bucket historique
          - 'NoncurrentVersionExpiration'
    solution:
      - aws s3api put-bucket-lifecycle-configuration --bucket historique --lifecycle-configuration file://lifecycle_policy.json
  - texte: 'Garde la liste des versions de `journal.txt` dans le fichier `versions.json`'
    indice: 'aws s3api list-object-versions --bucket historique --prefix journal.txt > versions.json'
    apres: [3]
    verif:
      - fichier-contient-dans-env: [versions.json, 'VersionId']
    solution:
      - aws s3api list-object-versions --bucket historique --prefix journal.txt > versions.json
:::

## Vérifie tes acquis

:::quiz
Que fait le versionnement quand on envoie un objet qui existe déjà ?

- [ ] Il refuse l'envoi
- [ ] Il écrase l'objet sans trace
- [x] Il garde l'ancien et crée une nouvelle version
- [ ] Il renomme l'objet avec la date

> Chaque écriture ajoute une version identifiée par un `VersionId`.
:::

:::quiz
Dans la politique du dépôt, que supprime `NoncurrentVersionExpiration` à 365 jours ?

- [x] Les versions remplacées depuis plus d'un an
- [ ] La version courante après un an
- [ ] Tout le bucket
- [ ] Les objets de plus d'un an, courants ou non

> Seules les versions non courantes sont concernées ; la version actuelle reste.
:::

:::quiz
Que signifie `"Filter": {}` dans une règle ?

- [ ] La règle ne s'applique à rien
- [ ] La règle est désactivée
- [x] La règle s'applique à tous les objets du bucket
- [ ] La règle s'applique aux objets vides

> Un filtre vide ne restreint aucun objet.
:::

:::quiz
Pourquoi ne pas conserver toutes les versions indéfiniment ?

- [ ] S3 limite à 10 versions
- [ ] Les anciennes versions deviennent illisibles
- [x] Le stockage se remplit et des données personnelles ne doivent pas être gardées sans limite
- [ ] Le versionnement se désactive tout seul

> La durée de conservation doit équilibrer besoin de restauration, coût et obligations légales.
:::
