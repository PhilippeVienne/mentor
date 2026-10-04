"""Données et clients de test partagés par tous les exercices."""
from datetime import datetime, timezone

import pytest
from django.contrib.auth import get_user_model
from rest_framework.test import APIClient

import helpers_jwt


@pytest.fixture(autouse=True)
def cle_publique_de_test(settings):
    """Les jetons des tests sont signés avec une paire de clés fabriquée à la volée."""
    settings.OIDC_PUBLIC_KEY = helpers_jwt.CLE_PUBLIQUE


@pytest.fixture
def donnees(db):
    """Deux associations et trois événements (deux ouverts, un fermé)."""
    from agenda.models import Asso, Evenement, Inscription

    cine = Asso.objects.create(nom="Ciné-club")
    fanfare = Asso.objects.create(nom="Fanfare")
    courts = Evenement.objects.create(
        asso=cine, titre="Soirée courts-métrages", date=datetime(2099, 3, 12, 17, 0, tzinfo=timezone.utc), places=2
    )
    Evenement.objects.create(
        asso=fanfare, titre="Concert de printemps", date=datetime(2099, 5, 2, 18, 0, tzinfo=timezone.utc), places=50
    )
    Evenement.objects.create(
        asso=cine,
        titre="Ciné-débat",
        date=datetime(2020, 1, 10, 19, 0, tzinfo=timezone.utc),
        places=10,
        ouvert=False,
    )
    Inscription.objects.create(evenement=courts, email="bob@example.org")
    return {"cine": cine, "fanfare": fanfare, "courts": courts}


@pytest.fixture
def alice(db):
    return get_user_model().objects.create(username="alice@example.org")


@pytest.fixture
def api():
    """Client d'API sans identification."""
    return APIClient()


@pytest.fixture
def api_lecteur(alice):
    """Client identifié, sans aucun rôle (jeton réduit à `{}`)."""
    client = APIClient()
    client.force_authenticate(user=alice, token={})
    return client


@pytest.fixture
def api_staff(alice):
    """Client identifié avec le rôle `staff`."""
    client = APIClient()
    client.force_authenticate(user=alice, token={"resource_access": {"agenda-api": {"roles": ["staff"]}}})
    return client
