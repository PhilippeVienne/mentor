---
id: documentation
title: "Documenter son API"
summary: "Docstrings, schéma OpenAPI, page Swagger et document partagé : donner aux autres de quoi utiliser ton API."
minutes: 35
objectives:
  - Expliquer pourquoi et pour qui on documente une API
  - Écrire des docstrings qui alimentent la documentation
  - Générer un schéma OpenAPI avec `generateschema`
  - Rédiger la documentation d'un point d'accès pour des personnes extérieures et la garder à jour
---

Tu as construit une API. Une autre association veut s'y connecter : quelle URL appeler ? avec quel jeton ? quel rôle faut-il ? que renvoie-t-elle ? Sans documentation, elle n'a que deux solutions : te poser toutes ces questions, ou lire ton code. Documenter, c'est répondre **une fois** à ces questions pour toutes les personnes qui viendront.

## Trois niveaux de documentation

| Niveau | Pour qui | Comment |
| --- | --- | --- |
| Docstrings dans le code | Les développeur·se·s du projet | Texte entre triples guillemets, affiché par DRF |
| Schéma / page interactive | Les développeur·se·s des applications clientes | Généré automatiquement depuis le code |
| Document partagé | Les partenaires extérieurs | Rédigé à la main, court et précis |

## Les docstrings

Une **docstring** est le texte entre triples guillemets placé juste sous la ligne `class` ou `def`. DRF la reprend comme description de la vue, à la fois dans la page web de l'API navigable et dans le schéma généré.

```python
class AssoViewSet(viewsets.ReadOnlyModelViewSet):
    """Associations et leurs événements."""

    queryset = Asso.objects.prefetch_related("evenements").order_by("nom")
    serializer_class = AssoSerializer
```

Au passage, `prefetch_related("evenements")` charge en une seule requête supplémentaire les événements de toutes les associations (sans lui, Django ferait une requête par association) : c'est le pendant de `select_related` pour les relations « un vers plusieurs ». Sans docstring, la description est vide. Les vues d'Adhésion en contiennent beaucoup, par exemple celle de `CardTypesViewset` (« Renvoie les différents types de cartes. Pour énumerer les types dispos pour un membre, ajouter ?member=id ») ou la docstring détaillée de `va_check` qui liste ses paramètres.

Une requête `OPTIONS` sur une URL (le verbe HTTP qui demande à quoi sert une adresse et ce qu'on peut y faire) renvoie aussi cette description et les formats acceptés :

```console
OPTIONS /v1/assos/ -> 200
Allow: GET, HEAD, OPTIONS
{"name": "Asso List", "description": "Associations et leurs événements.", "renders": ["application/json", "text/html"], ...}
```

## Le schéma OpenAPI

**OpenAPI** est un format standard pour décrire une API (ses URL, verbes, paramètres, réponses). À partir de ce format, des outils fabriquent des pages interactives où l'on peut essayer chaque appel (**Swagger UI** est la plus connue : une page web générée depuis le schéma, qui décrit chaque point d'accès et permet de le tester en un clic), ou génèrent des clients dans d'autres langages. DRF sait produire le schéma avec une commande :

```bash
python manage.py generateschema --title "API Agenda" --api_version 1.0.0 > schema.yml
```

Ligne à ligne : `generateschema` est la commande de gestion de DRF ; `--title` et `--api_version` renseignent l'en-tête du document ; `> schema.yml` enregistre la sortie dans un fichier. Un fichier `.yml` est écrit en **YAML**, un format texte proche du JSON mais sans accolades, où l'indentation montre l'imbrication.

Voici un **extrait** du fichier obtenu. La vraie sortie est bien plus longue : elle commence par `/v1/evenements/` (les chemins suivent l'ordre d'enregistrement au routeur), détaille chaque verbe, ses paramètres et ses réponses, et écrit les lettres accentuées de façon échappée (`\xE9` pour « é »).

```yaml
openapi: 3.0.2
info:
  title: API Agenda
  version: 1.0.0
paths:
  # ... extrait : dans le vrai fichier, le premier chemin est /v1/evenements/
  /v1/assos/:
    get:
      operationId: listAssos
      description: "Associations et leurs \xE9v\xE9nements."
```

Dans l'extrait, la description reprend la docstring de la vue. Selon la version, la commande demande des paquets facultatifs (`pyyaml`, `uritemplate`, `inflection`) : l'erreur t'indique celui qui manque (dans les labos, ils sont déjà installés).

:::warning La commande plante dès qu'un filtre django-filter est actif
Avec les versions des labos (DRF 3.18 et `django-filter` 26), la commande s'arrête sur `AttributeError: 'DjangoFilterBackend' object has no attribute 'get_schema_operation_parameters'` dès qu'une vue déclare `DjangoFilterBackend` dans `filter_backends` : DRF demande à chaque filtre de décrire ses paramètres, et celui-ci ne sait pas le faire. Le contournement est une petite sous-classe d'`AutoSchema` qui ignore les filtres sans cette méthode :

```python
# agenda/schema.py
from rest_framework.schemas.openapi import AutoSchema


class SchemaAgenda(AutoSchema):
    def get_filter_parameters(self, path, method):
        if not self.allows_filters(path, method):
            return []
        parametres = []
        for backend in self.view.filter_backends:
            if hasattr(backend, "get_schema_operation_parameters"):
                parametres += backend().get_schema_operation_parameters(self.view)
        return parametres
```

Puis, dans `settings.py`, `REST_FRAMEWORK = {..., "DEFAULT_SCHEMA_CLASS": "agenda.schema.SchemaAgenda"}`. Le schéma décrit alors la pagination, la recherche et le tri, mais pas les filtres exacts (`?ouvert=` et `?asso=`) : signale-les dans la documentation que tu écris à la main. Si tu utilises une autre version de `django-filter`, essaie d'abord sans contournement.
:::

On peut aussi servir le schéma par une URL avec `get_schema_view` (dans `rest_framework.schemas`), puis le brancher sur une page Swagger.

:::info Et dans l'équipe ?
Adhésion déclare `django-rest-swagger==2.2.0` et expose une page `/swagger` avec `get_swagger_view(title="Adhesion API")`. Cette bibliothèque repose sur un ancien système de schéma de DRF et n'est plus la voie recommandée : pour un nouveau projet, pars plutôt du schéma OpenAPI de DRF. Dans tous les cas, vérifie la compatibilité avec ta version de DRF.
:::

## Le document pour les personnes extérieures

Un schéma complet est utile à qui développe. Pour un partenaire qui veut juste appeler un point d'accès, un court document vaut mieux. Adhésion en contient un dans son dépôt, `API_DOC_EXTERN.md`, dont l'introduction dit qu'une documentation complète et automatique est possible, mais qu'il est plus simple de partager ce document aux utilisateurs extérieurs.

Pour chaque point d'accès, il donne :

- l'**URL** (`https://api-adhesion.example.org/va_check`) ;
- la **protection** : comment obtenir les identifiants du client Keycloak ;
- le **rôle Keycloak** requis ;
- les **méthodes** autorisées et le `Content-Type` ;
- un **exemple de requête** et un **exemple de réponse**.

Voici le même modèle pour notre API fictive :

```console
Point d'accès : POST /v1/evenements/
Protection    : jeton Bearer (Keycloak), rôle « staff »
Méthodes      : POST, OPTIONS      Content-Type : application/json
```

```json
{ "asso": "Ciné-club", "titre": "Ciné-débat", "date": "2099-01-01T10:00:00Z", "places": 40 }
```

Réponse `201` :

```json
{
  "id": 2,
  "asso": "Ciné-club",
  "titre": "Ciné-débat",
  "date": "2099-01-01T11:00:00+01:00",
  "places": 40,
  "places_restantes": 40,
  "ouvert": true
}
```

:::warning Une documentation écrite à la main vieillit
En relisant le code actuel d'Adhésion, on constate un écart : le document externe décrit `/va_check` comme un `POST` avec un corps JSON et le rôle `24h`, alors que la vue `va_check` du dépôt répond à des `GET` avec des paramètres d'URL et exige le rôle `vachecker` (et le document décrit aussi un point `/24h` qu'on ne retrouve pas dans `urls.py`). Quand tu changes un comportement, mets la documentation à jour **dans la même merge request** (la demande de fusion GitLab : une proposition de modification du code, relue par l'équipe avant d'être intégrée), ou génère-la depuis le code.
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  L'API des événements fonctionne, mais elle n'est pas documentée : ses vues n'ont aucune docstring, et la commande `generateschema` plante. Tu vas ajouter les docstrings, contourner le plantage, produire le schéma OpenAPI et rédiger la fiche d'un point d'accès pour un partenaire. Les deux premières étapes sont vérifiées par `pytest -q test_documentation.py` ; les suivantes par des commandes et par le contenu des fichiers que tu crées.
commands:
  - cp -R /opt/exercices/base/. .
  - cp -R /opt/exercices/06-documentation/. .
steps:
  - text: 'Ajoute à `AssoViewSet` la docstring `"""Associations et leurs événements."""`, juste sous la ligne `class`. Une requête `OPTIONS /v1/assos/` doit la renvoyer comme description : `test_docstring_des_assos` doit passer'
    hint: 'La docstring est la première chose dans le corps de la classe, entre triples guillemets. Elle s''écrit avant `queryset`.'
    checks:
      - command-succeeds: '/opt/outils/verifier-tests 06-documentation docstring_des_assos'
    solution:
      - write:
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
                """Associations et leurs événements."""

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
  - text: 'Ajoute une docstring à `EvenementViewSet` et à son action `fermer`. `test_docstrings_des_evenements` doit passer'
    hint: 'Pour la méthode `fermer`, la docstring se place juste sous la ligne `def`, avant le code.'
    after: [1]
    checks:
      - command-succeeds: '/opt/outils/verifier-tests 06-documentation docstrings_des_evenements'
    solution:
      - write:
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
                """Associations et leurs événements."""

                queryset = Asso.objects.prefetch_related("evenements").order_by("nom")
                serializer_class = AssoSerializer


            class EvenementViewSet(viewsets.ModelViewSet):
                """Événements organisés par les associations."""

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
  - text: 'Lance `python manage.py generateschema --title "API Agenda" --api_version 1.0.0` : la commande plante (`AttributeError ... get_schema_operation_parameters`). Crée `agenda/schema.py` avec une sous-classe d''`AutoSchema` qui ignore les filtres sans cette méthode, et déclare-la avec `DEFAULT_SCHEMA_CLASS` dans `REST_FRAMEWORK`. La commande doit alors réussir'
    hint: 'Le code de la sous-classe est donné dans la leçon (encart sur `DjangoFilterBackend`). Le réglage se met dans `projet/settings.py` : `"DEFAULT_SCHEMA_CLASS": "agenda.schema.SchemaAgenda"`.'
    checks:
      - command-succeeds: '/opt/outils/verifier-doc-api schema'
    solution:
      - write:
          agenda/schema.py: |
            from rest_framework.schemas.openapi import AutoSchema


            class SchemaAgenda(AutoSchema):
                """Contournement : DjangoFilterBackend n'a pas get_schema_operation_parameters avec DRF 3.18."""

                def get_filter_parameters(self, path, method):
                    if not self.allows_filters(path, method):
                        return []
                    parametres = []
                    for backend in self.view.filter_backends:
                        if hasattr(backend, "get_schema_operation_parameters"):
                            parametres += backend().get_schema_operation_parameters(self.view)
                    return parametres
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
                "DEFAULT_PAGINATION_CLASS": "rest_framework.pagination.PageNumberPagination",
                "PAGE_SIZE": 20,
                "DEFAULT_SCHEMA_CLASS": "agenda.schema.SchemaAgenda",
            }
  - text: 'Génère le fichier `schema.yml` avec `python manage.py generateschema --title "API Agenda" --api_version 1.0.0 > schema.yml`, puis ouvre-le : il décrit `/v1/evenements/` et reprend la docstring des associations'
    hint: 'La commande est celle de l''étape précédente, avec `> schema.yml` à la fin pour enregistrer la sortie dans un fichier. Regarde-le avec `less schema.yml` (touche `q` pour quitter).'
    after: [1, 3]
    checks:
      - command-succeeds: '/opt/outils/verifier-doc-api schema-fichier'
    solution:
      - 'python manage.py generateschema --title "API Agenda" --api_version 1.0.0 > schema.yml'
  - text: 'Rédige `API_DOC_EXTERN.md`, la fiche du point d''accès `POST /v1/evenements/` pour un partenaire : l''URL et la méthode, la protection (jeton `Bearer`), le rôle requis (`staff`), le `Content-Type`, un exemple de requête et la réponse `201`'
    hint: 'Reprends le modèle de la leçon : une ligne « Point d''accès », une ligne « Protection », une ligne « Méthodes », puis un exemple de requête JSON et un exemple de réponse.'
    checks:
      - command-succeeds: '/opt/outils/verifier-doc-api fiche'
    solution:
      - write:
          API_DOC_EXTERN.md: |-
            # API Agenda : documentation pour les partenaires

            ## Créer un événement

            Point d'accès : POST /v1/evenements/
            Protection    : jeton Bearer (Keycloak), rôle « staff »
            Méthodes      : POST, OPTIONS      Content-Type : application/json

            Exemple de requête :

            ```json
            { "asso": "Ciné-club", "titre": "Ciné-débat", "date": "2099-01-01T10:00:00Z", "places": 40 }
            ```

            Réponse 201 (créé) :

            ```json
            { "id": 2, "asso": "Ciné-club", "titre": "Ciné-débat", "places": 40, "places_restantes": 40, "ouvert": true }
            ```
:::

## Vérifie tes acquis

:::quiz
D'où DRF tire-t-il la description d'une vue dans la documentation générée ?

- [ ] Du nom du fichier
- [ ] Du message de commit
- [x] De la docstring de la classe ou de la méthode
- [ ] Du nom de la base de données

> DRF réutilise la docstring comme description, dans la page navigable comme dans le schéma.
:::

:::quiz
À quoi sert un schéma OpenAPI ?

- [x] À décrire l'API dans un format standard exploitable par des outils (pages interactives, génération de clients)
- [ ] À accélérer les réponses de l'API
- [ ] À remplacer les permissions
- [ ] À stocker les jetons

> OpenAPI décrit l'interface : il ne change ni les performances ni la sécurité.
:::

:::quiz
Qu'est-ce qu'un document partagé comme `API_DOC_EXTERN.md` doit absolument indiquer pour qu'un partenaire appelle un point d'accès ?

- [ ] La liste de tous les modèles Django
- [ ] Le mot de passe de la base de données
- [x] L'URL, la protection et le rôle requis, les méthodes, et un exemple de requête et de réponse
- [ ] L'historique complet des migrations

> Le partenaire a besoin de ce qu'il faut pour appeler, pas de l'intérieur du projet.
:::

:::quiz
Quel est le principal risque d'une documentation rédigée à la main ?

- [ ] Elle est illisible
- [ ] Elle ralentit l'API
- [x] Elle peut ne plus correspondre au code après une modification
- [ ] Elle est interdite en production

> Sans mise à jour conjointe avec le code, elle devient trompeuse.
:::
