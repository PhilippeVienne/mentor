## Faire l'état des lieux

| Où lire | Ce que tu y trouves |
| --- | --- |
| `requirements.txt` | Dépendances Python ; `Django==3.1.13` = exactement cette version |
| `package.json` | Dépendances JavaScript ; `^16.13.1` = 16.13.1 ou toute 16.x plus récente |
| `Dockerfile` | Version du langage (`FROM python:3.9-slim`) |
| Notes de version | Ce qui est ajouté, déprécié, retiré |

## Chemin Django

`3.1` → `3.2` (LTS) → `4.2` (LTS) → `5.2` (LTS). Un palier = tests verts + CI verte + fusion avant de passer au suivant. Django 5.2 demande au moins Python 3.10.

## Commandes de contrôle

| Besoin | Commande |
| --- | --- |
| Lancer les tests | `python manage.py test` |
| Transformer les dépréciations en erreurs | `python -W error::DeprecationWarning manage.py test` |
| Vérifier la configuration | `python manage.py check` |
| Aucune migration surprise | `python manage.py makemigrations --check --dry-run` |
| Dépendances incompatibles | `pip check` |
| Tout avertissement de configuration en erreur | `python manage.py check --fail-level WARNING` |
| Mesurer la couverture | `coverage run --source="." manage.py test` puis `coverage report` |

## API retirées (Django 4.0 et 5.0)

| Ancien | Nouveau |
| --- | --- |
| `force_text` | `force_str` |
| `ugettext_lazy` | `gettext_lazy` |
| `django.conf.urls.url` | `django.urls.re_path` (ou `path`) |
| `django.utils.timezone.utc` (déprécié en 4.1, retiré en 5.0) | `datetime.timezone.utc` |
| Avertissement `models.W042` | `DEFAULT_AUTO_FIELD = "django.db.models.AutoField"` |

## Front React

`react-scripts` → Vite (`vite`, `@vitejs/plugin-react`, `vite.config.js`, `index.html` à la racine, fichiers `.jsx`, variables `VITE_…` lues par `import.meta.env`) → React 17 → Material-UI v5 (`@mui/material`, codemods) → React 18 (`createRoot`).

## Renovate sur un projet en retard

| Besoin | Réglage |
| --- | --- |
| Plafonner Django au palier | `"allowedVersions": "<3.3"` dans une `packageRules` |
| Regrouper des paquets liés | `"groupName": "dépendances Django"` |
| Étiqueter les failles | `"vulnerabilityAlerts": {"labels": ["Priority::Critical"]}` |

## Passer la main

Tag Git avant chaque palier · journal de montée de version (quoi, pourquoi, laissé de côté, retour arrière) · versions des outils dans le `README` · dépendances abandonnées listées avec un ticket.
