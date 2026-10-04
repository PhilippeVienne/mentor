"""Tests d'API de la leçon 7 : à écrire par toi, un par scénario. Lance-les avec `pytest -q`."""
from datetime import datetime, timezone

from django.contrib.auth import get_user_model
from django.test import override_settings
from rest_framework.test import APITestCase

import helpers_jwt
from agenda.models import Asso, Evenement

CORPS = {"asso": "Ciné-club", "titre": "Ciné-débat", "date": "2099-01-01T10:00:00Z", "places": 40}


def jeton_staff():
    return {"resource_access": {"agenda-api": {"roles": ["staff"]}}}


class EvenementApiTests(APITestCase):
    def setUp(self):
        self.asso = Asso.objects.create(nom="Ciné-club")
        self.evenement = Evenement.objects.create(
            asso=self.asso,
            titre="Soirée courts-métrages",
            date=datetime(2099, 3, 12, 17, 0, tzinfo=timezone.utc),
            places=2,
        )
        self.alice = get_user_model().objects.create(username="alice@example.org")

    def test_sans_identification_401(self):
        self.fail("À écrire : un GET sans identification sur /v1/evenements/ doit répondre 401")

    def test_creation_interdite_sans_role_403(self):
        self.fail("À écrire : un POST sans rôle doit répondre 403 et ne rien créer en base")

    def test_creation_avec_role_staff_201(self):
        self.fail("À écrire : un POST avec le rôle staff doit répondre 201")

    def test_donnees_invalides_400(self):
        self.fail("À écrire : un POST avec 0 place doit répondre 400, avec l'erreur sous « places »")

    @override_settings(OIDC_PUBLIC_KEY=helpers_jwt.CLE_PUBLIQUE)
    def test_jeton_signe_par_une_autre_cle_401(self):
        self.fail("À écrire : un jeton fabriqué avec helpers_jwt.AUTRE_CLE_PRIVEE doit donner 401")
