#!/usr/bin/env python3
"""Vérifie qu'un fichier .sql contient UNE requête SELECT qui interroge vraiment la base.

Usage : python3 /opt/exercices/verif-requete.py FICHIER.sql ATTENDU ATTENDU_MODIFIE [tri]

  - le fichier ne doit contenir qu'une seule instruction, SELECT ou WITH, sans méta-commande psql (\\…) ;
  - la requête est exécutée par ce script (psql -f), en lecture seule, avec un délai maximal ;
  - sa sortie (psql -At) doit être égale à ATTENDU (\\n = retour à la ligne) ;
  - puis elle est exécutée de nouveau dans une transaction annulée où les données ont été MODIFIÉES (noms en
    majuscules, places en plus, une adhérente ajoutée) : la sortie doit alors être égale à ATTENDU_MODIFIE.
    Une réponse écrite « en dur » (SELECT 'Chloé', 'Durand') réussit la première comparaison mais pas la seconde ;
  - avec `tri`, l'ordre des lignes n'est pas comparé (la consigne n'impose pas de tri).
Code de sortie 0 = requête acceptée.
"""

import os
import re
import subprocess
import sys

MODIFICATION = """
UPDATE assos SET nom = upper(nom);
UPDATE adherents SET prenom = upper(prenom), nom = upper(nom);
UPDATE evenements SET titre = upper(titre), places_restantes = places_restantes + 100;
INSERT INTO adherents (prenom, nom, email, asso_id) VALUES ('ZOÉ', 'ZED', 'zoe.zed@example.org', 2);
INSERT INTO inscriptions (adherent_id, evenement_id)
    SELECT id, 3 FROM adherents WHERE email = 'zoe.zed@example.org';
"""


def echec(message):
    print(message, file=sys.stderr)
    sys.exit(1)


def psql(entree, lecture_seule):
    env = dict(os.environ)
    options = "-c statement_timeout=5000"
    if lecture_seule:
        options += " -c default_transaction_read_only=on"
    env["PGOPTIONS"] = options
    fait = subprocess.run(
        ["psql", "-q", "-At", "-v", "ON_ERROR_STOP=1", "-f", "-"],
        input=entree, capture_output=True, text=True, env=env, timeout=12,
    )
    if fait.returncode != 0:
        echec(fait.stderr.strip() or "psql a échoué")
    return fait.stdout


def lignes(texte, tri):
    resultat = [ligne.rstrip() for ligne in texte.strip().splitlines()]
    return sorted(resultat) if tri else resultat


def main():
    if len(sys.argv) < 4:
        echec(__doc__)
    fichier, attendu, attendu_modifie = sys.argv[1:4]
    tri = "tri" in sys.argv[4:]
    try:
        with open(fichier, encoding="utf-8") as f:
            texte = f.read(20000)
    except OSError as exc:
        echec(f"fichier illisible : {exc}")
    sans = re.sub(r"/\*.*?\*/", " ", texte, flags=re.S)
    sans = re.sub(r"--[^\n]*", " ", sans)
    if "\\" in sans:
        echec("pas de méta-commande psql dans le fichier")
    instructions = [s for s in sans.split(";") if s.strip()]
    if len(instructions) != 1 or not re.match(r"(?is)\s*(select|with)\b", instructions[0]):
        echec("le fichier doit contenir une seule requête SELECT terminée par un point-virgule")
    requete = instructions[0].strip() + ";\n"

    reel = psql(requete, lecture_seule=True)
    if lignes(reel, tri) != lignes(attendu.replace("\\n", "\n"), tri):
        echec("le résultat de la requête n'est pas celui attendu")
    modifie = psql(f"BEGIN;\n{MODIFICATION}\nSET LOCAL transaction_read_only = on;\n{requete}ROLLBACK;\n", lecture_seule=False)
    if lignes(modifie, tri) != lignes(attendu_modifie.replace("\\n", "\n"), tri):
        echec("la requête ne lit pas les tables : elle ne s'adapte pas aux données")


main()
