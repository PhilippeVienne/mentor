"""Tests de la leçon 5 : `pytest -q test_filtres.py` te dit ce qu'il reste à faire."""


def titres(reponse):
    corps = reponse.json()
    liste = corps["results"] if isinstance(corps, dict) else corps
    return [e["titre"] for e in liste]


def test_recherche_et_tri(donnees, api_lecteur):
    reponse = api_lecteur.get("/v1/evenements/?search=Ciné")
    assert reponse.status_code == 200
    assert sorted(titres(reponse)) == ["Ciné-débat", "Soirée courts-métrages"], (
        "?search=Ciné doit trouver les titres qui contiennent « Ciné » et les associations qui commencent par « Ciné »"
    )
    assert titres(api_lecteur.get("/v1/evenements/?ordering=-date")) == [
        "Concert de printemps",
        "Soirée courts-métrages",
        "Ciné-débat",
    ], "?ordering=-date trie du plus récent au plus ancien"
    assert titres(api_lecteur.get("/v1/evenements/?ordering=-places"))[0] == "Concert de printemps"


def test_filtres_exacts(donnees, api_lecteur):
    ferme = api_lecteur.get("/v1/evenements/?ouvert=false")
    assert titres(ferme) == ["Ciné-débat"], "?ouvert=false ne garde que les événements fermés"
    de_la_fanfare = api_lecteur.get(f"/v1/evenements/?asso={donnees['fanfare'].pk}")
    assert titres(de_la_fanfare) == ["Concert de printemps"], "?asso=<id> filtre sur l'association"


def test_pagination_globale(donnees, api_lecteur):
    reponse = api_lecteur.get("/v1/evenements/")
    corps = reponse.json()
    assert isinstance(corps, dict), "la réponse paginée est un objet, plus une simple liste"
    assert corps["count"] == 3 and corps["next"] is None and corps["previous"] is None
    assert len(corps["results"]) == 3
    assert api_lecteur.get("/v1/evenements/?page=9").status_code == 404, "une page inexistante donne 404"


def test_taille_de_page_choisie_par_le_client(donnees, api_lecteur):
    from agenda.pagination import EvenementsPagination

    assert EvenementsPagination.page_size == 20
    assert EvenementsPagination.max_page_size == 100, "plafond : 100 éléments par page"
    reponse = api_lecteur.get("/v1/evenements/?page_size=1").json()
    assert len(reponse["results"]) == 1, "?page_size=1 doit renvoyer un seul événement"
    assert reponse["count"] == 3 and reponse["next"] is not None


def test_export_csv(donnees, api_staff, api_lecteur):
    refus = api_lecteur.get("/v1/evenements/export/")
    assert refus.status_code == 403, "l'export est réservé au rôle staff"
    reponse = api_staff.get("/v1/evenements/export/?ouvert=true")
    assert reponse.status_code == 200
    assert reponse["Content-Type"].startswith("text/csv"), "l'export est un fichier CSV"
    assert "evenements.csv" in reponse["Content-Disposition"]
    lignes = reponse.content.decode().splitlines()
    assert lignes[0] == "titre,asso,date", "première ligne : les en-têtes de colonnes"
    assert len(lignes) == 3, "?ouvert=true filtre aussi l'export : 2 événements ouverts + l'en-tête"
    assert lignes[1].startswith("Soirée courts-métrages,Ciné-club,2099-03-12")
