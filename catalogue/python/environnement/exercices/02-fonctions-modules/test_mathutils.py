"""Tests de la leçon 2 : `pytest -q` te dit ce qu'il reste à faire."""

from mathutils import carre, est_pair, moyenne


def test_carre():
    assert carre(4) == 16, "carre(4) doit valoir 16"
    assert carre(-3) == 9, "un carré n'est jamais négatif : carre(-3) doit valoir 9"


def test_moyenne():
    assert moyenne([2, 4, 6]) == 4, "moyenne([2, 4, 6]) doit valoir 4"
    assert moyenne([10]) == 10, "la moyenne d'un seul nombre est ce nombre"


def test_moyenne_documentee():
    assert moyenne.__doc__ and moyenne.__doc__.strip(), (
        "ajoute une docstring (une phrase entre triples guillemets) juste sous « def moyenne »"
    )


def test_est_pair():
    assert est_pair(2) is True, "est_pair(2) doit retourner True"
    assert est_pair(7) is False, "est_pair(7) doit retourner False"
    assert est_pair(0) is True, "0 est pair"
