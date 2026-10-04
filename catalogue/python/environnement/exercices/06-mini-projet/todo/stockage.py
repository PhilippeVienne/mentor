"""Lecture et écriture des tâches dans un fichier JSON."""


def charger(chemin):
    """Retourne la liste des tâches du fichier JSON, ou [] si le fichier n'existe pas encore."""
    raise NotImplementedError("À toi de jouer : json.load dans un try / except FileNotFoundError")


def sauver(chemin, taches):
    """Écrit la liste des tâches dans le fichier JSON (UTF-8, accents lisibles)."""
    raise NotImplementedError("À toi de jouer : json.dump(..., ensure_ascii=False)")
