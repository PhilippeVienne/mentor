---
title: "Examen de validation — Python"
draw: 12
pass_mark: 80
minutes: 20
shuffle: true
---

Cet examen s'adresse aux personnes qui **savent déjà programmer en Python** et veulent valider tout le parcours sans refaire les labos : variables et types, fonctions et modules, listes, dictionnaires et boucles, fichiers et exceptions, environnements virtuels, et la construction d'un petit projet testé.

**Les règles :**

- À chaque tentative, le serveur tire **12 questions au hasard** dans un grand pool, et mélange les réponses.
- Tu as **20 minutes** et il faut au moins **80 %** de bonnes réponses (10 sur 12) pour réussir.
- Une réponse laissée vide compte comme fausse ; avant d'envoyer, tu peux revoir et modifier tes réponses.
- Après un échec, un court délai est imposé avant de réessayer ; les questions changent d'une tentative à l'autre.
- La correction détaillée, avec les explications, s'affiche une fois l'examen envoyé.

Réussir l'examen **valide le parcours** (et te donne l'XP et les badges correspondants) ; les leçons restent disponibles si tu veux t'entraîner.

:::quiz
Quel est le type du résultat de `3 / 2` en Python 3 ?

- [ ] `int`, car 3 et 2 sont des entiers
- [x] `float`
- [ ] `str`, car le résultat s'affiche comme du texte
- [ ] Aucun : la division d'entiers lève une erreur

> `/` retourne toujours un `float`, même quand le résultat tombe juste (`4 / 2` donne `2.0`). Pour un quotient entier, on utilise `//`.
:::

:::quiz
Que vaut `"7" + "3"` ?

- [ ] `10`, car Python additionne les chiffres
- [ ] `"10"`, car le résultat est converti en texte
- [ ] Une `TypeError` : on ne peut pas additionner des textes
- [x] `"73"`

> Avec deux `str`, l'opérateur `+` **concatène**. Ce n'est pas une addition numérique.
:::

:::quiz
Que va afficher `print(f"{2 + 3} points")` ?

- [ ] `2 + 3 points`, car le texte est affiché tel quel
- [ ] `{5} points`, car les accolades sont conservées
- [ ] Une `SyntaxError`, car on ne peut pas calculer dans une chaîne
- [x] `5 points`

> Dans une f-string, tout ce qui est entre `{}` est évalué comme une expression Python, puis remplacé par son résultat.
:::

:::quiz
Lequel de ces noms **n'est pas** un nom de variable valide ?

- [x] `2eme_place`
- [ ] `_brouillon`, car il commence par un tiret bas
- [ ] `nom_complet`, car il contient un tiret bas
- [ ] `age2`, car il se termine par un chiffre

> Un nom de variable commence par une lettre ou `_`, puis peut contenir des chiffres. `2eme_place` provoque une `SyntaxError`.
:::

:::quiz
Que vaut `17 % 5` ?

- [ ] `3`, le quotient entier de 17 par 5
- [x] `2`, le reste de la division de 17 par 5
- [ ] `3.4`, le résultat de la division
- [ ] `85`, le produit de 17 par 5

> `%` est le **modulo** : le reste de la division entière (17 = 3 × 5 + 2). Le quotient serait `17 // 5`.
:::

:::quiz
Quelle commande lance le fichier `script.py` depuis le terminal ?

- [ ] `python3 --run script.py`
- [x] `python3 script.py`
- [ ] `import script.py`
- [ ] `run python3 script.py`

> On passe le nom du fichier à l'interpréteur : `python3 script.py`. Il n'existe pas d'option `--run`, et `import` s'écrit dans un programme Python, pas dans le terminal.
:::

:::quiz
Que signifie `NameError: name 'prenom' is not defined` ?

- [x] Une variable `prenom` est utilisée sans avoir été créée
- [ ] Le texte `prenom` est trop long pour une variable
- [ ] Le fichier `prenom.py` est introuvable
- [ ] La variable `prenom` existe mais contient un nombre

> `NameError` : Python ne connaît pas ce nom. Cause fréquente : une faute de frappe, ou une variable utilisée avant d'être définie.
:::

:::quiz
Parmi ces valeurs, laquelle est de type `float` ?

- [ ] `4`, car c'est un nombre
- [ ] `"4.0"`, car elle contient un point
- [ ] `True`, car elle vaut 1
- [x] `4.0`

> Le type dépend de l'écriture : `4` est un `int`, `4.0` un `float`, `"4.0"` une `str` (entre guillemets), `True` un `bool`.
:::

:::quiz
Quelle fonction retourne `None` quand on l'appelle ?

- [ ] Une fonction dont la dernière ligne est `return 0`
- [x] Une fonction qui affiche avec `print` sans `return`
- [ ] Une fonction qui contient `return n * n`
- [ ] Une fonction qui retourne une chaîne vide

> Sans instruction `return`, Python renvoie `None`. Afficher n'est pas retourner.
:::

:::quiz
Avec `def saluer(nom, politesse="Bonjour"): return f"{politesse} {nom}"`, que retourne `saluer("Ada")` ?

- [ ] `"Ada"`, car la politesse est optionnelle donc ignorée
- [x] `"Bonjour Ada"`
- [ ] Une `TypeError`, car il manque un argument
- [ ] `"None Ada"`, car `politesse` n'a pas reçu de valeur

> Un paramètre qui a une valeur par défaut devient facultatif ; quand on ne le donne pas, la valeur par défaut est utilisée.
:::

:::quiz
Quelle écriture permet d'utiliser `carre` ensuite **sans** préfixe, depuis le module `mathutils` ?

- [ ] `import mathutils`
- [ ] `import carre from mathutils`
- [x] `from mathutils import carre`
- [ ] `include mathutils.carre`

> Avec `import mathutils`, il faut écrire `mathutils.carre(3)`. `from mathutils import carre` rend `carre` directement disponible.
:::

:::quiz
Que vaut `__name__` dans `mathutils.py` quand un autre fichier fait `import mathutils` ?

- [ ] `"__main__"`, comme pour tout fichier
- [x] `"mathutils"`, le nom du module
- [ ] `None`, car le fichier n'est pas lancé
- [ ] `"import"`, car il est importé

> `__name__` vaut `"__main__"` seulement dans le fichier lancé directement. Dans un module importé, c'est son nom.
:::

:::quiz
À quoi sert la docstring d'une fonction ?

- [ ] À donner un type de retour obligatoire à la fonction
- [ ] À empêcher d'appeler la fonction avec de mauvais arguments
- [x] À décrire ce que fait la fonction
- [ ] À exécuter du code automatiquement avant chaque appel

> La docstring est un simple texte documentaire, entre triples guillemets, placé juste après la ligne `def`. Python ne la contrôle pas.
:::

:::quiz
Pourquoi une fonction qui `retourne` son résultat est-elle plus facile à tester qu'une fonction qui l'affiche ?

- [ ] Parce que `pytest` ne sait pas lire ce que `print` affiche
- [ ] Parce que `return` rend la fonction plus rapide à exécuter
- [x] Un `assert` compare directement la valeur retournée à l'attendue
- [ ] Parce qu'une fonction qui affiche ne peut avoir aucun paramètre

> Une valeur retournée se compare dans un `assert`. Capturer l'affichage est possible, mais plus lourd.
:::

:::quiz
Quelle erreur Python signale un bloc mal aligné après un `def` ?

- [ ] `BlockError`
- [ ] `SpaceError`
- [x] `IndentationError`
- [ ] `FormatError`

> L'indentation fait partie de la syntaxe de Python : un corps de fonction mal décalé provoque une `IndentationError`.
:::

:::quiz
Avec `def f(a, b=2): return a * b`, quel appel est **invalide** ?

- [x] `f(b=3)`
- [ ] `f(5)`, car `b` n'est pas donné
- [ ] `f(a=5, b=3)`, car on nomme les arguments
- [ ] `f(5, b=3)`, car on mélange position et nom

> Les arguments sans valeur par défaut sont obligatoires. On peut nommer les arguments et les mélanger tant que les positionnels viennent en premier.
:::

:::quiz
Que vaut `[10, 20, 30, 40][1:3]` ?

- [ ] `[10, 20, 30]`, les trois premiers éléments
- [ ] `[20, 30, 40]`, à partir du rang 1
- [ ] `[10, 30]`, un élément sur deux
- [x] `[20, 30]` : du rang 1 inclus au rang 3 exclu

> Une tranche `[a:b]` commence au rang `a` (inclus) et s'arrête avant le rang `b`.
:::

:::quiz
Que retourne `len({"a": 1, "b": 2, "c": 3})` ?

- [x] `3`, le nombre de paires clé-valeur
- [ ] `6`, car il y a 3 clés et 3 valeurs
- [ ] `1`, car c'est un seul dictionnaire
- [ ] Une `TypeError`, car `len` ne marche que sur les listes

> `len` d'un dictionnaire donne le nombre de clés, donc de paires.
:::

:::quiz
Que se passe-t-il avec `d = {"a": 1}` puis `d["b"]` ?

- [x] Python lève une `KeyError`
- [ ] Python retourne `None` sans erreur
- [ ] Python crée la clé `"b"` avec la valeur `0`
- [ ] Python retourne `"b"`

> L'accès par crochets exige que la clé existe. Pour une valeur par défaut, utilise `d.get("b")`.
:::

:::quiz
Que produit `for i, x in enumerate(["a", "b"], start=1):` à la première itération ?

- [ ] `i` vaut `0` et `x` vaut `"a"`
- [ ] `i` vaut `"a"` et `x` vaut `1`
- [x] `i` vaut `1` et `x` vaut `"a"`
- [ ] `i` vaut `1` et `x` vaut `"b"`

> `enumerate(..., start=1)` numérote à partir de 1 ; chaque itération donne le couple (numéro, élément).
:::

:::quiz
Quelle est la différence entre `sorted(ma_liste)` et `ma_liste.sort()` ?

- [ ] `sorted` trie en place ; `.sort()` retourne une nouvelle liste
- [ ] Il n'y en a aucune : les deux font exactement la même chose
- [x] `sorted` retourne une nouvelle liste triée
- [ ] `sorted` ne fonctionne que sur les listes de nombres

> `sorted()` laisse la liste d'origine intacte et retourne le résultat ; `.sort()` modifie l'objet et retourne `None`.
:::

:::quiz
Que contient `[x * 2 for x in range(3)]` ?

- [x] `[0, 2, 4]`
- [ ] `[2, 4, 6]`
- [ ] `[0, 1, 2]`
- [ ] `[0, 2, 4, 6]`

> `range(3)` produit 0, 1, 2 ; chaque valeur est doublée.
:::

:::quiz
Que retourne `"a" in {"a": 1, "b": 2}` ?

- [ ] `False`, car `"a"` n'est pas une valeur
- [x] `True`
- [ ] `1`, la valeur associée à la clé
- [ ] Une `TypeError`, car on ne peut pas tester un dictionnaire

> Sur un dictionnaire, `in` regarde les clés. Pour tester une valeur : `1 in d.values()`.
:::

:::quiz
Après `a = [1, 2]`, `b = a` puis `b.append(3)`, que vaut `a` ?

- [ ] `[1, 2]`, car `b` est une copie indépendante
- [ ] `[3]`, car `append` remplace le contenu
- [x] `[1, 2, 3]`
- [ ] Une erreur : on ne peut pas modifier `b`

> L'affectation ne copie pas : elle ajoute un deuxième nom sur le même objet. Pour copier : `b = a.copy()` ou `list(a)`.
:::

:::quiz
Quel mode d'ouverture ajoute du texte **à la fin** d'un fichier existant sans l'effacer ?

- [ ] `"w"` (*write*)
- [ ] `"r"` (*read*)
- [ ] `"x"` (*exclusive*)
- [x] `"a"` (*append*)

> `"w"` écrase le fichier, `"r"` ne permet que la lecture, `"x"` échoue si le fichier existe. `"a"` écrit à la suite.
:::

:::quiz
À quoi sert le bloc `else` d'un `try` / `except` ?

- [ ] À exécuter du code seulement quand une exception a eu lieu
- [x] À exécuter du code quand le `try` a réussi sans exception
- [ ] À exécuter du code quoi qu'il arrive, même après une erreur
- [ ] À remplacer le bloc `except` quand on ne connaît pas l'erreur

> `except` gère l'erreur, `else` s'exécute en l'absence d'erreur, `finally` s'exécute dans tous les cas.
:::

:::quiz
Quand le bloc `finally` s'exécute-t-il ?

- [ ] Seulement s'il y a eu une exception
- [x] Toujours
- [ ] Seulement s'il n'y a pas eu d'exception
- [ ] Seulement si le `try` contient un `return`

> `finally` sert à nettoyer (fermer, libérer) dans tous les cas.
:::

:::quiz
Que fait `raise ValueError("âge négatif")` ?

- [ ] Il affiche le message puis continue le programme
- [x] Il lève volontairement une exception `ValueError`
- [ ] Il rattrape une exception `ValueError` déjà levée
- [ ] Il convertit la valeur en entier

> `raise` signale une erreur à l'appelant. Si personne ne la rattrape, le programme s'arrête avec une trace.
:::

:::quiz
Pourquoi écrire `open(chemin, encoding="utf-8")` ?

- [ ] Pour ouvrir le fichier plus rapidement
- [ ] Pour chiffrer le contenu du fichier
- [ ] Parce que `open` refuse de s'exécuter sans cet argument
- [x] Pour lire et écrire les accents de la même façon partout

> Sans encodage explicite, Python utilise celui de la machine, et le même fichier peut être lu différemment. UTF-8 est le choix sûr.
:::

:::quiz
Que fait `"  bonjour\n".strip()` ?

- [ ] Il retourne `"BONJOUR"`
- [ ] Il retourne `"  bonjour"` : seul le retour à la ligne disparaît
- [x] Il retourne `"bonjour"`
- [ ] Il retourne `["bonjour"]`, une liste de mots

> `strip()` enlève tous les espaces blancs (espaces, tabulations, retours à la ligne) au début et à la fin du texte.
:::

:::quiz
Quelle exception obtient-on avec `int("12.5")` ?

- [ ] `TypeError`, car `12.5` est un `float`
- [ ] `KeyError`, car la clé `12.5` n'existe pas
- [ ] Aucune : Python retourne `12`
- [x] `ValueError`

> `int()` accepte un texte à condition qu'il représente un entier. `"12.5"` a le bon type (`str`) mais un contenu invalide.
:::

:::quiz
Quelle exception obtient-on avec `[1, 2][5]` ?

- [x] `IndexError`
- [ ] `KeyError`, car `5` est introuvable
- [ ] `ValueError`, car `5` est trop grand
- [ ] Aucune : Python retourne `None`

> `IndexError` pour une liste, `KeyError` pour un dictionnaire.
:::

:::quiz
Que crée la commande `python3 -m venv .venv` ?

- [ ] Une machine virtuelle complète avec un système Linux
- [ ] Un fichier `requirements.txt` vide
- [ ] Une copie de tous les paquets installés sur la machine
- [x] Un dossier `.venv` avec son propre Python et `pip`

> Un venv n'est qu'un dossier : un interpréteur, `pip`, et un emplacement pour les paquets du projet.
:::

:::quiz
Que change `source .venv/bin/activate` dans le terminal ?

- [x] `python` et `pip` désignent désormais ceux du venv
- [ ] Il installe les paquets listés dans `requirements.txt`
- [ ] Il supprime le Python du système
- [ ] Il ouvre une connexion à PyPI

> L'activation modifie le `PATH` du terminal courant. `deactivate` annule l'effet.
:::

:::quiz
Que fait `pip freeze > requirements.txt` ?

- [ ] Il fige l'exécution du programme en cours
- [ ] Il installe les paquets listés dans le fichier
- [ ] Il supprime les paquets que le projet n'utilise pas
- [x] Il écrit les paquets et leurs versions dans le fichier

> `pip freeze` affiche `paquet==version` pour chaque paquet installé ; la redirection `>` l'enregistre.
:::

:::quiz
Quelle ligne de `.gitignore` empêche de commiter les fichiers compilés automatiquement par Python ?

- [x] `__pycache__/`
- [ ] `requirements.txt`
- [ ] `src/`
- [ ] `README.md`

> Python écrit des `.pyc` dans des dossiers `__pycache__` : ce sont des fichiers générés, qu'on ne versionne pas.
:::

:::quiz
Dans l'environnement d'entraînement du portail (sans accès réseau), pourquoi `pip install requests` échoue-t-il ?

- [x] Parce que le conteneur n'a aucun accès à Internet, donc pas à PyPI
- [ ] Parce que `requests` n'existe pas pour Python 3.13
- [ ] Parce que `pip` est interdit aux utilisateurs non administrateurs
- [ ] Parce que le venv n'est pas activé

> Pour la sécurité, les environnements réels sont isolés du réseau ; seuls les paquets installés à la construction de l'image sont disponibles.
:::

:::quiz
Pourquoi écrire `pytest==8.3.5` plutôt que `pytest` dans `requirements.txt` ?

- [ ] Parce que `pip` refuse les paquets sans version
- [ ] Pour que `pytest` s'installe plus vite
- [x] Pour que tout le monde installe la même version
- [ ] Parce que la version `8.3.5` est la seule qui existe

> Épingler les versions rend l'environnement **reproductible** : le projet ne casse pas parce qu'une nouvelle version est sortie.
:::

:::quiz
Que fait `deactivate` ?

- [ ] Il supprime le dossier `.venv`
- [ ] Il désinstalle tous les paquets du projet
- [ ] Il arrête le programme Python en cours
- [x] Il quitte le venv

> `deactivate` ne supprime rien : il rétablit seulement le `PATH` d'origine.
:::

:::quiz
Où ranger un mot de passe ou une clé d'API d'un projet ?

- [ ] Dans `requirements.txt`, avec les autres réglages
- [ ] Directement dans le code, pour que l'équipe le retrouve
- [ ] Dans le `README.md`, pour le documenter
- [x] Dans un fichier `.env` ignoré par Git, jamais dans le dépôt

> Un secret commité reste dans l'historique Git, même supprimé ensuite : il faut alors le révoquer. Les projets de l'équipe utilisent des variables d'environnement.
:::

:::quiz
À quoi sert `add_subparsers` dans `argparse` ?

- [ ] À lancer plusieurs programmes en parallèle
- [ ] À vérifier que les arguments sont des nombres
- [x] À déclarer des sous-commandes qui ont chacune leurs arguments
- [ ] À écrire l'aide dans un fichier

> Comme `git commit` ou `docker run`, une interface avec plusieurs actions utilise des sous-commandes.
:::

:::quiz
Dans le cycle TDD, que fait-on **en premier** ?

- [ ] On écrit tout le code puis on cherche les erreurs
- [ ] On relit le code pour l'améliorer
- [x] On écrit un test qui échoue pour le comportement voulu
- [ ] On supprime les anciens tests

> Rouge, vert, amélioration : le test échouant précise ce que le code doit faire avant de l'écrire.
:::

:::quiz
Quelle est la différence entre `json.dump` et `json.dumps` ?

- [ ] `dump` retourne une chaîne ; `dumps` écrit dans un fichier
- [ ] `dumps` accepte plusieurs objets d'un coup
- [x] `dump` écrit dans un fichier ouvert
- [ ] Il n'y en a aucune : l'une est un alias de l'autre

> Le `s` veut dire *string* : `dumps` produit une chaîne, `dump` écrit dans un objet fichier.
:::

:::quiz
Que fait `pytest -q -k ajouter` ?

- [ ] Il ajoute un test nommé `ajouter`
- [x] Il ne lance que les tests dont le nom contient `ajouter`
- [ ] Il lance tous les tests sauf ceux qui contiennent `ajouter`
- [ ] Il affiche la liste des tests sans les lancer

> `-k` filtre les tests par expression sur leur nom ; `-q` réduit la verbosité.
:::

:::quiz
Que signifie un programme qui se termine avec le code de sortie `0` ?

- [ ] Qu'une erreur grave est survenue
- [x] Que tout s'est bien passé
- [ ] Que le programme n'a rien affiché
- [ ] Qu'il a été interrompu par l'utilisateur

> Par convention, `0` = succès. Les scripts et les outils comme la CI se servent de ce code pour savoir si une commande a réussi.
:::

:::quiz
Pourquoi afficher les messages d'erreur sur `sys.stderr` plutôt que sur la sortie normale ?

- [x] Pour les séparer du résultat
- [ ] Parce que `print` ne sait pas afficher d'accents
- [ ] Parce que `stderr` s'affiche en rouge
- [ ] Parce que `stdout` est réservé à Python

> La sortie normale (`stdout`) et celle des erreurs (`stderr`) sont deux flux distincts : `programme > resultat.txt` n'enregistre pas les erreurs.
:::

:::quiz
Que permet `python3 -m todo` quand le dossier `todo/` contient un fichier `__main__.py` ?

- [x] Lancer le paquet `todo` comme un programme
- [ ] Créer un nouveau module nommé `todo`
- [ ] Importer `todo` sans exécuter son code
- [ ] Tester le paquet `todo` avec `pytest`

> `-m` exécute un module ou un paquet ; pour un paquet, c'est son `__main__.py` qui s'exécute.
:::

:::quiz
Que fait un fichier `conftest.py` à la racine d'un projet, même vide ?

- [ ] Il désactive automatiquement les tests lents
- [ ] Il installe les dépendances du projet
- [ ] Il convertit les tests en scripts exécutables
- [x] Il indique à `pytest` la racine importable du projet

> `pytest` ajoute le dossier d'un `conftest.py` au chemin d'import, ce qui permet `import todo` depuis `tests/`. Il sert aussi à partager des *fixtures*.
:::
