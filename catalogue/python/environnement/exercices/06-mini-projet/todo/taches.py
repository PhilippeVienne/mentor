"""Logique métier : une tâche est un dictionnaire {"texte": ..., "fait": False}. Pas d'entrée/sortie ici."""


def ajouter(taches, texte):
    """Retourne une NOUVELLE liste avec la tâche ajoutée à la fin (la liste d'origine ne change pas)."""
    raise NotImplementedError("À toi de jouer : taches + [{'texte': texte, 'fait': False}]")


def terminer(taches, numero):
    """Retourne une nouvelle liste où la tâche numéro `numero` (à partir de 1) est faite. IndexError si elle n'existe pas."""
    raise NotImplementedError("À toi de jouer : copie la liste, puis modifie la bonne tâche")


def formater(taches):
    """Retourne le texte à afficher : une ligne « 1. [ ] Acheter du pain » par tâche ([x] si faite)."""
    raise NotImplementedError("À toi de jouer : enumerate(taches, start=1)")
