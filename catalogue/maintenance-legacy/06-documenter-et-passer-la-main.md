---
id: documenter-et-passer-la-main
titre: "Documenter et passer la main"
resume: "Laisser derrière toi un journal de montée de version, des vérifications simples et un projet que la personne suivante peut reprendre."
duree: 30
objectifs:
  - Expliquer pourquoi documenter une montée de version fait partie du travail
  - Rédiger un journal de montée de version utile à la personne suivante
  - Vérifier automatiquement que l'environnement correspond à ce qui est attendu
---

Dans un an, tu auras quitté l'équipe, ou simplement oublié ce que tu as fait. La personne qui ouvrira le projet découvrira un `requirements.txt` modifié et aucune explication. Elle sera exactement dans ta situation du début de ce parcours : face à un héritage sans mode d'emploi.

## À quoi ça sert, et pourquoi à la fin ?

Une montée de version laisse des **traces invisibles** : pourquoi telle bibliothèque est plafonnée, quelle correction a été nécessaire, ce qui a été volontairement laissé de côté. Si ces décisions ne sont pas écrites, la prochaine personne les redécouvrira à ses dépens, ou les défera sans le savoir. Documenter ne prend qu'un quart d'heure et en économise des dizaines.

## Un journal de montée de version

Le **journal** est un fichier court (`docs/montees-de-version.md`, par exemple) qui garde une entrée par palier. Voici un modèle (les dates, numéros de merge request et de ticket sont des exemples inventés) :

```markdown
## Django 3.1.13 vers 3.2 (LTS)

- Date et merge request : 2026-10-03, !42
- Pourquoi : Django 3.1 n'est plus maintenu.
- Changements de code : ajout de `DEFAULT_AUTO_FIELD` dans `settings.py`.
- Dépendances touchées : `djangorestframework` 3.12.1 vers 3.12.4.
- Laissé de côté : remplacement de `django-rest-swagger` (non maintenu), voir ticket #17.
- Retour arrière : revenir au tag `avant-django-3.2` et réappliquer la sauvegarde de la base.
```

Chaque ligne répond à une question que la personne suivante se posera : pourquoi, quoi, quoi **pas**, et comment annuler.

## Poser des repères dans le dépôt

Quelques gestes simples qui s'ajoutent au journal :

1. **Un tag Git avant chaque palier** (`git tag avant-django-3.2`). Un *tag* est une étiquette fixe sur un commit : revenir en arrière devient une commande.
2. **Un fichier de dépendances unique et clair.** L'API d'Adhésion contient `requirements.txt` (une vingtaine de lignes, les dépendances directes) et `requirements-old.txt` (une liste d'une centaine de lignes). Celui-ci ressemble à un ancien gel complet des versions, conservé comme référence : à confirmer avec l'équipe. Si tu as la même situation, écris dans le `README` lequel fait foi et supprime l'autre quand il ne sert plus.
3. **Les versions des outils dans le `README`** : version de Python, de Node.js, commande pour lancer les tests.
4. **Les dépendances suspectes ou abandonnées listées** (comme `django-rest-swagger` dans Adhésion, ou la ligne `django-rest-framework==0.1.0` de PlanningAPI) avec un ticket (une fiche de suivi dans GitLab) pour chacune.

:::info Pour les projets que tu ne maintiens pas seul·e
L'équipe fonctionne avec des équipes qui changent chaque année. Un journal à jour, c'est ce qui permet à une nouvelle recrue de reprendre PlanningAPI sans appeler la personne précédente.
:::

## Vérifier que l'environnement est conforme

Une vérification automatique vaut mieux qu'une consigne écrite. Ce script Python compare les versions installées à celles que tu attends (il a été exécuté tel quel) :

```python
from importlib.metadata import PackageNotFoundError, version

ATTENDUES = {"Django": "5.2", "sqlparse": "0.5"}

for nom, prefixe in ATTENDUES.items():
    try:
        installee = version(nom)
    except PackageNotFoundError:
        print(f"{nom}: absent")
        continue
    etat = "ok" if installee.startswith(prefixe) else "À REVOIR"
    print(f"{nom}: {installee} (attendu {prefixe}.x) {etat}")
```

Ligne à ligne :

- `ATTENDUES` associe chaque paquet au début de version qu'on attend.
- `version(nom)` demande à Python quelle version est réellement installée ; si le paquet n'existe pas, `PackageNotFoundError` est levée, et le script affiche « absent ».
- `startswith(prefixe)` compare le début du numéro : `5.2.17` commence bien par `5.2`.
- Le `print` affiche une ligne par paquet, avec `ok` ou `À REVOIR`.

Exemple de sortie sur un environnement où `sqlparse` est plus récent que prévu :

```console
Django: 5.2.17 (attendu 5.2.x) ok
sqlparse: 0.6.0 (attendu 0.5.x) À REVOIR
```

Pour une vérification plus générale, `pip check` signale les dépendances dont les versions sont incompatibles entre elles :

```bash
pip check
```

Si tout va bien, la commande répond `No broken requirements found.`.

:::tip Mets-le en CI
Une vérification lancée à chaque pipeline attrape les dérives, par exemple quand quelqu'un modifie `requirements.txt` sans mettre à jour l'image `Dockerfile`.
:::

## Passer la main

Avant de partir, relis le projet comme si tu le découvrais :

- Le `README` explique comment lancer le projet et les tests.
- Le journal contient chaque palier, y compris ce qui a été reporté.
- La CI est verte et Renovate est configuré (leçon précédente).
- Les accès (le *registry*, le serveur qui stocke les images Docker du projet, et les secrets, c'est-à-dire les mots de passe et jetons) sont notés **sans** mot de passe en clair.

## Entraîne-toi

:::labo
moteur: reel
intro: |
  Le projet `adherents` vient d'être monté en Django 5.2 : il est dans ton dossier de travail, déjà enregistré dans un dépôt Git (le dossier contient `README.md`, `requirements.txt`, `requirements-old.txt` et le script `verifier_env.py`). Tu prépares la passation. Pour exécuter Python avec Django 5.2, utilise `/opt/venvs/django52/bin/python`. Les commandes fonctionnent hors ligne.
commandes:
  - cp -R /opt/exercices/06-passer-la-main/. .
  - git init -q
  - git config --global user.name "Apprenant"
  - git config --global user.email "apprenant@exemple.invalid"
  - git add -A
  - git commit -q -m "Projet monté en Django 5.2"
etapes:
  - texte: 'Écris le journal `docs/montees-de-version.md` avec une entrée `## Django 3.1 vers 3.2` qui contient les lignes `Pourquoi`, `Laissé de côté` et `Retour arrière`'
    indice: 'Crée le dossier `docs`, puis le fichier avec `nano docs/montees-de-version.md`. Suis le modèle de la leçon : un titre `## Django 3.1 vers 3.2`, puis une ligne `- Pourquoi : …`, `- Laissé de côté : …` et `- Retour arrière : …`.'
    verif:
      - commande-reussit: '/opt/outils/verifier-legacy doc journal'
    solution:
      - ecrire:
          docs/montees-de-version.md: |
            ## Django 3.1 vers 3.2 (LTS)

            - Pourquoi : Django 3.1 n'est plus maintenu.
            - Changements de code : ajout de `DEFAULT_AUTO_FIELD` dans `settings.py`.
            - Laissé de côté : passage de `re_path` à `path`.
            - Retour arrière : revenir au tag `avant-django-3.2`.

  - texte: 'Pose le tag Git `avant-django-3.2` sur le commit actuel'
    indice: '`git tag avant-django-3.2` ; `git tag` seul liste les tags.'
    verif:
      - commande-reussit: git rev-parse -q --verify refs/tags/avant-django-3.2
    solution:
      - git tag avant-django-3.2

  - texte: 'Complète `README.md` : indique les versions `Python 3.10` et `Django 5.2`, et la commande `python manage.py test`'
    indice: 'Remplace « À écrire. » dans `README.md` (avec `nano`) par quelques lignes, par exemple « Versions : Python 3.10, Django 5.2 » puis « Tests : `python manage.py test` ».'
    verif:
      - commande-reussit: '/opt/outils/verifier-legacy doc readme'
    solution:
      - |-
        printf '%s\n' 'Versions : Python 3.10, Django 5.2.' 'Tests : `python manage.py test`.' >> README.md

  - texte: 'Lance `verifier_env.py` avec le Python de Django 5.2 et garde le résultat dans `verification.txt` ; la ligne de Django doit se terminer par `ok`'
    indice: '`/opt/venvs/django52/bin/python verifier_env.py > verification.txt`. Le script compare les versions installées à celles qu''il attend.'
    verif:
      - commande-reussit: '/opt/outils/verifier-legacy sortie verification.txt verification'
      - fichier-contient-dans-env: [verification.txt, '(?m)^Django: 5\.2\.\d+ \(attendu 5\.2\.x\) ok$']
    solution:
      - /opt/venvs/django52/bin/python verifier_env.py > verification.txt

  - texte: 'Vérifie la cohérence de l''environnement Django 5.2 avec `pip check` et écris le résultat dans `pip-check.txt`'
    indice: 'Lance pip par le Python de l''environnement : `/opt/venvs/django52/bin/python -m pip check > pip-check.txt`.'
    verif:
      - commande-reussit: '/opt/outils/verifier-legacy sortie pip-check.txt pip-check'
      - fichier-contient-dans-env: [pip-check.txt, 'No broken requirements found']
    solution:
      - /opt/venvs/django52/bin/python -m pip check > pip-check.txt

  - texte: 'Dis dans le `README.md` que `requirements.txt` fait foi, puis supprime l''ancien gel `requirements-old.txt`'
    indice: 'Ajoute une phrase qui cite `requirements.txt` à la fin de `README.md` (`>>` ajoute à la fin), puis `rm requirements-old.txt`.'
    apres: [3]
    verif:
      - commande-reussit: '/opt/outils/verifier-legacy doc readme-requirements'
    solution:
      - printf '%s\n' 'Les dépendances sont dans `requirements.txt` (il fait foi).' >> README.md
      - rm requirements-old.txt
:::

## Vérifie tes acquis

:::quiz
Quelle information le journal de montée de version doit-il contenir en priorité ?

- [ ] La liste complète des commits
- [ ] Le nom de la personne qui a relu la merge request
- [x] Ce qui a été fait, pourquoi, ce qui a été laissé de côté et comment revenir en arrière
- [ ] Le temps passé sur chaque correction

> Ces quatre réponses sont celles que la personne suivante cherchera en cas de problème.
:::

:::quiz
À quoi sert `git tag avant-django-3.2` posé avant un palier ?

- [ ] À accélérer la CI
- [ ] À supprimer les anciennes branches
- [x] À marquer un point de retour fixe, simple à retrouver
- [ ] À installer automatiquement Django 3.2

> Un tag est une étiquette sur un commit : tu peux y revenir sans chercher dans l'historique.
:::

:::quiz
Que fait `pip check` ?

- [ ] Il met à jour toutes les dépendances
- [x] Il signale les dépendances installées dont les versions sont incompatibles entre elles
- [ ] Il lance les tests du projet
- [ ] Il liste les failles de sécurité connues

> C'est un contrôle de cohérence de l'environnement, pas une mise à jour ni une analyse de sécurité.
:::
