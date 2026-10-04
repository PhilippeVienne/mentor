"""Tests de la leçon 3."""

import inspect

from collections_exo import compter_mots, en_majuscules, inverser, somme_pairs


def test_compter_mots():
    assert compter_mots("a b a c a b") == {"a": 3, "b": 2, "c": 1}, "attendu : {'a': 3, 'b': 2, 'c': 1}"
    assert compter_mots("") == {}, "un texte vide donne un dictionnaire vide"


def test_en_majuscules():
    assert en_majuscules(["ada", "alan"]) == ["ADA", "ALAN"]
    assert en_majuscules([]) == []


def test_en_majuscules_utilise_une_comprehension():
    source = inspect.getsource(en_majuscules)
    assert "[" in source and " for " in source, "écris cette fonction avec une compréhension de liste : [ ... for ... in ... ]"


def test_somme_pairs():
    assert somme_pairs([1, 2, 3, 4, 10]) == 16, "2 + 4 + 10 = 16"
    assert somme_pairs([1, 3]) == 0, "aucun nombre pair : la somme vaut 0"
    assert somme_pairs([]) == 0


def test_inverser():
    assert inverser({"a": 1, "b": 2}) == {1: "a", 2: "b"}
    assert inverser({}) == {}
