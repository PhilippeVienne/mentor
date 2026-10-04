---
titre: "Examen de validation — TypeScript"
tirage: 5
seuil: 80
duree: 10
melange: true
---

Cet examen valide les bases de TypeScript dans l'équipe : types, unions, interfaces, fonctions, génériques, `tsconfig.json`, typage des réponses d'API, ESLint et Prettier.

:::quiz
Quel mot-clé permet de donner un nom à une union comme `"a" | "b"` ?

- [ ] `enum`
- [x] `type`
- [ ] `const`
- [ ] `let`

> `type Statut = "a" | "b"` crée un alias réutilisable.
:::

:::quiz
Que produit `tsc` à partir d'un fichier `.ts` correct ?

- [ ] Un fichier `.ts` optimisé et plus léger
- [ ] Un binaire exécutable pour le système
- [ ] Un fichier `.json` de types
- [x] Un fichier `.js`, sans types

> Les types sont effacés : le navigateur ne reçoit que du JavaScript.
:::

:::quiz
Que signifie `lieu: string | null` dans une interface ?

- [x] Obligatoire, mais `null` est permis
- [ ] Le champ peut être absent de l'objet sans erreur
- [ ] Le champ est un tableau
- [ ] Le champ ne peut contenir que `null`

> Obligatoire, mais `null` est une valeur permise. Pour un champ omissible, on écrit `lieu?: string`.
:::

:::quiz
Un paramètre `valeur: string | number` : comment appeler `valeur.toFixed(2)` ?

- [ ] Directement : `tsc` ajoute seul la vérification nécessaire
- [ ] En ajoutant `?` après `valeur`
- [x] Après un test `typeof`
- [ ] Impossible, `toFixed` n'existe pas

> Le test `typeof` réduit l'union à `number` dans la branche concernée.
:::

:::quiz
Quel est l'inconvénient principal de `any` ?

- [ ] Il ralentit fortement le programme à l'exécution
- [ ] Il est interdit par `tsc`
- [ ] Il convertit tout en texte
- [x] Il désactive les contrôles

> Une valeur `any` accepte tout : les erreurs réapparaissent à l'exécution.
:::

:::quiz
Que renvoie une fonction annotée `: void` ?

- [ ] Toujours `null`
- [x] Rien d'utile
- [ ] Un tableau vide
- [ ] Une `Promise`

> `void` signale qu'on n'utilise pas la valeur de retour.
:::

:::quiz
Que vaut `T` dans `premier(["a", "b"])` pour `function premier<T>(l: T[]): T | undefined` ?

- [ ] `any`
- [ ] `unknown`
- [x] `string`
- [ ] `string[]`

> `tsc` infère `T` à partir de l'argument : un tableau de `string` donne `T = string`.
:::

:::quiz
Quelle est la raison d'être d'un générique ?

- [x] Réutiliser un code pour plusieurs types
- [ ] Accélérer l'exécution du JavaScript produit
- [ ] Ignorer les erreurs de type dans certains fichiers
- [ ] Importer une bibliothèque externe de types

> Le générique évite de dupliquer le code tout en conservant des types précis.
:::

:::quiz
Quel type décrit une fonction `async` qui renvoie un nombre ?

- [ ] `number`
- [ ] `void`
- [ ] `Array<number>`
- [x] `Promise<number>`

> Une fonction `async` renvoie toujours une `Promise` de sa valeur.
:::

:::quiz
Que fait `Partial<Evenement>` ?

- [ ] Retire les propriétés de type texte
- [x] Rend toutes les propriétés facultatives
- [ ] Rend l'objet immuable
- [ ] Duplique l'interface

> Pratique pour décrire un objet de modifications partielles.
:::

:::quiz
Dans un `tsconfig.json`, quelle option active le plus de vérifications d'un coup ?

- [x] `strict`
- [ ] `skipLibCheck`
- [ ] `noEmit`
- [ ] `include`

> `strict` regroupe notamment `strictNullChecks` et `strictPropertyInitialization`.
:::

:::quiz
Que fait `skipLibCheck: true` ?

- [ ] Ignore tous les fichiers du projet écrits en TypeScript
- [ ] Désactive le mode `strict` pour tout le projet
- [x] Saute le contrôle des types des bibliothèques
- [ ] Supprime le dossier `node_modules` à chaque build

> Cela accélère la vérification : seuls les types de ton propre code sont contrôlés en profondeur.
:::

:::quiz
À quoi sert `"paths": { "@/*": ["./src/*"] }` ?

- [ ] À exclure le dossier `src` de la vérification
- [x] À créer un raccourci d'import
- [ ] À renommer les fichiers
- [ ] À installer des paquets listés dans src

> On écrit `@/db/types` au lieu d'un chemin relatif comme `../../db/types`.
:::

:::quiz
Quelle commande vérifie les types d'un projet sans produire de JavaScript ?

- [ ] `npx prettier --check`
- [ ] `npx eslint --fix`
- [ ] `npm audit`
- [x] `npx tsc --noEmit`

> `--noEmit` limite `tsc` à la vérification.
:::

:::quiz
`const donnees = (await reponse.json()) as Evenement[]` : que se passe-t-il si le serveur renvoie un autre format ?

- [ ] `tsc` détecte l'erreur dès la compilation du projet
- [ ] Une exception est levée automatiquement par le moteur JavaScript
- [x] Rien n'est détecté à l'exécution
- [ ] Le JSON est converti

> `as` n'ajoute aucune vérification à l'exécution.
:::

:::quiz
Pourquoi typer le résultat de `JSON.parse` en `unknown` plutôt qu'en `any` ?

- [x] Pour obliger à vérifier avant usage
- [ ] Pour que le JSON soit validé automatiquement à l'exécution
- [ ] Pour accélérer l'analyse
- [ ] Pour éviter les exceptions de syntaxe de l'analyse

> Avec `unknown`, accéder à une propriété sans test est refusé.
:::

:::quiz
Que signifie le retour `valeur is Evenement` ?

- [ ] La fonction renvoie un nouvel objet de type `Evenement`
- [x] Si `true`, la valeur est un `Evenement` pour `tsc`
- [ ] La fonction vérifie seule tous les champs de la valeur
- [ ] La valeur est convertie

> C'est à l'auteur·rice de la fonction d'écrire des tests corrects.
:::

:::quiz
Quel outil reformate le code (indentation, guillemets, virgules finales) ?

- [ ] ESLint seul
- [ ] `tsc`
- [ ] Node.js
- [x] Prettier

> `prettier --write` réécrit les fichiers selon le `.prettierrc`.
:::

:::quiz
Quel outil signale une variable inutilisée ou un `any` explicite ?

- [x] ESLint
- [ ] Prettier
- [ ] Le navigateur
- [ ] `npm install`

> Ce sont des règles de lint (`no-unused-vars`, `no-explicit-any`).
:::

:::quiz
Que fait `prettier --check .` ?

- [ ] Il reformate tout le projet en place
- [ ] Il corrige les types du projet entier
- [x] Il signale les fichiers mal formatés
- [ ] Il supprime les commentaires et les lignes vides

> Pratique en vérification automatique : il échoue si un fichier n'est pas formaté.
:::

:::quiz
Tu vois `Object is possibly 'null'` ou `'x' is possibly 'null'`. Quelle correction est la bonne ?

- [ ] Ajouter `as any`
- [ ] Désactiver `strict`
- [ ] Supprimer le type de `x` dans le fichier
- [x] Tester `x !== null` avant usage

> Le test réduit le type ; les autres solutions masquent le problème.
:::

:::quiz
Que signifie `reponse.ok` après un `fetch` ?

- [ ] Le JSON reçu est valide et complet
- [x] Le statut HTTP est un succès (2xx)
- [ ] Le type est correct
- [ ] Le serveur est en HTTPS

> Il faut le vérifier avant de lire le corps : un statut 404 ou 500 renvoie aussi une réponse.
:::

:::quiz
Dans MiniShop, comment sont produits les types de la base de données (`src/db/types.ts`) ?

- [ ] Ils sont écrits à la main, table par table
- [ ] Ils sont téléchargés depuis un paquet npm public
- [x] Générés depuis le schéma de la base
- [ ] Ils sont déduits à chaque requête SQL envoyée

> Le script `generate-types` les génère ; l'en-tête demande de ne pas les modifier à la main.
:::
