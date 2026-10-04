-- Base « asso » du labo : données FICTIVES (associations et adhérent·e·s inventées).
CREATE TABLE assos (
    id serial PRIMARY KEY,
    nom text NOT NULL
);
CREATE TABLE adherents (
    id serial PRIMARY KEY,
    nom text NOT NULL,
    email text,
    asso_id integer NOT NULL REFERENCES assos (id)
);
INSERT INTO assos (nom) VALUES ('Robotique'), ('Photographie');
INSERT INTO adherents (nom, email, asso_id) VALUES
    ('Camille Durand', 'camille@exemple.invalid', 1),
    ('Samir Benali', 'samir@exemple.invalid', 1),
    ('Lou Martin', 'lou@exemple.invalid', 2),
    ('Inès Moreau', 'ines@exemple.invalid', 2),
    ('Alex Petit', 'alex@exemple.invalid', 1);
