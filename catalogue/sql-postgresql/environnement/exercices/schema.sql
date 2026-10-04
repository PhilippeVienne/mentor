-- Schéma fictif du parcours : associations, adhérent·e·s, événements, inscriptions.
CREATE TABLE assos (
    id       integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    nom      text NOT NULL UNIQUE,
    cree_le  date NOT NULL DEFAULT CURRENT_DATE
);

CREATE TABLE adherents (
    id       integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    prenom   text NOT NULL,
    nom      text NOT NULL,
    email    text NOT NULL UNIQUE,
    asso_id  integer REFERENCES assos (id)
);

CREATE TABLE evenements (
    id               integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    asso_id          integer NOT NULL REFERENCES assos (id),
    titre            text NOT NULL,
    lieu             text,
    debut            timestamptz NOT NULL,
    places_restantes integer NOT NULL DEFAULT 0
);

CREATE TABLE inscriptions (
    adherent_id  integer REFERENCES adherents (id) ON DELETE CASCADE,
    evenement_id integer REFERENCES evenements (id),
    inscrit_le   timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (adherent_id, evenement_id)
);
