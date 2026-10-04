---
id: select
titre: "SELECT : filtrer, trier, agréger"
resume: "Interroger une table avec SELECT, WHERE, ORDER BY, LIMIT, puis résumer les données avec GROUP BY."
duree: 30
objectifs:
  - Écrire un `SELECT` avec `WHERE`, `ORDER BY` et `LIMIT`
  - Raisonner correctement avec `NULL`
  - Compter et additionner avec `count`, `sum` et `GROUP BY`
---

« Combien de personnes sont inscrites à la Coupe de robotique ? » « Qui n'a pas d'association ? » Ces questions se posent à une base de données avec un seul verbe, `SELECT`, que tu vas utiliser tous les jours.

On reprend le schéma de la leçon précédente (les tables `assos`, `adherents`, `evenements` et `inscriptions`), avec ses données : dans le labo, elles sont déjà chargées dans la base `asso`. Une **requête** est une demande écrite en SQL ; `SELECT` (« sélectionne ») est celle qui **lit** des données sans jamais les modifier.

## Lire la bonne forme

Une requête `SELECT` se lit dans l'ordre où on l'écrit, mais PostgreSQL l'exécute dans un autre ordre :

| Écrit | Rôle | Exécuté |
| --- | --- | --- |
| `SELECT` | Les colonnes à afficher | 5e |
| `FROM` | La table lue (« depuis ») | 1er |
| `WHERE` | Filtre les **lignes** | 2e |
| `GROUP BY` / `HAVING` | Regroupe, puis filtre les **groupes** | 3e et 4e |
| `ORDER BY` | Trie le résultat | 6e |
| `LIMIT` | Garde les N premières lignes | 7e |

Un **alias** est un nom provisoire donné avec `AS` (`count(*) AS total` : la colonne s'appellera `total`). Conséquence pratique : `WHERE` ne peut pas utiliser un alias défini dans le `SELECT`, puisqu'il est évalué avant.

## Choisir, filtrer, trier

```sql
SELECT prenom, nom FROM adherents;
```

Ligne par ligne : `SELECT prenom, nom` liste les colonnes voulues, `FROM adherents` indique la table, `;` termine la requête.

Les colonnes demandées sont affichées pour toutes les lignes :

```console
 prenom |  nom
--------+--------
 Alice  | Martin
 Bilal  | Haddad
 Chloé  | Durand
 David  | Roux
 Emma   | Petit
(5 rows)
```

Avec `WHERE` (« où »), on ne garde que les lignes qui vérifient une condition :

```sql
SELECT prenom, nom, email FROM adherents WHERE asso_id = 2;
```

```console
 prenom |  nom   |          email
--------+--------+--------------------------
 Chloé  | Durand | chloe.durand@example.org
 David  | Roux   | david.roux@example.org
(2 rows)
```

Autres filtres courants : `<>` (différent), `>=` (supérieur ou égal), `AND` (et), `OR` (ou), `IN (1, 2)` (parmi ces valeurs), `BETWEEN` (entre deux bornes), et `LIKE` pour chercher un motif (`%` remplace n'importe quelle suite de caractères). `ILIKE` ignore la casse (majuscules ou minuscules), ce que `LIKE` ne fait pas. Ci-dessous, `'d%'` veut dire « commence par d » :

```sql
SELECT prenom, nom FROM adherents WHERE nom ILIKE 'd%';
```

```console
 prenom |  nom
--------+--------
 Chloé  | Durand
(1 row)
```

`ORDER BY` trie (`ASC` croissant, par défaut ; `DESC` décroissant) et `LIMIT` coupe : `LIMIT 3` ne garde que les trois premières lignes.

```sql
SELECT prenom, nom FROM adherents ORDER BY nom DESC LIMIT 3;
```

```console
 prenom |  nom
--------+--------
 David  | Roux
 Emma   | Petit
 Alice  | Martin
(3 rows)
```

:::tip
Évite `SELECT *` dans le code d'une application : si quelqu'un ajoute une colonne, ta requête change de forme sans que tu l'aies décidé. En exploration dans `psql`, c'est en revanche très pratique.
:::

## NULL : ni zéro, ni vide

`NULL` signifie « valeur inconnue ou absente ». Comparer quoi que ce soit à `NULL` avec `=` ne donne ni vrai ni faux, mais « inconnu », et `WHERE` ne garde que les « vrai » :

```sql
SELECT prenom FROM adherents WHERE asso_id = NULL;
```

```console
 prenom
--------
(0 rows)
```

La bonne écriture est `IS NULL` (ou `IS NOT NULL`) :

```sql
SELECT prenom FROM adherents WHERE asso_id IS NULL;
```

```console
 prenom
--------
 Emma
(1 row)
```

## Agréger : count, sum, GROUP BY

Une **fonction d'agrégation** résume plusieurs lignes en une seule valeur : `count` (nombre de lignes), `sum` (somme), `avg` (moyenne), `min` et `max` (plus petite et plus grande valeur).

```sql
SELECT count(*) AS total, count(asso_id) AS avec_asso FROM adherents;
```

```console
 total | avec_asso
-------+-----------
     5 |         4
(1 row)
```

`count(*)` compte les lignes ; `count(colonne)` ignore les `NULL`. D'où 5 contre 4.

Avec `GROUP BY` (« regroupe par »), on obtient un résultat **par groupe** : une ligne par valeur distincte de `asso_id`.

```sql
SELECT asso_id,
       count(*)              AS nb_evenements,
       sum(places_restantes) AS places
FROM evenements
GROUP BY asso_id
ORDER BY asso_id;
```

```console
 asso_id | nb_evenements | places
---------+---------------+--------
       1 |             2 |     50
       2 |             2 |     18
(2 rows)
```

`HAVING` filtre les groupes une fois calculés, là où `WHERE` filtre les lignes avant le regroupement :

```sql
SELECT asso_id, count(*) AS nb_evenements
FROM evenements
GROUP BY asso_id
HAVING count(*) >= 2;
```

:::warning Toute colonne du SELECT doit être regroupée ou agrégée
Écrire `SELECT asso_id, titre, count(*) … GROUP BY asso_id` échoue : PostgreSQL ne sait pas quel `titre` afficher pour un groupe qui en contient plusieurs. Ajoute la colonne au `GROUP BY` ou agrège-la.
:::

## Entraîne-toi

Un `SELECT` ne laisse aucune trace dans la base : pour que le serveur puisse vérifier ton travail, tu **écris ta requête dans un fichier `.sql`**, et c'est le serveur qui l'exécute. Pour créer le fichier en une ligne :

```shell
echo "SELECT prenom FROM adherents;" > resultat.sql
```

- `echo "…"` affiche le texte entre guillemets ; `> resultat.sql` est une **redirection** du terminal : le texte est écrit dans le fichier `resultat.sql` au lieu de s'afficher. Tu peux aussi écrire le fichier avec `nano resultat.sql`.
- Teste ta requête avec `psql -At -f resultat.sql` : `-f` exécute le fichier, `-A` supprime l'alignement en colonnes et `-t` supprime l'en-tête et le « (N rows) ». Il ne reste que les valeurs, séparées par `|`.
- Le serveur relance ta requête sur les vraies tables : une réponse recopiée à la main ne passe pas.

:::labo
moteur: reel
intro: |
  Le schéma et les données de la leçon 1 sont chargés dans la base `asso` (5 adhérent·e·s, 4 événements). Pour chaque étape, écris la **requête** dans le fichier `.sql` demandé (avec `nano` ou `echo "…" > fichier.sql`) et teste-la avec `psql -At -f fichier.sql`. Le serveur exécute lui-même ton fichier et compare son résultat : il vérifie aussi que ta requête lit bien les tables (une réponse recopiée à la main ne passe pas). L'ordre et les colonnes demandés comptent.
commandes:
  - /opt/exercices/demarrer.sh
  - psql -q -v ON_ERROR_STOP=1 -f /opt/exercices/schema.sql -f /opt/exercices/donnees.sql
etapes:
  - texte: 'Écris dans `robotique.sql` une requête qui donne le prénom et le nom (dans cet ordre) des adhérent·e·s de l''association `2`, triés par prénom'
    indice: 'Une requête, un fichier : `echo "SELECT prenom, nom FROM adherents WHERE asso_id = 2 ORDER BY prenom;" > robotique.sql` (ou `nano robotique.sql`). Teste-la avec `psql -At -f robotique.sql` : tu dois lire `Chloé|Durand` puis `David|Roux`.'
    verif:
      - commande-reussit: python3 /opt/exercices/verif-requete.py robotique.sql 'Chloé|Durand\nDavid|Roux' 'CHLOÉ|DURAND\nDAVID|ROUX\nZOÉ|ZED'
    solution:
      - printf '%s\n' "SELECT prenom, nom FROM adherents WHERE asso_id = 2 ORDER BY prenom;" > robotique.sql
  - texte: 'Écris dans `sans-asso.sql` une requête qui donne le prénom des adhérent·e·s **sans association**'
    indice: 'Une valeur absente se teste avec `IS NULL`, jamais avec `= NULL`. Teste avec `psql -At -f sans-asso.sql`.'
    verif:
      - commande-reussit: python3 /opt/exercices/verif-requete.py sans-asso.sql 'Emma' 'EMMA'
    solution:
      - printf '%s\n' "SELECT prenom FROM adherents WHERE asso_id IS NULL;" > sans-asso.sql
  - texte: 'Écris dans `noms-d.sql` une requête qui donne les noms de famille (colonne `nom`) qui commencent par « d », majuscule ou minuscule'
    indice: '`ILIKE ''d%''` ignore la casse ; pense à ne sélectionner que la colonne `nom`. Teste avec `psql -At -f noms-d.sql`.'
    verif:
      - commande-reussit: python3 /opt/exercices/verif-requete.py noms-d.sql 'Durand' 'DURAND'
    solution:
      - printf '%s\n' "SELECT nom FROM adherents WHERE nom ILIKE 'd%';" > noms-d.sql
  - texte: 'Écris dans `compte.sql` une requête qui donne, sur une seule ligne, le nombre total d''adhérent·e·s, puis le nombre de ceux qui ont une association (dans cet ordre)'
    indice: '`count(*)` compte toutes les lignes ; `count(asso_id)` ignore les `NULL`. Sépare les deux par une virgule dans le `SELECT`. Teste avec `psql -At -f compte.sql` : tu dois lire `5|4`.'
    verif:
      - commande-reussit: python3 /opt/exercices/verif-requete.py compte.sql '5|4' '6|5'
    solution:
      - printf '%s\n' "SELECT count(*), count(asso_id) FROM adherents;" > compte.sql
  - texte: 'Écris dans `par-asso.sql` une requête qui donne, pour chaque association (colonne `asso_id` de `evenements`), son `asso_id`, le nombre d''événements et la somme de leurs `places_restantes`, triés par `asso_id`'
    indice: 'Reprends l''exemple `GROUP BY` du cours en ajoutant `ORDER BY asso_id`, sans alias ni `HAVING`. Teste avec `psql -At -f par-asso.sql`.'
    verif:
      - commande-reussit: python3 /opt/exercices/verif-requete.py par-asso.sql '1|2|50\n2|2|18' '1|2|250\n2|2|218'
    solution:
      - printf '%s\n' "SELECT asso_id, count(*), sum(places_restantes) FROM evenements GROUP BY asso_id ORDER BY asso_id;" > par-asso.sql
:::

## Vérifie tes acquis

:::quiz
Quelle requête liste les adhérent·e·s sans association ?

- [ ] `SELECT prenom FROM adherents WHERE asso_id = NULL;`
- [ ] `SELECT prenom FROM adherents WHERE asso_id = 0;`
- [x] `SELECT prenom FROM adherents WHERE asso_id IS NULL;`
- [ ] `SELECT prenom FROM adherents WHERE asso_id = '';`

> Il faut `IS NULL`. Avec `= NULL`, la condition n'est jamais vraie et la requête ne renvoie aucune ligne.
:::

:::quiz
Sur nos 5 adhérent·e·s dont 4 ont une association, que renvoie `SELECT count(asso_id) FROM adherents;` ?

- [x] 4
- [ ] 5
- [ ] 1
- [ ] 0

> `count(colonne)` ne compte que les valeurs non `NULL`, alors que `count(*)` compte toutes les lignes.
:::

:::quiz
Quelle est la différence entre `WHERE` et `HAVING` ?

- [ ] `WHERE` s'utilise avec `SELECT`, `HAVING` avec `UPDATE`
- [ ] `HAVING` est une version plus rapide de `WHERE`
- [x] `WHERE` filtre les lignes avant le regroupement, `HAVING` filtre les groupes après
- [ ] Il n'y en a aucune, les deux sont interchangeables

> `HAVING` peut utiliser une agrégation comme `count(*)`, ce que `WHERE` ne peut pas faire, car il est évalué avant le regroupement.
:::

:::quiz
Que fait `SELECT prenom FROM adherents ORDER BY nom DESC LIMIT 3;` ?

- [ ] Elle renvoie les trois premiers prénoms par ordre alphabétique de nom
- [ ] Elle renvoie les trois premières lignes enregistrées, triées ensuite
- [x] Elle trie par nom décroissant puis garde les trois premières lignes
- [ ] Elle supprime les lignes au-delà de la troisième

> Le tri est appliqué avant `LIMIT`. Un `SELECT` ne modifie jamais les données.
:::
