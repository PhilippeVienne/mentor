"""Interface en ligne de commande avec argparse."""


def construire_parser():
    """Retourne le parseur : option --fichier et sous-commandes ajouter, lister, terminer."""
    raise NotImplementedError("À toi de jouer : argparse.ArgumentParser et add_subparsers")


def main(argv=None):
    """Lit les arguments, appelle taches.py et stockage.py, affiche le résultat. Retourne 0 si tout va bien."""
    raise NotImplementedError("À toi de jouer : relie le parseur, le stockage et la logique")
