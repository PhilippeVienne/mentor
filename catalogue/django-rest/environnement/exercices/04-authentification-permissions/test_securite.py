"""Tests de la leçon 4 : `pytest -q test_securite.py` te dit ce qu'il reste à faire."""
import pytest
from rest_framework import exceptions
from rest_framework.test import APIRequestFactory

from helpers_jwt import AUTRE_CLE_PRIVEE, fabriquer_jeton

CORPS = {"asso": "Ciné-club", "titre": "Ciné-débat", "date": "2099-01-01T10:00:00Z", "places": 40}


def requete_avec(jeton=None):
    en_tetes = {"HTTP_AUTHORIZATION": f"Bearer {jeton}"} if jeton else {}
    return APIRequestFactory().get("/v1/evenements/", **en_tetes)


def test_authentification_jeton_valide(db):
    from agenda.auth import KeycloakJWTAuthentication

    resultat = KeycloakJWTAuthentication().authenticate(requete_avec(fabriquer_jeton(roles=["staff"])))
    assert resultat is not None, "un jeton valide doit identifier la personne"
    utilisateur, claims = resultat
    assert utilisateur.username == "alice@example.org", "l'utilisateur est retrouvé par l'email du jeton"
    assert claims["resource_access"]["agenda-api"]["roles"] == ["staff"], "request.auth contient les claims"


def test_authentification_sans_en_tete(db):
    from agenda.auth import KeycloakJWTAuthentication

    assert KeycloakJWTAuthentication().authenticate(requete_avec()) is None, (
        "sans en-tête Authorization, la classe renvoie None (requête anonyme)"
    )


def test_authentification_jeton_refuse(db):
    from agenda.auth import KeycloakJWTAuthentication

    for jeton in ("pas-un-jeton", fabriquer_jeton(cle=AUTRE_CLE_PRIVEE), fabriquer_jeton(expire_dans=-10)):
        with pytest.raises(exceptions.AuthenticationFailed):
            KeycloakJWTAuthentication().authenticate(requete_avec(jeton))


def test_en_tete_bearer_annonce():
    from agenda.auth import KeycloakJWTAuthentication

    en_tete = KeycloakJWTAuthentication().authenticate_header(requete_avec())
    assert en_tete and en_tete.startswith("Bearer"), "authenticate_header doit annoncer le schéma Bearer"


def test_roles_et_permissions():
    from types import SimpleNamespace

    from agenda.permissions import HasRoleStaff, HasRoleStaffOrReadOnly, roles_du_jeton

    staff = SimpleNamespace(auth={"resource_access": {"agenda-api": {"roles": ["staff"]}}}, method="POST")
    autre_app = SimpleNamespace(auth={"resource_access": {"autre": {"roles": ["staff"]}}}, method="POST")
    sans_jeton = SimpleNamespace(auth=None, method="POST")
    lecture = SimpleNamespace(auth={}, method="GET")
    assert roles_du_jeton(staff) == {"staff"}
    assert roles_du_jeton(autre_app) == set(), "seuls les rôles de CETTE application comptent"
    assert roles_du_jeton(sans_jeton) == set(), "un jeton absent ou mal formé donne un ensemble vide"
    assert HasRoleStaff().has_permission(staff, None) is True
    assert HasRoleStaff().has_permission(lecture, None) is False
    assert HasRoleStaffOrReadOnly().has_permission(lecture, None) is True, "la lecture est libre"
    assert HasRoleStaffOrReadOnly().has_permission(autre_app, None) is False, "l'écriture exige le rôle"
    assert HasRoleStaffOrReadOnly().has_permission(staff, None) is True


def test_api_fermee_par_defaut(donnees, api):
    reponse = api.get("/v1/evenements/")
    assert reponse.status_code == 401, "sans jeton, l'API répond 401"
    assert reponse["WWW-Authenticate"].startswith("Bearer")
    faux = api.get("/v1/evenements/", HTTP_AUTHORIZATION="Bearer pas-un-jeton")
    assert faux.status_code == 401, "un jeton invalide donne aussi 401"
    bon = api.get("/v1/evenements/", HTTP_AUTHORIZATION=f"Bearer {fabriquer_jeton()}")
    assert bon.status_code == 200, "un jeton valide, même sans rôle, peut lire"


def test_ecriture_reservee_au_role_staff(donnees, api):
    from agenda.models import Evenement

    sans_role = f"Bearer {fabriquer_jeton()}"
    avec_role = f"Bearer {fabriquer_jeton(roles=['staff'])}"
    refus = api.post("/v1/evenements/", CORPS, format="json", HTTP_AUTHORIZATION=sans_role)
    assert refus.status_code == 403, "sans le rôle staff, l'écriture est refusée (403)"
    assert Evenement.objects.count() == 3, "un refus ne doit rien créer"
    ok = api.post("/v1/evenements/", CORPS, format="json", HTTP_AUTHORIZATION=avec_role)
    assert ok.status_code == 201, "avec le rôle staff, la création réussit"
    assert api.get("/v1/evenements/", HTTP_AUTHORIZATION=sans_role).status_code == 200, "la lecture reste libre"
