---
id: tsconfig
title: "Configurer tsconfig.json et le vérificateur"
summary: "Lire le tsconfig de MiniShop : mode strict, résolution des modules, alias `@/` et vérification en ligne de commande."
minutes: 30
objectives:
  - Lancer le vérificateur avec `npx tsc --noEmit`
  - Expliquer le rôle de `strict`, `noEmit`, `include` et `paths`
  - Savoir pourquoi le mode strict évite des bogues
---

Tu clones un projet de l'équipe et `npm run ts-check` affiche des dizaines d'erreurs, alors que ton éditeur est calme. Ou l'inverse. Dans les deux cas, la réponse est dans un fichier : `tsconfig.json`, qui règle **comment** `tsc` vérifie le projet.

## À quoi ça sert, et pourquoi ?

Un correcteur d'orthographe peut être réglé en « indulgent » ou en « strict ». `tsc` aussi. Le fichier `tsconfig.json` (la **configuration** de TypeScript pour le projet) dit à `tsc` quels fichiers regarder, quelle version de JavaScript viser et à quel point être sévère. Chaque projet a le sien : c'est pourquoi le même code peut passer dans l'un et échouer dans l'autre.

## Lancer la vérification

```bash
npx tsc --init
npx tsc --noEmit
npm run ts-check
```

- `npx` exécute un outil installé dans le projet (ici `tsc`, installé avec `npm install --save-dev typescript`).
- `npx tsc --init` crée un `tsconfig.json` de départ, commenté.
- `npx tsc --noEmit` vérifie tous les fichiers listés dans le `tsconfig.json`, sans produire de JavaScript.
- `npm run ts-check` lance le script de même nom du `package.json` : c'est le raccourci que MiniShop fournit.

:::warning Pas de fichier en argument
Si tu écris `npx tsc fichier.ts`, `tsc` ignorait le `tsconfig.json` dans les anciennes versions. Avec TypeScript 6 et suivants, il refuse avec l'erreur `TS5112` (option `--ignoreConfig` pour passer outre). Dans un projet, lance toujours `tsc` **sans nom de fichier**.
:::

## Le tsconfig de MiniShop

Voici l'essentiel du fichier de MiniShop (sans les plugins de Next.js, le framework web construit sur React qu'il utilise) :

```json
{
    "compilerOptions": {
        "target": "ES2017",
        "lib": ["dom", "dom.iterable", "esnext"],
        "strict": true,
        "noEmit": true,
        "module": "esnext",
        "moduleResolution": "bundler",
        "esModuleInterop": true,
        "resolveJsonModule": true,
        "isolatedModules": true,
        "skipLibCheck": true,
        "jsx": "react-jsx",
        "paths": {
            "@/*": ["./src/*"]
        }
    },
    "include": ["**/*.ts", "**/*.tsx"],
    "exclude": ["node_modules"]
}
```

Le fichier est du **JSON** : des paires `"nom": valeur` entre accolades. Le bloc `compilerOptions` contient les réglages ; `include` et `exclude` désignent les fichiers. Voici ce que fait chaque option :

| Option | Rôle |
| --- | --- |
| `strict` | Active toute la famille des vérifications strictes (voir ci-dessous) |
| `noEmit` | Ne produit aucun `.js` : Next.js s'occupe de la transformation |
| `target` / `lib` | Version de JavaScript visée et bibliothèques connues (`dom` = API du navigateur) |
| `module` / `moduleResolution` | Comment lire les `import` ; `bundler` imite la façon dont un **bundler** retrouve les fichiers importés (par exemple sans écrire leur extension). Un bundler (« empaqueteur », comme Next.js ou Vite) est l'outil qui regroupe tous les fichiers du projet en quelques fichiers prêts pour le navigateur |
| `isolatedModules` | Chaque fichier doit pouvoir être transformé seul, comme l'exigent les outils de build |
| `skipLibCheck` | Ne revérifie pas les fichiers de types des bibliothèques (plus rapide) |
| `paths` | Alias d'import : `@/db/types` pointe vers `src/db/types` |
| `include` / `exclude` | Quels fichiers vérifier |

Le frontend d'Adhésion (Angular) a son propre `tsconfig.json`, avec d'autres options (`target: ES2022`, décorateurs expérimentaux pour Angular : un **décorateur** est une annotation comme `@Component(…)` placée devant une classe, que le framework lit ; l'option `experimentalDecorators` active l'ancienne version de cette syntaxe) et **sans** `strict`. Un projet peut donc être plus ou moins sévère : lis le sien avant de supposer.

## Ce que change le mode strict

Une **vérification stricte** est une règle supplémentaire qui rend `tsc` plus exigeant. `strict: true` est un interrupteur qui active plusieurs vérifications d'un coup. Les deux plus visibles :

- `strictNullChecks` : `null` et `undefined` ne se glissent plus dans n'importe quel type.
- `strictPropertyInitialization` : une propriété de classe doit recevoir une valeur.

```ts
interface Evenement {
  titre: string;
  lieu: string | null;
}

function lieuEnMajuscules(evenement: Evenement): string {
  return evenement.lieu.toUpperCase();
}

class Association {
  nom: string;
}
```

```console
evenements.ts(7,10): error TS18047: 'evenement.lieu' is possibly 'null'.
evenements.ts(11,3): error TS2564: Property 'nom' has no initializer and is not definitely assigned in the constructor.
```

Ligne à ligne : dans `lieuEnMajuscules`, on appelle une méthode sur une valeur qui peut être `null` ; dans la classe `Association`, la propriété `nom` est déclarée mais jamais remplie (elle serait donc `undefined`). Avec `"strict": false` dans le `tsconfig.json`, ces deux fautes **passent sans bruit** : c'est ce qui rend du code ancien (comme des classes `StudySchool` dont les champs ne sont jamais initialisés) acceptable dans un projet non strict.

:::info Le défaut a changé
Dans TypeScript 6 et suivants, `strict` vaut `true` par défaut quand il n'est pas écrit. Les versions précédentes (Adhésion utilise `~5.9`) le laissaient à `false`. Écris donc toujours l'option explicitement dans ton `tsconfig.json`.
:::

## Corriger les erreurs d'un projet, dans l'ordre

1. Lance `npx tsc --noEmit` et lis **la première** erreur.
2. Corrige-la en **adaptant le code** (test de `null`, bon type de paramètre…).
3. Relance : une seule correction peut en faire disparaître plusieurs.
4. Ne désactive pas `strict` pour faire taire `tsc` : c'est cacher le problème.

## Entraîne-toi

:::lab
engine: real
intro: |
  Ton dossier de travail contient un petit projet inspiré d'Adhésion : un `tsconfig.json` **indulgent** (`strict` vaut `false`), un `package.json` et trois fichiers dans `src/`. Aucune erreur n'apparaît quand on lance `npx tsc --noEmit`, mais des fautes sont cachées. Tu vas durcir la configuration, corriger ce qui apparaît, ajouter un alias d'import et un script `ts-check`. Pour éditer, utilise `nano`. Le portail contrôle ton travail sur une copie propre, avec ses propres contrôles : `@ts-ignore`, `@ts-nocheck` et `any` ne font que taire `tsc`, ils ne valident pas l'étape.
commands:
  - cp -R /opt/exercices/04-tsconfig/. .
  - lier-outils
steps:
  - text: 'Dans `tsconfig.json`, passe l''option `strict` à `true`'
    hint: 'Ouvre le fichier avec `nano tsconfig.json` et remplace `"strict": false` par `"strict": true`.'
    checks:
      - command-succeeds: 'verifier-ts 04 strict'
    solution:
      - |
        sed -i 's/"strict": false/"strict": true/' tsconfig.json
  - text: 'Corrige `src/lieu.ts` : avec `strict`, `tsc` signale que `lieu` peut être `null`. Renvoie `"Lieu à confirmer"` dans ce cas'
    hint: 'Teste `if (evenement.lieu === null)` avant d''appeler `toUpperCase()`. Pour voir l''erreur sans attendre l''étape 1, lance `npx tsc --noEmit --strict`.'
    after: [1]
    checks:
      - command-succeeds: 'verifier-ts 04 lieu'
    solution:
      - |
        cat > src/lieu.ts <<'EOF'
        export interface Evenement {
          titre: string;
          lieu: string | null;
        }

        export function lieuEnMajuscules(evenement: Evenement): string {
          if (evenement.lieu === null) {
            return "Lieu à confirmer";
          }
          return evenement.lieu.toUpperCase();
        }
        EOF
  - text: 'Corrige `src/association.ts` : les propriétés de la classe `Association` ne sont jamais initialisées'
    hint: 'Donne une valeur initiale à chaque propriété, par exemple `nom = "";` et `adherents = 0;`.'
    after: [1]
    checks:
      - command-succeeds: 'verifier-ts 04 association'
    solution:
      - |
        cat > src/association.ts <<'EOF'
        export class Association {
          nom = "";
          adherents = 0;
        }
        EOF
  - text: '`src/main.ts` importe `@/lieu`, que `tsc` ne trouve pas. Ajoute dans `compilerOptions` un alias `paths` pour que `@/` désigne le dossier `src/`'
    hint: 'Ajoute `"paths": { "@/*": ["./src/*"] },` dans `compilerOptions` (comme le tsconfig d''MiniShop de la leçon).'
    checks:
      - command-succeeds: 'verifier-ts 04 alias'
    solution:
      - |
        sed -i 's|"noEmit": true,|"noEmit": true,\n        "paths": { "@/*": ["./src/*"] },|' tsconfig.json
  - text: 'Ajoute dans `package.json` un script `ts-check` qui lance `tsc --noEmit`, puis lance `npm run ts-check` : il doit réussir'
    hint: 'Dans la section `scripts`, ajoute `"ts-check": "tsc --noEmit"` (sans oublier la virgule entre deux scripts).'
    after: [1, 2, 3, 4]
    checks:
      - command-succeeds: 'verifier-ts 04 script'
    solution:
      - |-
        sed -i 's|"build": "echo pas de build ici"|"build": "echo pas de build ici",\n        "ts-check": "tsc --noEmit"|' package.json
:::

## Vérifie tes acquis

:::quiz
Que fait l'option `"noEmit": true` ?

- [ ] Elle désactive la vérification des types
- [x] Elle empêche `tsc` d'écrire des fichiers JavaScript
- [ ] Elle interdit les `import` relatifs
- [ ] Elle supprime les commentaires du code

> Le projet est vérifié, mais la transformation en JavaScript est confiée à un autre outil (Next.js pour MiniShop).
:::

:::quiz
Dans le tsconfig de MiniShop, à quoi sert `"paths": { "@/*": ["./src/*"] }` ?

- [ ] À exclure le dossier `src` de la vérification
- [ ] À installer les paquets listés dans `src`
- [x] À écrire `@/db/types` au lieu d'un long chemin relatif
- [ ] À renommer les fichiers `.ts` en `.js`

> C'est un alias d'import : `@/` désigne le dossier `src/`.
:::

:::quiz
Avec `strict: true`, que se passe-t-il pour `evenement.lieu.toUpperCase()` quand `lieu` est de type `string | null` ?

- [ ] Le code compile, `null` est converti en texte
- [x] `tsc` signale que `lieu` est peut-être `null`
- [ ] L'erreur n'apparaît qu'à l'exécution
- [ ] `tsc` ajoute un test automatiquement

> C'est `strictNullChecks`, inclus dans `strict`.
:::

:::quiz
Un projet voisin n'active pas `strict` et ne montre aucune erreur. Que peux-tu en conclure ?

- [ ] Que son code est sans bogue
- [x] Que `tsc` y est moins sévère : des fautes possibles ne sont pas détectées
- [ ] Que `tsc` n'y est jamais lancé
- [ ] Que les types y sont inutiles

> L'absence d'erreur dépend de la sévérité de la configuration, pas seulement de la qualité du code.
:::
