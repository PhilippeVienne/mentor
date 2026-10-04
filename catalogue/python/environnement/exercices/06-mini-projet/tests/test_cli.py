import json

import pytest

from todo import cli


def test_parser_connait_les_trois_commandes():
    parser = cli.construire_parser()
    assert parser.parse_args(["ajouter", "Pain"]).commande == "ajouter"
    assert parser.parse_args(["lister"]).commande == "lister"
    assert parser.parse_args(["terminer", "1"]).numero == 1


def test_parser_option_fichier_par_defaut():
    assert cli.construire_parser().parse_args(["lister"]).fichier == "taches.json"


def test_commande_inconnue_est_refusee():
    with pytest.raises(SystemExit):
        cli.construire_parser().parse_args(["supprimer"])


def test_ajouter_puis_lister(tmp_path, capsys):
    chemin = str(tmp_path / "t.json")
    assert cli.main(["--fichier", chemin, "ajouter", "Acheter du pain"]) == 0
    assert json.loads((tmp_path / "t.json").read_text(encoding="utf-8")) == [{"texte": "Acheter du pain", "fait": False}]
    capsys.readouterr()
    assert cli.main(["--fichier", chemin, "lister"]) == 0
    assert "1. [ ] Acheter du pain" in capsys.readouterr().out


def test_terminer(tmp_path, capsys):
    chemin = str(tmp_path / "t.json")
    cli.main(["--fichier", chemin, "ajouter", "Acheter du pain"])
    assert cli.main(["--fichier", chemin, "terminer", "1"]) == 0
    capsys.readouterr()
    cli.main(["--fichier", chemin, "lister"])
    assert "1. [x] Acheter du pain" in capsys.readouterr().out


def test_terminer_numero_inconnu_retourne_une_erreur(tmp_path, capsys):
    chemin = str(tmp_path / "t.json")
    assert cli.main(["--fichier", chemin, "terminer", "5"]) == 1, "un numéro inconnu doit retourner 1, sans faire planter le programme"
    assert "5" in capsys.readouterr().err
