from django.test import TestCase

from .models import Adherent
from .utils import etiquette, maintenant_utc


class ReferenceTests(TestCase):
    """Filet de sécurité posé avant la montée de version."""

    def test_nom_complet(self):
        adherent = Adherent.objects.create(nom="  marie curie ")
        self.assertEqual(adherent.nom_complet(), "Marie Curie")

    def test_etiquette(self):
        self.assertEqual(etiquette(Adherent(nom="ada lovelace")), "Ada Lovelace")

    def test_liste(self):
        Adherent.objects.create(nom="marie curie")
        reponse = self.client.get("/adherents/")
        self.assertEqual(reponse.json(), {"adherents": ["Marie Curie"]})

    def test_maintenant_est_en_utc(self):
        self.assertEqual(maintenant_utc().utcoffset().total_seconds(), 0)
