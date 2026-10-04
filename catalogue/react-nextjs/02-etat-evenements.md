---
id: etat-evenements
titre: "L'état et les événements"
resume: "Faire réagir un composant aux clics grâce à `useState`, sans jamais modifier les données en place."
duree: 35
objectifs:
  - Expliquer la différence entre une prop et un état
  - Utiliser `useState` et réagir à un événement (`onClick`)
  - Mettre à jour un tableau ou un objet de l'état sans le modifier en place
  - Calculer une valeur à partir de l'état plutôt que la stocker
---

Tu as un bouton « + » pour choisir une quantité. Tu cliques : le nombre doit changer à l'écran. Un composant qui ne fait que recevoir des props ne peut pas faire ça, car ses props ne changent pas d'elles-mêmes. Il lui faut une mémoire : c'est l'**état**.

## À quoi sert l'état

L'**état** (*state*) est une donnée que le composant **possède** et qui peut changer au fil du temps : la quantité choisie, le contenu du panier, le fait qu'un menu soit ouvert. Quand l'état change, React **rappelle ton composant** avec la nouvelle valeur et met l'écran à jour. Tu ne touches jamais à la page à la main : tu changes l'état, React s'occupe du reste.

| | Prop | État |
| --- | --- | --- |
| Qui la fournit ? | Le composant parent | Le composant lui-même |
| Peut-elle changer ? | Seulement si le parent la change | Oui, via la fonction de mise à jour |
| Exemple | `nom` d'un goodie | quantité choisie |

## `useState` ligne à ligne

`useState` est un **hook** : une fonction fournie par React, dont le nom commence par `use`, qui donne une capacité à un composant (ici, avoir un état). Voici un sélecteur de quantité, comme celui du bouton d'ajout au panier de MiniShop (qui commence lui aussi par `const [qty, setQty] = useState(1)`).

```tsx
import { useState } from "react";

function ChoixQuantite({
  stock,
  onAjouter,
}: Readonly<{ stock: number; onAjouter: (quantite: number) => void }>) {
  const [quantite, setQuantite] = useState(1);

  return (
    <div>
      <button type="button" disabled={quantite <= 1} onClick={() => setQuantite((q) => Math.max(1, q - 1))}>
        −
      </button>
      <span>{quantite}</span>
      <button type="button" disabled={quantite >= stock} onClick={() => setQuantite((q) => Math.min(stock, q + 1))}>
        +
      </button>
      <button
        type="button"
        onClick={() => {
          onAjouter(quantite);
          setQuantite(1);
        }}
      >
        Ajouter au panier
      </button>
    </div>
  );
}
```

- `const [quantite, setQuantite] = useState(1);` : `useState(1)` crée un état dont la valeur de départ est `1`. Il renvoie une paire : la **valeur actuelle** (`quantite`) et la **fonction pour la changer** (`setQuantite`). On les nomme avec la convention `x` / `setX`.
- `onClick={() => …}` : un **gestionnaire d'événement**. Tu donnes à React la fonction à appeler quand la personne clique. Attention : on passe la fonction (`() => …`), on ne l'appelle pas (`setQuantite(2)` écrit directement dans `onClick` s'exécuterait à chaque affichage).
- `setQuantite((q) => Math.max(1, q - 1))` : on passe une fonction qui reçoit l'ancienne valeur `q` et renvoie la nouvelle. `Math.max(1, …)` empêche de descendre sous 1, `Math.min(stock, …)` de dépasser le stock.
- `type="button"` : un bouton placé dans un formulaire l'enverrait par défaut ; `type="button"` dit qu'il sert seulement à cliquer.
- `disabled={quantite <= 1}` : le bouton se grise tout seul quand la condition est vraie. L'affichage est **déduit** de l'état.
- `{quantite}` entre les balises `<span>` affiche la valeur actuelle de l'état. À chaque changement, React rappelle le composant et le `<span>` se met à jour.
- `onAjouter` est une prop de type fonction : le composant prévient son parent (« la personne veut ajouter 3 exemplaires ») sans savoir ce que le parent en fait. Les données descendent, les événements remontent.

Au premier affichage, ce composant produit le HTML suivant (`renderToStaticMarkup` ne montre que l'état initial) :

```console
<div><button type="button" disabled="">−</button><span>1</span><button type="button">+</button><button type="button">Ajouter au panier</button></div>
```

On voit bien `1` et le premier bouton désactivé.

## Ne modifie jamais l'état en place

Pour un tableau (le panier) ou un objet, la règle est : **on ne modifie pas l'ancien, on en fabrique un nouveau**. React compare l'ancienne et la nouvelle valeur ; si c'est la même référence (le même tableau modifié), il croit que rien n'a changé et ne met pas l'écran à jour.

Voici la logique d'un panier, écrite comme des fonctions pures (qui ne modifient rien et renvoient un résultat). Elle suit celle de `cartStore.ts` de MiniShop (`addItem` : si l'article est déjà là, la quantité augmente sans dépasser le maximum).

```ts
type Ligne = { id: number; nom: string; prixCents: number; stock: number; quantite: number };
type LigneEntree = Omit<Ligne, "quantite">;

export function ajouter(panier: Ligne[], entree: LigneEntree): Ligne[] {
  if (entree.stock <= 0) return panier;
  const index = panier.findIndex((l) => l.id === entree.id);
  if (index === -1) {
    return [...panier, { ...entree, quantite: 1 }];
  }
  return panier.map((l, i) =>
    i === index ? { ...l, quantite: Math.min(l.quantite + 1, l.stock) } : l,
  );
}

export function retirer(panier: Ligne[], id: number): Ligne[] {
  return panier.filter((l) => l.id !== id);
}
```

- `type Ligne` décrit une ligne du panier ; `LigneEntree = Omit<Ligne, "quantite">` est la même chose sans la quantité (ce que fournit une fiche produit). `export` rend la fonction utilisable depuis un autre fichier.
- `if (entree.stock <= 0) return panier;` : un article épuisé ne s'ajoute pas, on renvoie le panier tel quel.
- `panier.findIndex(…)` renvoie la position de la première ligne qui vérifie la condition, ou `-1` s'il n'y en a aucune.
- `[...panier, nouvelleLigne]` : la **syntaxe de décomposition** `...` recopie les éléments d'un tableau dans un nouveau tableau. L'original ne bouge pas.
- `{ ...l, quantite: … }` : même idée pour un objet. On recopie tout, puis on remplace une seule propriété.
- `panier.filter((l) => l.id !== id)` garde toutes les lignes sauf celle qu'on retire : c'est le contraire de `push`, sans rien modifier.
- `map` et `filter` renvoient toujours un **nouveau** tableau. Évite `push`, `splice` et `panier[0].quantite = 2`, qui modifient en place.

Essayons-les avec une gourde dont le stock est de 2 (les fonctions `totalArticles` et `totalCents` sont expliquées juste après) :

```ts
const gourde = { id: 1, nom: "Gourde Éco", prixCents: 1250, stock: 2 };
const p1 = ajouter([], gourde);
const p2 = ajouter(p1, gourde);
const p3 = ajouter(p2, gourde);
console.log(p1[0].quantite, p2[0].quantite, p3[0].quantite);
console.log(p1 === p2);
console.log(totalArticles(p3), totalCents(p3));
```

```console
1 2 2
false
2 2500
```

Lecture : la quantité monte à 2 puis reste plafonnée par le stock. `p1 === p2` est `false` : chaque ajout a produit un nouveau tableau, et `p1` n'a jamais été modifié.

## Calculer plutôt que stocker

Le nombre total d'articles se **déduit** du panier. Ne le stocke pas dans un second état, tu risquerais d'oublier de le mettre à jour.

```ts
const totalArticles = (panier: Ligne[]) => panier.reduce((somme, l) => somme + l.quantite, 0);
const totalCents = (panier: Ligne[]) => panier.reduce((somme, l) => somme + l.prixCents * l.quantite, 0);
```

`reduce` parcourt le tableau en accumulant une valeur (`somme`, qui démarre à `0`). Avec deux gourdes à 12,50 € : `totalArticles` vaut `2` et `totalCents` vaut `2500`. MiniShop a exactement ces deux calculs, `selectTotalItems` et `selectTotalPrice`.

:::tip Quand l'état doit être partagé : un store
Si plusieurs composants éloignés ont besoin du même état (le panier est affiché dans l'en-tête, dans la fiche produit et dans la caisse), le remonter de parent en parent devient pénible. MiniShop utilise **Zustand**, une petite bibliothèque de « store » : un état global auquel n'importe quel composant se branche avec `useCartStore(...)`. Le principe reste identique : on ne modifie jamais en place, et chaque action (`addItem`, `removeItem`…) fabrique un nouvel état. Le panier y est aussi enregistré dans le `localStorage` du navigateur (un petit espace de stockage que le navigateur garde sur l'ordinateur de la personne, même après la fermeture de l'onglet) pendant 48 heures.
:::

## Entraîne-toi

:::labo
moteur: reel
intro: |
  Démarre ton environnement. Tu vas écrire la logique d'un panier de goodies (`src/panier.ts`) puis un sélecteur de quantité (`src/ChoixQuantite.tsx`), avec `nano` (`Ctrl+O` puis `Entrée` enregistre, `Ctrl+X` quitte).

  Il n'y a pas de navigateur ici : des **tests** (de petits programmes qui vérifient ton travail) cliquent à ta place sur les boutons de ta page fictive, avec `@testing-library`, et regardent ce qui s'affiche. `npx vitest run panier-ajouter` lance les tests dont le nom de fichier contient `panier-ajouter`. Les tests de la logique du panier **gèlent** leurs données (`Object.freeze`) : si ton code modifie un tableau en place au lieu d'en fabriquer un nouveau, JavaScript lève une erreur et le test échoue.
commandes:
  - cp -R /opt/exercices/commun/. .
  - cp -R /opt/exercices/02-etat-evenements/. .
  - lier-outils
etapes:
  - texte: >-
      Dans `src/panier.ts`, écris `ajouter(panier, entree)` : elle renvoie un **nouveau** tableau. Un stock nul ou négatif ne change rien ; un article absent du panier devient une ligne de quantité 1 ; un article déjà présent gagne une unité, sans jamais dépasser son stock. Vérifie avec `npx vitest run panier-ajouter`.
    indice: >-
      Cherche l'article avec `panier.findIndex(…)`. S'il est absent : `[...panier, { ...entree, quantite: 1 }]`. Sinon : `panier.map((l, i) => i === index ? { ...l, quantite: Math.min(l.quantite + 1, l.stock) } : l)`.
    verif:
      - commande-reussit: controler tests 02-etat-evenements panier-ajouter
    solution:
      - ecrire:
          'src/panier.ts': |
            // La logique du panier, écrite comme des fonctions pures : elles ne modifient jamais leurs arguments.
            export type Ligne = { id: number; nom: string; prixCents: number; stock: number; quantite: number };
            export type LigneEntree = Omit<Ligne, "quantite">;

            export function ajouter(panier: Ligne[], entree: LigneEntree): Ligne[] {
              if (entree.stock <= 0) return panier;
              const index = panier.findIndex((l) => l.id === entree.id);
              if (index === -1) {
                return [...panier, { ...entree, quantite: 1 }];
              }
              return panier.map((l, i) =>
                i === index ? { ...l, quantite: Math.min(l.quantite + 1, l.stock) } : l,
              );
            }

            export function retirer(panier: Ligne[], id: number): Ligne[] {
              return panier;
            }

            export const totalArticles = (panier: Ligne[]): number => 0;
            export const totalCents = (panier: Ligne[]): number => 0;
  - texte: >-
      Toujours dans `src/panier.ts`, écris `retirer(panier, id)` (un nouveau tableau sans la ligne d'identifiant `id`), puis `totalArticles` et `totalCents`, qui **calculent** le nombre d'articles et le prix total à partir du panier (sans rien stocker). Vérifie avec `npx vitest run panier-totaux`.
    indice: >-
      `retirer` utilise `panier.filter((l) => l.id !== id)`. Les totaux utilisent `panier.reduce((somme, l) => somme + …, 0)` : `l.quantite` pour le nombre d'articles, `l.prixCents * l.quantite` pour le prix.
    verif:
      - commande-reussit: controler tests 02-etat-evenements panier-totaux
    solution:
      - ecrire:
          'src/panier.ts': |
            // La logique du panier, écrite comme des fonctions pures : elles ne modifient jamais leurs arguments.
            export type Ligne = { id: number; nom: string; prixCents: number; stock: number; quantite: number };
            export type LigneEntree = Omit<Ligne, "quantite">;

            export function ajouter(panier: Ligne[], entree: LigneEntree): Ligne[] {
              if (entree.stock <= 0) return panier;
              const index = panier.findIndex((l) => l.id === entree.id);
              if (index === -1) {
                return [...panier, { ...entree, quantite: 1 }];
              }
              return panier.map((l, i) =>
                i === index ? { ...l, quantite: Math.min(l.quantite + 1, l.stock) } : l,
              );
            }

            export function retirer(panier: Ligne[], id: number): Ligne[] {
              return panier.filter((l) => l.id !== id);
            }

            export const totalArticles = (panier: Ligne[]): number => panier.reduce((somme, l) => somme + l.quantite, 0);
            export const totalCents = (panier: Ligne[]): number =>
              panier.reduce((somme, l) => somme + l.prixCents * l.quantite, 0);
  - texte: >-
      Dans `src/ChoixQuantite.tsx`, ajoute un état `quantite` avec `useState(1)`. Affiche-le dans le `<span>`, fais monter la quantité d'une unité au clic sur `+` et descendre au clic sur `−`, en passant une **fonction** à `setQuantite`. Vérifie avec `npx vitest run quantite-clics`.
    indice: >-
      `const [quantite, setQuantite] = useState(1);` puis `onClick={() => setQuantite((q) => q + 1)}` sur le bouton `+` (et `q - 1` sur le bouton `−`). N'oublie pas `import { useState } from "react";`.
    verif:
      - commande-reussit: controler tests 02-etat-evenements quantite-clics
      - commande-reussit: contient src/ChoixQuantite.tsx 'useState\s*(<[^>]*>)?\s*\(\s*1\s*\)'
    solution:
      - ecrire:
          'src/ChoixQuantite.tsx': |
            import { useState } from "react";

            export function ChoixQuantite({
              stock,
              onAjouter,
            }: Readonly<{ stock: number; onAjouter: (quantite: number) => void }>) {
              const [quantite, setQuantite] = useState(1);

              return (
                <div>
                  <button type="button" onClick={() => setQuantite((q) => q - 1)}>
                    −
                  </button>
                  <span>{quantite}</span>
                  <button type="button" onClick={() => setQuantite((q) => q + 1)}>
                    +
                  </button>
                  <button type="button" onClick={() => {}}>
                    Ajouter au panier
                  </button>
                </div>
              );
            }
  - texte: >-
      Empêche les quantités absurdes : `−` est désactivé (`disabled`) quand la quantité vaut 1, `+` l'est quand elle atteint le `stock`, et la quantité ne sort jamais de l'intervalle de 1 à `stock`. L'état n'est pas dupliqué : l'affichage se **déduit** de `quantite`. Vérifie avec `npx vitest run quantite-bornes`.
    indice: >-
      Ajoute `disabled={quantite <= 1}` au bouton `−` et `disabled={quantite >= stock}` au bouton `+`, et protège les calculs avec `Math.max(1, q - 1)` et `Math.min(stock, q + 1)`.
    verif:
      - commande-reussit: controler tests 02-etat-evenements quantite-bornes
      - commande-reussit: contient src/ChoixQuantite.tsx '\bdisabled\b'
    apres: [3]
    solution:
      - ecrire:
          'src/ChoixQuantite.tsx': |
            import { useState } from "react";

            export function ChoixQuantite({
              stock,
              onAjouter,
            }: Readonly<{ stock: number; onAjouter: (quantite: number) => void }>) {
              const [quantite, setQuantite] = useState(1);

              return (
                <div>
                  <button type="button" disabled={quantite <= 1} onClick={() => setQuantite((q) => Math.max(1, q - 1))}>
                    −
                  </button>
                  <span>{quantite}</span>
                  <button type="button" disabled={quantite >= stock} onClick={() => setQuantite((q) => Math.min(stock, q + 1))}>
                    +
                  </button>
                  <button type="button" onClick={() => {}}>
                    Ajouter au panier
                  </button>
                </div>
              );
            }
  - texte: >-
      Branche le bouton « Ajouter au panier » : au clic, il appelle `onAjouter` avec la quantité choisie (les données descendent par les props, les événements remontent par les fonctions), puis remet la quantité à 1. Vérifie avec `npx vitest run quantite-ajout`, puis lance toute la suite avec `npx vitest run`.
    indice: >-
      Le gestionnaire est `onClick={() => { onAjouter(quantite); setQuantite(1); }}`.
    verif:
      - commande-reussit: controler tests 02-etat-evenements
      - commande-reussit: controler types 02-etat-evenements
      - commande-reussit: contient src/ChoixQuantite.tsx '\bonAjouter\s*\('
    apres: [1, 2, 3, 4]
    solution:
      - ecrire:
          'src/ChoixQuantite.tsx': |
            import { useState } from "react";

            export function ChoixQuantite({
              stock,
              onAjouter,
            }: Readonly<{ stock: number; onAjouter: (quantite: number) => void }>) {
              const [quantite, setQuantite] = useState(1);

              return (
                <div>
                  <button type="button" disabled={quantite <= 1} onClick={() => setQuantite((q) => Math.max(1, q - 1))}>
                    −
                  </button>
                  <span>{quantite}</span>
                  <button type="button" disabled={quantite >= stock} onClick={() => setQuantite((q) => Math.min(stock, q + 1))}>
                    +
                  </button>
                  <button type="button" onClick={() => {
                    onAjouter(quantite);
                    setQuantite(1);
                  }}>
                    Ajouter au panier
                  </button>
                </div>
              );
            }
:::

## Vérifie tes acquis

:::quiz
Quelle ligne crée un état `ouvert` qui vaut `false` au départ ?

- [ ] `const ouvert = useState(false);`
- [ ] `const [setOuvert, ouvert] = useState(false);`
- [x] `const [ouvert, setOuvert] = useState(false);`
- [ ] `let ouvert = false;`

> `useState` renvoie une paire `[valeur, fonction de mise à jour]`, dans cet ordre.
:::

:::quiz
Pourquoi `panier.push(ligne)` ne met-il pas l'écran à jour correctement ?

- [ ] Parce que `push` est supprimé dans React
- [x] Parce que le tableau est modifié en place : sa référence ne change pas et React ne voit pas de changement
- [ ] Parce que `push` ne fonctionne pas sur les tableaux typés
- [ ] Parce qu'il faut toujours utiliser `setTimeout`

> Il faut fournir à `setPanier` un nouveau tableau, par exemple `[...panier, ligne]`.
:::

:::quiz
Que fait ce code : `<button onClick={setQuantite(2)}>` ?

- [ ] Il met la quantité à 2 au clic
- [x] Il appelle `setQuantite(2)` à chaque affichage, au lieu d'attendre le clic
- [ ] Il est refusé par le navigateur
- [ ] Il désactive le bouton

> On doit passer une fonction : `onClick={() => setQuantite(2)}`.
:::

:::quiz
Tu as un état `panier`. Où calculer le nombre total d'articles ?

- [ ] Dans un second `useState` mis à jour à chaque ajout
- [x] Directement dans le composant, avec `reduce` sur `panier`
- [ ] Dans une prop que le parent doit fournir
- [ ] Dans le `localStorage`

> Une valeur qui se déduit d'une autre est calculée, pas stockée : elle ne peut jamais se désynchroniser.
:::
