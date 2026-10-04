"""Tests de la leçon 6 : `pytest -q test_documentation.py` te dit ce qu'il reste à faire."""


def description(reponse):
    assert reponse.status_code == 200, reponse.content
    return reponse.json()["description"]


def test_docstring_des_assos(donnees, api_lecteur):
    texte = description(api_lecteur.options("/v1/assos/"))
    assert "Associations et leurs événements" in texte, (
        "la docstring de AssoViewSet doit commencer par « Associations et leurs événements »"
    )


def test_docstrings_des_evenements(donnees, api_lecteur, api_staff):
    assert description(api_lecteur.options("/v1/evenements/")).strip(), (
        "EvenementViewSet doit avoir une docstring (OPTIONS /v1/evenements/ la renvoie)"
    )
    fermer = api_staff.options(f"/v1/evenements/{donnees['courts'].pk}/fermer/")
    assert description(fermer).strip(), "l'action fermer doit avoir sa propre docstring"
