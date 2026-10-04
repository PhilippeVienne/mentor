from django.test import TestCase

from .models import Adherent


class NomCompletTests(TestCase):
    def test_nom_complet_met_une_majuscule_a_chaque_mot(self):
        adherent = Adherent.objects.create(nom="  marie curie ")
        self.assertEqual(adherent.nom_complet(), "Marie Curie")
