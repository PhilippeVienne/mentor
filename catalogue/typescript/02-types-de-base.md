---
id: types-de-base
title: "Types de base, unions et interfaces"
summary: "Décrire la forme de tes données : types primitifs, unions, `null`, interfaces et alias de type."
minutes: 40
objectives:
  - Annoter des variables et des tableaux avec les types de base
  - Écrire une union de valeurs littérales et gérer `null`
  - Décrire un objet avec une `interface`
  - Réduire un type union avec un test (`typeof`, `=== null`)
---

« Cette inscription est-elle confirmée ? » Si la réponse est une chaîne libre (`"confirmée"`, `"Confirmee"`, `"ok"`…), un oubli d'accent suffit à casser le programme. TypeScript permet de dire précisément **quelles valeurs sont permises**.

## À quoi ça sert, et pourquoi ?

Quand tu remplis un formulaire papier, chaque case dit ce qu'on attend : une date, un nom, une case à cocher. Dans un programme, une **donnée** (un événement, une inscription…) est rangée dans des variables et des objets. Sans description, rien n'indique ce qu'ils contiennent. Cette leçon t'apprend à écrire ces descriptions, c'est-à-dire à définir la « forme » de tes données une fois pour toutes. `tsc` vérifie ensuite que tout le programme la respecte.

## Les types de base

Un **type primitif** est un type de valeur simple. Voici ceux dont tu auras besoin :

| Type | Exemple | Remarque |
| --- | --- | --- |
| `string` | `"Gala"` | Texte |
| `number` | `120`, `3.5` | Entiers et décimaux, même type |
| `boolean` | `true` | |
| `string[]` | `["Camille", "Yasmine"]` | Tableau de textes (aussi `Array<string>`) |
| `null` / `undefined` | | Absence de valeur, à traiter explicitement |

Tu n'es pas obligé·e d'annoter chaque variable : `const places = 120` est déjà un `number`. On annote surtout les **paramètres de fonctions** et les **structures de données** partagées.

## Les unions : « l'un ou l'autre »

Une **union** est un type qui accepte plusieurs possibilités. Le symbole `|` signifie « ou ». Avec des valeurs littérales, on obtient une liste fermée. MiniShop fait exactement cela dans son fichier `src/db/types.ts` : `PurchaseType = "cash" | "cash+tpe" | "check" | "online" | "payapp" | "tpe"`. Le même fichier décrit les colonnes optionnelles de la base avec `string | null`.

Voici notre version, pour des inscriptions à un événement :

```ts
type StatutInscription = "en_attente" | "confirmee" | "annulee";

let statut: StatutInscription = "confirmee";
statut = "confirmé";
```

```console
statut.ts(4,1): error TS2820: Type '"confirmé"' is not assignable to type 'StatutInscription'. Did you mean '"confirmee"'?
```

- `type StatutInscription = …` crée un **alias** : un nom que l'on donne à un type pour le réutiliser (comme une variable, mais pour un type).
- `let statut: StatutInscription = "confirmee";` : on crée une variable dont le type est cet alias ; la valeur est autorisée.
- `statut = "confirmé";` : faute d'accent. `tsc` refuse et propose la bonne valeur. Sans ce contrôle, l'erreur serait passée inaperçue.

## Les interfaces : la forme d'un objet

Un **objet** regroupe plusieurs valeurs nommées (ses *propriétés*). Une `interface` décrit les propriétés d'un objet. Un point d'interrogation rend une propriété facultative ; `string | null` dit qu'elle est obligatoire mais peut valoir `null`.

```ts
interface Evenement {
  id: number;
  titre: string;
  lieu: string | null;
  places: number;
  description?: string;
}

const soiree: Evenement = { id: 1, titre: "Soirée d'intégration", lieu: null, places: 120 };
```

Ligne à ligne : `interface Evenement { … }` ne crée aucun objet, elle écrit seulement le **plan**. Chaque ligne est `nom: type;`. La dernière constante `soiree` est un vrai objet, annoté `: Evenement` ; `tsc` vérifie qu'il a toutes les propriétés obligatoires, avec les bons types. Si tu oublies `places`, ou si tu écris `places: "beaucoup"`, c'est une erreur.

MiniShop décrit ainsi chaque table (`interface Category { id: Generated<number>; name: string | null; … }`), et Adhésion définit des classes comme `StudySchool` (`id`, `name`, `short_name`) dans `metadata.service.ts`.

:::cards
### `interface`

Décrit la forme d'un objet. Elle peut être étendue (`interface Concert extends Evenement`). C'est le choix habituel pour des objets.

### `type`

Donne un nom à n'importe quel type : une union, un tableau, un objet. Seul `type` permet d'écrire `type Statut = "a" | "b"`.
:::

## Réduire une union

Avec `lieu: string | null`, appeler `.toUpperCase()` directement est refusé, car `null` n'a pas cette méthode. Il faut d'abord **vérifier** : TypeScript suit le test et affine le type (*narrowing*).

```ts
function afficherLieu(evenement: Evenement): string {
  return evenement.lieu.toUpperCase();
}

function afficherLieuSur(evenement: Evenement): string {
  if (evenement.lieu === null) {
    return "Lieu à confirmer";
  }
  return evenement.lieu.toUpperCase();
}

function decrire(valeur: string | number): string {
  if (typeof valeur === "string") {
    return valeur.toUpperCase();
  }
  return valeur.toFixed(2);
}
```

```console
lieu.ts(2,10): error TS18047: 'evenement.lieu' is possibly 'null'.
```

Ligne à ligne : dans `afficherLieu`, TypeScript refuse l'appel car `lieu` peut valoir `null`, et `null.toUpperCase()` planterait. Dans `afficherLieuSur`, le test `if (… === null)` traite d'abord ce cas et sort (`return`) ; après ce test, `tsc` **sait** que `lieu` est un texte. Dans `decrire`, `typeof valeur === "string"` vaut « si c'est un texte » ; sinon c'est forcément un nombre.

Dans `decrire`, après le `if`, `valeur` est forcément un `number` : `toFixed` est donc autorisé. L'opérateur `?.` (ex. `evenement.description?.length`) et `??` (ex. `?? 0`) sont des raccourcis pour les valeurs absentes.

:::warning Évite `any`
`any` désactive la vérification : une valeur `any` accepte tout, partout. C'est une porte de sortie, pas un type. Quand tu ne connais pas la forme d'une donnée, préfère `unknown` (leçon 5).
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  Ton dossier de travail contient cinq petits fichiers TypeScript : `statut.ts`, `evenement.ts`, `lieu.ts`, `decrire.ts` et `usage.ts`. Chacun illustre une notion de la leçon et contient une erreur à corriger. `npx tsc --noEmit` vérifie les types de tous les fichiers ; `npx tsx fichier.ts` exécute un fichier TypeScript (tsx le transforme en JavaScript puis le lance). Pour éditer, utilise `nano`. Le portail contrôle ton travail sur une copie propre, avec ses propres contrôles : `@ts-ignore`, `@ts-nocheck` et `any` ne font que taire `tsc`, ils ne valident pas l'étape.
commands:
  - cp -R /opt/exercices/02-types-de-base/. .
  - lier-outils
steps:
  - text: 'Dans `statut.ts`, la valeur `"confirmé"` n''est pas dans l''union `StatutInscription` : corrige-la'
    hint: 'Lis le message : `tsc` propose la bonne valeur (sans accent).'
    checks:
      - command-succeeds: 'verifier-ts 02 statut'
    solution:
      - |
        sed -i 's/"confirmé"/"confirmee"/' statut.ts
  - text: 'Dans `evenement.ts`, l''objet `soiree` ne respecte pas l''interface `Evenement` : complète-le'
    hint: 'Il manque une propriété obligatoire. Ajoute `places: 120`.'
    checks:
      - command-succeeds: 'verifier-ts 02 evenement'
    solution:
      - |
        sed -i 's/lieu: null }/lieu: null, places: 120 }/' evenement.ts
  - text: 'Dans `lieu.ts`, traite le cas où `lieu` vaut `null` (renvoie `"Lieu à confirmer"`) : `npx tsx lieu.ts` ne doit plus planter'
    hint: 'Avant `toUpperCase()`, ajoute `if (evenement.lieu === null) { return "Lieu à confirmer"; }`.'
    checks:
      - command-succeeds: 'verifier-ts 02 lieu'
    solution:
      - |
        cat > lieu.ts <<'EOF'
        interface Evenement {
          id: number;
          titre: string;
          lieu: string | null;
          places: number;
        }

        function afficherLieu(evenement: Evenement): string {
          if (evenement.lieu === null) {
            return "Lieu à confirmer";
          }
          return evenement.lieu.toUpperCase();
        }

        console.log(afficherLieu({ id: 1, titre: "Gala", lieu: null, places: 200 }));
        console.log(afficherLieu({ id: 2, titre: "Soirée", lieu: "Amphi Chappe", places: 120 }));
        EOF
  - text: 'Dans `decrire.ts`, utilise `typeof` pour que les textes soient mis en majuscules et les nombres affichés avec deux décimales (`3.50`)'
    hint: 'Écris `if (typeof valeur === "string") { return valeur.toUpperCase(); }` avant le `return valeur.toFixed(2);`.'
    checks:
      - command-succeeds: 'verifier-ts 02 decrire'
    solution:
      - |
        cat > decrire.ts <<'EOF'
        function decrire(valeur: string | number): string {
          if (typeof valeur === "string") {
            return valeur.toUpperCase();
          }
          return valeur.toFixed(2);
        }

        console.log(decrire("gala"));
        console.log(decrire(3.5));
        EOF
  - text: '`usage.ts` importe un type `Inscription` qui n''existe pas encore : crée `inscription.ts` qui exporte une interface `Inscription` (`id`, `statut` de type `StatutInscription`, `commentaire` facultatif)'
    hint: 'Dans `inscription.ts`, écris `export type StatutInscription = ...` (l''union de la leçon) puis `export interface Inscription { ... }`. Le champ facultatif se note `commentaire?: string`.'
    checks:
      - command-succeeds: 'verifier-ts 02 inscription'
    solution:
      - |-
        cat > inscription.ts <<'EOF'
        export type StatutInscription = "en_attente" | "confirmee" | "annulee";

        export interface Inscription {
          id: number;
          statut: StatutInscription;
          commentaire?: string;
        }
        EOF
:::

## Vérifie tes acquis

:::quiz
Quel type décrit exactement les valeurs `"en_attente"`, `"confirmee"` et `"annulee"` ?

- [ ] `string`
- [ ] `string[]`
- [ ] `enum`
- [x] `"en_attente" | "confirmee" | "annulee"`

> Une union de littéraux limite les valeurs permises ; `string` accepterait n'importe quel texte.
:::

:::quiz
Que signifie `description?: string` dans une interface ?

- [x] La propriété peut être absente de l'objet
- [ ] La propriété vaut toujours `null`
- [ ] La propriété est un tableau
- [ ] La propriété est en lecture seule

> Le `?` rend la propriété facultative : sa valeur est alors `string | undefined`.
:::

:::quiz
Un paramètre est de type `string | null`. Que faut-il faire avant d'appeler `.toUpperCase()` dessus ?

- [ ] Rien, `tsc` ajoute la vérification
- [ ] Le convertir avec `as any`
- [x] Tester qu'il n'est pas `null`
- [ ] Le déclarer avec `?`

> Après `if (valeur === null) return …`, TypeScript sait que `valeur` est une `string`.
:::

:::quiz
Quelle est la différence essentielle entre `lieu?: string` et `lieu: string | null` ?

- [ ] Aucune : ce sont deux écritures du même type
- [ ] Le premier interdit `null`, le second interdit `undefined`
- [x] Le premier peut être omis ; le second doit être présent mais peut valoir `null`
- [ ] Le second ne peut jamais contenir de texte

> Un champ facultatif peut manquer. Un champ `| null` doit exister, même s'il est vide : c'est ce que fait la base de données de MiniShop.
:::
