---
title: "Examen de validation — HTML, CSS et JavaScript"
draw: 5
pass_mark: 80
minutes: 10
shuffle: true
---

Cet examen valide les bases du web : structure HTML, mise en forme CSS, JavaScript, DOM, appels d'API avec `fetch` et outils `npm`.

:::quiz
Quelle balise HTML indique le contenu principal d'une page, à n'utiliser qu'une fois ?

- [ ] `<section>`
- [x] `<main>`
- [ ] `<article>`

> `<main>` désigne le contenu principal. `<section>` et `<article>` peuvent se répéter.
:::

:::quiz
Une image ne charge pas : que s'affiche-t-il à sa place si `alt="Logo du club"` est renseigné ?

- [x] Le texte « Logo du club »
- [ ] Le nom du fichier de l'image
- [ ] Rien : `alt` n'est lu que par les moteurs de recherche

> Le texte alternatif remplace l'image si elle est indisponible, et il est lu par les lecteurs d'écran.
:::

:::quiz
Un `<h1>` est suivi directement d'un `<h4>`, pour avoir un titre plus petit. Quel est le problème ?

- [ ] Aucun : les niveaux servent uniquement à la taille du texte
- [ ] Un seul niveau de titre est autorisé par page
- [x] Le niveau saute, ce qui casse le plan de la page ; la taille se règle en CSS

> Les niveaux de titres structurent le document. L'apparence est le rôle de CSS.
:::

:::quiz
Quel est l'intérêt de `<label for="email">` associé à `<input id="email">` ?

- [x] Un clic sur le libellé active le champ, et les lecteurs d'écran annoncent le libellé
- [ ] Le champ devient obligatoire
- [ ] Le champ est vérifié comme une adresse e-mail

> `for` et `id` relient le libellé au champ. L'obligation passe par `required`, la vérification d'e-mail par `type="email"`.
:::

:::quiz
Deux règles CSS ciblent le même élément avec la même spécificité : `p { color: red; }` puis, plus bas, `p { color: blue; }`. Quelle couleur s'applique ?

- [ ] Rouge, car c'est la première règle
- [x] Bleu, car à spécificité égale la dernière règle gagne
- [ ] Rouge, car les couleurs en anglais sont prioritaires

> C'est la « cascade » : à spécificité égale, l'ordre d'écriture décide.
:::

:::quiz
Un élément a `width: 300px`, `padding: 20px` de chaque côté et `box-sizing: border-box`. Quelle largeur occupe-t-il, sans bordure ?

- [ ] 340 px
- [x] 300 px
- [ ] 320 px

> Avec `border-box`, `width` inclut le `padding` et la bordure. Avec `content-box`, il aurait occupé 340 px.
:::

:::quiz
Tu veux afficher des cartes sur trois colonnes égales, quelle déclaration utilises-tu sur le conteneur ?

- [ ] `display: flex; columns: 3`
- [ ] `display: block; grid: 3`
- [x] `display: grid; grid-template-columns: repeat(3, 1fr)`

> `repeat(3, 1fr)` crée trois colonnes qui se partagent l'espace disponible.
:::

:::quiz
Que fait cette media query : `@media (max-width: 600px) { .menu { flex-direction: column; } }` ?

- [x] Le menu passe en colonne sur les écrans d'au plus 600 px
- [ ] Le menu passe en colonne sur les écrans d'au moins 600 px
- [ ] Le menu passe en colonne sur tous les écrans

> `max-width` fixe une largeur maximale : la règle vise les petits écrans.
:::

:::quiz
Que vaut `[3, 8, 12].map(n => n * 2)` ?

- [ ] `[12, 24]`
- [x] `[6, 16, 24]`
- [ ] `[3, 8, 12, 6, 16, 24]`

> `map` renvoie un nouveau tableau de même taille, chaque élément étant transformé.
:::

:::quiz
Que renvoie `[5, 12, 8, 20].find(n => n > 10)` ?

- [ ] `[12, 20]`
- [ ] `true`
- [x] `12`

> `find` renvoie le premier élément qui satisfait la condition. Pour tous les éléments, c'est `filter`.
:::

:::quiz
Pourquoi `const evenement = { places: 5 }; evenement.places = 4;` ne provoque-t-il pas d'erreur ?

- [ ] Parce que `const` ne s'applique pas aux objets
- [x] Parce que `const` interdit la réaffectation de la variable, pas la modification de l'objet
- [ ] Parce que JavaScript ignore silencieusement les erreurs

> `evenement = {}` lèverait une erreur, mais modifier une propriété de l'objet est permis.
:::

:::quiz
Quelle expression est vraie ?

- [x] `5 == "5"`
- [ ] `5 === "5"`
- [ ] `5 !== 5`

> `==` convertit le texte en nombre. `===` compare aussi le type et donne `false`. C'est pourquoi on préfère `===`.
:::

:::quiz
Tu affiches dans la page un commentaire saisi par une autre personne. Quelle affectation est sûre ?

- [x] `element.textContent = commentaire`
- [ ] `element.innerHTML = commentaire`
- [ ] `element.outerHTML = commentaire`

> `textContent` traite la saisie comme du texte. `innerHTML` et `outerHTML` interprètent le HTML et ouvrent la porte à une faille XSS.
:::

:::quiz
Un `fetch` reçoit une réponse `500`. Dans quel cas la promesse est-elle rejetée ?

- [ ] Pour tout statut supérieur ou égal à 400
- [ ] Pour le statut 500 uniquement
- [x] Seulement en cas de problème réseau ; un statut 500 donne une réponse avec `ok` à `false`

> `fetch` ne rejette pas sur les erreurs HTTP. Il faut tester `reponse.ok` soi-même.
:::

:::quiz
Dans `import formater, { prixTotal } from "./prix.js"`, que désigne `formater` ?

- [ ] Un export nommé appelé `formater`
- [ ] Un paquet npm installé dans `node_modules/`
- [x] L'export par défaut du module `./prix.js`

> Sans accolades, on importe l'export par défaut sous le nom de son choix. Les accolades désignent les exports nommés.
:::

:::quiz
Dans quel champ de `package.json` déclare-t-on un raccourci lancé par `npm run dev` ?

- [ ] `dependencies`
- [x] `scripts`
- [ ] `devDependencies`

> `scripts` associe un nom à une commande. `dependencies` et `devDependencies` listent des paquets.
:::
