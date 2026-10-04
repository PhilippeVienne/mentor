-- Ajoute 20 000 adhérent·e·s fictif·ve·s pour que l'index serve vraiment (leçon « Index et EXPLAIN »).
INSERT INTO adherents (prenom, nom, email, asso_id)
SELECT 'Prénom' || g, 'Nom' || g, 'adherent' || g || '@example.org', 1
FROM generate_series(1, 20000) AS g;
ANALYZE adherents;
