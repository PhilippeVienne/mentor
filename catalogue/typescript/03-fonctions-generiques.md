---
id: fonctions-generiques
title: "Fonctions et génériques"
summary: "Typer les paramètres et les retours, puis écrire des fonctions qui marchent avec n'importe quel type grâce aux génériques."
minutes: 40
objectives:
  - Annoter les paramètres, les paramètres facultatifs et le retour d'une fonction
  - Écrire et appeler une fonction générique `<T>`
  - Lire un type générique comme `Promise<T>` ou `Partial<T>`
  - Filtrer un tableau avec un type guard
---

Une fonction, c'est une petite machine : on lui donne des valeurs (les **paramètres**), elle en renvoie une autre (le **retour**). Typer une fonction, c'est afficher sur la machine ce qu'elle accepte et ce qu'elle produit, comme l'étiquette d'un appareil (« 5 V, pas plus »). Une fois que c'est fait, chaque appel est contrôlé.

Tu as écrit une fonction `premier(liste)` qui renvoie le premier élément d'un tableau. Pour des nombres, pour des textes, pour des événements… Faut-il la recopier trois fois en changeant le type ? Non : c'est le rôle des **génériques**.

## Typer une fonction

On annote chaque paramètre, et on peut annoter la valeur de retour après les parenthèses :

```ts
interface Association {
  nom: string;
  adherents: number;
}

function ajouterAdherents(asso: Association, nombre: number, motif?: string): Association {
  console.log(motif ?? "sans motif");
  return { ...asso, adherents: asso.adherents + nombre };
}

function journaliser(message: string): void {
  console.log(message);
}
```

- `ajouterAdherents` reçoit une association, un nombre, et éventuellement un motif. Elle renvoie une **nouvelle** association (`{ ...asso, adherents: … }` copie l'objet en changeant une propriété).
- `motif?: string` : un paramètre facultatif vaut `string | undefined` dans la fonction.
- `: Association` après les parenthèses : le type de retour. Si tu l'écris, `tsc` vérifie que **toutes** les branches de la fonction le respectent.
- `void` : la fonction ne renvoie rien d'utile.

Sur les fonctions exportées d'un projet, annoter le retour sert de **contrat** : MiniShop le fait dans `buildGalleryImageItems(images, selectedVariationIds): GalleryImageItem[]`.

## Les génériques : un type en paramètre

Un **générique** est une fonction (ou un type) qui marche pour plusieurs types, comme une boîte aux dimensions variables : on indique à chaque usage ce qu'elle contient (une boîte *de* nombres, une boîte *de* textes). `<T>` est un **paramètre de type** : un nom (par convention `T`) que `tsc` remplace par le vrai type à chaque appel.

```ts
function premier<T>(liste: T[]): T | undefined {
  return liste[0];
}

const n = premier([3, 4, 5]);
const s = premier(["a", "b"]);
const vide = premier<string>([]);
n.toFixed(1);
```

```console
premier.ts(8,1): error TS18048: 'n' is possibly 'undefined'.
```

- `function premier<T>(liste: T[]): T | undefined` se lit : « pour n'importe quel type `T`, je prends un tableau de `T` et je renvoie un `T`, ou `undefined` si le tableau est vide ».
- Dans `premier([3, 4, 5])`, `tsc` **infère** `T = number` : `n` est un `number | undefined`.
- On peut aussi préciser le type à la main : `premier<string>([])`.
- L'erreur est légitime : un tableau vide n'a pas de premier élément !

Le même mécanisme existe pour les interfaces. Adhésion l'utilise sans le dire quand elle écrit `this.http.get<StudySchool[]>(…)` : le `<StudySchool[]>` indique le type de la réponse attendue.

```ts
interface Reponse<T> {
  donnees: T;
  recu: Date;
}

function envelopper<T>(donnees: T): Reponse<T> {
  return { donnees, recu: new Date() };
}

const rep = envelopper({ nom: "BdE", adherents: 800 });
console.log(rep.donnees.adherents);
```

Ligne à ligne : `Reponse<T>` est une interface avec un trou (`T`) à remplir. `envelopper` prend n'importe quelle donnée et la renvoie emballée avec la date de réception. Ici `rep.donnees` garde le type exact de l'objet passé : l'autocomplétion de l'éditeur connaît `adherents`.

## Quelques types génériques fournis

| Type | Sens |
| --- | --- |
| `Array<T>` ou `T[]` | Tableau de `T` |
| `Promise<T>` | Une « promesse » : une valeur `T` qui n'est pas encore disponible (une réponse réseau, par exemple), que l'on attend avec `await` dans une fonction `async` (leçon 5) |
| `Partial<T>` | Comme `T`, mais toutes les propriétés deviennent facultatives |
| `Record<K, V>` | Objet dont les clés sont de type `K` et les valeurs de type `V` |

```ts
interface Evenement {
  id: number;
  titre: string;
  brouillon: boolean;
}

function modifier(e: Evenement, changements: Partial<Evenement>): Evenement {
  return { ...e, ...changements };
}

const gala: Evenement = { id: 1, titre: "Gala", brouillon: true };
modifier(gala, { titre: "Gala 2027" });
modifier(gala, { titer: "Gala" });
```

```console
modifier.ts(13,18): error TS2561: Object literal may only specify known properties, but 'titer' does not exist in type 'Partial<Evenement>'. Did you mean to write 'titre'?
```

## Filtrer avec un type guard

Un **type guard** (« garde de type ») est un test que `tsc` sait exploiter pour affiner un type. Un `.filter()` peut retirer les `null` d'un tableau. Pour que `tsc` le sache, on écrit un **prédicat de type** : `e is Evenement`. C'est ce que fait MiniShop à la fin de `buildGalleryImageItems` : `.filter((image): image is GalleryImageItem => image !== null)`.

Ligne à ligne : `(Evenement | null)[]` est un tableau dont les cases contiennent un événement ou `null`. Le filtre garde les cases non nulles, et `e is Evenement` précise à `tsc` que le tableau obtenu ne contient plus que des `Evenement`.

```ts
const brouillons: (Evenement | null)[] = [gala, null];
const evenements = brouillons.filter((e): e is Evenement => e !== null);
console.log(evenements[0].titre);
```

:::info Le compilateur devient plus malin
Depuis TypeScript 5.5, `tsc` devine tout seul le prédicat pour un test simple comme `e !== null`. Le prédicat explicite reste utile pour les tests plus complexes, et tu le rencontreras dans beaucoup de code existant.
:::

:::tip Des noms de types parlants
`T` suffit pour un générique simple. Dès qu'il y en a deux, préfère des noms qui disent le rôle, comme `TEntree` et `TSortie`.
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  Ton dossier de travail contient cinq fichiers TypeScript, un par notion de la leçon : `adherents.ts`, `premier.ts`, `modifier.ts`, `filtre.ts` et `usage-enveloppe.ts`. Corrige-les un par un. `npx tsc --noEmit` vérifie les types ; `npx tsx fichier.ts` exécute un fichier. Utilise `nano` pour éditer. Le portail contrôle ton travail sur une copie propre, avec ses propres contrôles : `@ts-ignore`, `@ts-nocheck` et `any` ne font que taire `tsc`, ils ne valident pas l'étape.
commands:
  - cp -R /opt/exercices/03-fonctions-generiques/. .
  - lier-outils
steps:
  - text: 'Dans `adherents.ts`, annote les trois paramètres de `ajouterAdherents` (le dernier est facultatif) et son retour : `tsc` ne doit plus se plaindre, et `npx tsx adherents.ts` doit afficher `825`'
    hint: 'Les types sont `asso: Association`, `nombre: number` et `motif?: string`. Le retour est `Association`.'
    checks:
      - command-succeeds: 'verifier-ts 03 adherents'
    solution:
      - |
        cat > adherents.ts <<'EOF'
        interface Association {
          nom: string;
          adherents: number;
        }

        function ajouterAdherents(asso: Association, nombre: number, motif?: string): Association {
          console.log(motif ?? "sans motif");
          return { ...asso, adherents: asso.adherents + nombre };
        }

        const bde: Association = { nom: "BdE", adherents: 800 };
        console.log(ajouterAdherents(bde, 25).adherents);
        EOF
  - text: 'Dans `premier.ts`, rends `premier` générique pour qu''elle accepte aussi bien des nombres que des textes'
    hint: 'Remplace `number` par un paramètre de type : `function premier<T>(liste: T[]): T | undefined`.'
    checks:
      - command-succeeds: 'verifier-ts 03 premier'
    solution:
      - |
        cat > premier.ts <<'EOF'
        function premier<T>(liste: T[]): T | undefined {
          return liste[0];
        }

        const n = premier([3, 4, 5]);
        const s = premier(["a", "b"]);

        console.log(n, s);

        // `s` est un texte (ou `undefined`) : `toFixed` n'existe pas, cette ligne DOIT être une erreur.
        // @ts-expect-error
        console.log(s.toFixed(1));
        EOF
  - text: 'Dans `modifier.ts`, fais accepter à `modifier` un objet de changements partiel, avec `Partial<Evenement>`'
    hint: 'Le deuxième paramètre doit être de type `Partial<Evenement>`.'
    checks:
      - command-succeeds: 'verifier-ts 03 modifier'
    solution:
      - |
        cat > modifier.ts <<'EOF'
        interface Evenement {
          id: number;
          titre: string;
          brouillon: boolean;
        }

        function modifier(e: Evenement, changements: Partial<Evenement>): Evenement {
          return { ...e, ...changements };
        }

        const gala: Evenement = { id: 1, titre: "Gala", brouillon: true };

        console.log(modifier(gala, { titre: "Gala 2027" }));

        // Une faute de frappe dans les changements DOIT rester une erreur.
        // @ts-expect-error
        modifier(gala, { titer: "Gala" });
        EOF
  - text: 'Dans `filtre.ts`, `evenements[0].titre` est refusé parce que `tsc` croit qu''il peut y avoir des `null` : donne au filtre un prédicat de type (`e is Evenement`)'
    hint: 'Écris le filtre sous la forme `(e): e is Evenement => e !== null && e.id > 0`.'
    checks:
      - command-succeeds: 'verifier-ts 03 filtre'
    solution:
      - |
        sed -i 's/(e) => e !== null/(e): e is Evenement => e !== null/' filtre.ts
  - text: '`usage-enveloppe.ts` utilise une fonction générique `envelopper` qui n''existe pas : crée `enveloppe.ts` qui exporte l''interface `Reponse<T>` (`donnees: T`, `recu: Date`) et la fonction `envelopper<T>(donnees: T): Reponse<T>`'
    hint: 'Reprends l''exemple de la leçon. N''oublie pas `export` devant l''interface et la fonction.'
    checks:
      - command-succeeds: 'verifier-ts 03 enveloppe'
    solution:
      - |-
        cat > enveloppe.ts <<'EOF'
        export interface Reponse<T> {
          donnees: T;
          recu: Date;
        }

        export function envelopper<T>(donnees: T): Reponse<T> {
          return { donnees, recu: new Date() };
        }
        EOF
:::

## Vérifie tes acquis

:::quiz
Dans `function premier<T>(liste: T[]): T | undefined`, que représente `T` ?

- [ ] Une variable globale
- [ ] Le mot-clé `type`
- [x] Un type choisi à chaque appel, déduit ou précisé
- [ ] Le type `Tableau`

> Pour `premier(["a", "b"])`, `T` vaut `string`. Pour `premier([3])`, il vaut `number`.
:::

:::quiz
Pourquoi `premier([3, 4, 5]).toFixed(1)` est-il refusé ?

- [ ] Parce que `toFixed` n'existe pas sur `number`
- [x] Parce que le résultat peut être `undefined` si le tableau est vide
- [ ] Parce que `T` n'a pas été précisé
- [ ] Parce qu'un générique ne peut pas renvoyer de nombre

> Le type de retour est `T | undefined` : il faut traiter le cas `undefined`.
:::

:::quiz
Que signifie `motif?: string` dans la liste des paramètres d'une fonction ?

- [ ] Le paramètre doit être `null`
- [x] L'appelant peut l'omettre
- [ ] Le paramètre est de type `boolean`

> Dans la fonction, sa valeur est `string | undefined` : traite l'absence avec un test ou `??`.
:::

:::quiz
À quoi sert `Partial<Evenement>` ?

- [ ] À rendre toutes les propriétés en lecture seule
- [ ] À supprimer les propriétés facultatives
- [x] À accepter un objet qui ne contient qu'une partie des propriétés
- [ ] À convertir l'objet en tableau

> `Partial<T>` rend chaque propriété facultative : pratique pour un objet de modifications.
:::
