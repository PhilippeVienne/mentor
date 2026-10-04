"""Réglages du projet « Agenda des associations » (domaine fictif de la formation)."""
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent

# Pas de vrai secret ici : cette clé ne sert qu'à l'exercice.
SECRET_KEY = "cle-factice-pour-la-formation-uniquement"
DEBUG = True
ALLOWED_HOSTS = ["*"]

INSTALLED_APPS = [
    "django.contrib.auth",
    "django.contrib.contenttypes",
    "django.contrib.sessions",
    "rest_framework",
    "django_filters",
    "agenda",
]

MIDDLEWARE = [
    "django.contrib.sessions.middleware.SessionMiddleware",
    "django.middleware.common.CommonMiddleware",
    "django.contrib.auth.middleware.AuthenticationMiddleware",
]

ROOT_URLCONF = "projet.urls"

DATABASES = {
    "default": {
        "ENGINE": "django.db.backends.sqlite3",
        "NAME": BASE_DIR / "db.sqlite3",
    }
}

LANGUAGE_CODE = "fr-fr"
TIME_ZONE = "Europe/Paris"
USE_TZ = True
DEFAULT_AUTO_FIELD = "django.db.models.BigAutoField"

# Jetons JWT (leçon 4) : application visée par les jetons et clé publique qui vérifie leur signature.
# Une clé PUBLIQUE n'est pas un secret ; les tests fabriquent leur propre paire de clés.
OIDC_CLIENT_ID = "agenda-api"
OIDC_PUBLIC_KEY = ""

REST_FRAMEWORK = {
    "DEFAULT_AUTHENTICATION_CLASSES": ["agenda.auth.KeycloakJWTAuthentication"],
    "DEFAULT_PERMISSION_CLASSES": ["rest_framework.permissions.IsAuthenticated"],
    "DEFAULT_PAGINATION_CLASS": "rest_framework.pagination.PageNumberPagination",
    "PAGE_SIZE": 20,
}
