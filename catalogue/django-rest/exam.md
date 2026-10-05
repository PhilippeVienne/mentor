---
title: "Examen de validation — API REST avec Django REST framework"
draw: 5
pass_mark: 80
minutes: 10
shuffle: true
---

Cet examen valide les bases de Django REST framework : API et HTTP, sérialiseurs, viewsets et routeurs, authentification par jeton et permissions, filtres et pagination, documentation, tests. Les exemples reprennent le domaine fictif du parcours (`Asso`, `Evenement`, `Inscription`).

:::quiz
Quelle est la différence entre une réponse `401` et une réponse `403` ?

- [x] `401` : personne non identifiée ; `403` : identifiée mais refusée
- [ ] `401` : la ressource n'existe pas ; `403` : les données envoyées sont invalides
- [ ] `401` : erreur du serveur ; `403` : erreur du client
- [ ] Aucune, ce sont deux noms du même code

> `401` signifie que l'authentification manque ou échoue, `403` que la personne est connue mais refusée. `404` = introuvable, `400` = données invalides.
:::

:::quiz
Quel verbe HTTP sert à modifier seulement quelques champs d'une ressource existante ?

- [ ] `POST`
- [ ] `GET`
- [x] `PATCH`
- [ ] `DELETE`

> `PATCH` modifie en partie, `PUT` remplace la ressource entière, `POST` crée.
:::

:::quiz
Que fait un sérialiseur DRF en sortie ?

- [ ] Il chiffre les données
- [x] Il convertit un objet en dictionnaire prêt pour le JSON
- [ ] Il enregistre l'objet dans la base de données
- [ ] Il choisit l'URL de la ressource demandée par le client

> En sortie, `serializer.data` produit des types simples (dictionnaires, listes, chaînes) que DRF rend en JSON.
:::

:::quiz
Après `s = EvenementSerializer(data=donnees)`, quelle étape faut-il avant d'appeler `s.save()` ?

- [ ] `s.data`
- [ ] `s.validated_data = donnees`
- [x] `s.is_valid()`
- [ ] `s.errors = {}`

> `save()` exige que `is_valid()` ait été appelé : c'est lui qui contrôle les données et remplit `validated_data`.
:::

:::quiz
Dans `class Meta`, que fait `read_only_fields = ("id",)` ?

- [x] Il est affiché, mais ignoré si le client l'envoie
- [ ] Le champ `id` disparaît de la réponse JSON du serveur
- [ ] Le champ `id` devient obligatoire dans chaque requête d'écriture
- [ ] Le champ `id` est chiffré avant d'être renvoyé

> Un champ en lecture seule apparaît en sortie mais n'est jamais pris en compte en entrée.
:::

:::quiz
Quelle méthode DRF appelle-t-il automatiquement pour valider uniquement le champ `places` ?

- [ ] `clean_places`
- [ ] `check_places`
- [x] `validate_places`
- [ ] `places_valid`

> Pour chaque champ, DRF cherche une méthode `validate_<champ>` sur le sérialiseur.
:::

:::quiz
Où trouve-t-on l'erreur levée dans `validate()` ?

- [x] Dans `serializer.errors["non_field_errors"]`
- [ ] Dans `serializer.errors["places"]`, comme une erreur de champ
- [ ] Dans `serializer.data`
- [ ] Dans `serializer.instance`

> Les erreurs qui ne concernent pas un champ précis sont placées sous `non_field_errors`.
:::

:::quiz
Un `SlugRelatedField(slug_field="nom", queryset=...)` sur une clé étrangère `asso` produit en JSON :

- [ ] Le numéro de l'association (`3`)
- [ ] L'URL de l'association
- [ ] L'association complète imbriquée
- [x] Le nom de l'association (`"Ciné-club"`)

> Le slug remplace l'identifiant par la valeur du champ indiqué. En entrée, il sert à retrouver l'objet grâce au `queryset`.
:::

:::quiz
Quelle classe de base choisir pour une ressource qui ne doit être que consultée ?

- [ ] `ModelViewSet`
- [x] `ReadOnlyModelViewSet`
- [ ] `APIView` avec une méthode `post`
- [ ] `SerializerMethodField`

> `ReadOnlyModelViewSet` ne propose que `list` et `retrieve`.
:::

:::quiz
Que fait `router.register("evenements", EvenementViewSet)` ?

- [ ] Il crée les tables correspondantes dans la base de données
- [ ] Il active l'authentification sur ce viewset
- [x] Il génère les URL de la liste et du détail du viewset
- [ ] Il ajoute des permissions à toutes les vues

> Le routeur déduit les URL à partir du préfixe et des actions du viewset.
:::

:::quiz
Tu écris `@action(detail=False, methods=["get"])` sur une méthode `export`. Quelle URL est créée ?

- [ ] `/evenements/<id>/export/`
- [x] `/evenements/export/`
- [ ] `/export/`
- [ ] Aucune, `detail=False` désactive l'action

> `detail=False` : l'action porte sur la collection, donc pas d'identifiant dans l'URL.
:::

:::quiz
Sans réglage particulier, quelle permission DRF applique-t-il à une vue ?

- [ ] `IsAuthenticated`
- [ ] `IsAdminUser`
- [ ] `DjangoModelPermissions`
- [x] `AllowAny`

> Par défaut tout est autorisé : on règle `DEFAULT_PERMISSION_CLASSES` pour fermer l'API par défaut.
:::

:::quiz
Dans le dépôt Adhésion (`adhesion/api/permissions.py`), quelle classe de base la classe `KeycloakJWTAuthentication` étend-elle ? (Ne la confonds pas avec celle de la leçon, écrite pour l'exemple.)

- [x] `mozilla_django_oidc.contrib.drf.OIDCAuthentication`
- [ ] `rest_framework.authentication.BasicAuthentication`
- [ ] `django.contrib.auth.backends.ModelBackend`
- [ ] `rest_framework.permissions.BasePermission`

> Elle étend `OIDCAuthentication` de `mozilla_django_oidc`. `BasePermission` sert aux permissions, pas à l'authentification.
:::

:::quiz
Où le client envoie-t-il un jeton Bearer ?

- [ ] Dans l'URL, après `?token=`
- [ ] Dans le corps JSON
- [x] Dans l'en-tête `Authorization: Bearer <jeton>`
- [ ] Dans un cookie `csrftoken`

> Le jeton voyage dans l'en-tête `Authorization`, précédé du mot `Bearer`.
:::

:::quiz
Tu déclares `permission_classes = [HasRoleStaffOrReadOnly]` qui autorise les méthodes sûres à tout le monde. Qu'arrive-t-il à un `GET` sans jeton ?

- [ ] Il est refusé en `401`
- [x] Il est accepté, car la liste remplace `DEFAULT_PERMISSION_CLASSES`
- [ ] Il est refusé en `403`
- [ ] Il est accepté seulement si `DEBUG` vaut `True`

> La liste de la vue remplace le réglage par défaut ; ajoute `IsAuthenticated` pour exiger une identification.
:::

:::quiz
Quelle URL permet de chercher « ciné » puis de trier par date décroissante ?

- [ ] `?q=ciné&sort=date&direction=desc`
- [x] `?search=ciné&ordering=-date`
- [ ] `?find=ciné&order=-date&page=1`
- [ ] `?search=ciné&ordering=date&desc=1`

> `SearchFilter` lit `search`, `OrderingFilter` lit `ordering` (le `-` inverse).
:::

:::quiz
Pourquoi ajouter `order_by(...)` au `queryset` d'une vue paginée ?

- [ ] Pour que la pagination fonctionne en JSON
- [ ] Pour accélérer l'authentification
- [x] Pour éviter des doublons ou des oublis entre pages dus à un ordre instable
- [ ] Pour que `count` soit exact

> Sans ordre défini, la base peut renvoyer les lignes dans un ordre différent d'une page à l'autre.
:::

:::quiz
Que contient le champ `count` d'une réponse paginée par `PageNumberPagination` ?

- [ ] Le nombre d'éléments de la page
- [ ] Le numéro de la page courante
- [x] Le nombre total d'éléments, toutes pages confondues
- [ ] Le nombre de pages

> `count` est le total ; `results` contient la page courante.
:::

:::quiz
Que fait `max_page_size` dans une classe de pagination ?

- [x] Il limite la taille de page que le client peut demander
- [ ] Il fixe la taille par défaut
- [ ] Il limite le nombre de pages
- [ ] Il désactive `page_size_query_param`

> Il plafonne la taille de page demandée via `page_size_query_param`.
:::

:::quiz
D'où DRF tire-t-il la description d'une vue dans la page navigable et le schéma ?

- [ ] Du nom de la classe uniquement
- [x] De la docstring
- [ ] D'un fichier `README.md`
- [ ] Du `Meta.description`

> La docstring de la classe ou de la méthode est reprise comme description.
:::

:::quiz
Quelle commande génère un schéma OpenAPI d'un projet DRF ?

- [ ] `python manage.py makeschema`
- [ ] `python manage.py openapi --export`
- [x] `python manage.py generateschema`
- [ ] `python manage.py collectstatic --schema`

> `generateschema` est la commande fournie par DRF.
:::

:::quiz
Dans un test, que permet `self.client.force_authenticate(user=u, token=t)` ?

- [ ] Générer un vrai jeton JWT
- [x] Envoyer les requêtes comme si `u` était identifié·e, sans authentification
- [ ] Désactiver toutes les permissions de l'API pendant le test
- [ ] Créer l'utilisateur `u` dans la base de données du test

> Il fixe `request.user` et `request.auth` ; les permissions s'appliquent ensuite normalement.
:::

:::quiz
Un test envoie un `POST` avec des données invalides à une vue protégée, avec le bon rôle. Quel code attends-tu ?

- [ ] `201`
- [ ] `200`
- [ ] `403`
- [x] `400`

> Des données invalides donnent `400` avec le détail des erreurs. `403` aurait signalé un problème de droits.
:::

:::quiz
Pourquoi `APIRequestFactory` seul ne suffit-il pas pour tester une API de bout en bout ?

- [ ] Il est réservé à la production
- [ ] Il ne gère pas le JSON
- [x] Il fabrique une requête mais ne l'envoie pas à une vue
- [ ] Il supprime la base de données

> Il construit la requête ; c'est à toi de la passer à une vue. `self.client` envoie la requête et renvoie une vraie réponse.
:::
