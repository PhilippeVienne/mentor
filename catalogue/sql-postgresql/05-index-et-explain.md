---
id: index-et-explain
titre: "Index et plan de requête avec EXPLAIN"
resume: "Comprendre pourquoi une requête est lente, lire son plan avec EXPLAIN et savoir quand créer un index."
duree: 30
objectifs:
  - Expliquer ce qu'est un index et son coût
  - "Lire un plan `EXPLAIN` : `Seq Scan` ou `Index Scan`"
  - Savoir quelles colonnes PostgreSQL indexe d'office, et lesquelles non
---

Imagine une page qui s'affiche instantanément avec 200 lignes en base, puis qui rame avec 200 000. La requête n'a pas changé : c'est la façon dont PostgreSQL la résout qui ne tient plus la charge. Cette leçon suppose que tu sais lire un `SELECT … WHERE` (voir la leçon sur `SELECT`) ; dans le labo, la table `adherents` est volontairement remplie de 20 000 lignes pour que la différence se voie.

## Lire tout, ou aller droit au but

Sans index, pour trouver les adhérent·e·s dont le nom est « Haddad », PostgreSQL lit **toute la table** ligne par ligne : c'est un *sequential scan*. Un **index** est comme l'index à la fin d'un livre : une structure triée, séparée de la table, qui indique où se trouvent les lignes cherchées.

| Sans index | Avec index |
| --- | --- |
| Lecture de toute la table | Lecture de quelques entrées de l'index, puis des lignes concernées |
| Temps proportionnel à la taille de la table | Temps qui augmente très peu avec la taille |

## EXPLAIN : demander le plan

`EXPLAIN` (« explique ») placé devant une requête affiche son **plan** : la méthode que PostgreSQL compte employer pour la résoudre. Il ne l'exécute **pas**.

```sql
EXPLAIN SELECT * FROM adherents WHERE nom = 'Haddad';
```

Sur le petit jeu de données des premières leçons (5 adhérent·e·s), PostgreSQL garde toujours un `Seq Scan`, car toute la table tient dans une seule page de stockage : un index n'apporterait rien. Pour que l'exemple soit reproductible, le labo ajoute **20 000 adhérent·e·s fictif·ve·s** avant de commencer. L'instruction utilisée est la suivante (tu n'as pas à la retaper) :

```sql
INSERT INTO adherents (prenom, nom, email, asso_id)
SELECT 'Prénom' || g, 'Nom' || g, 'adherent' || g || '@example.org', 1
FROM generate_series(1, 20000) AS g;
ANALYZE adherents;
```

- `generate_series(1, 20000) AS g` fabrique une table temporaire d'un seul nombre `g` qui va de 1 à 20 000.
- Le `SELECT` produit une ligne par valeur de `g` : `'Prénom' || g` **concatène** (`||` colle des textes bout à bout) pour obtenir `Prénom1`, `Prénom2`…
- `INSERT INTO … SELECT …` insère toutes ces lignes d'un coup.
- `ANALYZE adherents;` demande à PostgreSQL de **mesurer** le contenu de la table (nombre de lignes, répartition des valeurs). Ce sont ces statistiques qui lui permettent de choisir un plan.

La requête `EXPLAIN` ci-dessus donne alors (tes chiffres peuvent différer un peu) :

```console
                         QUERY PLAN
------------------------------------------------------------
 Seq Scan on adherents  (cost=0.00..457.06 rows=1 width=53)
   Filter: (nom = 'Haddad'::text)
```

- `Seq Scan` : lecture de toute la table, ligne après ligne.
- `cost=0.00..457.06` : une estimation du travail (coût de démarrage, puis coût total), en unité arbitraire. Compare-la entre deux plans, ne l'interprète pas comme des millisecondes.
- `rows=1` : le nombre de lignes que PostgreSQL **estime** trouver (ici, Bilal Haddad).
- `width=53` : la taille moyenne estimée d'une ligne, en octets.
- `Filter` : la condition appliquée **après** lecture de chaque ligne.

On crée un index sur la colonne filtrée :

```sql
CREATE INDEX idx_adherents_nom ON adherents (nom);
```

```console
CREATE INDEX
```

Le même `EXPLAIN` donne maintenant un autre plan :

```console
                                     QUERY PLAN
------------------------------------------------------------------------------------
 Index Scan using idx_adherents_nom on adherents  (cost=0.29..8.30 rows=1 width=53)
   Index Cond: (nom = 'Haddad'::text)
```

`CREATE INDEX nom_de_l_index ON table (colonne)` construit l'index. `Index Scan` : PostgreSQL passe par l'index, la condition est dans `Index Cond`. Le coût estimé a été divisé par plus de cinquante (de 457 à 8).

## Mesurer pour de vrai : EXPLAIN ANALYZE

`EXPLAIN ANALYZE` **exécute** la requête et ajoute les temps réels :

```console
 Seq Scan on adherents  (cost=0.00..457.06 rows=1 width=53) (actual time=0.007..1.353 rows=1 loops=1)
   Filter: (nom = 'Haddad'::text)
   Rows Removed by Filter: 20004
 Planning Time: 0.229 ms
 Execution Time: 1.379 ms
```

`actual time` et `Execution Time` sont des durées réelles en millisecondes. `Rows Removed by Filter: 20004` est le signe typique d'un index manquant : PostgreSQL a lu plus de 20 000 lignes pour en garder une. Après `CREATE INDEX`, la même commande affiche `Index Scan` et un `Execution Time` d'environ 0,03 ms.

:::danger EXPLAIN ANALYZE exécute vraiment la requête
Avec un `UPDATE` ou un `DELETE`, les modifications sont **réellement faites**. Enveloppe-les dans une transaction que tu annules :

```sql
BEGIN;
EXPLAIN ANALYZE DELETE FROM inscriptions WHERE evenement_id = 3;
ROLLBACK;
```
:::

## Quand créer un index

PostgreSQL crée **automatiquement** un index pour chaque `PRIMARY KEY` et chaque `UNIQUE` (par exemple `adherents_email_key`). Il ne le fait **pas** pour les clés étrangères : `adherents.asso_id` et `inscriptions.evenement_id` n'en ont pas tant que tu n'en crées pas.

Un index est utile pour une colonne :

- utilisée souvent dans un `WHERE` ;
- utilisée dans une condition de jointure (`ON a.asso_id = s.id`) ;
- sur une table qui contient beaucoup de lignes.

Mais un index a un coût : il occupe de la place et **ralentit chaque `INSERT`, `UPDATE` et `DELETE`**, car il doit être tenu à jour.

:::warning Un index n'est pas toujours utilisé
Sur une petite table, PostgreSQL préfère souvent un `Seq Scan` : lire quelques pages est plus rapide que passer par l'index. De même, `WHERE nom LIKE '%addad'` (joker au début) ne peut pas utiliser un index classique. Ne crée pas d'index « au cas où » : mesure d'abord avec `EXPLAIN`.
:::

## Entraîne-toi

:::labo
moteur: reel
intro: |
  Le labo a chargé les quatre tables **et** 20 000 adhérent·e·s fictif·ve·s (puis lancé `ANALYZE adherents;`). Tu vas voir le plan changer en créant un index. Pour garder une trace vérifiable d'un plan, redirige-le dans un fichier : `psql -c "EXPLAIN …" > plan-avant.txt` (`>` écrit la sortie de la commande dans le fichier ; relis-le avec `cat`).
commandes:
  - /opt/exercices/demarrer.sh
  - psql -q -v ON_ERROR_STOP=1 -f /opt/exercices/schema.sql -f /opt/exercices/donnees.sql -f /opt/exercices/volume.sql
etapes:
  - texte: 'Avant de créer l''index, enregistre dans `plan-avant.txt` le plan (`EXPLAIN`) de `SELECT * FROM adherents WHERE nom = ''Haddad'';` : tu dois y lire `Seq Scan`'
    indice: 'Commande : `psql -c "EXPLAIN SELECT * FROM adherents WHERE nom = ''Haddad'';" > plan-avant.txt`, puis `cat plan-avant.txt`.'
    verif:
      - fichier-contient-dans-env: [plan-avant.txt, 'Seq Scan on adherents']
      - commande-reussit: 'printf "SET enable_indexscan = off;\nSET enable_bitmapscan = off;\nEXPLAIN SELECT * FROM adherents WHERE nom = ''Haddad'';\n" | psql -q -f - | cmp -s - plan-avant.txt'
    solution:
      - psql -c "EXPLAIN SELECT * FROM adherents WHERE nom = 'Haddad';" > plan-avant.txt
  - texte: 'Crée un index nommé `idx_adherents_nom` sur la colonne `nom` de `adherents`'
    indice: 'psql -c "CREATE INDEX idx_adherents_nom ON adherents (nom);"'
    apres: [1]
    verif:
      - sortie-contient:
          - psql -Atc "SELECT indexname FROM pg_indexes WHERE tablename = 'adherents' AND indexname = 'idx_adherents_nom'"
          - '^idx_adherents_nom$'
    solution:
      - psql -c "CREATE INDEX idx_adherents_nom ON adherents (nom);"
  - texte: 'Enregistre dans `plan-apres.txt` le plan de la **même** requête : il doit maintenant utiliser `Index Scan using idx_adherents_nom`'
    indice: 'Même commande qu''avant, avec `> plan-apres.txt`. Compare le coût (`cost=`) des deux fichiers.'
    apres: [2]
    verif:
      - fichier-contient-dans-env: [plan-apres.txt, 'Index Scan using idx_adherents_nom']
      - commande-reussit: 'psql -c "EXPLAIN SELECT * FROM adherents WHERE nom = ''Haddad'';" | cmp -s - plan-apres.txt'
    solution:
      - psql -c "EXPLAIN SELECT * FROM adherents WHERE nom = 'Haddad';" > plan-apres.txt
  - texte: 'Enregistre dans `analyse.txt` le résultat de `EXPLAIN ANALYZE` pour la même requête (il exécute la requête et ajoute les temps réels) : tu dois y trouver `Execution Time`'
    indice: 'Commande : `psql -c "EXPLAIN ANALYZE SELECT * FROM adherents WHERE nom = ''Haddad'';" > analyse.txt`.'
    apres: [2]
    verif:
      - fichier-contient-dans-env: [analyse.txt, '(?s)Index Scan using idx_adherents_nom.*Execution Time']
      - commande-reussit: 'COUT=$(psql -c "EXPLAIN SELECT * FROM adherents WHERE nom = ''Haddad'';" | grep -o "cost=[0-9.]*" | head -1) && grep -q "$COUT" analyse.txt && grep -q "actual time=" analyse.txt'
    solution:
      - psql -c "EXPLAIN ANALYZE SELECT * FROM adherents WHERE nom = 'Haddad';" > analyse.txt
  - texte: 'PostgreSQL n''indexe pas les clés étrangères : crée un index (le nom que tu veux) sur la seule colonne `evenement_id` de `inscriptions`, pour accélérer les jointures par événement'
    indice: 'psql -c "CREATE INDEX idx_inscriptions_evenement ON inscriptions (evenement_id);" — la clé primaire existante commence par `adherent_id`, elle ne sert pas ici.'
    verif:
      - sortie-contient:
          - psql -Atc "SELECT count(*) FROM pg_indexes WHERE tablename = 'inscriptions' AND indexdef LIKE '%(evenement_id)%'"
          - '^1$'
    solution:
      - psql -c "CREATE INDEX idx_inscriptions_evenement ON inscriptions (evenement_id);"
:::

## Vérifie tes acquis

:::quiz
Que fait `EXPLAIN SELECT * FROM adherents WHERE nom = 'Haddad';` ?

- [ ] Elle exécute la requête et affiche le temps réel
- [ ] Elle crée un index sur la colonne `nom`
- [x] Elle affiche le plan choisi par PostgreSQL, sans exécuter la requête
- [ ] Elle explique en français pourquoi la requête échoue

> `EXPLAIN` seul montre le plan estimé. Il faut `EXPLAIN ANALYZE` pour exécuter la requête et mesurer.
:::

:::quiz
Dans un plan, tu lis `Seq Scan on adherents` avec `Rows Removed by Filter: 20004`. Que suggères-tu ?

- [ ] Supprimer les lignes en trop de la table
- [x] Examiner si un index sur la colonne filtrée accélérerait la requête
- [ ] Ajouter `LIMIT 3` pour que PostgreSQL s'arrête plus tôt
- [ ] Rien : un `Seq Scan` est toujours le meilleur plan

> PostgreSQL a lu toute la table pour n'en garder qu'une ligne. Un index sur la colonne du `WHERE` est la piste classique.
:::

:::quiz
Quelle colonne PostgreSQL indexe-t-il automatiquement ?

- [ ] `adherents.asso_id`, car c'est une clé étrangère
- [ ] Toutes les colonnes `text`
- [ ] `inscriptions.inscrit_le`, car elle a une valeur par défaut
- [x] `adherents.email`, car elle est déclarée `UNIQUE`

> Les contraintes `PRIMARY KEY` et `UNIQUE` s'appuient sur un index. Les clés étrangères n'en reçoivent pas automatiquement.
:::

:::quiz
Quel est le principal inconvénient d'ajouter beaucoup d'index sur une table ?

- [ ] Les requêtes `SELECT` deviennent plus lentes
- [ ] Les données deviennent moins fiables
- [x] Chaque écriture est ralentie et l'espace disque augmente
- [ ] PostgreSQL refuse alors les nouvelles contraintes

> L'index doit être mis à jour à chaque insertion, modification ou suppression, et il occupe de la place.
:::
