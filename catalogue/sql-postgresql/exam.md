---
title: "Examen de validation — SQL et PostgreSQL"
draw: 5
pass_mark: 80
minutes: 10
shuffle: true
---

Cet examen valide les bases de SQL et de PostgreSQL : schéma et clés, `SELECT`, jointures, transactions, index, migrations et sauvegardes. Les exemples reprennent le schéma fictif du cours (`assos`, `adherents`, `evenements`, `inscriptions`).

:::quiz
Quelle contrainte garantit qu'aucune valeur n'apparaît deux fois dans la colonne `email` ?

- [ ] `NOT NULL`
- [ ] `DEFAULT`
- [x] `UNIQUE`
- [ ] `REFERENCES`

> `UNIQUE` interdit les doublons. `NOT NULL` interdit seulement l'absence de valeur.
:::

:::quiz
Quel est le rôle d'une clé primaire ?

- [x] Identifier chaque ligne de la table de façon unique
- [ ] Relier la table à une autre table
- [ ] Accélérer uniquement les requêtes d'écriture
- [ ] Interdire toute modification de la ligne

> Une clé primaire est unique et jamais vide. C'est elle que les clés étrangères des autres tables référencent.
:::

:::quiz
Tu essaies de supprimer une association qui a encore des adhérent·e·s rattaché·e·s via une clé étrangère sans `ON DELETE CASCADE`. Que se passe-t-il ?

- [ ] Les adhérent·e·s sont supprimé·e·s en même temps
- [ ] Les adhérent·e·s sont rattaché·e·s à l'association numéro 1
- [x] PostgreSQL refuse la suppression avec une erreur de clé étrangère
- [ ] L'association est supprimée et les `asso_id` des adhérent·e·s restent orphelins

> Par défaut, une clé étrangère empêche de supprimer une ligne encore référencée. `CASCADE` demanderait explicitement la suppression en cascade.
:::

:::quiz
Que renvoie `SELECT prenom FROM adherents WHERE asso_id <> 1;` si une personne a `asso_id` à `NULL` ?

- [ ] Cette personne est incluse, car `NULL` est différent de 1
- [x] Cette personne est absente, car la comparaison avec `NULL` n'est jamais vraie
- [ ] La requête échoue avec une erreur
- [ ] Toutes les lignes sont renvoyées

> `NULL <> 1` donne « inconnu », pas « vrai ». Pour inclure ces lignes, ajoute `OR asso_id IS NULL`.
:::

:::quiz
Quelle requête donne le nombre d'événements par association ?

- [x] `SELECT asso_id, count(*) FROM evenements GROUP BY asso_id;`
- [ ] `SELECT asso_id, count(*) FROM evenements;`
- [ ] `SELECT count(*) FROM evenements ORDER BY asso_id;`
- [ ] `SELECT asso_id FROM evenements HAVING count(*) > 0 ORDER BY asso_id;`

> Pour avoir un résultat par association, il faut regrouper avec `GROUP BY asso_id`.
:::

:::quiz
Dans quel ordre PostgreSQL évalue-t-il ces clauses ?

- [ ] `SELECT`, puis `FROM`, puis `WHERE`
- [ ] `WHERE`, puis `FROM`, puis `SELECT`
- [x] `FROM`, puis `WHERE`, puis `SELECT`
- [ ] `ORDER BY`, puis `WHERE`, puis `FROM`

> On lit d'abord la table (`FROM`), on filtre les lignes (`WHERE`), puis on calcule les colonnes (`SELECT`). Le tri vient en dernier.
:::

:::quiz
Tu veux la liste de toutes les associations avec leur nombre d'événements, y compris celles qui n'en ont aucun. Quelle jointure choisis-tu depuis `assos` ?

- [ ] `JOIN evenements`
- [ ] `INNER JOIN evenements`
- [ ] Aucune, un `GROUP BY` suffit
- [x] `LEFT JOIN evenements`

> Seul `LEFT JOIN` conserve les associations sans événement. Avec `JOIN`, elles disparaissent du résultat.
:::

:::quiz
Un `LEFT JOIN` renvoie une ligne dont toutes les colonnes de la table de droite sont vides. Que cela signifie-t-il ?

- [ ] Que la table de droite est vide
- [x] Qu'aucune ligne de droite ne correspond à cette ligne de gauche
- [ ] Que la jointure a échoué parce que la condition `ON` est invalide
- [ ] Qu'il manque un index

> PostgreSQL complète avec des `NULL` quand aucune ligne de droite ne vérifie la condition `ON`.
:::

:::quiz
Quelle est la condition de jointure correcte pour relier `evenements e` à `assos s` ?

- [ ] `ON e.id = s.id`
- [ ] `ON e.titre = s.nom`
- [ ] `ON e.places_restantes = s.id`
- [x] `ON e.asso_id = s.id`

> On joint la clé étrangère (`evenements.asso_id`) à la clé primaire qu'elle référence (`assos.id`).
:::

:::quiz
Un `UPDATE` affiche `UPDATE 5000` alors que tu pensais modifier une seule ligne. Que fais-tu, si tu es dans une transaction ?

- [ ] `COMMIT`, car la requête s'est bien terminée
- [ ] Relancer la même requête pour corriger
- [ ] Fermer le terminal pour interrompre
- [x] `ROLLBACK`, puis vérifier le `WHERE`

> Tant que le `COMMIT` n'est pas fait, `ROLLBACK` annule la modification. Le compteur révèle un `WHERE` trop large.
:::

:::quiz
Que garantit une transaction ?

- [x] Que toutes les instructions sont enregistrées ensemble, ou aucune
- [ ] Que la requête est plus rapide
- [ ] Que personne d'autre ne peut jamais lire la table
- [ ] Que les erreurs de syntaxe sont corrigées automatiquement

> C'est l'atomicité : un bloc d'instructions réussit ou échoue d'un seul tenant.
:::

:::quiz
Que fait cette contrainte : `CHECK (places_restantes >= 0)` ?

- [ ] Elle calcule automatiquement le nombre de places restantes à chaque inscription
- [x] Elle refuse toute ligne dont le nombre de places serait négatif
- [ ] Elle remplace `NOT NULL`
- [ ] Elle supprime les événements complets

> Une contrainte `CHECK` vérifie une condition à chaque insertion ou modification, et refuse la ligne si elle est fausse.
:::

:::quiz
Dans un plan `EXPLAIN`, quel nœud indique que PostgreSQL lit toute la table ?

- [ ] `Index Scan`
- [ ] `Index Cond`
- [ ] `Planning Time`
- [x] `Seq Scan`

> `Seq Scan` (lecture séquentielle) parcourt toutes les lignes. `Index Scan` passe par un index.
:::

:::quiz
Tu vas utiliser `EXPLAIN ANALYZE` sur un `DELETE`. Quelle précaution prends-tu ?

- [ ] Aucune : `EXPLAIN` n'exécute jamais la requête
- [x] Tu l'enveloppes dans `BEGIN` et `ROLLBACK`, car `ANALYZE` exécute vraiment la suppression
- [ ] Tu ajoutes `LIMIT 1`
- [ ] Tu crées d'abord un index sur toutes les colonnes

> `EXPLAIN ANALYZE` exécute la requête pour mesurer son temps. Pour un `DELETE`, les lignes sont réellement supprimées sans transaction annulée.
:::

:::quiz
Une requête `WHERE asso_id = 2` est lente sur `adherents`, et `asso_id` est une clé étrangère. Que sais-tu ?

- [ ] PostgreSQL a déjà indexé cette colonne automatiquement
- [ ] Les clés étrangères ne peuvent pas être indexées
- [x] Aucun index n'existe par défaut sur une clé étrangère : tu peux en créer un
- [ ] La requête est lente à cause du `NOT NULL`

> PostgreSQL indexe les clés primaires et les colonnes `UNIQUE`, mais pas les clés étrangères.
:::

:::quiz
Pourquoi une migration est-elle préférable à un `ALTER TABLE` tapé directement en production ?

- [ ] Parce que `ALTER TABLE` est interdit en production
- [x] Parce que le changement est versionné, relu et rejouable sur tous les environnements
- [ ] Parce qu'une migration ne peut pas échouer
- [ ] Parce qu'une migration est plus rapide qu'un `ALTER TABLE`

> Une migration vit dans Git avec le code : elle est relue en merge request et appliquée de la même façon partout.
:::

:::quiz
Que fait `pg_dump -d asso -Fc -f asso.dump` ?

- [ ] Il restaure la base `asso` depuis `asso.dump`
- [x] Il écrit une sauvegarde de la base `asso` au format custom dans `asso.dump`
- [ ] Il supprime la base `asso` après en avoir fait une copie
- [ ] Il sauvegarde toutes les bases du serveur

> `pg_dump` sauvegarde **une** base. La restauration d'un fichier custom se fait avec `pg_restore`.
:::

:::quiz
Tu viens de réaliser une sauvegarde `pg_dump`. Que fais-tu pour t'assurer qu'elle est exploitable ?

- [ ] Tu vérifies que la taille du fichier est supérieure à zéro
- [ ] Tu la copies dans un second dossier du même serveur
- [x] Tu la restaures dans une base vide et tu vérifies les données
- [ ] Tu relances `pg_dump` pour comparer les deux fichiers

> Seul un test de restauration prouve que la sauvegarde sert à quelque chose.
:::
