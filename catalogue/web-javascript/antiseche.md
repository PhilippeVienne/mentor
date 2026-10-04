## HTML

| Balise | Rôle |
| --- | --- |
| `<header>`, `<nav>`, `<main>`, `<footer>` | Structure sémantique de la page |
| `<h1>` à `<h6>` | Titres (un seul `<h1>`, pas de niveau sauté) |
| `<a href="…">` | Lien |
| `<img src="…" alt="…">` | Image avec texte alternatif |
| `<label for="id">` + `<input id="id">` | Champ de formulaire relié à son libellé |

## CSS

| Besoin | Syntaxe |
| --- | --- |
| Cibler une balise, une classe, un id | `p`, `.classe`, `#id` |
| Boîte qui inclut marge intérieure et bordure | `box-sizing: border-box` |
| Aligner sur une ligne | `display: flex; gap: 1rem; justify-content: space-between` |
| Grille de 3 colonnes égales | `display: grid; grid-template-columns: repeat(3, 1fr)` |
| Règle pour écran large | `@media (min-width: 768px) { … }` |

## JavaScript

| Besoin | Syntaxe |
| --- | --- |
| Constante, variable | `const x = 1;` `let y = 2;` |
| Comparer | `===` et `!==` |
| Fonction fléchée | `const f = (a, b) => a + b;` |
| Transformer, filtrer, chercher | `liste.map(…)` `liste.filter(…)` `liste.find(…)` |
| Déstructurer, copier | `const { titre } = objet;` `{ ...objet, places: 1 }` |

## DOM et événements

| Besoin | Syntaxe |
| --- | --- |
| Sélectionner | `document.querySelector("#id")` `querySelectorAll(".classe")` |
| Changer un texte (sûr) | `element.textContent = "…"` |
| Gérer une classe | `element.classList.add("alerte")` |
| Écouter un événement | `element.addEventListener("click", () => { … })` |
| Empêcher l'envoi d'un formulaire | `evenement.preventDefault()` |
| Créer et ajouter | `document.createElement("li")` puis `liste.append(li)` |

## fetch

```js
const reponse = await fetch(url);
if (!reponse.ok) throw new Error(`Erreur ${reponse.status}`);
const donnees = await reponse.json();
```

## Modules et npm

| Besoin | Syntaxe |
| --- | --- |
| Export nommé, par défaut | `export const x = 1;` `export default f;` |
| Import | `import f, { x } from "./fichier.js";` |
| Installer | `npm install` · `npm install paquet` · `npm install --save-dev paquet` |
| Lancer un script | `npm run dev` |
| À committer | `package.json` et `package-lock.json`, jamais `node_modules/` |

## Dans le labo

| Besoin | Commande |
| --- | --- |
| Écrire un fichier | `nano fichier` (`Ctrl+O` puis `Entrée` pour enregistrer, `Ctrl+X` pour quitter) |
| « Ouvrir » une page | `verifier-page index.html` |
| Cliquer, remplir, lire un style | `verifier-page index.html --clic "#id" --saisie "#champ" texte --style .classe color rouge` |
| Lancer un script | `node script.js` |
| Lancer les tests | `node --test fichier.test.js` |
