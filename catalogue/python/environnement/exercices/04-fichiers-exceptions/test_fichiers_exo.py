"""Tests de la leçon 4 (les fichiers de test sont créés dans un dossier temporaire)."""

from fichiers_exo import ecrire_rapport, lire_config, lire_lignes, somme_nombres


def test_lire_lignes(tmp_path):
    fichier = tmp_path / "f.txt"
    fichier.write_text("a\n\nb\nc\n", encoding="utf-8")
    assert lire_lignes(str(fichier)) == ["a", "b", "c"], "les lignes vides sont ignorées, pas de \\n à la fin"


def test_somme_nombres(tmp_path):
    fichier = tmp_path / "n.txt"
    fichier.write_text("12\n7\n\nabc\n30\n", encoding="utf-8")
    assert somme_nombres(str(fichier)) == 49, "12 + 7 + 30 = 49 : « abc » est ignoré au lieu de faire planter le programme"


def test_somme_nombres_du_fichier_fourni():
    assert somme_nombres("nombres.txt") == 49


def test_ecrire_rapport(tmp_path):
    fichier = tmp_path / "rapport.txt"
    ecrire_rapport(str(fichier), [3, 4])
    assert fichier.read_text(encoding="utf-8").splitlines() == ["valeur: 3", "valeur: 4", "total: 7"]


def test_lire_config(tmp_path):
    fichier = tmp_path / "app.conf"
    fichier.write_text("langue=fr\ntheme=sombre\n", encoding="utf-8")
    assert lire_config(str(fichier)) == {"langue": "fr", "theme": "sombre"}


def test_lire_config_fichier_absent(tmp_path):
    assert lire_config(str(tmp_path / "absent.conf")) == {}, "un fichier absent donne {} (attrape FileNotFoundError)"
