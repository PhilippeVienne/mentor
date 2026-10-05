## Les verbes et les codes

| Verbe | Action viewset | Succès |
| --- | --- | :---: |
| `GET /ressources/` | `list` | `200` |
| `POST /ressources/` | `create` | `201` |
| `GET /ressources/1/` | `retrieve` | `200` |
| `PUT` / `PATCH /ressources/1/` | `update` / `partial_update` | `200` |
| `DELETE /ressources/1/` | `destroy` | `204` |

Erreurs : `400` données invalides · `401` pas identifié·e · `403` pas autorisé·e · `404` introuvable.

## Sérialiseur

```python
class EvenementSerializer(serializers.ModelSerializer):
    asso = serializers.SlugRelatedField(slug_field="nom", queryset=Asso.objects.all())
    places_restantes = serializers.SerializerMethodField()

    class Meta:
        model = Evenement
        fields = ("id", "asso", "titre", "places_restantes")
        read_only_fields = ("id",)

    def get_places_restantes(self, obj): ...
    def validate_places(self, value): ...   # un champ
    def validate(self, data): ...           # plusieurs champs
```

`s = Serializer(instance)` puis `s.data` · `s = Serializer(data=...)` puis `s.is_valid()`, `s.errors`, `s.validated_data`, `s.save()`.

## Vues et routeur

```python
class EvenementViewSet(viewsets.ModelViewSet):   # ReadOnlyModelViewSet : lecture seule
    queryset = Evenement.objects.order_by("date")
    serializer_class = EvenementSerializer

    @action(detail=True, methods=["post"])
    def fermer(self, request, pk=None): ...

router = routers.DefaultRouter()
router.register("evenements", EvenementViewSet)
urlpatterns = [path("v1/", include(router.urls))]
```

## Authentification et permissions

```python
REST_FRAMEWORK = {
    "DEFAULT_AUTHENTICATION_CLASSES": ["agenda.auth.KeycloakJWTAuthentication"],
    "DEFAULT_PERMISSION_CLASSES": ["rest_framework.permissions.IsAuthenticated"],
}
```

- `permission_classes = [IsAuthenticated, HasRoleStaff]` : toutes doivent accepter, et la liste **remplace** le réglage par défaut.
- Une permission : `has_permission(request, view)` ; sur un objet : `has_object_permission(request, view, obj)`.
- `request.user` = l'utilisateur, `request.auth` = le jeton (ou ses claims). `SAFE_METHODS` = `GET`, `HEAD`, `OPTIONS`.

## Filtres et pagination

| Besoin | Réglage | Appel |
| --- | --- | --- |
| Recherche | `SearchFilter` + `search_fields` (`^` début, `=` exact, `$` regex) | `?search=ciné` |
| Tri | `OrderingFilter` + `ordering_fields` | `?ordering=-date` |
| Filtre exact | `DjangoFilterBackend` + `filterset_fields` | `?ouvert=true` |
| Pages | `PageNumberPagination` (`page_size`, `page_size_query_param`, `max_page_size`) | `?page=2&page_size=50` |

## Documentation

```bash
python manage.py generateschema --title "API Agenda" --api_version 1.0.0 > schema.yml
```

Une docstring de vue devient sa description. Si `generateschema` plante sur `DjangoFilterBackend` (DRF 3.18), ajoute une classe de schéma de contournement (leçon 6). Pour les partenaires : URL, protection, rôle, méthodes, exemples de requête et de réponse.

## Tests

```python
class EvenementApiTests(APITestCase):
    def test_creation(self):
        self.client.force_authenticate(user=self.alice, token=jeton_staff())
        reponse = self.client.post("/v1/evenements/", corps, format="json")
        self.assertEqual(reponse.status_code, 201)
```

`python manage.py test` ou, avec pytest-django, `pytest -q` · `self.client.credentials(HTTP_AUTHORIZATION="Bearer ...")` pour un vrai jeton.
