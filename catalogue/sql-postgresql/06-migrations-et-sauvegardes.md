---
id: migrations-et-sauvegardes
title: "Migrations de schéma et sauvegardes"
summary: "Faire évoluer le schéma de façon versionnée, puis sauvegarder et restaurer avec pg_dump."
minutes: 35
objectives:
  - Expliquer ce qu'est une migration et pourquoi on ne modifie jamais une base à la main
  - Écrire une migration `ALTER TABLE` sûre sur une table déjà remplie
  - Sauvegarder et restaurer une base avec `pg_dump` et `pg_restore`
---

Le besoin change : il faut ajouter un numéro de téléphone aux adhérent·e·s. Si tu te connectes à la base de **production** (celle que les vrais utilisateur·rice·s utilisent) pour lancer un `ALTER TABLE` (l'instruction qui modifie la structure d'une table) à la main, personne ne saura que ça a été fait, ni comment le refaire sur la **préproduction** (la copie d'essai de la production) ou sur le poste de la personne d'à côté.

## Une migration, c'est du schéma versionné

Une **migration** est un fichier, rangé dans Git avec le code, qui décrit **un changement du schéma**. Les migrations s'appliquent dans l'ordre, et la base note lesquelles ont déjà été jouées. Résultat : tout le monde, du poste de développement à la production, arrive au même schéma.

Tu les croiseras sous deux formes dans les projets de l'équipe :

:::cards
### Django

Les migrations sont générées à partir des modèles Python. `python manage.py makemigrations` crée le fichier, `python manage.py migrate` l'applique, et `python manage.py sqlmigrate app 0002` affiche le SQL correspondant.

### MiniShop (Kysely)

Les migrations sont écrites à la main en TypeScript, numérotées dans `src/migrations/` (`001_initial_schema.ts`, `002_variations_rules.ts`…). Chacune exporte une fonction `up` (appliquer) et une fonction `down` (défaire).
:::

La règle commune : **une migration déjà appliquée ailleurs ne se modifie plus**. Pour corriger, on en écrit une nouvelle.

## Une migration sûre

Ajoutons le téléphone. Une colonne facultative ne pose aucun problème : les lignes existantes y reçoivent simplement `NULL` (« pas de valeur »).

```sql
ALTER TABLE adherents ADD COLUMN telephone text;
```

```console
ALTER TABLE
```

Si on la veut obligatoire (`NOT NULL`), il faut que **toutes les lignes existantes** aient déjà une valeur, sinon PostgreSQL refuse. La bonne méthode tient en trois temps, dans une transaction (`BEGIN` … `COMMIT`, bloc « tout ou rien » vu à la leçon précédente). Les lignes qui commencent par `--` sont des commentaires SQL, ignorés par PostgreSQL :

```sql
BEGIN;

-- 1. ajouter la colonne, facultative
ALTER TABLE adherents ADD COLUMN telephone text;

-- 2. remplir les lignes existantes
UPDATE adherents SET telephone = 'inconnu' WHERE telephone IS NULL;

-- 3. rendre la colonne obligatoire
ALTER TABLE adherents ALTER COLUMN telephone SET NOT NULL;

COMMIT;
```

```console
BEGIN
ALTER TABLE
UPDATE 5
ALTER TABLE
COMMIT
```

Comme PostgreSQL gère les changements de schéma dans une transaction (leçon précédente), si une étape échoue **rien** n'est appliqué : on ne laisse pas la base à moitié migrée. En Kysely, la même migration s'écrirait dans `up`, avec dans `down` son contraire :

```typescript
export async function up(db: Kysely<DB>): Promise<void> {
    await sql`ALTER TABLE adherents ADD COLUMN telephone text`.execute(db);
}

export async function down(db: Kysely<DB>): Promise<void> {
    await sql`ALTER TABLE adherents DROP COLUMN telephone`.execute(db);
}
```

:::danger DROP COLUMN et DROP TABLE détruisent les données
`down` défait le **schéma**, pas les **données** : la colonne supprimée emporte son contenu avec elle. Avant une migration destructrice, fais une sauvegarde (section suivante) et teste-la sur une copie de la base.
:::

## Sauvegarder avec pg_dump

`pg_dump` produit une copie logique d'**une** base. Elle est cohérente (prise à un instant précis) et n'empêche pas les autres de continuer à lire et écrire.

```bash
pg_dump -h localhost -U app -d asso -Fc -f asso.dump
```

Dans un terminal, une ligne qui commence par le nom d'un programme (`pg_dump`) le lance avec les **options** qui suivent :

- `-h`, `-U`, `-d` : serveur (*host*), utilisateur (*user*) et base (*database*). Dans le labo, la connexion est déjà réglée pour toi : tu peux les omettre.
- `-Fc` : format *custom*, compressé, que l'on restaure avec `pg_restore`.
- `-f` : le fichier de sortie.

Le mot de passe ne se tape pas dans la commande : utilise la variable d'environnement `PGPASSWORD` ou un fichier `~/.pgpass`. Dans un conteneur Docker Compose, on exécute `pg_dump` dans le conteneur :

```bash
docker compose exec -T db pg_dump -U app -d asso > asso.sql
```

`docker compose exec -T db …` exécute la commande dans le service `db` du fichier Compose, et `>` écrit la sortie dans le fichier `asso.sql` de ta machine. Ici la sortie est du SQL en clair (format par défaut), qui se rejoue avec `psql`.

## Restaurer, et surtout tester

On restaure **dans une base vide**, jamais par-dessus la production :

```bash
createdb -h localhost -U app asso_restauration
pg_restore -h localhost -U app -d asso_restauration asso.dump
psql -h localhost -U app -d asso_restauration -c "SELECT count(*) FROM adherents;"
```

`createdb` crée une base vide ; `pg_restore -d` y recharge le contenu de la sauvegarde ; `psql -c` compte enfin les adhérent·e·s de la copie.

La dernière commande vérifie que les données sont bien revenues. Un fichier de sauvegarde que tu n'as jamais restauré est une **supposition**, pas une sauvegarde.

:::info Les sauvegardes automatiques dans l'équipe
Le groupe `dev` de l'équipe sur GitLab contient un dépôt `docker-postgres-backup` : une image qui lance `pg_dump` périodiquement (tâche cron) et écrit les sauvegardes dans un dossier `/backup`. Pour savoir comment sont sauvegardées les bases d'un projet donné, demande à l'équipe Infra.
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  Les quatre tables sont chargées dans la base `asso` (5 adhérent·e·s). Tu vas ajouter une colonne `telephone` obligatoire sans casser les lignes existantes, puis sauvegarder et restaurer la base. Dans le labo, `psql`, `pg_dump`, `pg_restore` et `createdb` sont déjà connectés au serveur : inutile de donner `-h` ni `-U`. Les étapes sont vérifiées sur l'état de la base et sur les fichiers produits.
commands:
  - /opt/exercices/demarrer.sh
  - psql -q -v ON_ERROR_STOP=1 -f /opt/exercices/schema.sql -f /opt/exercices/donnees.sql
steps:
  - text: 'Ajoute à `adherents` une colonne `telephone` de type `text`, **facultative** pour l''instant'
    hint: 'psql -c "ALTER TABLE adherents ADD COLUMN telephone text;"'
    checks:
      - output-contains:
          - psql -Atc "SELECT count(*) FROM information_schema.columns WHERE table_name = 'adherents' AND column_name = 'telephone'"
          - '^1$'
    solution:
      - psql -c "ALTER TABLE adherents ADD COLUMN telephone text;"
  - text: 'Remplis les lignes existantes : mets `inconnu` dans `telephone` partout où il vaut `NULL`'
    hint: 'Commande : `psql -c "UPDATE adherents SET telephone = ''inconnu'' WHERE telephone IS NULL;"`. Le compteur doit annoncer `UPDATE 5`.'
    after: [1]
    checks:
      - output-contains:
          - psql -Atc "SELECT count(*) FROM adherents WHERE telephone = 'inconnu'"
          - '^5$'
    solution:
      - psql -c "UPDATE adherents SET telephone = 'inconnu' WHERE telephone IS NULL;"
  - text: 'Rends la colonne `telephone` obligatoire avec `SET NOT NULL`'
    hint: 'psql -c "ALTER TABLE adherents ALTER COLUMN telephone SET NOT NULL;"'
    after: [2]
    checks:
      - output-contains:
          - psql -Atc "SELECT is_nullable FROM information_schema.columns WHERE table_name = 'adherents' AND column_name = 'telephone'"
          - '^NO$'
    solution:
      - psql -c "ALTER TABLE adherents ALTER COLUMN telephone SET NOT NULL;"
  - text: 'Sauvegarde la base `asso` au format personnalisé dans le fichier `asso.dump`'
    hint: 'pg_dump -Fc -f asso.dump — fais-le après la migration, pour que la sauvegarde contienne la colonne `telephone`.'
    after: [3]
    checks:
      - env-file-exists: asso.dump
      - command-succeeds: pg_restore -f - asso.dump | grep -q telephone
    solution:
      - pg_dump -Fc -f asso.dump
  - text: 'Teste la sauvegarde : crée une base vide `asso_restauration` puis restaure-y `asso.dump`'
    hint: 'createdb asso_restauration, puis pg_restore -d asso_restauration asso.dump. Vérifie avec psql -d asso_restauration -c "SELECT count(*) FROM adherents;".'
    after: [4]
    checks:
      - output-contains:
          - psql -d asso_restauration -Atc "SELECT count(*) FROM adherents WHERE telephone = 'inconnu'"
          - '^5$'
    solution:
      - createdb asso_restauration
      - pg_restore -d asso_restauration asso.dump
:::

## Vérifie tes acquis

:::quiz
Pourquoi ne modifie-t-on pas une migration déjà appliquée sur un autre environnement ?

- [ ] Parce que Git interdit de modifier un fichier déjà commité
- [ ] Parce que les migrations sont chiffrées
- [x] Parce que l'autre base l'a déjà jouée : la modification ne s'y appliquerait jamais, et les schémas divergeraient
- [ ] Parce qu'une migration ne peut contenir qu'une instruction

> La base mémorise les migrations déjà jouées. Pour changer le schéma, on ajoute une nouvelle migration à la suite.
:::

:::quiz
Tu dois ajouter une colonne `NOT NULL` à une table qui contient déjà des lignes. Quelle approche est la plus sûre ?

- [ ] Ajouter directement `ADD COLUMN … NOT NULL` sans valeur par défaut
- [x] Ajouter la colonne facultative, remplir les lignes existantes, puis poser `SET NOT NULL`
- [ ] Vider la table, puis ajouter la colonne
- [ ] Ajouter la colonne en production, puis la modifier en développement

> Les lignes existantes ont besoin d'une valeur avant la contrainte. Le tout peut tenir dans une transaction pour tout annuler en cas d'échec.
:::

:::quiz
Que contient la fonction `down` d'une migration de type Kysely ?

- [ ] Le code qui sauvegarde les données avant la migration
- [x] Les instructions qui défont le changement de schéma de `up`
- [ ] Les instructions qui restaurent aussi les données supprimées
- [ ] Un test qui vérifie que la migration a réussi

> `down` annule le changement de structure. Les données perdues par un `DROP COLUMN` ne reviennent pas : elles viennent d'une sauvegarde.
:::

:::quiz
Quand peut-on dire qu'une sauvegarde `pg_dump` est fiable ?

- [ ] Quand le fichier existe et n'est pas vide
- [ ] Quand la commande s'est terminée sans message
- [ ] Quand elle est stockée sur le même serveur que la base
- [x] Quand on a déjà réussi à la restaurer dans une base vide

> Seule une restauration réussie prouve que la sauvegarde est exploitable. Et garder la copie sur le même serveur n'aide pas si le serveur disparaît.
:::
