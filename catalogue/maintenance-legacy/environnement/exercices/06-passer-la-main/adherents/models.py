from django.db import models
from django.utils.translation import gettext_lazy as _


class Adherent(models.Model):
    nom = models.CharField(_("nom"), max_length=100)
    email = models.EmailField(_("courriel"), blank=True)

    def nom_complet(self):
        return self.nom.strip().title()
