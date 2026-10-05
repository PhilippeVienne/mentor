## Schéma

| Besoin | SQL |
| --- | --- |
| Créer une table | `CREATE TABLE t (id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY, …)` |
| Clé étrangère | `col integer REFERENCES autre (id)` |
| Contraintes | `NOT NULL` · `UNIQUE` · `DEFAULT valeur` · `CHECK (condition)` |
| Ajouter une colonne | `ALTER TABLE t ADD COLUMN col type` |
| Ajouter une contrainte | `ALTER TABLE t ADD CONSTRAINT nom CHECK (…)` |
| Insérer | `INSERT INTO t (a, b) VALUES (1, 'x') RETURNING id` |
| Modifier / supprimer | `UPDATE t SET a = 1 WHERE …` · `DELETE FROM t WHERE …` |

## Interroger

| Besoin | SQL |
| --- | --- |
| Lire des colonnes | `SELECT a, b FROM t` |
| Filtrer | `WHERE a = 1 AND b IN (1, 2)` |
| Motif de texte | `LIKE 'D%'` · `ILIKE 'd%'` (sans la casse) |
| Valeur absente | `IS NULL` · `IS NOT NULL` (jamais `= NULL`) |
| Trier, limiter | `ORDER BY a DESC LIMIT 10` |
| Compter | `count(*)` (lignes) · `count(col)` (non `NULL`) |
| Agréger | `sum` · `avg` · `min` · `max` avec `GROUP BY` |
| Filtrer les groupes | `HAVING count(*) >= 2` |

Ordre d'exécution : `FROM` → `WHERE` → `GROUP BY` → `HAVING` → `SELECT` → `ORDER BY` → `LIMIT`.

## Jointures

| Besoin | SQL |
| --- | --- |
| Lignes avec correspondance | `FROM a JOIN b ON b.a_id = a.id` |
| Garder toute la gauche | `FROM a LEFT JOIN b ON b.a_id = a.id` |
| Trouver ce qui manque | `LEFT JOIN … WHERE b.id IS NULL` |
| Compter avec `LEFT JOIN` | `count(b.id)`, pas `count(*)` |

## Transactions

| Besoin | SQL |
| --- | --- |
| Ouvrir | `BEGIN;` |
| Valider | `COMMIT;` |
| Annuler | `ROLLBACK;` |

Avant de valider un `UPDATE` ou un `DELETE`, lis le nombre de lignes touchées (`UPDATE 1`).

## Index et plan

| Besoin | SQL |
| --- | --- |
| Voir le plan | `EXPLAIN requête` |
| Mesurer pour de vrai | `EXPLAIN ANALYZE requête` (exécute la requête !) |
| Créer un index | `CREATE INDEX idx_t_col ON t (col)` |
| Lecture complète / par l'index | `Seq Scan` / `Index Scan` |

Indexés d'office : clés primaires et colonnes `UNIQUE`. Pas les clés étrangères.

## psql, migrations et sauvegardes

| Besoin | Commande |
| --- | --- |
| Lister les tables | `\dt` |
| Décrire une table | `\d table` |
| Quitter | `\q` |
| Sauvegarder (format custom) | `pg_dump -h hôte -U utilisateur -d base -Fc -f base.dump` |
| Restaurer dans une base vide | `pg_restore -h hôte -U utilisateur -d base_vide base.dump` |
| Migrations Django | `makemigrations` · `migrate` · `sqlmigrate app 0002` |
| Migrations Kysely (MiniShop) | fonctions `up` et `down` dans `src/migrations/NNN_nom.ts` |
