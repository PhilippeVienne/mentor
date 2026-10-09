---
id: authentification-permissions
title: "Authentification par jeton OIDC et permissions"
summary: "Savoir qui appelle l'API (authentification) puis décider ce qu'elle a le droit de faire (permissions par rôle)."
minutes: 50
objectives:
  - Distinguer authentification et permission, `401` et `403`
  - Expliquer ce qu'est un jeton JWT et le rôle de Keycloak (OIDC)
  - Écrire une permission par rôle et l'appliquer à un viewset
  - Combiner plusieurs permissions et relire celles de l'API d'Adhésion
---

Jusqu'ici, n'importe qui peut lire, créer et supprimer des événements. Pour une API qui expose des données d'adhérent·e·s, c'est inacceptable. Deux questions se posent, dans cet ordre : **qui es-tu ?** (authentification) puis **as-tu le droit de faire ça ?** (permission).

## Deux étapes, deux codes d'erreur

| Étape | Question | Échec | Code |
| --- | --- | --- | :---: |
| Authentification | Qui appelle ? | Pas de preuve d'identité, ou preuve invalide | `401` |
| Permission | A-t-il le droit ? | Identifié·e, mais pas autorisé·e | `403` |

À retenir : `401` = « je ne sais pas qui tu es », `403` = « je sais qui tu es, et c'est non ».

## Le jeton : une preuve d'identité qui voyage avec la requête

Un client ne peut pas renvoyer son mot de passe à chaque requête. Il obtient donc auprès d'un serveur d'identité un **jeton** (*token*), une chaîne de caractères qui sert de preuve, et l'envoie dans l'en-tête `Authorization` :

```console
GET /v1/evenements/ HTTP/1.1
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

`Bearer` signifie « porteur » : qui présente le jeton est considéré comme son propriétaire. Il ne faut donc jamais le publier.

Les jetons utilisés dans l'équipe sont des **JWT** (*JSON Web Token*) : un jeton qui contient lui-même des informations (appelées *claims*), comme l'adresse email de la personne et ses rôles, avec une **signature** qui prouve qu'ils n'ont pas été modifiés. Ils sont délivrés par **Keycloak**, le serveur d'identité auquel l'API d'Adhésion fait référence dans ses réglages (`KEYCLOAK_URL`, `REALM_NAME`). Le protocole qui décrit cette délégation s'appelle **OIDC** (*OpenID Connect*). L'API d'Adhésion utilise la bibliothèque `mozilla_django_oidc` (version `1.2.4`), dont elle étend la classe `OIDCAuthentication`.

Un jeton décodé ressemble à ceci (extrait fictif) :

```json
{
  "email": "alice@example.org",
  "aud": "agenda-api",
  "exp": 1893456000,
  "resource_access": {
    "agenda-api": { "roles": ["staff"] }
  }
}
```

- `email` : qui est la personne ;
- `aud` (*audience*) : pour quelle application le jeton est destiné ;
- `exp` : date d'expiration ;
- `resource_access` : les **rôles** de la personne, regroupés par application (par « client » Keycloak). Le nom `resource_access` est celui qu'utilise le code de permissions d'Adhésion.

## Brancher l'authentification dans DRF

Une **classe d'authentification** lit la requête et renvoie `(utilisateur, jeton)`, ou `None` si la requête ne tente pas de s'authentifier. Version simplifiée de ce que fait Adhésion, avec la bibliothèque `PyJWT` :

```python
# agenda/auth.py
import jwt
from django.conf import settings
from django.contrib.auth import get_user_model
from rest_framework import exceptions
from rest_framework.authentication import BaseAuthentication, get_authorization_header


class KeycloakJWTAuthentication(BaseAuthentication):
    def authenticate(self, request):
        parts = get_authorization_header(request).split()
        if not parts or parts[0].lower() != b"bearer":
            return None
        if len(parts) != 2:
            raise exceptions.AuthenticationFailed("En-tête Authorization invalide.")
        try:
            claims = jwt.decode(
                parts[1],
                settings.OIDC_PUBLIC_KEY,
                algorithms=["RS256"],
                audience=settings.OIDC_CLIENT_ID,
            )
        except jwt.PyJWTError:
            raise exceptions.AuthenticationFailed("Jeton invalide ou expiré.")
        user, _ = get_user_model().objects.get_or_create(username=claims["email"])
        return user, claims

    def authenticate_header(self, request):
        return 'Bearer realm="agenda"'
```

Ligne à ligne :

1. `get_authorization_header(request).split()` découpe l'en-tête `Authorization` en `["Bearer", "<jeton>"]`.
2. Pas d'en-tête, ou autre schéma : on renvoie `None`. DRF essaie alors la classe suivante, et finalement la requête reste **anonyme**.
3. `jwt.decode(...)` vérifie la **signature** avec la clé publique du serveur d'identité (`RS256` : signature par clé privée, vérification par clé publique), la date d'expiration et l'`audience`. Si quelque chose cloche, on lève `AuthenticationFailed`, qui devient une réponse `401`.
4. `get_or_create(username=...)` retrouve (ou crée) l'utilisateur Django correspondant à l'email du jeton. Adhésion, elle, retrouve l'utilisateur par un identifiant présent dans le jeton et synchronise nom et email avec Keycloak.
5. Le couple renvoyé alimente `request.user` (l'utilisateur) et `request.auth` (ici le dictionnaire des claims).
6. `authenticate_header` fournit l'en-tête `WWW-Authenticate` : sans lui, DRF répondrait `403` au lieu de `401` quand l'identification manque. Le `realm` (« domaine ») qu'il annonce est un simple nom pour la zone protégée, ici `agenda` ; ne le confonds pas avec le *realm* de Keycloak (le réglage `REALM_NAME` d'Adhésion), qui est un espace regroupant les personnes et les applications d'une même organisation.

On l'active dans `settings.py` :

```python
REST_FRAMEWORK = {
    "DEFAULT_AUTHENTICATION_CLASSES": ["agenda.auth.KeycloakJWTAuthentication"],
    "DEFAULT_PERMISSION_CLASSES": ["rest_framework.permissions.IsAuthenticated"],
}
```

La liste d'authentification est essayée dans l'ordre. Celle d'Adhésion en contient trois : `SyncedKeycloakAuthentication`, puis `SessionAuthentication` (pour l'interface web de Django) et `BasicAuthentication` (identifiant et mot de passe envoyés à chaque requête). `DEFAULT_PERMISSION_CLASSES` est la règle appliquée partout où la vue n'en précise pas ; la valeur par défaut de DRF est de **tout autoriser**, donc ce réglage est ton filet de sécurité.

:::danger Ne mets jamais un secret dans le code
Le `settings.py` d'Adhésion donne une valeur par défaut en clair à `OIDC_RP_CLIENT_SECRET`. Ne reproduis pas ce réflexe : un secret par défaut dans un dépôt est un secret public. Lis-les uniquement depuis l'environnement (`os.environ["..."]`), sans valeur de repli, comme le montre le cours *CI GitLab* du portail (la CI, ou intégration continue, est un robot qui lance des scripts automatiquement à chaque modification du code ; on y range les secrets dans des variables).
:::

## Les permissions par rôle

Une **permission** est une classe avec une méthode `has_permission(request, view)` qui renvoie `True` ou `False`. Voici le modèle utilisé par Adhésion (classes `KeycloakHasRolePermission`, `HasRoleStaff`…), adapté à notre jeton :

```python
# agenda/permissions.py
from django.conf import settings
from rest_framework.permissions import SAFE_METHODS, BasePermission


def roles_du_jeton(request):
    claims = request.auth if isinstance(request.auth, dict) else {}
    acces = claims.get("resource_access", {}).get(settings.OIDC_CLIENT_ID, {})
    return set(acces.get("roles", []))


class HasRole(BasePermission):
    required_roles = []

    def has_permission(self, request, view):
        return set(self.required_roles) <= roles_du_jeton(request)


class HasRoleStaff(HasRole):
    required_roles = ["staff"]


class HasRoleStaffOrReadOnly(HasRoleStaff):
    def has_permission(self, request, view):
        return request.method in SAFE_METHODS or super().has_permission(request, view)
```

- `roles_du_jeton` lit les rôles dans `request.auth`, en renvoyant un ensemble vide si le jeton n'a pas la forme attendue.
- `HasRole` est une classe de base : `set(required_roles) <= roles` signifie « tous les rôles exigés sont présents ».
- `HasRoleStaff` fixe le rôle exigé : on a ainsi une classe par rôle, avec un nom parlant (Adhésion en a une dizaine : `HasRoleInte`, `HasRoleTrezo`, `HasRoleWei`…).
- `SAFE_METHODS` (`GET`, `HEAD`, `OPTIONS`) sont les verbes qui ne modifient rien : `HEAD` fait comme `GET` mais ne renvoie que les en-têtes, sans le corps (pour vérifier qu'une ressource existe), et `OPTIONS` demande à quoi sert une URL et quels verbes elle accepte. `HasRoleStaffOrReadOnly` laisse lire tout le monde et réserve l'écriture au rôle `staff`.

On les applique à un viewset avec `permission_classes`, ou à une seule action :

```python
from rest_framework.permissions import IsAuthenticated

class EvenementViewSet(viewsets.ModelViewSet):
    permission_classes = [IsAuthenticated, HasRoleStaffOrReadOnly]

    @action(detail=True, methods=["post"], permission_classes=[HasRoleStaff])
    def fermer(self, request, pk=None):
        ...
```

Les permissions d'une liste s'**additionnent** (ET logique) : toutes doivent accepter. Ici, il faut être identifié·e **et** avoir le droit.

Résultats observés :

```console
GET  /v1/evenements/    sans jeton        -> 401 {"detail": "Informations d'authentification non fournies."}
GET  /v1/evenements/    jeton sans rôle   -> 200
POST /v1/evenements/    jeton sans rôle   -> 403 {"detail": "Vous n'avez pas la permission d'effectuer cette action."}
POST /v1/evenements/    rôle staff        -> 201
```

:::warning `permission_classes` remplace le réglage par défaut
Si tu écris `permission_classes = [HasRoleStaffOrReadOnly]` seul, tu **remplaces** `IsAuthenticated` : les `GET` redeviennent publics, même sans jeton, car la classe laisse passer les verbes de lecture. C'est précisément le comportement de `HasRoleStaffOrReadOnly` dans Adhésion (utilisée pour les types d'adhésion et les années d'étude). Si c'est voulu, très bien ; sinon ajoute `IsAuthenticated` dans la liste, comme ci-dessus.
:::

Dernier niveau, la permission **sur un objet** : `has_object_permission(request, view, obj)` s'appelle quand la vue appelle `self.get_object()`. Elle sert quand le droit dépend de l'objet (« seul·e l'organisateur·rice modifie son événement »).

## Entraîne-toi

:::lab
engine: real
intro: |
  Ton API des événements est publique : n'importe qui peut tout faire. Dans ton dossier de travail, les fichiers `agenda/auth.py` et `agenda/permissions.py` sont vides, et `projet/settings.py` ne règle rien. Pour tester sans serveur Keycloak, les tests fabriquent eux-mêmes des jetons JWT signés (avec une paire de clés RSA créée à la volée, voir `helpers_jwt.py`) : tu peux lire ce fichier, mais tu n'as pas à le modifier. Lance `pytest -q test_securite.py` pour voir les sept tests échouer, puis corrige-les dans l'ordre.
commands:
  - cp -R /opt/exercices/base/. .
  - cp -R /opt/exercices/04-authentification-permissions/. .
steps:
  - text: 'Dans `agenda/auth.py`, écris la méthode `authenticate(self, request)` de `KeycloakJWTAuthentication` : elle lit l''en-tête `Authorization`, renvoie `None` s''il n''y a pas de jeton `Bearer`, vérifie le jeton avec `jwt.decode(...)` (clé `settings.OIDC_PUBLIC_KEY`, algorithme `RS256`, audience `settings.OIDC_CLIENT_ID`), lève `AuthenticationFailed` s''il est invalide, et retourne `(utilisateur, claims)`. Les trois tests `authentification` doivent passer'
    hint: 'Reprends le code de la leçon. L''utilisateur se retrouve avec `get_user_model().objects.get_or_create(username=claims["email"])`. Puis `pytest -q test_securite.py -k authentification`.'
    checks:
      - command-succeeds: '/opt/outils/verifier-tests 04-authentification-permissions authentification 3'
    solution:
      - write:
          agenda/auth.py: |
            import jwt
            from django.conf import settings
            from django.contrib.auth import get_user_model
            from rest_framework import exceptions
            from rest_framework.authentication import BaseAuthentication, get_authorization_header


            class KeycloakJWTAuthentication(BaseAuthentication):
                def authenticate(self, request):
                    parts = get_authorization_header(request).split()
                    if not parts or parts[0].lower() != b"bearer":
                        return None
                    if len(parts) != 2:
                        raise exceptions.AuthenticationFailed("En-tête Authorization invalide.")
                    try:
                        claims = jwt.decode(
                            parts[1],
                            settings.OIDC_PUBLIC_KEY,
                            algorithms=["RS256"],
                            audience=settings.OIDC_CLIENT_ID,
                        )
                    except jwt.PyJWTError:
                        raise exceptions.AuthenticationFailed("Jeton invalide ou expiré.")
                    user, _ = get_user_model().objects.get_or_create(username=claims["email"])
                    return user, claims
  - text: 'Ajoute `authenticate_header(self, request)` à la classe : elle retourne `''Bearer realm="agenda"''`. Sans elle, DRF répondrait `403` au lieu de `401` à une personne non identifiée. `test_en_tete_bearer_annonce` doit passer'
    hint: 'Une seule ligne dans la méthode : `return ''Bearer realm="agenda"''`.'
    after: [1]
    checks:
      - command-succeeds: '/opt/outils/verifier-tests 04-authentification-permissions en_tete_bearer'
    solution:
      - write:
          agenda/auth.py: |
            import jwt
            from django.conf import settings
            from django.contrib.auth import get_user_model
            from rest_framework import exceptions
            from rest_framework.authentication import BaseAuthentication, get_authorization_header


            class KeycloakJWTAuthentication(BaseAuthentication):
                def authenticate(self, request):
                    parts = get_authorization_header(request).split()
                    if not parts or parts[0].lower() != b"bearer":
                        return None
                    if len(parts) != 2:
                        raise exceptions.AuthenticationFailed("En-tête Authorization invalide.")
                    try:
                        claims = jwt.decode(
                            parts[1],
                            settings.OIDC_PUBLIC_KEY,
                            algorithms=["RS256"],
                            audience=settings.OIDC_CLIENT_ID,
                        )
                    except jwt.PyJWTError:
                        raise exceptions.AuthenticationFailed("Jeton invalide ou expiré.")
                    user, _ = get_user_model().objects.get_or_create(username=claims["email"])
                    return user, claims

                def authenticate_header(self, request):
                    return 'Bearer realm="agenda"'
  - text: 'Dans `agenda/permissions.py`, écris `roles_du_jeton(request)` (les rôles de `request.auth["resource_access"][settings.OIDC_CLIENT_ID]["roles"]`, ou un ensemble vide), la classe `HasRole`, `HasRoleStaff` (rôle `staff`) et `HasRoleStaffOrReadOnly` (lecture libre, écriture réservée au rôle). `test_roles_et_permissions` doit passer'
    hint: '`HasRole.has_permission` compare des ensembles : `set(self.required_roles) <= roles_du_jeton(request)`. Pour la lecture libre, teste `request.method in SAFE_METHODS`.'
    checks:
      - command-succeeds: '/opt/outils/verifier-tests 04-authentification-permissions roles_et_permissions'
    solution:
      - write:
          agenda/permissions.py: |
            from django.conf import settings
            from rest_framework.permissions import SAFE_METHODS, BasePermission


            def roles_du_jeton(request):
                claims = request.auth if isinstance(request.auth, dict) else {}
                acces = claims.get("resource_access", {}).get(settings.OIDC_CLIENT_ID, {})
                return set(acces.get("roles", []))


            class HasRole(BasePermission):
                required_roles = []

                def has_permission(self, request, view):
                    return set(self.required_roles) <= roles_du_jeton(request)


            class HasRoleStaff(HasRole):
                required_roles = ["staff"]


            class HasRoleStaffOrReadOnly(HasRoleStaff):
                def has_permission(self, request, view):
                    return request.method in SAFE_METHODS or super().has_permission(request, view)
  - text: 'Dans `projet/settings.py`, remplis `REST_FRAMEWORK` : `DEFAULT_AUTHENTICATION_CLASSES` avec `"agenda.auth.KeycloakJWTAuthentication"` et `DEFAULT_PERMISSION_CLASSES` avec `IsAuthenticated`. Sans jeton valide, l''API doit répondre `401` partout. `test_api_fermee_par_defaut` doit passer'
    hint: 'Les deux réglages sont des listes de chemins en texte, par exemple `["rest_framework.permissions.IsAuthenticated"]`.'
    after: [1, 2]
    checks:
      - command-succeeds: '/opt/outils/verifier-tests 04-authentification-permissions api_fermee'
    solution:
      - write:
          projet/settings.py: |
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
            }
  - text: 'Dans `EvenementViewSet` (`agenda/views.py`), ajoute `permission_classes = [IsAuthenticated, HasRoleStaffOrReadOnly]`, et réserve l''action `fermer` au rôle `staff` avec `permission_classes=[HasRoleStaff]`. Sans le rôle, un `POST` doit donner `403` et ne rien créer. `test_ecriture_reservee_au_role_staff` doit passer'
    hint: 'Importe `IsAuthenticated` (depuis `rest_framework.permissions`) et tes classes (depuis `.permissions`). La liste de la vue remplace le réglage par défaut : garde `IsAuthenticated` dedans.'
    after: [3, 4]
    checks:
      - command-succeeds: '/opt/outils/verifier-tests 04-authentification-permissions ecriture_reservee'
    solution:
      - write:
          agenda/views.py: |-
            from django.utils import timezone
            from rest_framework import viewsets
            from rest_framework.decorators import action
            from rest_framework.permissions import IsAuthenticated
            from rest_framework.response import Response

            from .models import Asso, Evenement
            from .permissions import HasRoleStaff, HasRoleStaffOrReadOnly
            from .serializers import AssoSerializer, EvenementSerializer


            class AssoViewSet(viewsets.ReadOnlyModelViewSet):
                queryset = Asso.objects.prefetch_related("evenements").order_by("nom")
                serializer_class = AssoSerializer


            class EvenementViewSet(viewsets.ModelViewSet):
                queryset = Evenement.objects.select_related("asso").order_by("date")
                serializer_class = EvenementSerializer
                permission_classes = [IsAuthenticated, HasRoleStaffOrReadOnly]

                def get_queryset(self):
                    queryset = super().get_queryset()
                    if self.request.query_params.get("a_venir") == "true":
                        queryset = queryset.filter(date__gte=timezone.now())
                    return queryset

                @action(detail=True, methods=["post"], permission_classes=[HasRoleStaff])
                def fermer(self, request, pk=None):
                    """Ferme les inscriptions à l'événement."""
                    evenement = self.get_object()
                    evenement.ouvert = False
                    evenement.save(update_fields=["ouvert"])
                    return Response(self.get_serializer(evenement).data)
:::

## Vérifie tes acquis

:::quiz
Un client envoie un jeton expiré. Quel code de réponse attends-tu avec la classe d'authentification de la leçon ?

- [ ] `403`, car le client n'a pas le droit
- [ ] `200`, car l'URL existe
- [x] `401`, car l'authentification échoue
- [ ] `404`, car le jeton est introuvable

> Un jeton invalide ou expiré fait lever `AuthenticationFailed`, donc `401`. `403` est réservé aux personnes identifiées qui n'ont pas le droit.
:::

:::quiz
Que contient `request.auth` avec la classe d'authentification de la leçon ?

- [ ] L'objet utilisateur Django
- [x] Les claims décodés du jeton
- [ ] Le mot de passe de la personne
- [ ] La liste des permissions de la vue

> Le deuxième élément du couple renvoyé par `authenticate()` alimente `request.auth` ; l'utilisateur va dans `request.user`.
:::

:::quiz
Que se passe-t-il si tu déclares `permission_classes = [HasRoleStaffOrReadOnly]` sans `IsAuthenticated` ?

- [ ] L'API refuse toutes les requêtes
- [ ] Le réglage par défaut s'ajoute à ta liste
- [ ] Seules les personnes ayant le rôle `staff` peuvent lire
- [x] Les lectures (`GET`) deviennent possibles sans jeton

> La liste de la vue remplace les permissions par défaut, et la classe autorise les méthodes de lecture à tout le monde.
:::

:::quiz
Quelle valeur de `DEFAULT_PERMISSION_CLASSES` est le plus sûre pour démarrer ?

- [ ] `AllowAny`, qui est aussi le réglage par défaut de DRF
- [x] `IsAuthenticated`, quitte à ouvrir ensuite des exceptions explicites
- [ ] Aucune, pour que chaque vue décide
- [ ] `SAFE_METHODS`

> Mieux vaut fermer par défaut et ouvrir ensuite. `SAFE_METHODS` n'est pas une classe de permission mais une liste de verbes.
:::
