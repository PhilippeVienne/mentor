from datetime import datetime, timezone

from django.utils.encoding import force_str


def maintenant_utc():
    """Date et heure actuelles, en UTC."""
    return datetime.now(tz=timezone.utc)


def etiquette(adherent):
    """Texte affiché pour un adhérent dans les listes."""
    return force_str(adherent.nom_complet())
