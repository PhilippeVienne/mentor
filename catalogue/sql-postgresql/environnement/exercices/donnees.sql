-- Jeu de données fictif (aucune donnée réelle).
INSERT INTO assos (nom, cree_le) VALUES
    ('Ciné-club', '2019-09-01'),
    ('Robotique', '2021-02-15'),
    ('Jazz',      '2023-10-01');

INSERT INTO adherents (prenom, nom, email, asso_id) VALUES
    ('Alice', 'Martin', 'alice.martin@example.org', 1),
    ('Bilal', 'Haddad', 'bilal.haddad@example.org', 1),
    ('Chloé', 'Durand', 'chloe.durand@example.org', 2),
    ('David', 'Roux',   'david.roux@example.org',    2),
    ('Emma',  'Petit',  'emma.petit@example.org',    NULL);

INSERT INTO evenements (asso_id, titre, lieu, debut, places_restantes) VALUES
    (1, 'Soirée Kubrick',     'Salle A', '2026-11-12 20:00+01', 30),
    (1, 'Courts-métrages',    'Salle B', '2026-12-03 19:30+01', 20),
    (2, 'Coupe de robotique', 'Gymnase', '2026-11-21 14:00+01', 10),
    (2, 'Atelier soudure',    NULL,      '2026-12-10 18:00+01',  8);

INSERT INTO inscriptions (adherent_id, evenement_id) VALUES
    (1, 1), (2, 1), (1, 2), (3, 3), (4, 3), (3, 4);
