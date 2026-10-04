---
id: bases-javascript
titre: "JavaScript : variables, fonctions, tableaux et objets"
resume: "Les briques du langage : déclarer, calculer, boucler sur des listes et décrire des données, avec des scripts que tu lances toi-même."
duree: 30
objectifs:
  - Déclarer des variables avec `const` et `let`
  - Écrire une fonction, y compris une fonction fléchée
  - Transformer un tableau avec `map`, `filter` et `find`
  - Lire et créer des objets, y compris avec la déstructuration
---

HTML décrit, CSS habille, mais la page reste figée. **JavaScript** est un langage de programmation qui ajoute le **comportement** : réagir à un clic, calculer, charger des données. Un **programme** (ou **script**) est une suite d'instructions écrites dans un fichier, que l'ordinateur exécute de haut en bas. Avant de toucher à la page, il faut parler le langage : dans cette leçon, tes scripts s'exécutent dans le terminal avec **Node.js**, un programme qui sait lire et exécuter du JavaScript hors d'un navigateur. Tu lances un script avec `node nom-du-fichier.js`. Dans un navigateur, les mêmes exemples marchent aussi dans la **console** (touche F12).

Pour écrire tes fichiers, tu utilises toujours `nano nom-du-fichier.js` (enregistrer : `Ctrl+O` puis `Entrée` ; quitter : `Ctrl+X`), ou l'éditeur VS Code du portail s'il est proposé.

## Variables et types

Une **variable** est une boîte portant un nom, dans laquelle on range une valeur pour la réutiliser. Chaque valeur a un **type** : du texte, un nombre, vrai ou faux…

```js
const nom = "Club Photo";   // ne sera pas réaffectée
let places = 20;            // peut changer
places = places - 1;

const ouvert = true;        // booléen
const prix = 4.5;           // nombre
const rien = null;          // « volontairement vide »

console.log(`${nom} : ${places} places restantes`);
```

Ligne par ligne (tout ce qui suit `//` est un **commentaire** : l'ordinateur l'ignore, il est là pour les humains) :

- `const nom = "Club Photo";` crée une variable `nom` qui vaut le texte `"Club Photo"` (un texte s'écrit entre guillemets, c'est une **chaîne de caractères**). `const` signifie que `nom` ne pourra plus être réaffectée.
- `let places = 20;` crée une variable `places` qui vaut le nombre 20. Avec `let`, la valeur peut changer.
- `places = places - 1;` calcule `places - 1` (donc 19) et range le résultat dans `places`. Le signe `=` veut dire « range à gauche ce qui est calculé à droite », ce n'est pas une égalité mathématique.
- `const ouvert = true;` crée un **booléen**, une valeur qui ne peut être que `true` (vrai) ou `false` (faux).
- `const prix = 4.5;` crée un **nombre** (avec un point, pas une virgule).
- `const rien = null;` vaut `null`, « il n'y a volontairement rien ».
- `console.log(…)` **affiche** ce qu'on lui donne dans le terminal. C'est ton meilleur ami pour comprendre ce qui se passe.

Le résultat affiché est `Club Photo : 19 places restantes`.

- `const` par défaut ; `let` seulement si la valeur doit changer. Évite `var`, hérité des anciennes versions.
- Les accents graves (les *backticks*, touche `AltGr` + `7` sur un clavier français) créent un texte dans lequel `${…}` insère la valeur d'une variable.

## Comparer : `===`

Pour savoir si deux valeurs sont égales, on les compare :

```js
console.log(5 === "5");   // false : un nombre n'est pas un texte
console.log(5 == "5");    // true  : conversion implicite, source de bugs
```

`===` compare la valeur **et** le type : le nombre 5 n'est pas le texte `"5"`. L'égalité souple `==` convertit les types en cachette et réserve des surprises. Utilise toujours `===` et `!==` (« différent de »).

## Fonctions

Une **fonction** est une recette nommée : on lui donne des ingrédients (les **paramètres**), elle fait un calcul et rend un résultat avec `return`. On l'écrit une fois, on l'appelle autant de fois qu'on veut.

```js
function prixTotal(prix, quantite) {
  return prix * quantite;
}

const prixAvecRemise = (prix, quantite) => {
  const total = prix * quantite;
  return quantite >= 5 ? total * 0.9 : total;
};

console.log(prixTotal(4.5, 2));       // 9
console.log(prixAvecRemise(4.5, 10)); // 40.5
```

Ligne par ligne :

- `function prixTotal(prix, quantite) {` déclare une fonction nommée `prixTotal` qui reçoit deux paramètres, `prix` et `quantite`. Le contenu de la fonction est entre accolades `{ … }`.
- `return prix * quantite;` rend le produit des deux (`*` est la multiplication). La fonction s'arrête là.
- `const prixAvecRemise = (prix, quantite) => {` crée une **fonction fléchée** : la même chose écrite avec `=>` et rangée dans une constante. C'est la syntaxe la plus courante dans les frontends modernes.
- `const total = prix * quantite;` calcule un total intermédiaire, visible seulement dans la fonction.
- `return quantite >= 5 ? total * 0.9 : total;` utilise l'opérateur `? :`, un `si` en une ligne : si `quantite >= 5` (supérieur ou égal à 5), le résultat est `total * 0.9` (10 % de remise), sinon c'est `total`.
- `console.log(prixTotal(4.5, 2));` **appelle** la fonction avec 4,5 et 2 et affiche ce qu'elle rend : `9`.

## Tableaux

Un **tableau** est une liste ordonnée de valeurs, écrite entre crochets. Ses méthodes `map`, `filter` et `find` reçoivent chacune une fonction qu'elles appliquent à chaque élément.

```js
const prix = [4.5, 12, 8, 20];

const doubles = prix.map(p => p * 2);          // [9, 24, 16, 40]
const abordables = prix.filter(p => p < 10);   // [4.5, 8]
const premierCher = prix.find(p => p > 10);    // 12

prix.push(30);                                  // ajoute à la fin
console.log(prix.length);                       // 5

for (const p of prix) {
  console.log(p);
}
```

Ligne par ligne :

- `const prix = [4.5, 12, 8, 20];` crée un tableau de quatre nombres.
- `prix.map(p => p * 2)` appelle la fonction fléchée `p => p * 2` (« pour chaque `p`, rends `p * 2` ») sur chaque élément.
- `prix.filter(p => p < 10)` garde les éléments pour lesquels la fonction répond `true`.
- `prix.find(p => p > 10)` rend le premier élément pour lequel la fonction répond `true`.
- `prix.push(30)` ajoute un élément à la fin du tableau : le tableau lui-même change (c'est permis, même avec `const`, voir l'avertissement plus bas).
- `prix.length` est le nombre d'éléments.
- `for (const p of prix) { … }` est une **boucle** : elle exécute le bloc une fois pour chaque élément, `p` valant l'élément courant.

En résumé :

- `map` produit un **nouveau** tableau de même taille, chaque valeur transformée.
- `filter` garde seulement les valeurs pour lesquelles la fonction renvoie `true`.
- `find` renvoie le premier élément qui convient, ou `undefined` (« non défini ») s'il n'y en a pas.

## Objets

Un **objet** regroupe des valeurs nommées, appelées **propriétés**. C'est la forme de presque toutes les données échangées avec une API.

```js
const evenement = {
  titre: "Sortie photo",
  lieu: "Campus",
  places: 20,
  tags: ["extérieur", "débutants"]
};

console.log(evenement.titre);        // Sortie photo
console.log(evenement["lieu"]);      // Campus

const { titre, places } = evenement; // déstructuration
console.log(titre, places);          // Sortie photo 20

const copie = { ...evenement, places: 19 }; // copie avec une valeur changée
console.log(copie.places, evenement.places); // 19 20
```

Ligne par ligne :

- `const evenement = { … };` crée un objet avec quatre propriétés : chaque ligne est `nom: valeur`, séparées par des virgules. La propriété `tags` contient elle-même un tableau.
- `evenement.titre` lit la propriété `titre`. `evenement["lieu"]` est une autre écriture équivalente.
- `const { titre, places } = evenement;` est la **déstructuration** : elle crée en une ligne les variables `titre` et `places`, en recopiant les propriétés de l'objet qui portent ces noms.
- `{ ...evenement, places: 19 }` fabrique un **nouvel** objet : les trois petits points `...` recopient toutes les propriétés de `evenement`, puis `places: 19` remplace la valeur de `places`. L'original ne change pas, d'où l'affichage `19 20`.

Les tableaux d'objets sont partout :

```js
const evenements = [
  { titre: "Sortie photo", places: 20 },
  { titre: "Atelier retouche", places: 0 },
  { titre: "Exposition", places: 50 }
];

const disponibles = evenements
  .filter(e => e.places > 0)
  .map(e => e.titre);

console.log(disponibles); // [ 'Sortie photo', 'Exposition' ]
```

On **enchaîne** deux méthodes : `filter` garde les événements qui ont encore des places (`e.places > 0`), puis `map` ne retient que leur titre (`e.titre`). Le résultat est un tableau de titres.

:::warning const ne rend pas l'objet immuable
`const` interdit de **réaffecter** la variable, pas de modifier le contenu d'un tableau ou d'un objet. `const liste = []; liste.push(1);` fonctionne. En revanche, `liste = [];` provoque une erreur. Pour créer une version modifiée sans changer l'originale, utilise `{ ...objet }` ou `map` / `filter`. C'est la base de la gestion d'état de React.
:::

## Entraîne-toi

:::labo
moteur: reel
intro: |
  Tu vas écrire de petits scripts et les lancer avec `node`. Ton dossier contient déjà `evenements.js` (des données à compléter), `statistiques.js` (deux fonctions à écrire) et `statistiques.test.js` (les tests de la dernière étape). Écris les autres fichiers avec `nano`. L'outil de vérification regarde ce que tes scripts **affichent** ou **renvoient**, pas la façon dont tu les as écrits, mais il les essaie aussi avec d'autres valeurs que celles de l'énoncé : écrire le résultat « en dur » ne valide pas l'étape.
commandes:
  - cp -R /opt/exercices/03-bases-javascript/. .
etapes:
  - texte: >-
      Crée `boutique.js` avec une constante `nom` (`"Club Photo"`), une variable `places` (`20`) que tu diminues de 1, puis un `console.log` avec un texte à backticks qui affiche exactement `Club Photo : 19 places restantes`. Lance `node boutique.js`.
    indice: >-
      ``console.log(`${nom} : ${places} places restantes`);`` après avoir écrit `places = places - 1;`. Les backticks sont obligatoires pour que `${…}` fonctionne.
    verif:
      - commande-reussit: 'verifier-web 03 boutique'
    solution:
      - |
        cat > boutique.js <<'EOF'
        const nom = "Club Photo";
        let places = 20;
        places = places - 1;
        console.log(`${nom} : ${places} places restantes`);
        EOF
  - texte: >-
      Crée `prix.js` avec la fonction `prixTotal(prix, quantite)` et la fonction fléchée `prixAvecRemise(prix, quantite)` (10 % de remise à partir de 5 articles), puis affiche `prixTotal(4.5, 2)` et `prixAvecRemise(4.5, 10)`, un résultat par ligne. `node prix.js` doit afficher `9` puis `40.5`.
    indice: >-
      Reprends les deux fonctions de la leçon et termine par deux `console.log(…)`, un par appel.
    verif:
      - commande-reussit: 'verifier-web 03 prix'
    solution:
      - |
        cat > prix.js <<'EOF'
        function prixTotal(prix, quantite) {
          return prix * quantite;
        }

        const prixAvecRemise = (prix, quantite) => {
          const total = prix * quantite;
          return quantite >= 5 ? total * 0.9 : total;
        };

        console.log(prixTotal(4.5, 2));
        console.log(prixAvecRemise(4.5, 10));
        EOF
  - texte: >-
      Ouvre `evenements.js` (le tableau `evenements` est déjà écrit) et ajoute sous la ligne de consigne : une constante `disponibles` (les titres des événements qui ont encore des places, avec `filter` puis `map`) et une constante `grand` (le premier événement qui a plus de 30 places, avec `find`). Affiche `disponibles`, puis `grand.titre`. `node evenements.js` doit afficher `[ 'Sortie photo', 'Exposition' ]` puis `Exposition`.
    indice: >-
      `const disponibles = evenements.filter(e => e.places > 0).map(e => e.titre);` puis `const grand = evenements.find(e => e.places > 30);` et deux `console.log`.
    verif:
      - commande-reussit: 'verifier-web 03 evenements'
    solution:
      - |
        cat >> evenements.js <<'EOF'
        const disponibles = evenements.filter(e => e.places > 0).map(e => e.titre);
        const grand = evenements.find(e => e.places > 30);
        console.log(disponibles);
        console.log(grand.titre);
        EOF
  - texte: >-
      Crée `copie.js` : un objet `evenement` avec `titre: "Sortie photo"` et `places: 20`, une déstructuration `const { titre, places } = evenement;` qui affiche `Sortie photo 20`, puis une copie `{ ...evenement, places: 19 }` dont tu affiches les places suivies de celles de l'original : `19 20`.
    indice: >-
      `console.log(titre, places);` met un espace entre les deux valeurs. Même principe pour `console.log(copie.places, evenement.places);`.
    verif:
      - commande-reussit: 'verifier-web 03 copie'
    solution:
      - |
        cat > copie.js <<'EOF'
        const evenement = { titre: "Sortie photo", places: 20 };
        const { titre, places } = evenement;
        console.log(titre, places);
        const copie = { ...evenement, places: 19 };
        console.log(copie.places, evenement.places);
        EOF
  - texte: >-
      Complète les deux fonctions de `statistiques.js` : `moyenne(nombres)` (la moyenne, ou `0` pour un tableau vide) et `titresDisponibles(evenements)` (les titres des événements qui ont encore des places). Lance les tests avec `node --test statistiques.test.js` : ils exécutent ton fichier et vérifient ce que renvoient tes fonctions.
    indice: >-
      Pour la moyenne, additionne avec une boucle `for…of` puis divise par `nombres.length`. Pour les titres, c'est le même `filter` puis `map` qu'à l'étape 3, dans une fonction qui `return` le résultat.
    verif:
      - commande-reussit: 'verifier-web 03 stats'
    solution:
      - |
        cat > statistiques.js <<'EOF'
        function moyenne(nombres) {
          if (nombres.length === 0) {
            return 0;
          }
          let somme = 0;
          for (const n of nombres) {
            somme = somme + n;
          }
          return somme / nombres.length;
        }

        function titresDisponibles(evenements) {
          return evenements.filter(e => e.places > 0).map(e => e.titre);
        }
        EOF
:::

## Vérifie tes acquis

:::quiz
Que vaut `[1, 2, 3, 4].filter(n => n % 2 === 0)` ?

- [ ] `[1, 3]`
- [ ] `[2, 4, 6, 8]`
- [x] `[2, 4]`

> `filter` garde les éléments pour lesquels la fonction renvoie `true` : ici les nombres pairs. Il n'en change pas les valeurs (c'est le rôle de `map`).
:::

:::quiz
Que se passe-t-il avec `const total = 1; total = 2;` ?

- [x] Une erreur : on ne peut pas réaffecter une constante
- [ ] `total` vaut 2 sans erreur
- [ ] `total` reste à 1 sans erreur

> Réaffecter une variable `const` lève une `TypeError`. Il faut `let` si la valeur doit changer.
:::

:::quiz
Quel opérateur de comparaison faut-il privilégier en JavaScript ?

- [ ] `==`, car il est plus tolérant
- [ ] `=`, qui compare les deux valeurs
- [x] `===`, qui compare la valeur et le type

> `=` affecte une valeur. `==` convertit les types et donne des résultats déroutants (`0 == ""` est vrai). `===` est prévisible.
:::

:::quiz
Que contient `titre` après `const { titre } = { titre: "Expo", places: 5 };` ?

- [ ] L'objet complet `{ titre: "Expo", places: 5 }`
- [ ] `undefined`, car il manque le nom de l'objet
- [x] `"Expo"`

> La déstructuration extrait la propriété du même nom dans une variable.
:::
