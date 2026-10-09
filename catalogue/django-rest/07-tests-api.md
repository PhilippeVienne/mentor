---
id: tests-api
title: "Tester son API"
summary: "Écrire des tests qui appellent l'API comme un vrai client : succès, refus d'accès et données invalides."
minutes: 45
objectives:
  - Expliquer pourquoi tester une API et ce qu'il faut tester
  - Écrire un test avec `APITestCase` et `self.client`
  - Simuler une personne identifiée avec `force_authenticate`
  - Vérifier codes de statut et contenu des réponses
---

Tu modifies un sérialiseur, et sans le vouloir tu rends une route publique ou tu casses le format que lit l'application mobile. Personne ne s'en aperçoit avant la mise en production. Un **test automatique** rejoue, en quelques secondes, les scénarios importants à chaque changement, et la **CI** de ton projet (l'intégration continue : un robot qui lance les tests automatiquement chaque fois que tu envoies ton code avec `git push`) peut les lancer à chaque modification.

## Que tester dans une API ?

Pas le code ligne par ligne, mais le **contrat** : pour une requête donnée, quel code de statut et quel contenu reviennent. Pour chaque ressource, les cas utiles sont :

| Scénario | Code attendu |
| --- | :---: |
| Lecture par une personne autorisée | `200` |
| Appel sans identification | `401` |
| Écriture sans le bon rôle | `403` |
| Création valide | `201` |
| Données invalides | `400` |
| Ressource absente | `404` |

Les cas d'échec comptent autant que les succès : un test « sans rôle, c'est refusé » garde la sécurité en place quand le code change.

## Le client de test

DRF fournit `APITestCase`, une variante du `TestCase` de Django, dont l'attribut `self.client` joue le rôle d'un client HTTP : il envoie des requêtes à ton application **sans lancer de serveur** et renvoie la réponse. Chaque test s'exécute dans une base de données vide, annulée à la fin.

Pour se faire passer pour une personne identifiée sans fabriquer de vrai jeton, on utilise `force_authenticate` : il court-circuite l'authentification et fixe directement `request.user` et `request.auth` (ici, des claims avec le rôle `staff`).

```python
# agenda/tests.py
from datetime import datetime, timezone

from django.contrib.auth import get_user_model
from rest_framework.test import APITestCase

from .models import Asso, Evenement


def jeton_staff():
    return {"resource_access": {"agenda-api": {"roles": ["staff"]}}}


class EvenementApiTests(APITestCase):
    def setUp(self):
        self.asso = Asso.objects.create(nom="Ciné-club")
        self.evenement = Evenement.objects.create(
            asso=self.asso,
            titre="Soirée courts-métrages",
            date=datetime(2099, 3, 12, 17, 0, tzinfo=timezone.utc),
            places=2,
        )
        self.alice = get_user_model().objects.create(username="alice@example.org")

    def test_sans_identification_401(self):
        reponse = self.client.get("/v1/evenements/")
        self.assertEqual(reponse.status_code, 401)

    def test_creation_interdite_sans_role_403(self):
        self.client.force_authenticate(user=self.alice, token={})
        corps = {"asso": "Ciné-club", "titre": "Ciné-débat", "date": "2099-01-01T10:00:00Z", "places": 40}
        reponse = self.client.post("/v1/evenements/", corps, format="json")
        self.assertEqual(reponse.status_code, 403)
        self.assertEqual(Evenement.objects.count(), 1)

    def test_creation_avec_role_staff_201(self):
        self.client.force_authenticate(user=self.alice, token=jeton_staff())
        corps = {"asso": "Ciné-club", "titre": "Ciné-débat", "date": "2099-01-01T10:00:00Z", "places": 40}
        reponse = self.client.post("/v1/evenements/", corps, format="json")
        self.assertEqual(reponse.status_code, 201)
        self.assertEqual(reponse.json()["places_restantes"], 40)

    def test_donnees_invalides_400(self):
        self.client.force_authenticate(user=self.alice, token=jeton_staff())
        corps = {"asso": "Ciné-club", "titre": "x", "date": "2099-01-01T10:00:00Z", "places": 0}
        reponse = self.client.post("/v1/evenements/", corps, format="json")
        self.assertEqual(reponse.status_code, 400)
        self.assertIn("places", reponse.json())
```

Ligne à ligne :

1. `setUp` s'exécute avant **chaque** test : il prépare un petit jeu de données (une association, un événement, une personne).
2. `self.client.get(url)` envoie un `GET` ; sans `force_authenticate`, la requête est anonyme, donc `401`.
3. `force_authenticate(user=..., token={})` identifie la personne, mais sans aucun rôle (`token={}`) : la lecture passe, l'écriture est refusée en `403`.
4. `self.client.post(url, corps, format="json")` envoie `corps` encodé en JSON ; le `format` est important, sinon le client envoie un formulaire.
5. `reponse.status_code` est le code de statut, `reponse.json()` le corps décodé.
6. `assertEqual` compare ; `assertIn("places", ...)` vérifie que l'erreur est bien rangée sous le champ fautif.
7. Dans le test `403`, on contrôle aussi qu'**aucun événement n'a été créé** : un refus qui écrit quand même en base serait un bug grave.

On lance les tests avec :

```bash
python manage.py test
```

```console
Found 4 test(s).
Creating test database for alias 'default'...
....
----------------------------------------------------------------------
Ran 4 tests in 0.012s

OK
```

Chaque point est un test réussi ; un `F` signale un test en échec, avec le détail de la comparaison qui a échoué. Si ton projet utilise `pytest` avec `pytest-django` (le plugin qui apprend à `pytest` à lancer des tests Django), `APITestCase` et `self.client` fonctionnent de la même façon : c'est ce que tu fais dans le labo, avec `pytest -q`.

## Tester le bon jeton plutôt que le contourner

`force_authenticate` ne passe pas par ta classe d'authentification. Pour la tester, fabrique un vrai jeton signé (avec la bibliothèque `PyJWT`) et envoie-le dans l'en-tête :

```python
self.client.credentials(HTTP_AUTHORIZATION=f"Bearer {jeton}")
```

`credentials()` ajoute cet en-tête à toutes les requêtes suivantes du test. Avec un jeton invalide, on s'attend à un `401` ; avec un jeton valide sans rôle, à un `200` en lecture et un `403` en écriture.

:::warning Un test qui ne teste rien
Le fichier `tests.py` d'Adhésion contient un test `test_api_add_card` qui utilise `APIRequestFactory`. Cette fabrique construit seulement une **requête** ; elle ne l'envoie à aucune vue. Le test compare ensuite le corps de la requête fabriquée, pas celui d'une réponse de l'API : il ne vérifie donc pas le comportement de l'API. Pour appeler l'API de bout en bout, utilise `self.client` (`APIClient`) comme ci-dessus. Pour vérifier qu'un test détecte bien un problème, casse volontairement le code et regarde le test échouer. Le `tests.py` de PlanningAPI, lui, ne contient encore que le squelette généré par Django.
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  L'API des événements est terminée, protégée et paginée. Il lui manque ses tests : le fichier `test_agenda_api.py` contient le début de la classe `EvenementApiTests`, avec cinq tests qui échouent exprès (`self.fail(...)`). Écris-les un par un. Pour chacun, le portail vérifie deux choses : ton test **passe** sur l'API telle qu'elle est, et il **échoue** quand on la casse volontairement (on supprime la protection, une validation, etc.). Un test qui ne détecte rien n'est pas validé. Lance-les avec `pytest -q test_agenda_api.py`.
commands:
  - cp -R /opt/exercices/base/. .
  - cp -R /opt/exercices/07-tests-api/. .
steps:
  - text: 'Écris `test_sans_identification_401` : un `GET` sur `/v1/evenements/` sans identification doit répondre `401`'
    hint: '`reponse = self.client.get("/v1/evenements/")` puis `self.assertEqual(reponse.status_code, 401)`.'
    checks:
      - command-succeeds: '/opt/outils/verifier-mutation sans-authentification sans_identification'
    solution:
      - write:
          test_agenda_api.py: |
            """Tests d'API de la leçon 7 : à écrire par toi, un par scénario. Lance-les avec `pytest -q`."""
            from datetime import datetime, timezone

            from django.contrib.auth import get_user_model
            from django.test import override_settings
            from rest_framework.test import APITestCase

            import helpers_jwt
            from agenda.models import Asso, Evenement

            CORPS = {"asso": "Ciné-club", "titre": "Ciné-débat", "date": "2099-01-01T10:00:00Z", "places": 40}


            def jeton_staff():
                return {"resource_access": {"agenda-api": {"roles": ["staff"]}}}


            class EvenementApiTests(APITestCase):
                def setUp(self):
                    self.asso = Asso.objects.create(nom="Ciné-club")
                    self.evenement = Evenement.objects.create(
                        asso=self.asso,
                        titre="Soirée courts-métrages",
                        date=datetime(2099, 3, 12, 17, 0, tzinfo=timezone.utc),
                        places=2,
                    )
                    self.alice = get_user_model().objects.create(username="alice@example.org")

                def test_sans_identification_401(self):
                    reponse = self.client.get("/v1/evenements/")
                    self.assertEqual(reponse.status_code, 401)

                def test_creation_interdite_sans_role_403(self):
                    self.fail("À écrire : un POST sans rôle doit répondre 403 et ne rien créer en base")

                def test_creation_avec_role_staff_201(self):
                    self.fail("À écrire : un POST avec le rôle staff doit répondre 201")

                def test_donnees_invalides_400(self):
                    self.fail("À écrire : un POST avec 0 place doit répondre 400, avec l'erreur sous « places »")

                @override_settings(OIDC_PUBLIC_KEY=helpers_jwt.CLE_PUBLIQUE)
                def test_jeton_signe_par_une_autre_cle_401(self):
                    self.fail("À écrire : un jeton fabriqué avec helpers_jwt.AUTRE_CLE_PRIVEE doit donner 401")
  - text: 'Écris `test_creation_interdite_sans_role_403` : identifié·e **sans rôle** (`self.client.force_authenticate(user=self.alice, token={})`), un `POST` doit répondre `403` **et** ne rien créer (`Evenement.objects.count()` reste à `1`)'
    hint: 'Le corps à envoyer est la constante `CORPS`, avec `format="json"`. Après la réponse, compare `Evenement.objects.count()` à `1`.'
    after: [1]
    checks:
      - command-succeeds: '/opt/outils/verifier-mutation sans-controle-du-role creation_interdite --compte'
    solution:
      - write:
          test_agenda_api.py: |
            """Tests d'API de la leçon 7 : à écrire par toi, un par scénario. Lance-les avec `pytest -q`."""
            from datetime import datetime, timezone

            from django.contrib.auth import get_user_model
            from django.test import override_settings
            from rest_framework.test import APITestCase

            import helpers_jwt
            from agenda.models import Asso, Evenement

            CORPS = {"asso": "Ciné-club", "titre": "Ciné-débat", "date": "2099-01-01T10:00:00Z", "places": 40}


            def jeton_staff():
                return {"resource_access": {"agenda-api": {"roles": ["staff"]}}}


            class EvenementApiTests(APITestCase):
                def setUp(self):
                    self.asso = Asso.objects.create(nom="Ciné-club")
                    self.evenement = Evenement.objects.create(
                        asso=self.asso,
                        titre="Soirée courts-métrages",
                        date=datetime(2099, 3, 12, 17, 0, tzinfo=timezone.utc),
                        places=2,
                    )
                    self.alice = get_user_model().objects.create(username="alice@example.org")

                def test_sans_identification_401(self):
                    reponse = self.client.get("/v1/evenements/")
                    self.assertEqual(reponse.status_code, 401)

                def test_creation_interdite_sans_role_403(self):
                    self.client.force_authenticate(user=self.alice, token={})
                    reponse = self.client.post("/v1/evenements/", CORPS, format="json")
                    self.assertEqual(reponse.status_code, 403)
                    self.assertEqual(Evenement.objects.count(), 1)

                def test_creation_avec_role_staff_201(self):
                    self.fail("À écrire : un POST avec le rôle staff doit répondre 201")

                def test_donnees_invalides_400(self):
                    self.fail("À écrire : un POST avec 0 place doit répondre 400, avec l'erreur sous « places »")

                @override_settings(OIDC_PUBLIC_KEY=helpers_jwt.CLE_PUBLIQUE)
                def test_jeton_signe_par_une_autre_cle_401(self):
                    self.fail("À écrire : un jeton fabriqué avec helpers_jwt.AUTRE_CLE_PRIVEE doit donner 401")
  - text: 'Écris `test_creation_avec_role_staff_201` : identifié·e avec le rôle `staff` (`token=jeton_staff()`), un `POST` valide doit répondre `201`, et la réponse doit annoncer `places_restantes` égal à `40`'
    hint: '`reponse.json()["places_restantes"]` donne la valeur dans le corps de la réponse.'
    after: [2]
    checks:
      - command-succeeds: '/opt/outils/verifier-mutation staff-refuse role_staff places_restantes 40'
    solution:
      - write:
          test_agenda_api.py: |
            """Tests d'API de la leçon 7 : à écrire par toi, un par scénario. Lance-les avec `pytest -q`."""
            from datetime import datetime, timezone

            from django.contrib.auth import get_user_model
            from django.test import override_settings
            from rest_framework.test import APITestCase

            import helpers_jwt
            from agenda.models import Asso, Evenement

            CORPS = {"asso": "Ciné-club", "titre": "Ciné-débat", "date": "2099-01-01T10:00:00Z", "places": 40}


            def jeton_staff():
                return {"resource_access": {"agenda-api": {"roles": ["staff"]}}}


            class EvenementApiTests(APITestCase):
                def setUp(self):
                    self.asso = Asso.objects.create(nom="Ciné-club")
                    self.evenement = Evenement.objects.create(
                        asso=self.asso,
                        titre="Soirée courts-métrages",
                        date=datetime(2099, 3, 12, 17, 0, tzinfo=timezone.utc),
                        places=2,
                    )
                    self.alice = get_user_model().objects.create(username="alice@example.org")

                def test_sans_identification_401(self):
                    reponse = self.client.get("/v1/evenements/")
                    self.assertEqual(reponse.status_code, 401)

                def test_creation_interdite_sans_role_403(self):
                    self.client.force_authenticate(user=self.alice, token={})
                    reponse = self.client.post("/v1/evenements/", CORPS, format="json")
                    self.assertEqual(reponse.status_code, 403)
                    self.assertEqual(Evenement.objects.count(), 1)

                def test_creation_avec_role_staff_201(self):
                    self.client.force_authenticate(user=self.alice, token=jeton_staff())
                    reponse = self.client.post("/v1/evenements/", CORPS, format="json")
                    self.assertEqual(reponse.status_code, 201)
                    self.assertEqual(reponse.json()["places_restantes"], 40)

                def test_donnees_invalides_400(self):
                    self.fail("À écrire : un POST avec 0 place doit répondre 400, avec l'erreur sous « places »")

                @override_settings(OIDC_PUBLIC_KEY=helpers_jwt.CLE_PUBLIQUE)
                def test_jeton_signe_par_une_autre_cle_401(self):
                    self.fail("À écrire : un jeton fabriqué avec helpers_jwt.AUTRE_CLE_PRIVEE doit donner 401")
  - text: 'Écris `test_donnees_invalides_400` : avec le rôle `staff`, un `POST` à `0` place doit répondre `400`, et `"places"` doit figurer dans les erreurs de la réponse'
    hint: 'Fabrique un corps invalide à partir de `CORPS` : `{**CORPS, "titre": "x", "places": 0}`. Vérifie avec `self.assertIn("places", reponse.json())`.'
    after: [3]
    checks:
      - command-succeeds: '/opt/outils/verifier-mutation sans-validation donnees_invalides places'
    solution:
      - write:
          test_agenda_api.py: |
            """Tests d'API de la leçon 7 : à écrire par toi, un par scénario. Lance-les avec `pytest -q`."""
            from datetime import datetime, timezone

            from django.contrib.auth import get_user_model
            from django.test import override_settings
            from rest_framework.test import APITestCase

            import helpers_jwt
            from agenda.models import Asso, Evenement

            CORPS = {"asso": "Ciné-club", "titre": "Ciné-débat", "date": "2099-01-01T10:00:00Z", "places": 40}


            def jeton_staff():
                return {"resource_access": {"agenda-api": {"roles": ["staff"]}}}


            class EvenementApiTests(APITestCase):
                def setUp(self):
                    self.asso = Asso.objects.create(nom="Ciné-club")
                    self.evenement = Evenement.objects.create(
                        asso=self.asso,
                        titre="Soirée courts-métrages",
                        date=datetime(2099, 3, 12, 17, 0, tzinfo=timezone.utc),
                        places=2,
                    )
                    self.alice = get_user_model().objects.create(username="alice@example.org")

                def test_sans_identification_401(self):
                    reponse = self.client.get("/v1/evenements/")
                    self.assertEqual(reponse.status_code, 401)

                def test_creation_interdite_sans_role_403(self):
                    self.client.force_authenticate(user=self.alice, token={})
                    reponse = self.client.post("/v1/evenements/", CORPS, format="json")
                    self.assertEqual(reponse.status_code, 403)
                    self.assertEqual(Evenement.objects.count(), 1)

                def test_creation_avec_role_staff_201(self):
                    self.client.force_authenticate(user=self.alice, token=jeton_staff())
                    reponse = self.client.post("/v1/evenements/", CORPS, format="json")
                    self.assertEqual(reponse.status_code, 201)
                    self.assertEqual(reponse.json()["places_restantes"], 40)

                def test_donnees_invalides_400(self):
                    self.client.force_authenticate(user=self.alice, token=jeton_staff())
                    corps = {**CORPS, "titre": "x", "places": 0}
                    reponse = self.client.post("/v1/evenements/", corps, format="json")
                    self.assertEqual(reponse.status_code, 400)
                    self.assertIn("places", reponse.json())

                @override_settings(OIDC_PUBLIC_KEY=helpers_jwt.CLE_PUBLIQUE)
                def test_jeton_signe_par_une_autre_cle_401(self):
                    self.fail("À écrire : un jeton fabriqué avec helpers_jwt.AUTRE_CLE_PRIVEE doit donner 401")
  - text: 'Écris `test_jeton_signe_par_une_autre_cle_401` avec un **vrai** jeton : `helpers_jwt.fabriquer_jeton(roles=["staff"], cle=helpers_jwt.AUTRE_CLE_PRIVEE)` est signé par une clé que l''API ne connaît pas. Envoyé dans l''en-tête `Authorization: Bearer ...`, il doit donner `401`'
    hint: '`self.client.credentials(HTTP_AUTHORIZATION=f"Bearer {jeton}")` ajoute l''en-tête à toutes les requêtes suivantes du test.'
    after: [4]
    checks:
      - command-succeeds: '/opt/outils/verifier-mutation sans-signature autre_cle'
    solution:
      - write:
          test_agenda_api.py: |
            """Tests d'API de la leçon 7 : à écrire par toi, un par scénario. Lance-les avec `pytest -q`."""
            from datetime import datetime, timezone

            from django.contrib.auth import get_user_model
            from django.test import override_settings
            from rest_framework.test import APITestCase

            import helpers_jwt
            from agenda.models import Asso, Evenement

            CORPS = {"asso": "Ciné-club", "titre": "Ciné-débat", "date": "2099-01-01T10:00:00Z", "places": 40}


            def jeton_staff():
                return {"resource_access": {"agenda-api": {"roles": ["staff"]}}}


            class EvenementApiTests(APITestCase):
                def setUp(self):
                    self.asso = Asso.objects.create(nom="Ciné-club")
                    self.evenement = Evenement.objects.create(
                        asso=self.asso,
                        titre="Soirée courts-métrages",
                        date=datetime(2099, 3, 12, 17, 0, tzinfo=timezone.utc),
                        places=2,
                    )
                    self.alice = get_user_model().objects.create(username="alice@example.org")

                def test_sans_identification_401(self):
                    reponse = self.client.get("/v1/evenements/")
                    self.assertEqual(reponse.status_code, 401)

                def test_creation_interdite_sans_role_403(self):
                    self.client.force_authenticate(user=self.alice, token={})
                    reponse = self.client.post("/v1/evenements/", CORPS, format="json")
                    self.assertEqual(reponse.status_code, 403)
                    self.assertEqual(Evenement.objects.count(), 1)

                def test_creation_avec_role_staff_201(self):
                    self.client.force_authenticate(user=self.alice, token=jeton_staff())
                    reponse = self.client.post("/v1/evenements/", CORPS, format="json")
                    self.assertEqual(reponse.status_code, 201)
                    self.assertEqual(reponse.json()["places_restantes"], 40)

                def test_donnees_invalides_400(self):
                    self.client.force_authenticate(user=self.alice, token=jeton_staff())
                    corps = {**CORPS, "titre": "x", "places": 0}
                    reponse = self.client.post("/v1/evenements/", corps, format="json")
                    self.assertEqual(reponse.status_code, 400)
                    self.assertIn("places", reponse.json())

                @override_settings(OIDC_PUBLIC_KEY=helpers_jwt.CLE_PUBLIQUE)
                def test_jeton_signe_par_une_autre_cle_401(self):
                    jeton = helpers_jwt.fabriquer_jeton(roles=["staff"], cle=helpers_jwt.AUTRE_CLE_PRIVEE)
                    self.client.credentials(HTTP_AUTHORIZATION=f"Bearer {jeton}")
                    reponse = self.client.get("/v1/evenements/")
                    self.assertEqual(reponse.status_code, 401)
:::

## Vérifie tes acquis

:::quiz
Quelle classe de base utilises-tu pour écrire des tests qui appellent ton API ?

- [ ] `unittest.Mock`
- [ ] `serializers.Serializer`
- [x] `rest_framework.test.APITestCase`
- [ ] `viewsets.ViewSet`

> `APITestCase` fournit `self.client`, un client HTTP de test, et une base de données isolée.
:::

:::quiz
À quoi sert `force_authenticate(user=..., token=...)` ?

- [ ] À générer un vrai jeton signé
- [x] À simuler une personne identifiée sans passer par la classe d'authentification
- [ ] À désactiver les permissions
- [ ] À créer l'utilisateur en base

> Il fixe `request.user` et `request.auth` directement. Les permissions, elles, s'appliquent normalement.
:::

:::quiz
Pourquoi préciser `format="json"` dans `self.client.post(...)` ?

- [ ] Pour que la réponse soit en JSON
- [ ] Pour accélérer le test
- [ ] Pour contourner l'authentification
- [x] Pour que le corps soit envoyé en JSON plutôt qu'en formulaire

> Le format concerne la requête envoyée.
:::

:::quiz
Un test envoie une requête sans rôle suffisant. Quel contrôle ajoutes-tu en plus du code `403` ?

- [ ] Que la réponse dure moins d'une seconde
- [ ] Que le jeton est valide
- [x] Que la base de données n'a pas été modifiée
- [ ] Que le routeur contient la route

> Un refus doit être sans effet : on vérifie que rien n'a été créé ou supprimé.
:::
