---
id: api-rest-et-json
title: "À quoi sert une API REST ?"
summary: "Comprendre ce qu'est une API, une requête HTTP, du JSON, et ce que Django REST framework t'apporte."
minutes: 35
objectives:
  - Expliquer ce qu'est une API et pourquoi on en construit
  - Lire une requête et une réponse HTTP (verbe, URL, code de statut, corps)
  - Reconnaître du JSON et une URL « REST »
  - Situer Django REST framework par rapport à Django
---

Imagine que l'application mobile des associations veuille afficher la liste des événements. Elle n'a pas accès à la base de données (heureusement !) et ne sait pas lire du Python. Il faut un **intermédiaire** : un programme sur un serveur, qui répond à des questions précises dans un format que tout le monde comprend. C'est exactement le rôle d'une **API**.

## Une API, c'est un guichet

Une **API** (*Application Programming Interface*, interface de programmation) est un ensemble de « guichets » qu'un programme peut interroger pour lire ou modifier des données, sans connaître le fonctionnement interne du serveur. Comme un guichet de bibliothèque : tu demandes un livre, tu ne vas pas toi-même fouiller les rayons.

Deux rôles s'opposent :

- le **client** est celui qui demande (une application mobile, un site web, un script) ;
- le **serveur** est celui qui répond (ici, ton projet Django).

L'équipe en propose plusieurs. La plus utilisée est l'[API d'Adhésion](https://gitlab.example.org/equipe/adhesion/api), qui gère les adhérent·e·s : d'autres applications l'interrogent pour savoir, par exemple, si une personne a une adhésion valide.

## Le langage commun : HTTP

Client et serveur se parlent avec **HTTP**, le même protocole que ton navigateur. Une conversation = une **requête** (la question) suivie d'une **réponse**.

Une requête contient :

- une **URL** : l'adresse de la ressource demandée (`/v1/evenements/1/`) ;
- un **verbe** (ou méthode) : l'action voulue ;
- éventuellement des **en-têtes** (métadonnées, comme l'identité de l'appelant) ;
- éventuellement un **corps** : les données envoyées.

| Verbe | Sens | Exemple |
| --- | --- | --- |
| `GET` | Lire | Donne-moi l'événement 1 |
| `POST` | Créer | Voici un nouvel événement |
| `PUT` / `PATCH` | Remplacer / modifier en partie | Change la date de l'événement 1 |
| `DELETE` | Supprimer | Supprime l'événement 1 |

La réponse contient un **code de statut** (un nombre qui résume le résultat) et en général un corps.

| Code | Signification |
| --- | --- |
| `200` | OK |
| `201` | Créé |
| `204` | Fait, rien à renvoyer (après un `DELETE`) |
| `400` | Requête invalide (données incorrectes) |
| `401` | Pas identifié·e |
| `403` | Identifié·e, mais pas autorisé·e |
| `404` | Introuvable |

## Le format commun : JSON

Le corps est le plus souvent du **JSON**, un format texte qui décrit des données avec des accolades `{ }` (un objet : des paires nom/valeur) et des crochets `[ ]` (une liste). Il ressemble beaucoup aux dictionnaires et listes de Python, et tous les langages savent le lire.

Voici une requête puis sa réponse, telles qu'on les écrirait dans un terminal avec l'outil `curl` (un client HTTP en ligne de commande) :

```bash
curl -X GET https://api.example.org/v1/evenements/1/
```

Ligne à ligne : `curl` lance la requête, `-X GET` choisit le verbe, et l'argument final est l'URL. Le serveur répond :

```console
HTTP/1.1 200 OK
Content-Type: application/json

{"id": 1, "asso": "Ciné-club", "titre": "Soirée courts-métrages", "places": 30}
```

La première ligne donne le code de statut (`200 OK`), `Content-Type` annonce que le corps est du JSON, et le corps suit après la ligne vide.

Le même événement en JSON, mieux indenté :

```json
{
  "id": 1,
  "asso": "Ciné-club",
  "titre": "Soirée courts-métrages",
  "places": 30
}
```

## REST : des conventions pour que ce soit prévisible

**REST** n'est pas un logiciel mais un ensemble de conventions pour organiser une API autour de **ressources** (des « choses » comme un événement ou une association). Les deux idées à retenir :

- une URL désigne une ressource ou une collection, avec un **nom** et non un verbe : `/v1/evenements/` (la liste) et `/v1/evenements/1/` (un événement) ;
- le **verbe HTTP** dit l'action, le **code de statut** dit le résultat.

Ainsi `GET /v1/evenements/` liste, `POST /v1/evenements/` crée, `DELETE /v1/evenements/1/` supprime. Une personne qui connaît ces conventions devine comment utiliser ton API sans lire de documentation. C'est ce schéma que suit le routeur de l'API d'Adhésion (`/v1/members/`, `/v1/cards/`, `/v1/memberships/`…).

## Où intervient Django REST framework ?

Django sait déjà recevoir une requête et renvoyer une réponse, mais il est pensé pour des pages HTML. **Django REST framework** (DRF) est une bibliothèque qui ajoute les outils pour construire une API :

| Besoin | Outil DRF | Leçon |
| --- | --- | --- |
| Convertir un objet en JSON et vérifier le JSON reçu | Sérialiseur | 2 |
| Répondre aux requêtes d'une ressource | Vue / viewset, routeur | 3 |
| Savoir qui appelle et ce qu'il peut faire | Authentification, permissions | 4 |
| Chercher, trier, découper en pages, exporter | Filtres, pagination | 5 |
| Expliquer l'API aux autres | Documentation | 6 |
| Vérifier que tout marche | Tests d'API | 7 |

:::info Versions utilisées dans l'équipe
L'API d'Adhésion déclare `Django==3.1.13` et `djangorestframework==3.12.1` dans son `requirements.txt` ; [PlanningAPI](https://gitlab.example.org/equipe/dev/planning/planning-api) déclare `Django==3.1.1` et `djangorestframework==3.11.1`. Ce sont des versions anciennes. Les exemples de ce parcours ont été exécutés avec une version récente de Django et de DRF : ce qui change dans les anciennes versions est signalé.
:::

:::tip Essaie avec ton navigateur
Un navigateur fait un `GET` à chaque fois que tu ouvres une adresse. DRF fournit même une page web lisible pour chaque point d'accès (la *browsable API*), pratique pour explorer sans outil supplémentaire.
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  Une petite API d'événements est déjà écrite dans ton dossier de travail. Ton rôle est celui du **client** : tu vas écrire les requêtes HTTP qui l'interrogent, dans le fichier `requetes.py`, avec le client de test de Django REST framework (il envoie des requêtes à l'API sans lancer de serveur). Chaque fonction que tu écris retourne la **réponse** reçue, dont tu pourras lire `status_code` (le code de statut) et `json()` (le corps).

  Les tests sont lancés avec **pytest**, un outil qui exécute des fonctions de test et affiche `.` pour un test réussi et `F` pour un test en échec : `pytest -q` lance tous les tests, `pytest -q -k lister` un seul. Pour modifier un fichier, ouvre-le avec `nano requetes.py` (Ctrl+O puis Entrée pour enregistrer, Ctrl+X pour quitter) ou avec VS Code.
commands:
  - cp -R /opt/exercices/base/. .
  - cp -R /opt/exercices/01-api-rest-et-json/. .
steps:
  - text: 'Dans `requetes.py`, ajoute la fonction `lister()` qui envoie un `GET` sur `/v1/evenements/` et retourne la réponse : le test `test_lister` doit passer'
    hint: 'Le client sait envoyer chaque verbe : `client.get("/v1/evenements/")`. N''oublie pas le `return`. Puis `pytest -q -k lister`.'
    checks:
      - command-succeeds: '/opt/outils/verifier-tests 01-api-rest-et-json test_lister'
    solution:
      - |
        cat >> requetes.py <<'EOF'
        def lister():
            """Retourne la réponse à GET /v1/evenements/."""
            return client.get("/v1/evenements/")
        EOF
  - text: 'Ajoute `lire(pk)` : un `GET` sur `/v1/evenements/<pk>/` (par exemple `/v1/evenements/1/`). Le test vérifie le code `200` et le titre dans le JSON'
    hint: 'Une chaîne f-string insère le numéro dans l''URL : `client.get(f"/v1/evenements/{pk}/")`.'
    checks:
      - command-succeeds: '/opt/outils/verifier-tests 01-api-rest-et-json test_lire'
    solution:
      - |
        cat >> requetes.py <<'EOF'


        def lire(pk):
            """Retourne la réponse à GET /v1/evenements/<pk>/."""
            return client.get(f"/v1/evenements/{pk}/")
        EOF
  - text: 'Ajoute `creer(corps)` : un `POST` sur `/v1/evenements/` qui envoie `corps` **encodé en JSON**. Le serveur doit répondre `201` (créé)'
    hint: 'Le client de test accepte un dictionnaire et un format : `client.post(url, corps, format="json")`.'
    checks:
      - command-succeeds: '/opt/outils/verifier-tests 01-api-rest-et-json test_creer'
    solution:
      - |
        cat >> requetes.py <<'EOF'


        def creer(corps):
            """Envoie `corps` en JSON avec POST /v1/evenements/ et retourne la réponse."""
            return client.post("/v1/evenements/", corps, format="json")
        EOF
  - text: 'Ajoute `supprimer(pk)` : un `DELETE` sur `/v1/evenements/<pk>/`. Le serveur répond `204`, et l''événement doit ensuite être introuvable'
    hint: 'Même forme que `lire`, avec le verbe `delete` : `client.delete(f"/v1/evenements/{pk}/")`.'
    after: [2]
    checks:
      - command-succeeds: '/opt/outils/verifier-tests 01-api-rest-et-json test_supprimer'
    solution:
      - |
        cat >> requetes.py <<'EOF'


        def supprimer(pk):
            """Retourne la réponse à DELETE /v1/evenements/<pk>/."""
            return client.delete(f"/v1/evenements/{pk}/")
        EOF
  - text: 'Ajoute `introuvable()` : un `GET` sur l''événement numéro `9999`, qui n''existe pas. Le serveur répond `404`'
    hint: 'L''URL est `/v1/evenements/9999/`. Tu n''as rien de spécial à écrire : c''est le serveur qui dit « introuvable ».'
    checks:
      - command-succeeds: '/opt/outils/verifier-tests 01-api-rest-et-json test_introuvable'
    solution:
      - |
        cat >> requetes.py <<'EOF'


        def introuvable():
            """Retourne la réponse à la lecture d'un événement qui n'existe pas (numéro 9999)."""
            return client.get("/v1/evenements/9999/")
        EOF
  - text: 'Ajoute `invalide()` : un `POST` dont le corps est incomplet (seulement `{"asso": "Fanfare"}`, sans titre ni date). Le serveur répond `400` et nomme le champ fautif'
    hint: 'Comme `creer`, avec un corps réduit : `client.post("/v1/evenements/", {"asso": "Fanfare"}, format="json")`. Observe ensuite `reponse.json()` : une clé par champ en erreur.'
    checks:
      - command-succeeds: '/opt/outils/verifier-tests 01-api-rest-et-json test_invalide'
    solution:
      - |-
        cat >> requetes.py <<'EOF'


        def invalide():
            """Retourne la réponse à un POST dont le corps est incomplet (sans titre ni date)."""
            return client.post("/v1/evenements/", {"asso": "Fanfare"}, format="json")
        EOF
:::

## Vérifie tes acquis

:::quiz
Dans quel cas utilises-tu le verbe `POST` ?

- [ ] Pour lire la liste des événements
- [ ] Pour supprimer un événement
- [x] Pour créer un nouvel événement
- [ ] Pour demander la documentation

> `POST` envoie des données au serveur pour créer une ressource. La lecture se fait avec `GET` et la suppression avec `DELETE`.
:::

:::quiz
Le serveur répond `404`. Que cela signifie-t-il ?

- [ ] Les données envoyées sont invalides
- [ ] Tu n'es pas identifié·e
- [x] La ressource demandée n'existe pas
- [ ] La requête a réussi

> `404` signifie « introuvable ». Les données invalides donnent `400`, l'absence d'identification `401`.
:::

:::quiz
Quelle URL respecte le mieux les conventions REST pour supprimer l'événement numéro 7 ?

- [ ] `POST /supprimer_evenement/7/`
- [ ] `GET /evenements/7/supprimer/`
- [x] `DELETE /evenements/7/`
- [ ] `DELETE /supprimer/`

> L'URL désigne la ressource (un nom), et le verbe HTTP porte l'action.
:::
