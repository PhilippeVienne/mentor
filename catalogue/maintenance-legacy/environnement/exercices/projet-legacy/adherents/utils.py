from datetime import datetime

from django.utils.encoding import force_text
from django.utils.timezone import utc


def maintenant_utc():
    """Date et heure actuelles, en UTC."""
    return datetime.now(tz=utc)


def etiquette(adherent):
    """Texte affiché pour un adhérent dans les listes."""
    return force_text(adherent.nom_complet())
