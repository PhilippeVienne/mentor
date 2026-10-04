---
id: jointures
titre: "Jointures : croiser plusieurs tables"
resume: "Recomposer l'information répartie entre les tables avec JOIN et LEFT JOIN."
duree: 30
objectifs:
  - Écrire un `JOIN` en reliant clé étrangère et clé primaire
  - Choisir entre `JOIN` et `LEFT JOIN`
  - Traverser une table de liaison pour relier deux entités
---

Dans `adherents`, Alice a `asso_id = 1`. Utile pour PostgreSQL, mais pas pour la personne qui lit la liste : elle veut voir « Ciné-club ». Une **jointure** (*join* en anglais) recolle les morceaux que le schéma a séparés. Cette leçon suppose que tu sais lire un `SELECT … FROM … WHERE` (leçon précédente) ; le labo charge les mêmes données que d'habitude dans la base `asso`.

## Le principe

Une jointure associe chaque ligne d'une table aux lignes d'une autre table qui vérifient une condition, presque toujours « clé étrangère = clé primaire ».

```sql
SELECT a.prenom, a.nom, s.nom AS asso
FROM adherents a
JOIN assos s ON s.id = a.asso_id
ORDER BY a.id;
```

```console
 prenom |  nom   |   asso
--------+--------+-----------
 Alice  | Martin | Ciné-club
 Bilal  | Haddad | Ciné-club
 Chloé  | Durand | Robotique
 David  | Roux   | Robotique
(4 rows)
```

- `adherents a` et `assos s` donnent un **alias** court à chaque table (`a` et `s`) : on écrit `a.nom` au lieu de `adherents.nom`. `s.nom AS asso` renomme en plus la colonne du résultat.
- `ON s.id = a.asso_id` est la condition de jointure : « garde les couples de lignes où la clé primaire `s.id` est égale à la clé étrangère `a.asso_id` ».
- `ORDER BY a.id` trie par numéro d'adhérent·e, pour un résultat stable.
- On préfixe les colonnes (`a.nom`, `s.nom`) quand deux tables ont le même nom de colonne.

Où est passée Emma ? Elle n'a pas d'association : aucune ligne de `assos` ne lui correspond, donc `JOIN` (qui est un `INNER JOIN`) l'écarte.

## JOIN ou LEFT JOIN

`LEFT JOIN` (« jointure à gauche ») garde **toutes** les lignes de la table de gauche (celle de `FROM`) et complète avec `NULL` quand rien ne correspond.

```sql
SELECT a.prenom, s.nom AS asso
FROM adherents a
LEFT JOIN assos s ON s.id = a.asso_id
ORDER BY a.id;
```

```console
 prenom |   asso
--------+-----------
 Alice  | Ciné-club
 Bilal  | Ciné-club
 Chloé  | Robotique
 David  | Robotique
 Emma   |
(5 rows)
```

| Jointure | Garde |
| --- | --- |
| `JOIN` (`INNER JOIN`) | Seulement les lignes qui ont une correspondance des deux côtés |
| `LEFT JOIN` | Toutes les lignes de gauche, `NULL` à droite s'il n'y a rien |

Le `LEFT JOIN` sert aussi à **trouver ce qui manque**. Les adhérent·e·s sans aucune inscription :

```sql
SELECT a.prenom, a.nom
FROM adherents a
LEFT JOIN inscriptions i ON i.adherent_id = a.id
WHERE i.adherent_id IS NULL;
```

```console
 prenom |  nom
--------+-------
 Emma   | Petit
(1 row)
```

## Joindre plus de deux tables

Pour savoir qui s'est inscrit à quoi, on traverse la **table de liaison** `inscriptions` (une table dont les lignes ne font que relier deux autres tables). On enchaîne deux jointures :

```sql
SELECT a.prenom, e.titre
FROM inscriptions i
JOIN adherents  a ON a.id = i.adherent_id
JOIN evenements e ON e.id = i.evenement_id
ORDER BY e.debut, a.prenom;
```

```console
 prenom |       titre
--------+--------------------
 Alice  | Soirée Kubrick
 Bilal  | Soirée Kubrick
 Chloé  | Coupe de robotique
 David  | Coupe de robotique
 Alice  | Courts-métrages
 Chloé  | Atelier soudure
(6 rows)
```

Une jointure se combine avec ce que tu as vu : filtres, tris, agrégats. Les inscrit·e·s par événement :

```sql
SELECT e.titre, count(*) AS inscrits
FROM evenements e
JOIN inscriptions i ON i.evenement_id = e.id
GROUP BY e.titre
ORDER BY inscrits DESC, e.titre;
```

```console
       titre        | inscrits
--------------------+----------
 Coupe de robotique |        2
 Soirée Kubrick     |        2
 Atelier soudure    |        1
 Courts-métrages    |        1
(4 rows)
```

## Compter avec un LEFT JOIN

Combien d'adhérent·e·s par association, **y compris celles qui n'en ont aucun·e** ?

```sql
SELECT s.nom, count(a.id) AS adherents
FROM assos s
LEFT JOIN adherents a ON a.asso_id = s.id
GROUP BY s.nom
ORDER BY s.nom;
```

```console
    nom    | adherents
-----------+-----------
 Ciné-club |         2
 Jazz      |         0
 Robotique |         2
(3 rows)
```

:::warning count(*) ment sur un LEFT JOIN
Avec `count(*)`, l'association Jazz serait comptée avec **1** adhérent·e : la ligne existe, même si la partie droite est vide. Compte une colonne de la table de droite (`count(a.id)`) : les `NULL` ne sont pas comptés.
:::

:::danger Une jointure sans condition multiplie les lignes
Oublier le `ON` (ou écrire `FROM adherents, assos` sans `WHERE`) produit un **produit cartésien** : chaque ligne de gauche est associée à chaque ligne de droite. Ici, 5 × 3 = 15 lignes. Sur des tables de 100 000 lignes, la requête peut saturer le serveur : vérifie toujours la condition.
:::

## Entraîne-toi

Comme à la leçon précédente, tu écris chaque requête dans un fichier `.sql` (`echo "SELECT …;" > fichier.sql`, ou `nano fichier.sql`) et le serveur l'exécute pour la vérifier. Teste-la toi-même avec `psql -At -f fichier.sql` : `-f` exécute le fichier, `-A -t` ne gardent que les valeurs séparées par `|`.

:::labo
moteur: reel
intro: |
  Les quatre tables sont chargées dans la base `asso`. Pour chaque étape, écris la jointure demandée dans le fichier `.sql` indiqué et teste-la avec `psql -At -f fichier.sql`. Le serveur exécute lui-même ton fichier et compare son résultat ; une réponse recopiée à la main ne passe pas. Respecte l'**ordre des colonnes** et le **tri** demandés.
commandes:
  - /opt/exercices/demarrer.sh
  - psql -q -v ON_ERROR_STOP=1 -f /opt/exercices/schema.sql -f /opt/exercices/donnees.sql
etapes:
  - texte: 'Avec un `JOIN`, écris dans `adherents-asso.sql` une requête qui donne le prénom puis le nom de l''association de chaque adhérent·e qui en a une, triés par `adherents.id`'
    indice: 'Écris `SELECT a.prenom, s.nom FROM adherents a JOIN assos s ON s.id = a.asso_id ORDER BY a.id;` dans le fichier (avec `nano` ou `echo "…" > adherents-asso.sql`), puis teste avec `psql -At -f adherents-asso.sql`.'
    verif:
      - commande-reussit: python3 /opt/exercices/verif-requete.py adherents-asso.sql 'Alice|Ciné-club\nBilal|Ciné-club\nChloé|Robotique\nDavid|Robotique' 'ALICE|CINÉ-CLUB\nBILAL|CINÉ-CLUB\nCHLOÉ|ROBOTIQUE\nDAVID|ROBOTIQUE\nZOÉ|ROBOTIQUE'
    solution:
      - printf '%s\n' "SELECT a.prenom, s.nom FROM adherents a JOIN assos s ON s.id = a.asso_id ORDER BY a.id;" > adherents-asso.sql
  - texte: 'Refais la même liste dans `tous.sql` mais avec un `LEFT JOIN`, pour **garder Emma** (son association restera vide)'
    indice: 'Remplace `JOIN` par `LEFT JOIN` : la dernière ligne de la sortie de `psql -At -f tous.sql` doit être `Emma|`.'
    verif:
      - commande-reussit: python3 /opt/exercices/verif-requete.py tous.sql 'Alice|Ciné-club\nBilal|Ciné-club\nChloé|Robotique\nDavid|Robotique\nEmma|' 'ALICE|CINÉ-CLUB\nBILAL|CINÉ-CLUB\nCHLOÉ|ROBOTIQUE\nDAVID|ROBOTIQUE\nEMMA|\nZOÉ|ROBOTIQUE'
    solution:
      - printf '%s\n' "SELECT a.prenom, s.nom FROM adherents a LEFT JOIN assos s ON s.id = a.asso_id ORDER BY a.id;" > tous.sql
  - texte: 'Écris dans `sans-inscription.sql` une requête qui donne le prénom et le nom des adhérent·e·s qui **n''ont aucune inscription**'
    indice: 'Un `LEFT JOIN inscriptions i ON i.adherent_id = a.id`, puis `WHERE i.adherent_id IS NULL`. Teste avec `psql -At -f sans-inscription.sql`.'
    verif:
      - commande-reussit: python3 /opt/exercices/verif-requete.py sans-inscription.sql 'Emma|Petit' 'EMMA|PETIT'
    solution:
      - printf '%s\n' "SELECT a.prenom, a.nom FROM adherents a LEFT JOIN inscriptions i ON i.adherent_id = a.id WHERE i.adherent_id IS NULL;" > sans-inscription.sql
  - texte: 'Avec deux jointures à travers `inscriptions`, écris dans `qui-vient.sql` une requête qui donne le prénom puis le titre de l''événement de chaque inscription, triés par `evenements.debut` puis par prénom'
    indice: 'Pars de `inscriptions i`, joins `adherents a ON a.id = i.adherent_id` puis `evenements e ON e.id = i.evenement_id`, et termine par `ORDER BY e.debut, a.prenom`. Teste avec `psql -At -f qui-vient.sql`.'
    verif:
      - commande-reussit: python3 /opt/exercices/verif-requete.py qui-vient.sql 'Alice|Soirée Kubrick\nBilal|Soirée Kubrick\nChloé|Coupe de robotique\nDavid|Coupe de robotique\nAlice|Courts-métrages\nChloé|Atelier soudure' 'ALICE|SOIRÉE KUBRICK\nBILAL|SOIRÉE KUBRICK\nCHLOÉ|COUPE DE ROBOTIQUE\nDAVID|COUPE DE ROBOTIQUE\nZOÉ|COUPE DE ROBOTIQUE\nALICE|COURTS-MÉTRAGES\nCHLOÉ|ATELIER SOUDURE'
    solution:
      - printf '%s\n' "SELECT a.prenom, e.titre FROM inscriptions i JOIN adherents a ON a.id = i.adherent_id JOIN evenements e ON e.id = i.evenement_id ORDER BY e.debut, a.prenom;" > qui-vient.sql
  - texte: 'Écris dans `effectifs.sql` une requête qui donne le nom de chaque association (y compris `Jazz`, qui n''a personne) et son nombre d''adhérent·e·s, triés par nom d''association'
    indice: 'Pars de `assos s LEFT JOIN adherents a ON a.asso_id = s.id`, regroupe avec `GROUP BY s.nom` et compte `count(a.id)` (pas `count(*)`). Teste avec `psql -At -f effectifs.sql`.'
    verif:
      - commande-reussit: python3 /opt/exercices/verif-requete.py effectifs.sql 'Ciné-club|2\nJazz|0\nRobotique|2' 'CINÉ-CLUB|2\nJAZZ|0\nROBOTIQUE|3'
    solution:
      - printf '%s\n' "SELECT s.nom, count(a.id) FROM assos s LEFT JOIN adherents a ON a.asso_id = s.id GROUP BY s.nom ORDER BY s.nom;" > effectifs.sql
:::

## Vérifie tes acquis

:::quiz
Quelle jointure garde Emma, qui n'a pas d'association, dans la liste des adhérent·e·s avec leur association ?

- [ ] `JOIN assos s ON s.id = a.asso_id`
- [ ] `INNER JOIN assos s ON s.id = a.asso_id`
- [x] `LEFT JOIN assos s ON s.id = a.asso_id`, depuis `adherents a`
- [ ] `JOIN assos s ON s.id = a.id`

> Seul `LEFT JOIN` conserve les lignes de gauche sans correspondance. `JOIN` et `INNER JOIN` sont synonymes et écartent Emma.
:::

:::quiz
Pour relier les adhérent·e·s à leurs événements, par où passe-t-on ?

- [x] Par la table `inscriptions`, avec deux jointures
- [ ] Directement entre `adherents` et `evenements`, par leur `id`
- [ ] Par la table `assos`, avec une seule jointure
- [ ] Il faut copier les titres des événements dans `adherents`

> `adherents` et `evenements` ne se référencent pas directement : c'est `inscriptions`, la table de liaison, qui porte les deux clés étrangères.
:::

:::quiz
Que renvoie `SELECT count(*) AS lignes FROM adherents, assos;` avec 5 adhérent·e·s et 3 associations ?

- [ ] 5
- [ ] 4
- [ ] 8
- [x] 15

> Sans condition de jointure, on obtient le produit cartésien : 5 × 3 = 15 lignes.
:::

:::quiz
Pourquoi compter avec `count(a.id)` plutôt que `count(*)` dans un `LEFT JOIN` ?

- [ ] Parce que `count(*)` est interdit avec une jointure
- [x] Parce que `count(a.id)` ignore les `NULL` d'une association sans adhérent·e
- [ ] Parce que `count(a.id)` est toujours plus rapide
- [ ] Parce que `count(*)` ne compte que la première table

> Une association vide produit une ligne dont `a.id` vaut `NULL` : `count(*)` la compte, `count(a.id)` non.
:::
