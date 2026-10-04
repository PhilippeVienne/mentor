import pytest

from todo import taches


def test_ajouter():
    avant = []
    apres = taches.ajouter(avant, "Acheter du pain")
    assert apres == [{"texte": "Acheter du pain", "fait": False}]
    assert avant == [], "ajouter doit retourner une nouvelle liste sans modifier celle reçue"


def test_terminer():
    liste = [{"texte": "a", "fait": False}, {"texte": "b", "fait": False}]
    resultat = taches.terminer(liste, 2)
    assert resultat[1]["fait"] is True and resultat[0]["fait"] is False
    assert liste[1]["fait"] is False, "terminer doit retourner une copie, pas modifier la liste reçue"


def test_terminer_numero_inconnu():
    with pytest.raises(IndexError):
        taches.terminer([], 1)


def test_formater():
    liste = [{"texte": "Acheter du pain", "fait": False}, {"texte": "Écrire la doc", "fait": True}]
    assert taches.formater(liste) == "1. [ ] Acheter du pain\n2. [x] Écrire la doc"


def test_formater_liste_vide():
    assert taches.formater([]) == "Aucune tâche."
