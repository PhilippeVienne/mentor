---
id: monter-django
title: "Monter de version Django pas à pas"
summary: "Faire passer un projet Django de palier en palier, en suivant les avertissements et en corrigeant les API retirées."
minutes: 40
objectives:
  - Afficher les avertissements de dépréciation d'un projet Django
  - Reconnaître et corriger les API retirées les plus courantes
  - Dérouler la procédure d'un palier de montée de version, de la branche à la fusion
---

Tu as choisi ton chemin (3.1, puis 3.2, puis 4.2, puis 5.2) et écrit tes premiers tests. Il reste à franchir le premier palier. Comment savoir ce qu'il faut modifier, sans lire à la main les notes de plusieurs versions ?

## À quoi ça sert, et pourquoi pas à pas ?

Django te prévient avant de retirer quelque chose. Une fonction qui va disparaître continue de marcher, mais affiche un avertissement (*warning*). En traitant ces avertissements **avant** de changer de version, tu transformes un mur d'erreurs en une liste de petites corrections, que tu fais tranquillement sur la version actuelle.

Par défaut, Python cache ces avertissements. Voici un exemple, exécuté avec Django 5.2 : `format_html` sans argument est déprécié et sera retiré dans Django 6.0.

```python
from django.conf import settings
settings.configure()
from django.utils.html import format_html
print(format_html("<b>Bonjour</b>"))
```

- `settings.configure()` donne à Django une configuration minimale, pour l'utiliser sans projet complet.
- `format_html(...)` est une fonction de Django qui fabrique du HTML sûr.

Lancé avec l'option `-W default` de Python (qui affiche les avertissements), le script répond :

```console
demo_warn.py:4: RemovedInDjango60Warning: Calling format_html() without passing args or kwargs is deprecated.
  print(format_html("<b>Bonjour</b>"))
<b>Bonjour</b>
```

Avec `-W error::DeprecationWarning`, le même avertissement devient une **erreur** qui arrête le programme : c'est le réglage idéal en CI, pour ne rater aucun avertissement. Pour tout un projet :

```bash
python -W error::DeprecationWarning manage.py test
```

La première commande lance Python en transformant chaque avertissement de dépréciation en erreur, puis exécute les tests du projet. Tant qu'elle échoue, il reste du travail à faire sur la version actuelle.

## Les API retirées les plus courantes

Plusieurs fonctions dépréciées dans Django 3.x ont été supprimées en 4.0 ; une autre, `timezone.utc`, a été dépréciée en 4.1 puis supprimée en 5.0. Dans un projet 3.1, tu risques de les rencontrer toutes. On a vérifié ci-dessous que chacune lève bien une erreur d'import sous Django 5.2 :

| Ancien code | Retiré en | Erreur sous Django 5.2 | Remplacement |
| --- | --- | --- | --- |
| `from django.utils.encoding import force_text` | 4.0 | `ImportError: cannot import name 'force_text'` | `force_str` |
| `from django.utils.translation import ugettext_lazy` | 4.0 | `ImportError: cannot import name 'ugettext_lazy'` | `gettext_lazy` |
| `from django.conf.urls import url` | 4.0 | `ImportError: cannot import name 'url'` | `re_path` (ou `path`) |
| `from django.utils.timezone import utc` | 5.0 (déprécié dès 4.1) | `ImportError: cannot import name 'utc'` | `datetime.timezone.utc` |

Un `ImportError`, c'est Python qui dit « ce nom n'existe pas dans ce module ». Voici la correction d'un fichier `urls.py`, au format `diff` (le `-` est la ligne supprimée, le `+` la ligne ajoutée) :

```diff
-from django.conf.urls import url
+from django.urls import re_path

 urlpatterns = [
-    url(r"^api/", include("adherents.urls")),
+    re_path(r"^api/", include("adherents.urls")),
 ]
```

`re_path` accepte les mêmes expressions régulières que l'ancien `url`, donc le comportement reste identique. Tu peux simplifier en `path("api/", ...)` plus tard, dans un autre **commit** (un enregistrement daté de modifications dans Git) : une chose à la fois.

Les deux autres cas se corrigent de la même façon. Pour `timezone.utc`, c'est Python lui-même qui fournit l'équivalent :

```diff
-from django.utils.timezone import utc
+from datetime import timezone
 ...
-    return datetime.now(tz=utc)
+    return datetime.now(tz=timezone.utc)
```

Et pour `ugettext_lazy` ou `force_text`, on change seulement le nom : `from django.utils.translation import gettext_lazy as _` et `force_str(...)`.

## Les réglages et les migrations

Un autre sujet te rencontrera dès Django 3.2 : l'avertissement `models.W042`, qui dit « le type de clé primaire (l'identifiant automatique de chaque ligne) n'est pas précisé ». Pour garder le comportement existant, ajoute dans `settings.py` :

```python
DEFAULT_AUTO_FIELD = "django.db.models.AutoField"
```

Cela dit à Django de continuer à utiliser des identifiants `AutoField`, comme avant. Choisir `BigAutoField` serait une migration de base de données : à faire à part, pas pendant une montée de version.

Les **migrations** sont les fichiers qui décrivent l'évolution du schéma de la base. Une montée de version ne doit pas en créer de nouvelles par surprise. Cette commande le vérifie :

```bash
python manage.py makemigrations --check --dry-run
```

Elle simule la création de migrations (`--dry-run`) et échoue (`--check`) s'il y en aurait. Sur un projet sain, elle répond `No changes detected`.

Dernier contrôle utile : `python manage.py check --fail-level WARNING` lance les vérifications de configuration de Django et échoue au moindre avertissement, comme `models.W042`. Par défaut, `check` n'échoue que sur une vraie erreur.

## La procédure d'un palier

1. Crée une **branche** dédiée. Une branche est une ligne de développement parallèle dans Git : tu y travailles sans toucher à la version principale. `git switch -c django-3.2` fait deux choses, `-c` crée la branche `django-3.2` et `switch` s'y place.
2. Sur la version actuelle, lance les tests avec `-W error::DeprecationWarning` et corrige.
3. Modifie la version dans `requirements.txt` (`Django==3.2.25`, par exemple, en vérifiant la dernière correction de la série sur PyPI, le site public où sont publiés les paquets Python).
4. Réinstalle : `pip install -r requirements.txt`. `pip` est l'outil qui télécharge les paquets listés dans le fichier et les installe.
5. Lance `python manage.py check`, `makemigrations --check --dry-run`, puis les tests.
6. Corrige, puis ouvre une **merge request** : une proposition de modification que l'équipe relit avant de la **fusionner** dans la branche principale (c'est une *pull request* sur GitHub). La CI, le robot qui rejoue les tests, vérifie tout.
7. Déploie sur un environnement de test, fais vérifier les écrans critiques, fusionne.

Les dépendances de Django suivent le mouvement. Dans les dépôts de l'équipe, voici celles à regarder en premier (versions lues dans les fichiers) :

| Paquet | Version actuelle | À vérifier |
| --- | --- | --- |
| `djangorestframework` | 3.11.1 (PlanningAPI), 3.12.1 (Adhésion) | La compatibilité avec la version de Django visée, dans son README |
| `django-import-export` | 2.4.0 et 2.5.0 | Ses notes de version |
| `django-rest-swagger` | 2.2.0 (Adhésion) | Le projet n'est plus maintenu à ma connaissance : prévois un remplaçant |
| `psycopg2-binary` | 2.8.6 | Le support de ta version de Python |

:::tip Un commit par correction
Sépare « monter la version » des « corrections de code ». Si un test casse, tu sais quel commit incriminer, et tu peux le retirer sans tout perdre.
:::

:::danger Ne teste pas sur la vraie base
Une migration lancée sur la base de production (le serveur réel, utilisé par les adhérent·e·s) est difficile à annuler. Fais d'abord une sauvegarde, puis rejoue toute la montée sur une copie des données.
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  Le projet `adherents` est dans ton dossier de travail, avec un petit jeu de tests de référence (`adherents/tests.py`) qui passe sous Django 3.1. Il contient les pièges de la leçon. Quatre versions de Django sont installées dans `/opt/venvs/` : tu lances chaque palier avec le Python de l'environnement voulu, par exemple `/opt/venvs/django32/bin/python manage.py test`. Les commandes fonctionnent hors ligne.
commands:
  - cp -R /opt/exercices/projet-legacy/. .
  - cp /opt/exercices/tests-de-reference.py adherents/tests.py
steps:
  - text: 'Supprime l''avertissement `models.W042` : ajoute le réglage `DEFAULT_AUTO_FIELD` à `config/settings.py`'
    hint: 'Ajoute la ligne `DEFAULT_AUTO_FIELD = "django.db.models.AutoField"` à la fin de `config/settings.py`. Contrôle avec `/opt/venvs/django32/bin/python manage.py check --fail-level WARNING`.'
    checks:
      - command-succeeds: '/opt/outils/verifier-legacy code auto-field'
      - command-succeeds: /opt/venvs/django32/bin/python manage.py check --fail-level WARNING
      - command-succeeds: /opt/venvs/django32/bin/python manage.py check --fail-level WARNING
    solution:
      - echo 'DEFAULT_AUTO_FIELD = "django.db.models.AutoField"' >> config/settings.py

  - text: 'Dans `config/urls.py`, remplace `url` par `re_path`'
    hint: 'Deux changements : `from django.urls import include, re_path` en haut, et `re_path(r"^adherents/", ...)` à la place de `url(...)`.'
    checks:
      - command-succeeds: '/opt/outils/verifier-legacy code urls'
      - command-succeeds: '/opt/outils/verifier-legacy palier django32'
    solution:
      - write:
          config/urls.py: |
            from django.urls import include, re_path

            urlpatterns = [
                re_path(r"^adherents/", include("adherents.urls")),
            ]

  - text: 'Remplace `force_text` par `force_str` et `ugettext_lazy` par `gettext_lazy` dans tout le projet'
    hint: 'Deux fichiers sont concernés : `adherents/utils.py` (`force_text`) et `adherents/models.py` (`ugettext_lazy`). `sed -i` remplace un texte dans un fichier ; pour repérer les occurrences : `grep -rn "force_text\|ugettext" adherents config`.'
    checks:
      - command-succeeds: '/opt/outils/verifier-legacy code textes-depreces'
      - command-succeeds: '/opt/outils/verifier-legacy palier django32'
    solution:
      - sed -i 's/force_text/force_str/g' adherents/utils.py
      - sed -i 's/ugettext_lazy/gettext_lazy/g' adherents/models.py

  - text: 'Franchis le palier 3.2 : passe `requirements.txt` à `Django==3.2.25`, et vérifie que les tests passent avec Django 3.2 même quand les avertissements de dépréciation sont transformés en erreurs'
    hint: 'Écris `Django==3.2.25` dans `requirements.txt`, puis lance `/opt/venvs/django32/bin/python -W error::DeprecationWarning manage.py test`. S''il reste une erreur, lis le nom de la fonction dépréciée dans le message.'
    after: [1, 2, 3]
    checks:
      - command-succeeds: '/opt/outils/verifier-legacy code requirements-32'
      - command-succeeds: '/opt/outils/verifier-legacy palier django32 --erreurs-deprecation'
    solution:
      - echo 'Django==3.2.25' > requirements.txt
      - /opt/venvs/django32/bin/python -W error::DeprecationWarning manage.py test

  - text: 'Franchis le palier 4.2 : remplace `timezone.utc` de Django par `datetime.timezone.utc` dans `adherents/utils.py`, jusqu''à ce que les tests passent avec Django 4.2 sans avertissement'
    hint: 'Dans `adherents/utils.py`, importe `from datetime import datetime, timezone`, supprime l''import de `django.utils.timezone` et écris `datetime.now(tz=timezone.utc)`. Contrôle avec `/opt/venvs/django42/bin/python -W error::DeprecationWarning manage.py test`.'
    after: [4]
    checks:
      - command-succeeds: '/opt/outils/verifier-legacy code utc'
      - command-succeeds: '/opt/outils/verifier-legacy palier django42 --erreurs-deprecation'
    solution:
      - write:
          adherents/utils.py: |
            from datetime import datetime, timezone

            from django.utils.encoding import force_str


            def maintenant_utc():
                """Date et heure actuelles, en UTC."""
                return datetime.now(tz=timezone.utc)


            def etiquette(adherent):
                """Texte affiché pour un adhérent dans les listes."""
                return force_str(adherent.nom_complet())

  - text: 'Franchis le palier 5.2 : les tests passent avec Django 5.2, `makemigrations --check --dry-run` ne propose aucune migration, et `requirements.txt` demande Django 5.2'
    hint: 'Lance `/opt/venvs/django52/bin/python manage.py test`, puis `/opt/venvs/django52/bin/python manage.py makemigrations --check --dry-run`. Pour `requirements.txt`, écris `Django==5.2.17`.'
    after: [5]
    checks:
      - command-succeeds: '/opt/outils/verifier-legacy palier django52 --migrations'
      - command-succeeds: '/opt/outils/verifier-legacy code requirements-52'
    solution:
      - echo 'Django==5.2.17' > requirements.txt
      - /opt/venvs/django52/bin/python manage.py test
      - /opt/venvs/django52/bin/python manage.py makemigrations --check --dry-run
:::

## Vérifie tes acquis

:::quiz
Quel est l'intérêt de lancer les tests avec `-W error::DeprecationWarning` ?

- [ ] Ils s'exécutent plus vite
- [ ] Ils ignorent les fonctions dépréciées
- [x] Tout avertissement de dépréciation fait échouer les tests, donc aucun n'est oublié
- [ ] Ils mettent Django à jour automatiquement

> L'option transforme les avertissements en erreurs : tant qu'il en reste, le travail n'est pas fini.
:::

:::quiz
Ton projet contient `from django.utils.translation import ugettext_lazy`. Que se passe-t-il sous Django 5.2 ?

- [ ] Rien, la fonction est simplement dépréciée
- [x] Une `ImportError`, car elle a été retirée ; il faut utiliser `gettext_lazy`
- [ ] Django la remplace tout seul au démarrage
- [ ] Le texte n'est plus traduit, sans erreur

> La fonction a été retirée dans Django 4.0 ; elle n'existe plus sous 5.2.
:::

:::quiz
Que vérifie `python manage.py makemigrations --check --dry-run` pendant une montée de version ?

- [ ] Que la base de données est sauvegardée
- [ ] Que toutes les migrations ont été appliquées en production
- [x] Que la montée ne crée pas de migration inattendue
- [ ] Que les tests passent

> Si la commande échoue, la nouvelle version veut modifier le schéma : lis la migration proposée avant de continuer.
:::

:::quiz
Pourquoi remplacer `url()` par `re_path()` plutôt que directement par `path()` ?

- [ ] Parce que `path()` n'existe pas dans Django 4
- [x] Parce que `re_path()` garde exactement le même comportement : on change une seule chose à la fois
- [ ] Parce que `re_path()` est plus rapide
- [ ] Parce que `url()` est toujours disponible dans Django 5.2

> `path()` utilise une autre syntaxe que les expressions régulières ; la convertir est une seconde étape, séparée.
:::
