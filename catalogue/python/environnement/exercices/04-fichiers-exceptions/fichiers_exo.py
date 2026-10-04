"""Fichiers et exceptions : écris les quatre fonctions (lis test_fichiers_exo.py)."""


def lire_lignes(chemin):
    """Retourne la liste des lignes non vides du fichier, sans le retour à la ligne."""
    raise NotImplementedError("À toi de jouer : with open(chemin, encoding='utf-8') as f: ...")


def somme_nombres(chemin):
    """Retourne la somme des lignes qui sont des entiers ; ignore les lignes invalides."""
    raise NotImplementedError("À toi de jouer : int(ligne) dans un try / except ValueError")


def ecrire_rapport(chemin, valeurs):
    """Écrit une ligne « valeur: X » par valeur, puis « total: S » à la fin."""
    raise NotImplementedError("À toi de jouer : with open(chemin, 'w', encoding='utf-8') as f: ...")


def lire_config(chemin):
    """Lit un fichier « cle=valeur » (une paire par ligne) ; retourne {} si le fichier n'existe pas."""
    raise NotImplementedError("À toi de jouer : attrape FileNotFoundError")
