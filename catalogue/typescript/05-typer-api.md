---
id: typer-api
title: "Typer les réponses d'une API"
summary: "`fetch` ne sait pas ce que le serveur renvoie : décrire la réponse, puis la vérifier vraiment avec `unknown` et un type guard."
minutes: 40
objectives:
  - Typer une fonction `async` avec `Promise<T>`
  - Expliquer pourquoi `as` ne vérifie rien
  - Valider une réponse inconnue avec `unknown` et un type guard
---

Une **API** est un service web qui répond à tes requêtes, en général avec des données au format **JSON** (du texte structuré, comme un objet JavaScript). Ton programme les reçoit « de l'extérieur » : TypeScript n'a aucun moyen de savoir à l'avance ce qu'elles contiennent. Cette leçon montre comment garder des types fiables malgré tout.

Dans la leçon *API fetch*, tu as récupéré du JSON avec `await reponse.json()`. En TypeScript, la question devient : **de quel type est ce JSON ?** `tsc` ne peut pas le deviner, le serveur n'existe pas encore au moment de la compilation.

## `json()` renvoie `any`

Le type de retour de `reponse.json()` est `Promise<any>` : `tsc` laisse tout passer. On peut donc écrire, sans aucune erreur :

```ts
interface Evenement {
  id: number;
  titre: string;
  places: number;
  lieu: string | null;
}

async function chargerEvenementsNaif(url: string): Promise<Evenement[]> {
  const reponse = await fetch(url);
  return (await reponse.json()) as Evenement[];
}
```

- `fetch(url)` envoie la requête ; `await` attend la réponse sans bloquer la page. `reponse.json()` lit le corps de la réponse et le transforme en valeur JavaScript.
- `async` + `Promise<Evenement[]>` : la fonction renvoie ses résultats plus tard ; l'appelant fait `await`.
- `as Evenement[]` est une **affirmation** : « fais-moi confiance, c'est bien cela ». `tsc` ne vérifie rien.

Voyons ce que cela donne si le serveur envoie autre chose (essai avec un faux `fetch`, sous Node.js, le programme qui exécute du JavaScript hors du navigateur) :

```console
GALA undefined
```

La seconde valeur est `typeof naif[0].places` : `tsc` pensait recevoir un `number`, le programme a reçu `undefined`. Pire : le code n'a pas planté sur le moment, l'erreur se déclenchera plus loin, chez quelqu'un d'autre.

:::warning `as` n'est pas une conversion
`valeur as Type` ne transforme ni ne contrôle la valeur : c'est une promesse faite au compilateur. Chaque `as` sur une donnée venue de l'extérieur est un pari.
:::

## Accepter qu'on ne sait pas : `unknown`

`unknown` est le type « je ne sais pas encore ». Contrairement à `any`, `tsc` interdit d'utiliser la valeur tant qu'elle n'a pas été vérifiée :

```ts
const brut: unknown = JSON.parse('{"a":1}');
brut.a;
```

```console
brut.ts(2,1): error TS18046: 'brut' is of type 'unknown'.
```

## Un type guard pour vérifier à l'exécution

Un **type guard** est une fonction qui contrôle une valeur à l'exécution et, grâce à son type de retour, informe `tsc` du résultat. Une fonction dont le retour est `valeur is Evenement` dit à `tsc` : « si je renvoie `true`, la valeur est un `Evenement` ». À toi d'écrire les tests réellement :

```ts
function estEvenement(valeur: unknown): valeur is Evenement {
  if (typeof valeur !== "object" || valeur === null) {
    return false;
  }
  const candidat = valeur as Record<string, unknown>;
  return (
    typeof candidat.id === "number" &&
    typeof candidat.titre === "string" &&
    typeof candidat.places === "number" &&
    (typeof candidat.lieu === "string" || candidat.lieu === null)
  );
}

async function chargerEvenements(url: string): Promise<Evenement[]> {
  const reponse = await fetch(url);
  if (!reponse.ok) {
    throw new Error(`Erreur HTTP ${reponse.status}`);
  }
  const donnees: unknown = await reponse.json();
  if (!Array.isArray(donnees) || !donnees.every(estEvenement)) {
    throw new Error("Réponse inattendue du serveur");
  }
  return donnees;
}
```

Ligne à ligne :

- `estEvenement` reçoit une valeur `unknown`. Elle refuse d'abord ce qui n'est pas un objet (`typeof … !== "object"`, ou `null`).
- `Record<string, unknown>` dit « un objet dont les clés sont des textes et les valeurs inconnues » : on peut lire `candidat.id` sans présumer de son type.
- Les `typeof … === …` vérifient un à un les champs. `&&` signifie « et » : tout doit être vrai.
- `chargerEvenements` vérifie d'abord `reponse.ok` (la requête a-t-elle réussi ?), lit le JSON en `unknown`, puis `donnees.every(estEvenement)` teste **chaque** élément du tableau. Après ce test, `tsc` sait que `donnees` est un `Evenement[]`.

Avec le même faux serveur, la version vérifiée se comporte ainsi :

```console
[ { id: 1, titre: 'Gala', places: 200, lieu: null } ]
Réponse inattendue du serveur
```

Les données conformes passent, les autres déclenchent une **erreur claire à l'endroit où les données entrent**, pas trois fichiers plus loin.

![Une réponse d'API arrive en `unknown`, traverse le type guard, puis devient un `Evenement` sûr](images/validation-api.svg)

:::info Et si le projet est gros ?
Écrire un type guard à la main devient fastidieux avec de grosses structures. Des bibliothèques de validation (Zod, Valibot…) génèrent à la fois la vérification et le type. Choisis celle de ton projet plutôt que d'en ajouter une : ce cours ne dépend d'aucune.
:::

## Quand les types viennent d'ailleurs

MiniShop n'écrit pas à la main les types de sa base de données : le script `"generate-types": "kysely-codegen --dialect=postgres --out-file=./src/db/types.ts"` les **génère** depuis le schéma PostgreSQL. L'en-tête du fichier prévient : *« Please do not edit it manually »*. Les types restent ainsi synchronisés avec la base, ce qui est plus fiable que de les recopier. Côté Adhésion, `this.http.get<StudySchool[]>(…)` fait la même promesse que notre `as` : le type est écrit à la main, à garder cohérent avec l'API.

## Entraîne-toi

:::lab
engine: real
intro: |
  Il n'y a pas de réseau dans cet environnement : le fichier `serveur.ts` simule un serveur, avec une réponse correcte (`reponseCorrecte`) et une réponse « cassée » dont le champ `places` a disparu (`reponseCassee`). Tu vas écrire un type guard, l'utiliser pour charger les données, puis gérer l'erreur. `npx tsc --noEmit` vérifie les types ; `npx tsx fichier.ts` exécute un fichier. Utilise `nano` pour éditer. Le portail contrôle ton travail sur une copie propre, avec ses propres contrôles : `@ts-ignore`, `@ts-nocheck` et `any` ne font que taire `tsc`, ils ne valident pas l'étape.
commands:
  - cp -R /opt/exercices/05-typer-api/. .
  - lier-outils
steps:
  - text: 'Dans `brut.ts`, `brut` est de type `unknown` et `brut.a` est refusé : vérifie la valeur avant de lire `a`. `npx tsx brut.ts` doit afficher `1`'
    hint: 'Teste que `brut` est un objet non `null` qui contient `"a"` : `if (typeof brut === "object" && brut !== null && "a" in brut) { ... }`.'
    checks:
      - command-succeeds: 'verifier-ts 05 brut'
    solution:
      - |
        cat > brut.ts <<'EOF'
        const brut: unknown = JSON.parse('{"a":1}');

        if (typeof brut === "object" && brut !== null && "a" in brut) {
          console.log(brut.a);
        }
        EOF
  - text: 'Dans `garde.ts`, écris pour de bon le type guard `estEvenement` : il doit vérifier le type de chaque champ. `npx tsx garde.test.ts` doit réussir'
    hint: 'Reprends la fonction de la leçon : refuse ce qui n''est pas un objet, puis teste `id` (nombre), `titre` (texte), `places` (nombre) et `lieu` (texte ou `null`).'
    checks:
      - command-succeeds: 'verifier-ts 05 garde'
    solution:
      - |
        cat > garde.ts <<'EOF'
        import type { Evenement } from "./types";

        export function estEvenement(valeur: unknown): valeur is Evenement {
          if (typeof valeur !== "object" || valeur === null) {
            return false;
          }
          const candidat = valeur as Record<string, unknown>;
          return (
            typeof candidat.id === "number" &&
            typeof candidat.titre === "string" &&
            typeof candidat.places === "number" &&
            (typeof candidat.lieu === "string" || candidat.lieu === null)
          );
        }
        EOF
  - text: 'Dans `charger.ts`, remplace l''affirmation `as Evenement[]` par une vraie vérification avec `estEvenement` : la fonction doit lever une erreur si la réponse est mal formée. `npx tsx charger.test.ts` doit réussir'
    hint: 'Après `await lire()`, teste `!Array.isArray(donnees) || !donnees.every(estEvenement)` et lance `throw new Error("...")`.'
    after: [2]
    checks:
      - command-succeeds: 'verifier-ts 05 charger'
    solution:
      - |
        cat > charger.ts <<'EOF'
        import type { Evenement } from "./types";
        import { estEvenement } from "./garde";

        export async function chargerEvenements(lire: () => Promise<unknown>): Promise<Evenement[]> {
          const donnees = await lire();
          if (!Array.isArray(donnees) || !donnees.every(estEvenement)) {
            throw new Error("Réponse inattendue du serveur");
          }
          return donnees;
        }
        EOF
  - text: 'Dans `main.ts`, attrape l''erreur avec `try` et `catch` et affiche une ligne qui commence par `Erreur` : `npx tsx main.ts` doit se terminer normalement et afficher ce message'
    hint: 'Entoure le chargement et l''affichage d''un `try { ... } catch (erreur) { console.log(`Erreur : ${...}`); }`. Pour lire le message : `erreur instanceof Error ? erreur.message : String(erreur)`.'
    after: [3]
    checks:
      - command-succeeds: 'verifier-ts 05 main'
    solution:
      - |-
        cat > main.ts <<'EOF'
        import { chargerEvenements } from "./charger";
        import { reponseCassee } from "./serveur";

        try {
          const evenements = await chargerEvenements(reponseCassee);
          console.log(`Places du premier événement : ${evenements[0].places.toFixed(0)}`);
        } catch (erreur) {
          console.log(`Erreur : ${erreur instanceof Error ? erreur.message : String(erreur)}`);
        }
        EOF
:::

## Vérifie tes acquis

:::quiz
Quel est le type de `await reponse.json()` ?

- [ ] `unknown`
- [ ] `object`
- [x] `any`
- [ ] `Evenement[]`

> `json()` renvoie `Promise<any>` : `tsc` ne connaît pas la forme de la réponse.
:::

:::quiz
Que garantit `(await reponse.json()) as Evenement[]` ?

- [ ] Que le serveur renvoie bien des événements
- [ ] Que les champs manquants sont remplis par défaut
- [x] Rien : c'est une affirmation que `tsc` ne vérifie pas
- [ ] Que la requête a réussi

> `as` ne produit aucun code d'exécution. Si le serveur change, le programme s'en apercevra trop tard.
:::

:::quiz
En quoi `unknown` est-il plus sûr que `any` ?

- [x] Il interdit d'utiliser la valeur avant de l'avoir vérifiée
- [ ] Il vérifie la valeur à l'exécution
- [ ] Il est converti automatiquement en `string`
- [ ] Il est plus rapide à compiler

> Avec `unknown`, `tsc` impose un test (`typeof`, type guard…) avant tout accès.
:::

:::quiz
Que signifie le type de retour `valeur is Evenement` ?

- [ ] La fonction convertit la valeur en `Evenement`
- [ ] La fonction renvoie un objet `Evenement`
- [x] Si elle renvoie `true`, `tsc` considère la valeur comme un `Evenement`
- [ ] La fonction vérifie automatiquement chaque propriété

> `tsc` fait confiance au corps de la fonction : c'est à toi d'écrire des tests complets.
:::
