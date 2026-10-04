from todo import stockage


def test_charger_fichier_absent(tmp_path):
    assert stockage.charger(str(tmp_path / "absent.json")) == []


def test_sauver_puis_charger(tmp_path):
    chemin = str(tmp_path / "t.json")
    donnees = [{"texte": "Écrire la doc", "fait": False}]
    stockage.sauver(chemin, donnees)
    assert stockage.charger(chemin) == donnees


def test_accents_lisibles_dans_le_fichier(tmp_path):
    chemin = tmp_path / "t.json"
    stockage.sauver(str(chemin), [{"texte": "Écrire la doc", "fait": False}])
    assert "Écrire la doc" in chemin.read_text(encoding="utf-8"), "utilise ensure_ascii=False pour garder les accents"
