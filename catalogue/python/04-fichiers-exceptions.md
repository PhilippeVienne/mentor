---
id: fichiers-exceptions
titre: "Fichiers et exceptions"
resume: "Lis et écris des fichiers, et apprends à gérer les erreurs au lieu de les subir."
duree: 30
objectifs:
  - Lire et écrire un fichier texte avec `with open(...)`
  - Lever et attraper une exception avec `try` / `except`
  - Distinguer les erreurs courantes (`FileNotFoundError`, `ValueError`, `KeyError`)
  - Écrire une fonction robuste face à des données imparfaites
---

Un programme utile lit des fichiers (une configuration, des données d'adhérent·e·s, des journaux) et en écrit (un rapport, une sauvegarde). Et dans la vraie vie, **les données sont imparfaites** : le fichier est absent, une ligne est mal formée… Savoir gérer ces cas, c'est la différence entre un script qui plante et un outil fiable.

![Ouvrir un fichier avec « with », et rattraper une exception avec try / except](images/fichiers-exceptions.svg)

## Lire un fichier

```python
with open("nombres.txt", encoding="utf-8") as fichier:
    for ligne in fichier:
        print(ligne.strip())
```

- `open(chemin, encoding="utf-8")` ouvre le fichier. **Précise toujours l'encodage** : sans lui, le résultat dépend de la machine, et les accents sont le premier endroit où ça casse.
- `with ... as fichier` garantit que le fichier est **refermé** à la sortie du bloc, même en cas d'erreur.
- Itérer sur `fichier` donne les lignes une par une, **avec** le retour à la ligne : `.strip()` l'enlève (et les espaces autour).

Pour tout lire d'un coup : `fichier.read()` (un seul texte) ou `fichier.readlines()` (une liste de lignes).

## Écrire un fichier

```python
with open("rapport.txt", "w", encoding="utf-8") as fichier:
    fichier.write("total: 49\n")
```

| Mode | Effet |
| --- | --- |
| `"r"` (défaut) | lecture ; erreur si le fichier n'existe pas |
| `"w"` | écriture ; **écrase** le contenu existant |
| `"a"` | ajout à la fin du fichier |

:::warning Le mode "w" efface
Ouvrir un fichier en `"w"` le vide immédiatement, avant même que tu aies écrit quoi que ce soit. Pour ajouter à la suite, utilise `"a"`.
:::

## Les exceptions

Quand quelque chose tourne mal, Python **lève une exception**. Si personne ne la rattrape, le programme s'arrête avec une trace. Pour la rattraper :

```python
try:
    age = int("abc")
except ValueError:
    age = 0
```

Le bloc `try` s'exécute ; si une `ValueError` survient, Python saute au bloc `except`.

| Exception | Quand ? |
| --- | --- |
| `FileNotFoundError` | le fichier à ouvrir n'existe pas |
| `ValueError` | une valeur a le bon type mais un contenu invalide (`int("abc")`) |
| `KeyError` | une clé de dictionnaire n'existe pas |
| `IndexError` | un index de liste est hors limites |
| `TypeError` | une opération sur un mauvais type (`"a" + 1`) |

On peut **lever** soi-même une exception : `raise ValueError("âge négatif")`.

:::tip Attrape précisément
N'écris pas `except:` tout court (ou `except Exception:`) : tu cacherais aussi les vraies erreurs de programmation. Nomme l'exception que tu sais gérer, et laisse les autres remonter.
:::

## try, except, else, finally

```python
try:
    fichier = open("config.txt", encoding="utf-8")
except FileNotFoundError:
    print("pas de configuration, valeurs par défaut")
else:
    print("configuration chargée")   # seulement si aucune exception
```

`else` s'exécute quand il n'y a **pas** eu d'erreur, `finally` s'exécute **toujours** (pratique pour nettoyer). Avec `with`, tu as rarement besoin de `finally` pour fermer un fichier.

## Dans ton terminal

```shell run
printf "12\n7\n\nabc\n30\n" > nombres_essai.txt
python3 -c "print(open('nombres_essai.txt', encoding='utf-8').read().split())"
```

## Entraîne-toi

:::labo
moteur: reel
intro: |
  `fichiers_exo.py` contient quatre fonctions à écrire. Le fichier `nombres.txt` (une ligne vide, et une ligne `abc` qui n'est pas un nombre) te sert de cobaye. Les tests sont dans `test_fichiers_exo.py`.
commandes:
  - cp -R /opt/exercices/04-fichiers-exceptions/. .
etapes:
  - texte: 'Écris `lire_lignes(chemin)` : la liste des lignes **non vides**, sans retour à la ligne'
    indice: 'Utilise `with open(chemin, encoding="utf-8") as fichier:` puis une compréhension : `[ligne.strip() for ligne in fichier if ligne.strip()]`.'
    verif:
      - commande-reussit: 'pytest -q test_fichiers_exo.py -k lire_lignes'
    solution:
      - |
        cat > fichiers_exo.py <<'EOF'
        """Fichiers et exceptions."""


        def lire_lignes(chemin):
            """Retourne la liste des lignes non vides du fichier, sans le retour à la ligne."""
            with open(chemin, encoding="utf-8") as fichier:
                return [ligne.strip() for ligne in fichier if ligne.strip()]


        def somme_nombres(chemin):
            """Retourne la somme des lignes qui sont des entiers ; ignore les lignes invalides."""
            raise NotImplementedError("À toi de jouer : int(ligne) dans un try / except ValueError")


        def ecrire_rapport(chemin, valeurs):
            """Écrit une ligne « valeur: X » par valeur, puis « total: S » à la fin."""
            raise NotImplementedError("À toi de jouer : with open(chemin, 'w', encoding='utf-8') as f: ...")


        def lire_config(chemin):
            """Lit un fichier « cle=valeur » (une paire par ligne) ; retourne {} si le fichier n'existe pas."""
            raise NotImplementedError("À toi de jouer : attrape FileNotFoundError")
        EOF
  - texte: 'Écris `somme_nombres(chemin)` : la somme des lignes qui sont des entiers, en **ignorant** les lignes invalides'
    indice: 'Réutilise `lire_lignes`. Pour chaque ligne, `int(ligne)` dans un `try`, et un `except ValueError` qui passe à la suite.'
    apres: [1]
    verif:
      - commande-reussit: 'pytest -q test_fichiers_exo.py -k somme_nombres'
    solution:
      - |
        cat > fichiers_exo.py <<'EOF'
        """Fichiers et exceptions."""


        def lire_lignes(chemin):
            """Retourne la liste des lignes non vides du fichier, sans le retour à la ligne."""
            with open(chemin, encoding="utf-8") as fichier:
                return [ligne.strip() for ligne in fichier if ligne.strip()]


        def somme_nombres(chemin):
            """Retourne la somme des lignes qui sont des entiers ; ignore les lignes invalides."""
            total = 0
            for ligne in lire_lignes(chemin):
                try:
                    total += int(ligne)
                except ValueError:
                    pass  # « abc » n'est pas un nombre : on passe à la ligne suivante
            return total


        def ecrire_rapport(chemin, valeurs):
            """Écrit une ligne « valeur: X » par valeur, puis « total: S » à la fin."""
            raise NotImplementedError("À toi de jouer : with open(chemin, 'w', encoding='utf-8') as f: ...")


        def lire_config(chemin):
            """Lit un fichier « cle=valeur » (une paire par ligne) ; retourne {} si le fichier n'existe pas."""
            raise NotImplementedError("À toi de jouer : attrape FileNotFoundError")
        EOF
  - texte: 'Écris `ecrire_rapport(chemin, valeurs)` : une ligne `valeur: X` par valeur, puis `total: S`'
    indice: 'Ouvre le fichier en mode `"w"`. Écris avec `fichier.write(f"valeur: {valeur}\n")`, et `sum(valeurs)` pour le total.'
    apres: [1]
    verif:
      - commande-reussit: 'pytest -q test_fichiers_exo.py -k ecrire_rapport'
    solution:
      - |
        cat > fichiers_exo.py <<'EOF'
        """Fichiers et exceptions."""


        def lire_lignes(chemin):
            """Retourne la liste des lignes non vides du fichier, sans le retour à la ligne."""
            with open(chemin, encoding="utf-8") as fichier:
                return [ligne.strip() for ligne in fichier if ligne.strip()]


        def somme_nombres(chemin):
            """Retourne la somme des lignes qui sont des entiers ; ignore les lignes invalides."""
            total = 0
            for ligne in lire_lignes(chemin):
                try:
                    total += int(ligne)
                except ValueError:
                    pass  # « abc » n'est pas un nombre : on passe à la ligne suivante
            return total


        def ecrire_rapport(chemin, valeurs):
            """Écrit une ligne « valeur: X » par valeur, puis « total: S » à la fin."""
            with open(chemin, "w", encoding="utf-8") as fichier:
                for valeur in valeurs:
                    fichier.write(f"valeur: {valeur}\n")
                fichier.write(f"total: {sum(valeurs)}\n")


        def lire_config(chemin):
            """Lit un fichier « cle=valeur » (une paire par ligne) ; retourne {} si le fichier n'existe pas."""
            raise NotImplementedError("À toi de jouer : attrape FileNotFoundError")
        EOF
  - texte: 'Écris `lire_config(chemin)` : un dictionnaire `{cle: valeur}`, ou `{}` si le fichier **n''existe pas**'
    indice: 'Entoure l''appel à `lire_lignes` d''un `try` / `except FileNotFoundError`. Pour découper `cle=valeur`, `ligne.split("=", 1)`.'
    apres: [1]
    verif:
      - commande-reussit: 'pytest -q test_fichiers_exo.py -k lire_config'
    solution:
      - |
        cat > fichiers_exo.py <<'EOF'
        """Fichiers et exceptions."""


        def lire_lignes(chemin):
            """Retourne la liste des lignes non vides du fichier, sans le retour à la ligne."""
            with open(chemin, encoding="utf-8") as fichier:
                return [ligne.strip() for ligne in fichier if ligne.strip()]


        def somme_nombres(chemin):
            """Retourne la somme des lignes qui sont des entiers ; ignore les lignes invalides."""
            total = 0
            for ligne in lire_lignes(chemin):
                try:
                    total += int(ligne)
                except ValueError:
                    pass  # « abc » n'est pas un nombre : on passe à la ligne suivante
            return total


        def ecrire_rapport(chemin, valeurs):
            """Écrit une ligne « valeur: X » par valeur, puis « total: S » à la fin."""
            with open(chemin, "w", encoding="utf-8") as fichier:
                for valeur in valeurs:
                    fichier.write(f"valeur: {valeur}\n")
                fichier.write(f"total: {sum(valeurs)}\n")


        def lire_config(chemin):
            """Lit un fichier « cle=valeur » ; retourne {} si le fichier n'existe pas."""
            try:
                lignes = lire_lignes(chemin)
            except FileNotFoundError:
                return {}
            return dict(ligne.split("=", 1) for ligne in lignes)
        EOF
  - texte: 'Fais un rapport réel : exécute `somme_nombres("nombres.txt")` et écris le résultat avec `ecrire_rapport` dans `rapport.txt`'
    indice: 'Une ligne `python3 -c "..."` suffit : importe les deux fonctions depuis `fichiers_exo`, puis `ecrire_rapport("rapport.txt", [somme_nombres("nombres.txt")])`.'
    apres: [2, 3]
    verif:
      - fichier-contient-dans-env: [rapport.txt, '^total: 49$']
    solution:
      - python3 -c 'from fichiers_exo import ecrire_rapport, somme_nombres; ecrire_rapport("rapport.txt", [somme_nombres("nombres.txt")])'
:::

## Vérifie tes acquis

:::quiz
Pourquoi écrire `with open(...) as fichier:` plutôt que `fichier = open(...)` ?

- [ ] Pour que le fichier soit lu plus vite
- [x] Pour que le fichier soit refermé automatiquement, même si une erreur survient
- [ ] Parce que `open` ne marche pas sans `with`
- [ ] Pour ouvrir le fichier en écriture

> Le gestionnaire de contexte `with` ferme le fichier à la sortie du bloc dans tous les cas, ce qui évite de laisser des fichiers ouverts.
:::

:::quiz
Que se passe-t-il si tu ouvres un fichier existant avec `open("notes.txt", "w")` ?

- [ ] Les nouvelles lignes sont ajoutées à la fin
- [ ] Python lève une erreur parce que le fichier existe déjà
- [x] Le contenu du fichier est effacé dès l'ouverture
- [ ] Le fichier est ouvert en lecture seule

> `"w"` tronque le fichier. Pour ajouter à la suite, il faut le mode `"a"`.
:::

:::quiz
Quelle exception attraper pour traiter le cas d'un fichier de configuration absent ?

- [ ] `ValueError`
- [ ] `KeyError`
- [x] `FileNotFoundError`
- [ ] `IndexError`

> `open()` en lecture lève `FileNotFoundError` quand le fichier n'existe pas. Attraper `Exception` serait trop large.
:::

:::quiz
Pourquoi vaut-il mieux écrire `except ValueError:` que `except:` tout court ?

- [ ] Parce que `except:` est une erreur de syntaxe
- [x] Pour ne rattraper que l'erreur prévue et ne pas masquer les vrais bogues
- [ ] Parce que `except:` ralentit le programme
- [ ] Parce que Python exige un nom d'exception

> Un `except` trop large avale aussi les fautes de programmation (faute de frappe sur un nom, par exemple), et rend les bogues invisibles.
:::
