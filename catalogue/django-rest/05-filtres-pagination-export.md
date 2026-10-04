---
id: filtres-pagination-export
titre: "Filtres, pagination, import et export"
resume: "Chercher, trier et découper une liste en pages ; exporter en CSV ; importer des fichiers via l'admin."
duree: 45
objectifs:
  - Utiliser les paramètres d'URL (`?search=`, `?ordering=`, `?page=`) pour interroger une liste
  - Configurer `SearchFilter`, `OrderingFilter` et `DjangoFilterBackend`
  - Paginer une liste avec `PageNumberPagination`
  - Exporter en CSV et situer `django-import-export`
---

Avec quelques milliers d'adhérent·e·s, `GET /v1/members/` ne peut pas renvoyer toute la table en une réponse : ce serait lent et inutile. Le client veut une page à la fois, et la possibilité de chercher « Martin » ou de trier par date. Tout cela passe par des **paramètres d'URL** (*query parameters*), la partie après le `?` : `/v1/evenements/?search=ciné&ordering=-date&page=2`.

## Les backends de filtrage

DRF traite ces paramètres avec des **backends de filtrage**, des classes que tu listes dans `filter_backends`. Les trois courants :

| Backend | Paramètre | Rôle |
| --- | --- | --- |
| `SearchFilter` | `?search=texte` | Recherche textuelle dans les champs de `search_fields` |
| `OrderingFilter` | `?ordering=date` ou `-date` | Tri (le `-` inverse l'ordre) |
| `DjangoFilterBackend` | `?ouvert=true&asso=3` | Filtre exact sur les champs de `filterset_fields` |

Le dernier vient du paquet `django-filter` (à installer, et à ajouter `"django_filters"` à `INSTALLED_APPS`) ; les deux premiers sont dans DRF.

```python
from django_filters.rest_framework import DjangoFilterBackend
from rest_framework import filters, viewsets


class EvenementViewSet(viewsets.ModelViewSet):
    queryset = Evenement.objects.select_related("asso").order_by("date")
    serializer_class = EvenementSerializer
    filter_backends = [filters.SearchFilter, filters.OrderingFilter, DjangoFilterBackend]
    search_fields = ["titre", "^asso__nom"]
    ordering_fields = ["date", "places"]
    filterset_fields = ["ouvert", "asso"]
```

- `search_fields` : les champs inspectés par `?search=`. Un préfixe change la façon de comparer : `^` = commence par, `=` = égal exactement, `$` = expression régulière (un motif de recherche avancé), rien = contient. Ici, `titre` contient le texte et `asso__nom` (le nom de l'association, avec la notation Django `__` pour traverser une relation) commence par le texte.
- `ordering_fields` : les champs sur lesquels le client peut trier. Liste-les explicitement : trier sur n'importe quel champ pourrait être lent ou révéler des données.
- `filterset_fields` : les champs filtrables par égalité.

Adhésion combine `SearchFilter` et `DjangoFilterBackend` sur ses membres, avec `search_fields = ("$first_name", "$last_name", "^email", "=student_profile__student_number", "=cards__code")` ; PlanningAPI utilise `OrderingFilter` avec `ordering = ['debut']` pour ses créneaux.

:::info Vieux nom d'attribut dans les anciennes versions
Adhésion écrit `filter_fields`, le nom historique utilisé avec `django-filter==2.4.0`. Les versions récentes de `django-filter` attendent `filterset_fields`. Si ton filtre est silencieusement ignoré, vérifie le nom attendu par **ta** version.
:::

Pour un filtre sur mesure (« événements à venir »), la méthode `get_queryset()` de la leçon 3 reste le plus simple.

## Paginer

La **pagination** découpe une liste en pages. Sans elle, une liste de 10 000 lignes est renvoyée d'un bloc. On la règle dans `settings.py` :

```python
REST_FRAMEWORK = {
    "DEFAULT_PAGINATION_CLASS": "rest_framework.pagination.PageNumberPagination",
    "PAGE_SIZE": 20,
}
```

La réponse change alors de forme : la liste est enveloppée avec des informations de navigation.

```json
{
  "count": 2,
  "next": "http://testserver/v1/evenements/?page=2&page_size=1",
  "previous": null,
  "results": [
    { "id": 1, "asso": "Ciné-club", "titre": "Soirée courts-métrages", "...": "..." }
  ]
}
```

`count` est le nombre total d'éléments, `next` et `previous` sont les liens vers les pages voisines, `results` la page courante. Demander une page inexistante (`?page=9`) renvoie `404` avec `{"detail": "Page non valide."}`.

Pour laisser le client choisir la taille de page, on écrit une classe de pagination dédiée :

```python
from rest_framework.pagination import PageNumberPagination


class EvenementsPagination(PageNumberPagination):
    page_size = 20
    page_size_query_param = "page_size"
    max_page_size = 100


class EvenementViewSet(viewsets.ModelViewSet):
    pagination_class = EvenementsPagination
    # ...
```

- `page_size` : taille par défaut ;
- `page_size_query_param` : nom du paramètre qui permet de la changer (`?page_size=50`) ;
- `max_page_size` : plafond, pour qu'un client ne demande pas 1 000 000 de lignes.

C'est le schéma des classes `MembersPagination`, `CardsPagination` et `MembershipsPagination` d'Adhésion (20, 100 et 20 éléments par défaut ; maximum 100, 3 500 et 100). `MembersPagination` redéfinit aussi `get_paginated_response` pour renvoyer ses propres clés (`page`, `pages`, `page_size`…). Le réglage global d'Adhésion, lui, est `LimitOffsetPagination` (`?limit=` et `?offset=`).

:::tip Toujours un ordre stable
Paginer une liste non triée peut répéter ou sauter des éléments d'une page à l'autre, et Django émet un avertissement `UnorderedObjectListWarning`. Ajoute un `order_by(...)` à ton `queryset`.
:::

## Exporter en CSV

Un **CSV** est un fichier texte où chaque ligne est un enregistrement et les valeurs sont séparées par des virgules : il s'ouvre dans un tableur. L'API d'Adhésion expose `/v1/export_members/` qui renvoie un CSV (colonnes choisies par `?fields=`) et est réservé au rôle `staff`. Voici l'équivalent pour nos événements, en action de viewset :

```python
import csv

from django.http import HttpResponse


class EvenementViewSet(viewsets.ModelViewSet):
    # ... autres attributs ...

    @action(detail=False, methods=["get"], permission_classes=[HasRoleStaff])
    def export(self, request):
        reponse = HttpResponse(content_type="text/csv")
        reponse["Content-Disposition"] = 'attachment; filename="evenements.csv"'
        writer = csv.writer(reponse)
        writer.writerow(["titre", "asso", "date"])
        for e in self.filter_queryset(self.get_queryset()):
            writer.writerow([e.titre, e.asso.nom, e.date.isoformat()])
        return reponse
```

- `HttpResponse(content_type="text/csv")` : on sort du JSON, donc on renvoie une réponse Django classique.
- `Content-Disposition: attachment` fait télécharger le fichier avec ce nom.
- `csv.writer(reponse)` écrit directement dans la réponse, qui se comporte comme un fichier.
- `self.filter_queryset(...)` applique les filtres de l'URL : `?ouvert=true` filtre aussi l'export.

```console
titre,asso,date
Soirée courts-métrages,Ciné-club,2099-03-12T17:00:00+00:00
```

## Import et export dans l'admin : `django-import-export`

Adhésion (`django-import-export==2.5.0`) et PlanningAPI l'utilisent côté **interface d'administration Django**, pas côté API : elle ajoute des boutons « Importer » et « Exporter » (CSV, Excel…) à une page d'admin.

```python
from import_export import resources
from import_export.admin import ImportExportModelAdmin


class EvenementResource(resources.ModelResource):
    class Meta:
        model = Evenement
        fields = ("id", "asso", "titre", "date", "places")


@admin.register(Evenement)
class EvenementAdmin(ImportExportModelAdmin):
    resource_class = EvenementResource
```

La `Resource` décrit les colonnes du fichier, comme un sérialiseur décrit le JSON. Ajoute `"import_export"` à `INSTALLED_APPS`.

## Entraîne-toi

:::labo
moteur: reel
intro: |
  L'API des événements est protégée (les tests se connectent avec le rôle `staff` ou sans rôle) mais elle ne sait ni chercher, ni trier, ni paginer, ni exporter. Tu complètes `agenda/views.py`, `projet/settings.py` et un nouveau fichier `agenda/pagination.py`. Lance `pytest -q test_filtres.py` pour voir les cinq tests échouer, puis corrige-les dans l'ordre.
commandes:
  - cp -R /opt/exercices/base/. .
  - cp -R /opt/exercices/05-filtres-pagination-export/. .
etapes:
  - texte: 'Dans `EvenementViewSet`, active la recherche et le tri : `filter_backends = [filters.SearchFilter, filters.OrderingFilter]`, avec `search_fields` (`titre` et le début de `asso__nom`) et `ordering_fields` (`date` et `places`). `?search=Ciné` et `?ordering=-date` doivent fonctionner : `test_recherche_et_tri` doit passer'
    indice: 'Un préfixe `^` devant un champ de `search_fields` veut dire « commence par ». Il faut importer `filters` depuis `rest_framework`.'
    verif:
      - commande-reussit: '/opt/outils/verifier-tests 05-filtres-pagination-export recherche_et_tri'
    solution:
      - ecrire:
          agenda/views.py: |
            from django.utils import timezone
            from rest_framework import filters, viewsets
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
                filter_backends = [filters.SearchFilter, filters.OrderingFilter]
                search_fields = ["titre", "^asso__nom"]
                ordering_fields = ["date", "places"]

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
  - texte: 'Ajoute le filtre exact `DjangoFilterBackend` (importé de `django_filters.rest_framework`) à `filter_backends`, avec `filterset_fields = ["ouvert", "asso"]` : `?ouvert=false` et `?asso=<id>` doivent filtrer. `test_filtres_exacts` doit passer'
    indice: '`DjangoFilterBackend` se met dans la même liste que les deux autres filtres. Les champs filtrables se déclarent dans `filterset_fields`.'
    apres: [1]
    verif:
      - commande-reussit: '/opt/outils/verifier-tests 05-filtres-pagination-export filtres_exacts'
    solution:
      - ecrire:
          agenda/views.py: |
            from django.utils import timezone
            from django_filters.rest_framework import DjangoFilterBackend
            from rest_framework import filters, viewsets
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
                filter_backends = [filters.SearchFilter, filters.OrderingFilter, DjangoFilterBackend]
                search_fields = ["titre", "^asso__nom"]
                ordering_fields = ["date", "places"]
                filterset_fields = ["ouvert", "asso"]

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
  - texte: 'Dans `projet/settings.py`, active la pagination pour toute l''API : `DEFAULT_PAGINATION_CLASS` (`rest_framework.pagination.PageNumberPagination`) et `PAGE_SIZE` à `20`. La réponse devient un objet avec `count`, `next`, `previous` et `results`. `test_pagination_globale` doit passer'
    indice: 'Ajoute deux clés au dictionnaire `REST_FRAMEWORK`. Une page qui n''existe pas (`?page=9`) répond `404`.'
    verif:
      - commande-reussit: '/opt/outils/verifier-tests 05-filtres-pagination-export pagination_globale'
    solution:
      - ecrire:
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
                "DEFAULT_PAGINATION_CLASS": "rest_framework.pagination.PageNumberPagination",
                "PAGE_SIZE": 20,
            }
  - texte: 'Crée `agenda/pagination.py` avec la classe `EvenementsPagination` (`page_size = 20`, `page_size_query_param = "page_size"`, `max_page_size = 100`) et branche-la sur la vue avec `pagination_class`. `?page_size=1` doit alors renvoyer un seul événement. `test_taille_de_page` doit passer'
    indice: 'La classe hérite de `PageNumberPagination` (dans `rest_framework.pagination`). Dans la vue : `pagination_class = EvenementsPagination`, sans oublier l''import.'
    apres: [2]
    verif:
      - commande-reussit: '/opt/outils/verifier-tests 05-filtres-pagination-export taille_de_page'
    solution:
      - ecrire:
          agenda/pagination.py: |
            from rest_framework.pagination import PageNumberPagination


            class EvenementsPagination(PageNumberPagination):
                page_size = 20
                page_size_query_param = "page_size"
                max_page_size = 100
      - ecrire:
          agenda/views.py: |
            from django.utils import timezone
            from django_filters.rest_framework import DjangoFilterBackend
            from rest_framework import filters, viewsets
            from rest_framework.decorators import action
            from rest_framework.permissions import IsAuthenticated
            from rest_framework.response import Response

            from .models import Asso, Evenement
            from .pagination import EvenementsPagination
            from .permissions import HasRoleStaff, HasRoleStaffOrReadOnly
            from .serializers import AssoSerializer, EvenementSerializer


            class AssoViewSet(viewsets.ReadOnlyModelViewSet):
                queryset = Asso.objects.prefetch_related("evenements").order_by("nom")
                serializer_class = AssoSerializer


            class EvenementViewSet(viewsets.ModelViewSet):
                queryset = Evenement.objects.select_related("asso").order_by("date")
                serializer_class = EvenementSerializer
                permission_classes = [IsAuthenticated, HasRoleStaffOrReadOnly]
                pagination_class = EvenementsPagination
                filter_backends = [filters.SearchFilter, filters.OrderingFilter, DjangoFilterBackend]
                search_fields = ["titre", "^asso__nom"]
                ordering_fields = ["date", "places"]
                filterset_fields = ["ouvert", "asso"]

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
  - texte: 'Ajoute l''action `export` (`detail=False`, `GET`, réservée à `HasRoleStaff`) qui renvoie un fichier CSV `titre,asso,date` des événements **filtrés** par l''URL. `test_export_csv` doit passer'
    indice: 'Une `HttpResponse(content_type="text/csv")` se remplit avec `csv.writer(reponse)`. Parcours `self.filter_queryset(self.get_queryset())` pour que `?ouvert=true` s''applique aussi à l''export.'
    apres: [4]
    verif:
      - commande-reussit: '/opt/outils/verifier-tests 05-filtres-pagination-export export_csv'
    solution:
      - ecrire:
          agenda/views.py: |
            import csv

            from django.http import HttpResponse
            from django.utils import timezone
            from django_filters.rest_framework import DjangoFilterBackend
            from rest_framework import filters, viewsets
            from rest_framework.decorators import action
            from rest_framework.permissions import IsAuthenticated
            from rest_framework.response import Response

            from .models import Asso, Evenement
            from .pagination import EvenementsPagination
            from .permissions import HasRoleStaff, HasRoleStaffOrReadOnly
            from .serializers import AssoSerializer, EvenementSerializer


            class AssoViewSet(viewsets.ReadOnlyModelViewSet):
                queryset = Asso.objects.prefetch_related("evenements").order_by("nom")
                serializer_class = AssoSerializer


            class EvenementViewSet(viewsets.ModelViewSet):
                queryset = Evenement.objects.select_related("asso").order_by("date")
                serializer_class = EvenementSerializer
                permission_classes = [IsAuthenticated, HasRoleStaffOrReadOnly]
                pagination_class = EvenementsPagination
                filter_backends = [filters.SearchFilter, filters.OrderingFilter, DjangoFilterBackend]
                search_fields = ["titre", "^asso__nom"]
                ordering_fields = ["date", "places"]
                filterset_fields = ["ouvert", "asso"]

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

                @action(detail=False, methods=["get"], permission_classes=[HasRoleStaff])
                def export(self, request):
                    reponse = HttpResponse(content_type="text/csv")
                    reponse["Content-Disposition"] = 'attachment; filename="evenements.csv"'
                    writer = csv.writer(reponse)
                    writer.writerow(["titre", "asso", "date"])
                    for e in self.filter_queryset(self.get_queryset()):
                        writer.writerow([e.titre, e.asso.nom, e.date.isoformat()])
                    return reponse
:::

## Vérifie tes acquis

:::quiz
Comment un client trie-t-il les événements du plus récent au plus ancien avec `OrderingFilter` ?

- [ ] `?order=date`
- [ ] `?sort=desc`
- [x] `?ordering=-date`
- [ ] `?ordering=date&reverse=true`

> Le paramètre s'appelle `ordering` et le préfixe `-` inverse le tri. Le champ doit figurer dans `ordering_fields`.
:::

:::quiz
Que signifie le préfixe `^` dans `search_fields = ["^asso__nom"]` ?

- [ ] Correspondance exacte
- [ ] Expression régulière
- [x] Le champ commence par le texte recherché
- [ ] Recherche insensible aux accents

> `^` = commence par, `=` = exact, `$` = expression régulière. Sans préfixe, c'est « contient ».
:::

:::quiz
Quel est l'intérêt de `max_page_size` ?

- [ ] Imposer la même taille à tous les clients
- [ ] Désactiver la pagination
- [ ] Trier les résultats
- [x] Empêcher un client de demander une page démesurée

> Avec `page_size_query_param`, le client choisit la taille, mais pas au-delà du plafond.
:::

:::quiz
Dans Adhésion, où `django-import-export` est-il branché ?

- [ ] Dans les viewsets de l'API
- [x] Dans l'interface d'administration Django
- [ ] Dans le routeur
- [ ] Dans les classes de pagination

> Il fournit des boutons d'import et d'export dans l'admin. L'export CSV de l'API passe par une vue dédiée.
:::
