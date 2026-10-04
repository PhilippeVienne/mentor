"""Tests de la leçon 3 : `pytest -q test_vues.py` te dit ce qu'il reste à faire."""
from rest_framework.test import APIRequestFactory


def test_evenement_viewset_repond_a_la_liste(donnees):
    from agenda.views import EvenementViewSet

    requete = APIRequestFactory().get("/v1/evenements/")
    reponse = EvenementViewSet.as_view({"get": "list"})(requete)
    assert reponse.status_code == 200, "la liste doit répondre 200"
    assert len(reponse.data) == 3, "il y a trois événements"
    assert reponse.data[0]["titre"] == "Ciné-débat", "ordre par date croissante : le plus ancien d'abord"


def test_evenements_branches_sur_le_routeur(donnees, api):
    assert api.get("/v1/evenements/").status_code == 200, "GET /v1/evenements/ doit répondre 200"
    corps = {"asso": "Fanfare", "titre": "Bal", "date": "2099-06-01T20:00:00Z", "places": 80}
    creation = api.post("/v1/evenements/", corps, format="json")
    assert creation.status_code == 201, creation.content
    suppression = api.delete(f"/v1/evenements/{creation.json()['id']}/")
    assert suppression.status_code == 204
    assert api.get("/v1/").status_code == 200, "le DefaultRouter ajoute une page racine sous /v1/"


def test_assos_en_lecture_seule(donnees, api):
    assert api.get("/v1/assos/").status_code == 200, "GET /v1/assos/ doit répondre 200"
    assert api.post("/v1/assos/", {"nom": "Nouvelle"}, format="json").status_code == 405, (
        "une ressource en lecture seule refuse l'écriture (405)"
    )


def test_filtre_a_venir(donnees, api):
    tous = api.get("/v1/evenements/").json()
    a_venir = api.get("/v1/evenements/?a_venir=true").json()
    assert len(tous) == 3
    assert [e["titre"] for e in a_venir] == ["Soirée courts-métrages", "Concert de printemps"], (
        "?a_venir=true ne garde que les événements dont la date n'est pas passée"
    )


def test_action_fermer(donnees, api):
    reponse = api.post(f"/v1/evenements/{donnees['courts'].pk}/fermer/")
    assert reponse.status_code == 200, "POST /v1/evenements/<id>/fermer/ doit répondre 200"
    assert reponse.json()["ouvert"] is False
    assert api.get(f"/v1/evenements/{donnees['courts'].pk}/fermer/").status_code == 405, "seul POST est permis"
