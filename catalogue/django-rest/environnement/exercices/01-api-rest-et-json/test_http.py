"""Tests de la leçon 1 : chaque fonction de `requetes.py` doit envoyer la bonne requête."""
import requetes


def test_lister(donnees):
    reponse = requetes.lister()
    assert reponse.status_code == 200, "GET /v1/evenements/ doit répondre 200"
    assert reponse["Content-Type"].startswith("application/json"), "la réponse doit être du JSON"
    assert len(reponse.json()) == 3, "il y a trois événements dans la base de test"


def test_lire(donnees):
    reponse = requetes.lire(donnees["courts"].pk)
    assert reponse.status_code == 200, "GET /v1/evenements/<id>/ doit répondre 200"
    assert reponse.json()["titre"] == "Soirée courts-métrages"


def test_creer(donnees):
    reponse = requetes.creer({"asso": "Fanfare", "titre": "Bal", "date": "2099-06-01T20:00:00Z", "places": 80})
    assert reponse.status_code == 201, "un POST valide doit répondre 201 (créé)"
    assert reponse.json()["titre"] == "Bal"
    assert reponse.json()["id"], "la réponse contient l'identifiant du nouvel événement"


def test_supprimer(donnees):
    reponse = requetes.supprimer(donnees["courts"].pk)
    assert reponse.status_code == 204, "un DELETE réussi répond 204 (rien à renvoyer)"
    assert requetes.lire(donnees["courts"].pk).status_code == 404, "l'événement supprimé doit être introuvable"


def test_introuvable(donnees):
    reponse = requetes.introuvable()
    assert reponse.status_code == 404, "un événement qui n'existe pas donne 404"


def test_invalide(donnees):
    reponse = requetes.invalide()
    assert reponse.status_code == 400, "des données incomplètes donnent 400"
    assert "titre" in reponse.json(), "l'erreur indique le champ fautif"
