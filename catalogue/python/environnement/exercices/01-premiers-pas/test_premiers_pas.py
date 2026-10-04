"""Tests de la leçon 1 : lance-les avec `pytest -q` quand tes trois scripts sont prêts."""

import re
import subprocess
import sys


def lancer(script):
    """Exécute un script Python et retourne ce qu'il affiche."""
    resultat = subprocess.run(
        [sys.executable, script], capture_output=True, text=True, timeout=10
    )
    assert resultat.returncode == 0, f"{script} n'a pas pu s'exécuter :\n{resultat.stderr}"
    return resultat.stdout


def test_salut():
    sortie = lancer("salut.py").strip()
    assert sortie == "Bonjour Mentor !", f"salut.py doit afficher exactement « Bonjour Mentor ! », il affiche {sortie!r}"


def test_profil():
    sortie = lancer("profil.py").strip()
    assert re.fullmatch(r"Je m'appelle .+ et j'ai \d+ ans", sortie), (
        f"profil.py doit afficher « Je m'appelle <prénom> et j'ai <âge> ans », il affiche {sortie!r}"
    )


def test_calcul():
    sortie = lancer("calcul.py").split()
    assert sortie == ["2", "1", "343"], (
        f"calcul.py doit afficher 7 // 3, 7 % 3 et 7 ** 3 (un résultat par ligne), il affiche {sortie}"
    )
