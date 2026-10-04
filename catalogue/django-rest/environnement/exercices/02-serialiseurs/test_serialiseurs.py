"""Tests de la leçon 2 : `pytest -q test_serialiseurs.py` te dit ce qu'il reste à faire."""
from datetime import datetime, timezone

from agenda.serializers import AssoSerializer, EvenementSerializer


def test_asso_affichee_par_son_nom(donnees):
    courts = donnees["courts"]
    assert EvenementSerializer(courts).data["asso"] == "Ciné-club", "l'association doit s'afficher par son nom"
    entrant = EvenementSerializer(
        data={"asso": "Fanfare", "titre": "Bal", "date": "2099-06-01T20:00:00Z", "places": 80}
    )
    assert entrant.is_valid(), entrant.errors
    assert entrant.validated_data["asso"] == donnees["fanfare"], "en entrée, le nom doit retrouver l'association"


def test_places_restantes(donnees):
    data = EvenementSerializer(donnees["courts"]).data
    assert data["places_restantes"] == 1, "2 places moins 1 inscription : il en reste 1"


def test_places_positives(donnees):
    s = EvenementSerializer(
        data={"asso": "Fanfare", "titre": "Bal", "date": "2099-06-01T20:00:00Z", "places": 0}
    )
    assert not s.is_valid(), "0 place doit être refusé"
    assert "places" in s.errors, "l'erreur doit être rangée sous le champ « places »"
    assert "au moins une place" in str(s.errors["places"][0])


def test_evenement_passe_ne_peut_pas_etre_ouvert(donnees):
    s = EvenementSerializer(
        data={"asso": "Fanfare", "titre": "Vieux", "date": "2001-01-01T10:00:00Z", "places": 5, "ouvert": True}
    )
    assert not s.is_valid(), "un événement passé ne peut pas être ouvert"
    assert "non_field_errors" in s.errors, "l'erreur de validate() est rangée sous non_field_errors"
    ferme = EvenementSerializer(
        data={"asso": "Fanfare", "titre": "Vieux", "date": "2001-01-01T10:00:00Z", "places": 5, "ouvert": False}
    )
    assert ferme.is_valid(), "un événement passé mais fermé reste valide"


def test_asso_contient_ses_evenements(donnees):
    data = AssoSerializer(donnees["cine"]).data
    titres = sorted(e["titre"] for e in data["evenements"])
    assert titres == ["Ciné-débat", "Soirée courts-métrages"], "l'association doit lister ses événements"
