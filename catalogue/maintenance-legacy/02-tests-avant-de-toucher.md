---
id: tests-avant-de-toucher
title: "Écrire des tests avant de toucher au code"
summary: "Poser un filet de sécurité de tests de caractérisation pour savoir si une montée de version a cassé quelque chose."
minutes: 35
objectives:
  - Expliquer à quoi sert un test avant une montée de version
  - Écrire un test de caractérisation avec le `TestCase` de Django
  - Repérer ce qui est testé (ou non) dans les projets de l'équipe et décider par où commencer
---

Tu changes la version de Django, tu lances le site, la page d'accueil s'affiche. Tout va bien ? Peut-être, mais tu viens de vérifier une page sur des dizaines. Sans filet de sécurité, une montée de version est un saut sans parachute : tu ne sauras qu'en production ce qui a cassé.

## À quoi ça sert, et pourquoi avant ?

Un **test automatique** est un petit programme qui exécute une partie de ton code et vérifie que le résultat est celui attendu. Imagine un contrôle technique : une liste de points vérifiés toujours de la même façon, en quelques secondes. Si tu modifies ensuite une dépendance et que la liste passe encore, tu as de bonnes raisons de croire que rien d'important n'a bougé.

Il faut les écrire **avant** la montée de version. Après, tu ne saurais pas distinguer « ce test échoue parce que la nouvelle version a changé le comportement » de « ce test est mal écrit ».

## Où en sont les projets de l'équipe ?

Lu dans les dépôts, au moment de la rédaction :

| Projet | Ce qui existe |
| --- | --- |
| PlanningAPI | Un fichier `PlanningAPI/tests.py` qui ne contient que le commentaire généré par Django (`# Create your tests here.`). La CI exécute pourtant `coverage run --source="." manage.py test`. |
| API d'Adhésion | Des fichiers `tests.py` dans plusieurs applications. Ceux de `customer` annoncent en en-tête qu'ils **nécessitent un serveur Keycloak** (**Keycloak** est le logiciel qui gère les comptes et les connexions des utilisateurs : une application lui demande « cette personne est-elle bien connectée ? »). |
| Connecteur Paiement | `test_selenium.py`, un test qui **lance un vrai navigateur Firefox** et nécessite un serveur Django joignable par la banque. La CI exécute aussi `manage.py test paiement`. |

Conséquence : on ne peut pas se fier à « les tests sont verts » sans savoir ce qu'ils couvrent. Un fichier vide donne une CI verte, mais ne protège rien.

:::info Termes à connaître
- Un **test unitaire** vérifie une petite pièce de code seule (une méthode).
- Un **test d'intégration** vérifie que plusieurs pièces fonctionnent ensemble (une URL, la base de données, un service externe comme Keycloak). Une **URL** est l'adresse d'une page ou d'une API, par exemple `/adherents/`.
- Un test **Selenium** pilote un vrai navigateur comme le ferait une personne : il est utile mais lent et fragile, donc à réserver aux parcours critiques (le paiement, par exemple).
- La **couverture** (*coverage*) mesure le pourcentage de lignes exécutées par les tests. L'équipe l'utilise : `coverage report` est dans la CI de PlanningAPI.
:::

## Le test de caractérisation

Sur du code que tu n'as pas écrit, tu ne sais pas toujours ce qu'il « devrait » faire. Le **test de caractérisation** résout ce problème : tu ne juges pas le comportement actuel, tu le **fige**. Tu appelles le code, tu notes ce qu'il renvoie, et tu écris ce résultat dans le test. Si la montée de version change ce résultat, le test te le dit.

Prenons une application d'exemple, `adherents`, avec un modèle (une classe qui décrit une table de la base de données) :

```python
from django.db import models


class Adherent(models.Model):
    nom = models.CharField(max_length=100)
    email = models.EmailField(blank=True)

    def nom_complet(self):
        return self.nom.strip().title()
```

Et son test, dans `adherents/tests.py` :

```python
from django.test import TestCase

from .models import Adherent


class NomCompletTests(TestCase):
    """Test de caractérisation : il fige le comportement actuel, même discutable."""

    def test_nom_complet_met_une_majuscule_a_chaque_mot(self):
        adherent = Adherent.objects.create(nom="  marie curie ")
        self.assertEqual(adherent.nom_complet(), "Marie Curie")
```

Ligne à ligne :

- `TestCase` est la classe de base fournie par Django. Chaque test s'exécute dans une base de données temporaire, vidée ensuite : tu ne touches jamais aux vraies données.
- Les méthodes dont le nom commence par `test_` sont lancées automatiquement.
- `Adherent.objects.create(...)` enregistre une ligne dans cette base temporaire.
- `assertEqual(a, b)` échoue si `a` et `b` diffèrent : ici, le nom a été nettoyé (`strip`) puis mis en majuscules initiales (`title`).

Pour le lancer depuis la racine du projet :

```bash
python manage.py test
```

```console
Found 1 test(s).
Creating test database for alias 'default'...
System check identified no issues (0 silenced).
.
----------------------------------------------------------------------
Ran 1 test in 0.000s

OK
Destroying test database for alias 'default'...
```

Chaque point `.` est un test réussi ; une erreur afficherait `F` (échec) ou `E` (erreur). Cette sortie vient d'une exécution réelle sur un projet d'essai avec Django 5.2. Dans le labo, tu travailles sur un projet semblable, resté en Django 3.1 comme PlanningAPI.

## Tester une URL et remplacer un service externe

Pour tester une page ou une API, le `TestCase` de Django fournit `self.client`, un faux navigateur : `self.client.get("/adherents/")` envoie une requête à l'URL (sans serveur réel) et renvoie la réponse, avec son code HTTP (`200` signifie « succès ») et son contenu. L'application d'exemple expose `/adherents/prive/`, une page qui demande à Keycloak si le jeton de la personne est valide. Sans serveur Keycloak, l'appel échoue. On le remplace par un **faux** (*mock*), une fonction de substitution qui renvoie une réponse prévue à l'avance :

```python
from unittest.mock import patch

from django.test import TestCase

from .models import Adherent


class PriveTests(TestCase):
    @patch("adherents.views.verifier_jeton", return_value=True)
    def test_prive_compte_les_adherents(self, faux_jeton):
        Adherent.objects.create(nom="ada")
        reponse = self.client.get("/adherents/prive/", HTTP_AUTHORIZATION="Bearer faux")
        self.assertEqual(reponse.json(), {"nombre": 1})
        faux_jeton.assert_called_once_with("Bearer faux")
```

Ligne à ligne :

- `@patch("adherents.views.verifier_jeton", return_value=True)` remplace, le temps du test, la fonction `verifier_jeton` **là où la vue l'utilise** (`adherents.views`) par un faux qui répond toujours `True`. Le faux est passé au test en argument (`faux_jeton`).
- `self.client.get(...)` appelle la page, en envoyant l'en-tête `Authorization`.
- `reponse.json()` décode la réponse JSON (un format de texte qui décrit des données, comme `{"nombre": 1}`).
- `assert_called_once_with(...)` vérifie que la vue a bien interrogé Keycloak, une seule fois, avec ce jeton.

## Mesurer la couverture

Combien de lignes tes tests exécutent-ils vraiment ? L'outil `coverage` se place devant la commande de test, puis résume le résultat :

```bash
python -m coverage run manage.py test
python -m coverage report
```

```console
Name                    Stmts   Miss  Cover
-------------------------------------------
adherents/__init__.py       0      0   100%
adherents/keycloak.py       6      3    50%
adherents/models.py         7      0   100%
adherents/urls.py           3      0   100%
adherents/utils.py          7      1    86%
adherents/views.py         12      1    92%
-------------------------------------------
TOTAL                      35      5    86%
```

`Stmts` compte les instructions du fichier, `Miss` celles que les tests n'ont jamais exécutées. Ici, `keycloak.py` est à 50 % : c'est normal, le faux a remplacé son code. Un chiffre élevé ne prouve pas que les tests sont bons, mais un chiffre bas montre où il n'y a aucun filet.

## Par où commencer ?

Tu n'as pas besoin de tout couvrir. Choisis dans cet ordre :

1. **Ce qui rapporte ou coûte de l'argent** : l'adhésion, le paiement.
2. **Ce qui touche les données personnelles** : authentification, export.
3. **Ce qui change souvent ou a déjà cassé** : l'historique Git (la liste datée de toutes les modifications du dépôt) te le dit.

Ensuite, ajoute un test pour chaque URL importante : une requête, un code de réponse, une donnée clé. Quand le test dépend d'un service externe (Keycloak, la banque), remplace ce service par un **faux** (*mock*) pour que le test marche sans lui.

:::warning Un test qui dépend d'un serveur externe n'est pas un filet
Si les tests d'Adhésion exigent un serveur Keycloak, ils échoueront en CI même si ton code est bon. Isole ce qui peut l'être derrière un faux, et garde les tests qui nécessitent Keycloak pour une étape à part, bien identifiée.
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  Le projet `adherents` (en Django 3.1, comme PlanningAPI) est dans ton dossier de travail. Son fichier `adherents/tests.py` ne contient que le commentaire généré par Django : la CI serait verte, mais rien n'est protégé. Tu poses le filet de sécurité avec le Python de l'environnement `django31` : `/opt/venvs/django31/bin/python manage.py test`. Les commandes fonctionnent hors ligne, et il n'existe aucun serveur Keycloak.
commands:
  - cp -R /opt/exercices/projet-legacy/. .
steps:
  - text: 'Mesure la couverture actuelle du projet et écris le rapport dans `couverture-avant.txt`'
    hint: 'Lance d''abord `/opt/venvs/django31/bin/python -m coverage run manage.py test`, puis `/opt/venvs/django31/bin/python -m coverage report > couverture-avant.txt`.'
    checks:
      - command-succeeds: '/opt/outils/verifier-legacy couverture couverture-avant.txt'
    solution:
      - /opt/venvs/django31/bin/python -m coverage run manage.py test
      - /opt/venvs/django31/bin/python -m coverage report > couverture-avant.txt

  - text: 'Dans `adherents/tests.py`, écris le test de caractérisation de `nom_complet` (celui de la leçon) et fais-le passer'
    hint: 'Reprends la classe `NomCompletTests` de la leçon, avec ses deux `import`. Lance ensuite `/opt/venvs/django31/bin/python manage.py test` : tu dois lire `Ran 1 test` puis `OK`.'
    checks:
      - command-succeeds: '/opt/outils/verifier-legacy tests 2 1'
    solution:
      - write:
          adherents/tests.py: |
            from django.test import TestCase

            from .models import Adherent


            class NomCompletTests(TestCase):
                def test_nom_complet_met_une_majuscule_a_chaque_mot(self):
                    adherent = Adherent.objects.create(nom="  marie curie ")
                    self.assertEqual(adherent.nom_complet(), "Marie Curie")

  - text: 'Ajoute un test de l''URL `/adherents/` : crée un adhérent, appelle la page avec `self.client.get` et vérifie le code `200` et le nom renvoyé'
    hint: 'Une seconde classe de test, avec `reponse = self.client.get("/adherents/")`, puis `self.assertEqual(reponse.status_code, 200)` et `self.assertEqual(reponse.json(), {"adherents": ["Marie Curie"]})`.'
    after: [2]
    checks:
      - command-succeeds: '/opt/outils/verifier-legacy tests 3 2'
    solution:
      - write:
          adherents/tests.py: |
            from django.test import TestCase

            from .models import Adherent


            class NomCompletTests(TestCase):
                def test_nom_complet_met_une_majuscule_a_chaque_mot(self):
                    adherent = Adherent.objects.create(nom="  marie curie ")
                    self.assertEqual(adherent.nom_complet(), "Marie Curie")


            class ListeTests(TestCase):
                def test_la_liste_renvoie_les_noms_complets(self):
                    Adherent.objects.create(nom="  marie curie ")
                    reponse = self.client.get("/adherents/")
                    self.assertEqual(reponse.status_code, 200)
                    self.assertEqual(reponse.json(), {"adherents": ["Marie Curie"]})

  - text: 'Teste `/adherents/prive/` sans serveur Keycloak : remplace `verifier_jeton` par un faux avec `patch`'
    hint: 'Importe `from unittest.mock import patch` et décore le test avec `@patch("adherents.views.verifier_jeton", return_value=True)`. Le test reçoit le faux en argument. Sans faux, l''appel réseau échoue.'
    after: [3]
    checks:
      - command-succeeds: '/opt/outils/verifier-legacy tests 4 3'
    solution:
      - write:
          adherents/tests.py: |
            from unittest.mock import patch

            from django.test import TestCase

            from .models import Adherent


            class NomCompletTests(TestCase):
                def test_nom_complet_met_une_majuscule_a_chaque_mot(self):
                    adherent = Adherent.objects.create(nom="  marie curie ")
                    self.assertEqual(adherent.nom_complet(), "Marie Curie")


            class ListeTests(TestCase):
                def test_la_liste_renvoie_les_noms_complets(self):
                    Adherent.objects.create(nom="  marie curie ")
                    reponse = self.client.get("/adherents/")
                    self.assertEqual(reponse.status_code, 200)
                    self.assertEqual(reponse.json(), {"adherents": ["Marie Curie"]})


            class PriveTests(TestCase):
                @patch("adherents.views.verifier_jeton", return_value=True)
                def test_prive_compte_les_adherents(self, faux_jeton):
                    Adherent.objects.create(nom="ada")
                    reponse = self.client.get("/adherents/prive/", HTTP_AUTHORIZATION="Bearer faux")
                    self.assertEqual(reponse.json(), {"nombre": 1})
                    faux_jeton.assert_called_once_with("Bearer faux")

  - text: 'Mesure la couverture avec ces tests, écris le rapport dans `couverture-apres.txt` et vérifie qu''elle atteint au moins 80 %'
    hint: 'Relance `/opt/venvs/django31/bin/python -m coverage run manage.py test`, puis `/opt/venvs/django31/bin/python -m coverage report > couverture-apres.txt`. L''option `--fail-under=80` de `coverage report` échoue sous 80 %.'
    after: [4]
    checks:
      - command-succeeds: '/opt/outils/verifier-legacy couverture couverture-apres.txt --avec-tests --minimum 80'
    solution:
      - /opt/venvs/django31/bin/python -m coverage run manage.py test
      - /opt/venvs/django31/bin/python -m coverage report > couverture-apres.txt
:::

## Vérifie tes acquis

:::quiz
Pourquoi écrire les tests avant de monter de version et non après ?

- [ ] Parce que Django l'exige
- [ ] Parce que les tests accélèrent l'installation des dépendances
- [x] Pour distinguer un changement de comportement dû à la montée d'un test mal écrit
- [ ] Parce qu'une CI sans test ne démarre pas

> Écrits avant, les tests décrivent le comportement de référence ; s'ils échouent après, la cause est la montée de version.
:::

:::quiz
Qu'est-ce qu'un test de caractérisation ?

- [ ] Un test qui vérifie que le code respecte la spécification officielle
- [x] Un test qui fige le comportement actuel, même s'il est discutable
- [ ] Un test qui mesure la vitesse du code
- [ ] Un test écrit par l'auteur d'origine du projet

> On ne juge pas le résultat, on le note : le but est de détecter un changement, pas de valider une spécification.
:::

:::quiz
Le fichier `tests.py` de PlanningAPI ne contient que `# Create your tests here.`. Que penser d'une CI verte sur ce projet ?

- [x] Elle ne prouve presque rien, car presque aucun code n'est testé
- [ ] Elle prouve que le projet est prêt pour Django 5.2
- [ ] Elle prouve que la couverture est de 100 %
- [ ] Elle prouve que les dépendances sont à jour

> Sans test, `manage.py test` réussit à vide : le vert n'a de valeur que si des tests existent.
:::

:::quiz
Un test d'une API appelle un serveur Keycloak. Que faire pour qu'il tourne en CI sans ce serveur ?

- [ ] Désactiver le test
- [ ] Le lancer uniquement sur ton poste
- [x] Remplacer l'appel à Keycloak par un faux (*mock*) dans le test
- [ ] Ajouter `try/except` autour de tout le test

> Un faux renvoie une réponse prévue à l'avance ; le test vérifie ton code, pas le serveur externe.
:::
