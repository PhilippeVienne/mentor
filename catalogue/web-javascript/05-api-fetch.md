---
id: api-fetch
title: "Appeler une API avec fetch"
summary: "Récupérer des données JSON avec fetch et async/await, gérer les erreurs et les afficher dans la page."
minutes: 30
objectives:
  - Expliquer ce qu'est une API HTTP renvoyant du JSON
  - Appeler une API avec `fetch` et `await`
  - Gérer une erreur réseau ou une réponse en erreur
  - Afficher les données reçues dans la page
---

Jusqu'ici, tes données sont écrites en dur dans le JavaScript. En vrai, la liste des événements d'une association vit sur un **serveur** (un ordinateur toujours allumé qui répond aux demandes des navigateurs), et ta page doit la demander. C'est le rôle d'une API.

## Une API HTTP en JSON

Une API (*Application Programming Interface*, « interface de programmation ») est une adresse que l'on interroge pour obtenir des données plutôt qu'une page. On lui parle en **HTTP**, le protocole (les règles de conversation) du web : le navigateur envoie une **requête**, le serveur renvoie une **réponse**. Le format d'échange le plus courant est le **JSON** (*JavaScript Object Notation*), un texte qui ressemble aux objets JavaScript :

```json
[
  { "id": 1, "titre": "Sortie photo", "places": 20 },
  { "id": 2, "titre": "Atelier retouche", "places": 0 }
]
```

Ce JSON est une liste (les crochets) de deux objets (les accolades), chacun avec trois propriétés. Une différence avec JavaScript : en JSON, les noms des propriétés sont toujours entre guillemets doubles.

Dans cette leçon, on suppose une API fictive : `GET https://example.org/api/evenements` renvoie la liste ci-dessus. Dans le labo, il n'y a pas de réseau : `verifier-page` fournit à ta page un **faux `fetch`** qui répond à cette adresse avec le fichier `api-fictive/evenements.json` de ton dossier. Le code que tu écris est exactement celui que tu écrirais pour une vraie API.

Les méthodes HTTP courantes (le « verbe » de la requête) :

| Méthode | Usage |
| --- | --- |
| `GET` | Lire des données |
| `POST` | Créer une ressource |
| `PUT` / `PATCH` | Modifier une ressource |
| `DELETE` | Supprimer une ressource |

Chaque réponse porte un **code de statut** : `200` (OK), `404` (introuvable), `401` / `403` (non authentifié·e / interdit), `500` (erreur du serveur).

## Les promesses et await

La requête prend du temps : le navigateur ne doit pas se figer en attendant. `fetch` renvoie donc une **promesse** (*Promise*), un résultat « à venir ». Le mot-clé `await` (« attends ») attend ce résultat dans une fonction déclarée `async`.

```js
async function chargerEvenements() {
  const reponse = await fetch("https://example.org/api/evenements");
  const evenements = await reponse.json();
  return evenements;
}
```

Ligne par ligne :

- `async function chargerEvenements() {` déclare une fonction **asynchrone** : le mot `async` autorise `await` à l'intérieur, et la fonction renvoie elle-même une promesse.
- `const reponse = await fetch("…");` envoie la requête `GET` et attend les **en-têtes** de la réponse (statut, type de contenu). `reponse` est un objet qui décrit la réponse.
- `const evenements = await reponse.json();` lit le corps de la réponse et le convertit en objets JavaScript. C'est aussi asynchrone : second `await`.
- `return evenements;` rend la liste à celui qui a appelé la fonction.

## Gérer les erreurs

Piège classique : `fetch` ne lève **pas** d'erreur sur un statut `404` ou `500`. Elle échoue seulement si le réseau est coupé. Vérifie donc toujours `reponse.ok` (qui vaut `true` pour les statuts 200 à 299) :

```js
async function chargerEvenements() {
  const reponse = await fetch("https://example.org/api/evenements");
  if (!reponse.ok) {
    throw new Error(`Erreur ${reponse.status}`);
  }
  return reponse.json();
}

async function afficher() {
  try {
    const evenements = await chargerEvenements();
    console.log(`${evenements.length} événements`);
  } catch (erreur) {
    console.error("Impossible de charger les événements", erreur);
  }
}
```

- `if (!reponse.ok) { … }` : le `!` veut dire « n'est pas » ; si la réponse n'est pas OK, on arrête tout.
- `throw new Error(…)` **lance une erreur** : la fonction s'interrompt et l'erreur remonte à qui l'a appelée.
- `try { … } catch (erreur) { … }` essaie le bloc `try` ; si une erreur est lancée dedans, l'exécution saute au bloc `catch`, avec l'erreur dans la variable `erreur`.
- `evenements.length` est le nombre d'éléments de la liste, et `console.error` affiche un message d'erreur.

`try … catch` attrape à la fois l'erreur réseau et celle que tu as lancée avec `throw`.

## Afficher les données

On combine tout ce qui précède dans une petite page :

```html
<p id="statut">Chargement…</p>
<ul id="evenements"></ul>
<script src="app.js"></script>
```

```js
const statut = document.querySelector("#statut");
const liste = document.querySelector("#evenements");

async function afficher() {
  try {
    const reponse = await fetch("https://example.org/api/evenements");
    if (!reponse.ok) {
      throw new Error(`Erreur ${reponse.status}`);
    }
    const evenements = await reponse.json();

    for (const e of evenements) {
      const li = document.createElement("li");
      li.textContent = `${e.titre} (${e.places} places)`;
      liste.append(li);
    }
    statut.textContent = "";
  } catch (erreur) {
    statut.textContent = "Impossible de charger les événements.";
  }
}

afficher();
```

Ligne par ligne :

- Les deux premières lignes retrouvent le paragraphe `#statut` et la liste `#evenements` dans le DOM (voir la leçon précédente).
- Dans `try`, on envoie la requête, on vérifie le statut, on lit le JSON.
- La boucle `for…of` crée un `<li>` par événement, avec un texte du genre `Sortie photo (20 places)`, et l'ajoute à la liste.
- `statut.textContent = "";` efface le message « Chargement… » une fois la liste affichée.
- Dans `catch`, on remplace « Chargement… » par un message d'erreur lisible.
- La dernière ligne `afficher();` **appelle** la fonction : sans elle, rien ne se passe.

Une bonne interface montre toujours **trois états** : chargement, données, erreur. Ici le message « Chargement… » est remplacé par une liste ou par le message d'erreur.

Pour envoyer des données, on précise la méthode, l'en-tête et le corps :

```js
await fetch("https://example.org/api/inscriptions", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ evenement: 1, email: "camille@example.org" })
});
```

- Le second argument de `fetch` est un objet d'options : `method` choisit le verbe, `headers` ajoute des en-têtes (ici « le corps est du JSON ») et `body` est le contenu envoyé.
- `JSON.stringify` transforme un objet en texte JSON ; `JSON.parse` fait l'inverse.

:::info Les restrictions CORS
Un navigateur n'autorise une page à appeler une API d'une **autre origine** (autre domaine ou autre port) que si cette API l'accepte via des en-têtes CORS. Si tu vois une erreur « blocked by CORS policy » dans la console, ce n'est pas un bug de ton code : c'est la configuration du serveur de l'API qu'il faut ajuster.
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  Ton dossier contient `index.html` (un paragraphe `#statut` qui dit « Chargement… », une liste `#evenements` et un formulaire `#inscription` avec son champ `#email`), un `app.js` vide, et un dossier `api-fictive/` avec le fichier `evenements.json` que le faux serveur renvoie. Écris ton code dans `app.js` avec `nano app.js`. Pour tester, `verifier-page index.html` exécute ta page avec le faux `fetch` ; ajoute `--erreur 500` pour que le faux serveur réponde par une erreur 500, ou `--panne` pour simuler un réseau coupé. Seul `app.js` compte : le portail remet `index.html` et `api-fictive/` à l'identique avant de contrôler, et essaie aussi ton code avec d'autres données.
commands:
  - cp -R /opt/exercices/05-api-fetch/. .
steps:
  - text: >-
      Dans `app.js`, écris une fonction `async` qui appelle `fetch("https://example.org/api/evenements")`, lit le JSON et ajoute à `#evenements` un `<li>` par événement, de la forme `Sortie photo (20 places)`. Appelle la fonction à la fin du fichier.
    hint: >-
      Reprends l'exemple de la leçon. Sans le dernier `afficher();`, la fonction n'est jamais appelée et la liste reste vide.
    checks:
      - command-succeeds: 'verifier-web 05 liste'
    solution:
      - |
        cat > app.js <<'EOF'
        const statut = document.querySelector("#statut");
        const liste = document.querySelector("#evenements");

        async function afficher() {
          const reponse = await fetch("https://example.org/api/evenements");
          const evenements = await reponse.json();
          for (const e of evenements) {
            const li = document.createElement("li");
            li.textContent = `${e.titre} (${e.places} places)`;
            liste.append(li);
          }
        }

        afficher();
        EOF
  - text: >-
      Une fois la liste affichée, efface le message « Chargement… » : le paragraphe `#statut` doit devenir vide.
    hint: >-
      `statut.textContent = "";` à la fin du bloc qui remplit la liste.
    after: [1]
    checks:
      - command-succeeds: 'verifier-web 05 statut'
    solution:
      - |
        cat > app.js <<'EOF'
        const statut = document.querySelector("#statut");
        const liste = document.querySelector("#evenements");

        async function afficher() {
          const reponse = await fetch("https://example.org/api/evenements");
          const evenements = await reponse.json();
          for (const e of evenements) {
            const li = document.createElement("li");
            li.textContent = `${e.titre} (${e.places} places)`;
            liste.append(li);
          }
          statut.textContent = "";
        }

        afficher();
        EOF
  - text: >-
      Gère les erreurs : vérifie `reponse.ok` (et lance une erreur sinon), entoure le tout d'un `try … catch`, et dans le `catch` écris `Impossible de charger les événements.` dans `#statut`. Cela doit marcher avec `--erreur 500` (le serveur répond 500) comme avec `--panne` (réseau coupé), et la liste doit rester vide.
    hint: >-
      Lance `verifier-page index.html --erreur 500` : tant que le statut affiche « Chargement… » ou qu'une ligne `[ERREUR dans la page]` apparaît, c'est que l'erreur n'est pas attrapée.
    after: [2]
    checks:
      - command-succeeds: 'verifier-web 05 erreurs'
    solution:
      - |
        cat > app.js <<'EOF'
        const statut = document.querySelector("#statut");
        const liste = document.querySelector("#evenements");

        async function afficher() {
          try {
            const reponse = await fetch("https://example.org/api/evenements");
            if (!reponse.ok) {
              throw new Error(`Erreur ${reponse.status}`);
            }
            const evenements = await reponse.json();
            for (const e of evenements) {
              const li = document.createElement("li");
              li.textContent = `${e.titre} (${e.places} places)`;
              liste.append(li);
            }
            statut.textContent = "";
          } catch (erreur) {
            statut.textContent = "Impossible de charger les événements.";
          }
        }

        afficher();
        EOF
  - text: >-
      Fais fonctionner le formulaire `#inscription` : à l'envoi, empêche le rechargement, puis envoie avec `fetch` une requête `POST` vers `https://example.org/api/inscriptions`, avec l'en-tête `Content-Type: application/json` et un corps JSON `{ "email": … }` (`JSON.stringify`). Si la réponse est OK, écris `Inscription enregistrée.` dans `#statut`.
    hint: >-
      Dans l'écouteur `submit`, la fonction doit être `async` pour utiliser `await fetch(…, { method: "POST", headers: …, body: JSON.stringify({ email: … }) })`. L'adresse se lit dans `document.querySelector("#email").value`.
    after: [3]
    checks:
      - command-succeeds: 'verifier-web 05 envoi'
    solution:
      - |-
        cat > app.js <<'EOF'
        const statut = document.querySelector("#statut");
        const liste = document.querySelector("#evenements");
        const formulaire = document.querySelector("#inscription");
        const champ = document.querySelector("#email");

        async function afficher() {
          try {
            const reponse = await fetch("https://example.org/api/evenements");
            if (!reponse.ok) {
              throw new Error(`Erreur ${reponse.status}`);
            }
            const evenements = await reponse.json();
            for (const e of evenements) {
              const li = document.createElement("li");
              li.textContent = `${e.titre} (${e.places} places)`;
              liste.append(li);
            }
            statut.textContent = "";
          } catch (erreur) {
            statut.textContent = "Impossible de charger les événements.";
          }
        }

        formulaire.addEventListener("submit", async (evenement) => {
          evenement.preventDefault();
          try {
            const reponse = await fetch("https://example.org/api/inscriptions", {
              method: "POST",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify({ email: champ.value })
            });
            if (!reponse.ok) {
              throw new Error(`Erreur ${reponse.status}`);
            }
            statut.textContent = "Inscription enregistrée.";
          } catch (erreur) {
            statut.textContent = "Inscription impossible.";
          }
        });

        afficher();
        EOF
:::

## Vérifie tes acquis

:::quiz
Que se passe-t-il quand l'API répond `404` à un `fetch` ?

- [ ] `fetch` lève une erreur que `catch` attrape automatiquement
- [x] `fetch` réussit ; c'est à toi de tester `reponse.ok` ou `reponse.status`
- [ ] `fetch` renvoie `null`

> La promesse n'est rejetée qu'en cas de problème réseau. Un statut d'erreur HTTP donne une réponse valide avec `ok` à `false`.
:::

:::quiz
À quoi sert `await reponse.json()` ?

- [ ] À envoyer un objet au serveur
- [ ] À vérifier que la réponse est bien en JSON
- [x] À lire le corps de la réponse et à le convertir en objets JavaScript

> `json()` lit le corps (asynchrone) et analyse le texte JSON. Pour envoyer un objet, on utilise `JSON.stringify`.
:::

:::quiz
Où peut-on utiliser le mot-clé `await` ?

- [ ] Partout dans le code, y compris dans une fonction ordinaire
- [x] Dans une fonction déclarée `async` (ou dans un module)
- [ ] Uniquement dans un bloc `try`

> `await` suspend la fonction en attendant la promesse. Il faut donc qu'elle soit `async`, ou que le code soit dans un module JavaScript.
:::

:::quiz
Quelle méthode HTTP sert à créer une nouvelle ressource sur une API ?

- [ ] `GET`
- [ ] `DELETE`
- [x] `POST`

> `GET` lit, `POST` crée, `PUT` ou `PATCH` modifient, `DELETE` supprime.
:::
