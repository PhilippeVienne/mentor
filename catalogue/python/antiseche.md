## Lancer du Python

| Commande | Effet |
| --- | --- |
| `python3` | ouvre le mode interactif (REPL) ; `exit()` pour quitter |
| `python3 script.py` | lance un fichier |
| `python3 -c "print(1 + 1)"` | exécute une ligne |
| `python3 -m module` | lance un module ou un paquet (`__main__.py`) |
| `python3 --version` | affiche la version |

## Types et opérateurs

| Élément | Exemple | Remarque |
| --- | --- | --- |
| `str` `int` `float` `bool` | `"Mentor"` `42` `3.14` `True` | `type(x)` donne le type |
| `/` `//` `%` `**` | `7 / 2` `7 // 2` `7 % 2` `7 ** 2` | `3.5`, `3`, `1`, `49` |
| f-string | `f"Bonjour {prenom} !"` | l'expression entre `{}` est évaluée |
| Comparaisons | `==` `!=` `<` `>=` `in` | donnent un `bool` |

## Fonctions et modules

| Besoin | Écriture |
| --- | --- |
| Définir | `def carre(n): return n * n` |
| Valeur par défaut | `def saluer(nom, politesse="Bonjour"):` |
| Docstring | `"""Retourne n au carré."""` juste sous le `def` |
| Importer | `from mathutils import carre` · `import mathutils` |
| Programme principal | `if __name__ == "__main__": main()` |

## Collections et boucles

| Besoin | Écriture |
| --- | --- |
| Liste | `[1, 2, 3]` · `l.append(x)` · `l[-1]` · `l[1:3]` · `len(l)` |
| Dictionnaire | `{"a": 1}` · `d["b"] = 2` · `d.get("c", 0)` · `d.items()` |
| Boucle | `for x in l:` · `for i, x in enumerate(l, start=1):` |
| Compréhension | `[x * 2 for x in l if x > 0]` · `{v: k for k, v in d.items()}` |
| Trier | `sorted(l)` (nouvelle liste) · `l.sort()` (en place) |

## Fichiers et exceptions

| Besoin | Écriture |
| --- | --- |
| Lire | `with open(f, encoding="utf-8") as fh: for ligne in fh:` |
| Écrire / ajouter | `open(f, "w", ...)` écrase · `open(f, "a", ...)` ajoute |
| Rattraper | `try: ... except ValueError: ...` (puis `else`, `finally`) |
| Lever | `raise ValueError("message")` |
| Erreurs courantes | `FileNotFoundError` `ValueError` `KeyError` `IndexError` `TypeError` `NameError` |

## Environnements virtuels et projet

| Commande | Effet |
| --- | --- |
| `python3 -m venv .venv` | crée l'environnement virtuel |
| `source .venv/bin/activate` · `deactivate` | l'active · le quitte |
| `pip install nom` · `pip list` | installe (réseau nécessaire) · liste |
| `pip freeze > requirements.txt` | fige les versions |
| `pip install -r requirements.txt` | recrée l'environnement |
| `.gitignore` | `.venv/` et `__pycache__/` au minimum |

## Tests avec pytest

| Commande | Effet |
| --- | --- |
| `pytest -q` | lance tous les tests (`.` réussi, `F` échec) |
| `pytest -q test_x.py` | un seul fichier |
| `pytest -q -k nom` | les tests dont le nom contient `nom` |
| `assert valeur == attendu` | le test échoue si c'est faux |
| `with pytest.raises(IndexError):` | vérifie qu'une exception est levée |
