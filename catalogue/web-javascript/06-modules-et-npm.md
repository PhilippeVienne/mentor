---
id: modules-et-npm
title: "Modules JavaScript et outils npm"
summary: "Découper le code en modules, lire et créer un package.json, installer un paquet et lancer les scripts d'un projet."
minutes: 25
objectives:
  - Exporter et importer des fonctions avec `export` et `import`
  - Lire un `package.json` (dépendances et scripts)
  - Utiliser `npm install` et `npm run` sans committer `node_modules`
---

Ton `app.js` fait désormais 400 lignes et tu cherches une fonction dans le désordre. Tous les frontends modernes (React, Angular, Next.js) découpent le code en **modules** et récupèrent des bibliothèques avec **npm**. Cette leçon utilise le terminal du labo (commandes `node` et `npm`) et un éditeur (`nano`) comme dans les leçons précédentes.

## Les modules

Un **module** est un fichier JavaScript qui **exporte** (met à disposition) ce qu'il veut partager. Les autres fichiers l'**importent** (vont le chercher).

```js
// prix.js
export function prixTotal(prix, quantite) {
  return prix * quantite;
}

export const TVA = 0.2;

export default function formater(montant) {
  return montant.toFixed(2) + " €";
}
```

```js
// app.js
import formater, { prixTotal, TVA } from "./prix.js";

const total = prixTotal(4.5, 3);
console.log(formater(total));         // 13.50 €
console.log(formater(total * (1 + TVA))); // 16.20 €
```

Ligne par ligne, pour `prix.js` :

- `export function prixTotal(…)` définit une fonction et la rend disponible aux autres fichiers sous le nom `prixTotal`.
- `export const TVA = 0.2;` partage une constante : la TVA (taxe sur la valeur ajoutée) vaut 20 %.
- `export default function formater(montant) {` partage la fonction **principale** du fichier. `montant.toFixed(2)` arrondit un nombre à 2 décimales (en texte), et `+ " €"` y colle le symbole.

Pour `app.js` :

- `import formater, { prixTotal, TVA } from "./prix.js";` va chercher le fichier `prix.js` du même dossier : `formater` (hors accolades) est l'export par défaut, `prixTotal` et `TVA` (dans les accolades) sont les exports nommés.
- `prixTotal(4.5, 3)` vaut 13,5 ; `formater(total)` l'affiche `13.50 €`.
- `total * (1 + TVA)` ajoute 20 % : `16.20 €`.

À retenir :

- `export` partage une valeur **nommée** : on l'importe entre accolades, avec le même nom.
- `export default` désigne la valeur principale du fichier (une seule par fichier) : on l'importe sans accolades, avec le nom de son choix.
- Pour un fichier local, le chemin commence par `./` ou `../`. Sans ce préfixe, `import … from "react"` désigne un **paquet** installé (voir plus bas).

Pour utiliser les modules dans une page web, on déclare le script ainsi :

```html
<script type="module" src="app.js"></script>
```

Un script `type="module"` est différé automatiquement, et `import` y est autorisé. Avec Node.js (dans le terminal), c'est le fichier `package.json`, vu juste après, qui active les modules : on y écrit `"type": "module"`.

## npm et package.json

**npm** (*Node Package Manager*) est le gestionnaire de paquets de l'écosystème Node.js. Un **paquet** est un morceau de code réutilisable publié par quelqu'un d'autre. npm télécharge ces paquets et lance les outils de ton projet. Tout est décrit dans `package.json`, à la racine du projet. C'est un fichier JSON (voir la leçon précédente) :

```json
{
  "name": "club-photo",
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build"
  },
  "dependencies": {
    "dayjs": "^1.11.0"
  },
  "devDependencies": {
    "vite": "^5.0.0"
  }
}
```

- `name` et `version` identifient ton projet.
- `type: "module"` fait comprendre `import` et `export` à Node.
- `dependencies` : paquets nécessaires à l'exécution de l'application.
- `devDependencies` : outils utiles seulement pour développer (compilation, tests, lint).
- `scripts` : des raccourcis lancés avec `npm run <nom>`. Ici `npm run dev` démarre le serveur de développement.
- `^1.11.0` est un **numéro de version** (majeure.mineure.correctif). Le `^` accepte les mises à jour mineures et correctives (1.x.y) mais pas la version majeure suivante.

Les commandes à connaître :

```shell
npm install                 # installe tout ce que package.json déclare
npm install dayjs           # ajoute une dépendance
npm install --save-dev vite # ajoute une dépendance de développement
npm run dev                 # lance le script « dev »
npm test                    # raccourci pour « npm run test »
```

`npm install` crée le dossier `node_modules/` (le code téléchargé) et le fichier `package-lock.json`, qui fige les versions exactes pour que toute l'équipe ait les mêmes.

:::info Pas d'Internet dans le labo
`npm install dayjs` télécharge le paquet depuis Internet, ce qui est impossible dans ton environnement du labo (aucun accès au réseau). Pour t'entraîner quand même, un petit paquet local, `mini-date`, est fourni dans `/opt/paquets/mini-date` : `npm install /opt/paquets/mini-date` l'installe à partir de ce dossier, exactement comme un paquet venu d'Internet.
:::

:::warning Ne committe jamais node_modules
**Committer** veut dire enregistrer une version de ton projet dans Git, l'outil de suivi de versions. Le dossier `node_modules/` est énorme et se reconstruit avec `npm install` : ajoute `node_modules/` à un fichier `.gitignore` (la liste de ce que Git doit ignorer). En revanche, **committe** `package.json` **et** `package-lock.json`.
:::

:::tip Pour retrouver ton chemin dans un frontend
Quand tu ouvres un projet inconnu, lis d'abord son `package.json` : les `scripts` te disent comment le lancer, les `dependencies` quel framework il utilise (`react`, `next`, `@angular/core`…).
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  Tu pars d'un dossier de travail vide. Tu vas créer un petit projet Node.js : un `package.json`, un module `prix.js` importé par `app.js`, un script `npm run demo`, une dépendance installée avec `npm install` et un `.gitignore`. Écris les fichiers avec `nano`. L'outil de vérification lance tes commandes et regarde ce qu'elles affichent.
steps:
  - text: >-
      Crée le projet : `npm init -y` génère un `package.json` par défaut. Active ensuite les modules avec `npm pkg set type=module`, ce qui ajoute `"type": "module"` au fichier. Vérifie avec `npm pkg get type` : la commande doit afficher `"module"`.
    hint: >-
      Deux commandes à la suite : `npm init -y` puis `npm pkg set type=module`. Tu peux lire le résultat avec `cat package.json`.
    checks:
      - output-contains: ['npm pkg get type', '"module"']
    solution:
      - npm init -y
      - npm pkg set type=module
  - text: >-
      Crée le module `prix.js` (export nommé `prixTotal` et `TVA`, export par défaut `formater`, comme dans la leçon) et `app.js` qui les importe, puis affiche `formater(total)` et `formater(total * (1 + TVA))` pour `prixTotal(4.5, 3)`. `node app.js` doit afficher `13.50 €` puis `16.20 €`.
    hint: >-
      Dans `app.js`, la première ligne est `import formater, { prixTotal, TVA } from "./prix.js";`. Sans `"type": "module"` dans `package.json`, Node refuse `import`.
    after: [1]
    checks:
      - command-succeeds: 'verifier-web 06 modules'
    solution:
      - |
        cat > prix.js <<'EOF'
        export function prixTotal(prix, quantite) {
          return prix * quantite;
        }

        export const TVA = 0.2;

        export default function formater(montant) {
          return montant.toFixed(2) + " €";
        }
        EOF
      - |
        cat > app.js <<'EOF'
        import formater, { prixTotal, TVA } from "./prix.js";

        const total = prixTotal(4.5, 3);
        console.log(formater(total));
        console.log(formater(total * (1 + TVA)));
        EOF
  - text: >-
      Ajoute un script npm nommé `demo` qui lance `node app.js` : édite la section `scripts` de `package.json` avec `nano`, ou utilise `npm pkg set scripts.demo="node app.js"`. Lance-le avec `npm run demo`.
    hint: >-
      Dans `package.json`, la section ressemble à `"scripts": { "demo": "node app.js" }`. N'oublie pas les virgules entre les propriétés si tu édites à la main.
    after: [2]
    checks:
      - command-succeeds: 'verifier-web 06 script'
    solution:
      - npm pkg set scripts.demo="node app.js"
  - text: >-
      Installe le paquet local avec `npm install /opt/paquets/mini-date`. Le paquet doit apparaître dans la section `dependencies` de `package.json` (lis-le avec `cat package.json`) et dans le dossier `node_modules/`.
    hint: >-
      La commande est exactement `npm install /opt/paquets/mini-date`. Elle fonctionne sans Internet parce que le paquet est un dossier local.
    after: [1]
    checks:
      - command-succeeds: 'verifier-web 06 paquet'
    solution:
      - npm install /opt/paquets/mini-date
  - text: >-
      Utilise le paquet dans `app.js` : ajoute `import { dateFr } from "mini-date";` en haut du fichier, puis `console.log(dateFr(new Date(Date.UTC(2026, 9, 3))));` à la fin (attention : dans `Date.UTC`, les mois commencent à 0, donc `9` est octobre). `node app.js` doit maintenant afficher `03/10/2026` en dernière ligne.
    hint: >-
      Un nom de paquet s'importe sans `./` : `from "mini-date"`. Les deux lignes `import` doivent rester en haut du fichier.
    after: [2, 4]
    checks:
      - command-succeeds: 'verifier-web 06 utiliser'
    solution:
      - |
        cat > app.js <<'EOF'
        import formater, { prixTotal, TVA } from "./prix.js";
        import { dateFr } from "mini-date";

        const total = prixTotal(4.5, 3);
        console.log(formater(total));
        console.log(formater(total * (1 + TVA)));
        console.log(dateFr(new Date(Date.UTC(2026, 9, 3))));
        EOF
  - text: >-
      Crée un fichier `.gitignore` qui contient une ligne `node_modules/`, pour que Git n'enregistre jamais le dossier des paquets installés.
    hint: >-
      `echo "node_modules/" > .gitignore` fait le travail en une commande. Vérifie avec `cat .gitignore`.
    checks:
      - env-file-contains: ['.gitignore', '(?m)^/?node_modules/?\s*$']
    solution:
      - echo "node_modules/" > .gitignore
:::

## Vérifie tes acquis

:::quiz
Comment importer une fonction `prixTotal` exportée avec un `export` nommé depuis `./prix.js` ?

- [ ] `import prixTotal from "./prix.js"`
- [x] `import { prixTotal } from "./prix.js"`
- [ ] `import "prixTotal" from "./prix.js"`

> Un export nommé s'importe entre accolades. Sans accolades, `import x from` récupère l'export par défaut.
:::

:::quiz
Où range-t-on un outil utilisé uniquement pour compiler le projet, comme `vite` ?

- [ ] Dans `dependencies`
- [ ] Dans `scripts`
- [x] Dans `devDependencies`

> `devDependencies` regroupe les outils de développement, qui ne sont pas nécessaires à l'exécution du site en production.
:::

:::quiz
Que fait `npm run build` ?

- [ ] Il installe les dépendances du projet
- [ ] Il exécute directement la commande `build` du système
- [x] Il lance le script `build` défini dans `package.json`

> Les noms de `scripts` sont libres. `npm run` exécute la commande qui y est associée.
:::

:::quiz
Quels fichiers faut-il committer dans Git ?

- [ ] `node_modules/` et `package.json`
- [ ] Uniquement `package.json`
- [x] `package.json` et `package-lock.json`, sans `node_modules/`

> `node_modules/` se régénère avec `npm install`. Le fichier de verrouillage garantit les mêmes versions pour toute l'équipe.
:::
