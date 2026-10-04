#!/bin/sh
# Lance les tests de l'application. C'est cette commande que le job « tests » du pipeline devra lancer.
python3 -B -m unittest discover -s app
