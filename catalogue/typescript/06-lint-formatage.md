---
id: lint-formatage
title: "Lint et formatage : ESLint et Prettier"
summary: "Trois outils, trois rôles : `tsc` vérifie les types, ESLint repère les mauvaises pratiques, Prettier uniformise la mise en forme."
minutes: 30
objectives:
  - Distinguer `tsc`, ESLint et Prettier
  - Lire la configuration ESLint et Prettier de MiniShop
  - Lancer les contrôles et le formatage avec les scripts npm
---

Ta **merge request** (une demande de fusion : tu proposes tes modifications de code, et d'autres personnes les relisent avant de les intégrer au projet) est correcte, mais la relecture se perd en remarques sur les guillemets, les points-virgules et une variable jamais utilisée. Ces détails se règlent par des outils, pas par des commentaires : tu gardes l'attention des relecteur·rice·s pour le fond.

## À quoi ça sert, et pourquoi ?

Un **linter** (« peluche », comme celle qu'on retire d'un vêtement) est un programme qui relit ton code et repère ce qui est suspect : variable inutilisée, comparaison douteuse… Un **formateur** réécrit ton code avec une mise en forme uniforme, comme un correcteur qui remet en page un texte. En équipe, cela évite les débats de style et attrape des bogues avant la relecture.

## Trois outils, trois rôles

:::cards
### `tsc`

Vérifie les **types**. Il ne dit rien d'une variable inutilisée ni d'une indentation bancale.

### ESLint

Repère les **mauvaises pratiques** : variable inutilisée, `any` explicite, bogues probables. Il peut aussi corriger certaines fautes.

### Prettier

**Réécrit la mise en forme** (indentation, guillemets, virgules finales) selon un style unique. Il ne juge pas le fond du code.
:::

Les scripts du `package.json` de MiniShop reprennent exactement cette répartition :

| Script | Commande | Rôle |
| --- | --- | --- |
| `ts-check` | `tsc --noEmit` | Types |
| `lint` | `eslint` | Mauvaises pratiques |
| `prettier-check` | `prettier --check .` | Vérifie la mise en forme sans rien modifier |
| `prettier` | `prettier --write .` | Reformate les fichiers |

## La configuration de MiniShop

Le fichier `eslint.config.mjs` est très court (`.mjs` = un module JavaScript moderne, avec `import` et `export`) : il reprend les règles recommandées de Next.js et ignore quelques dossiers générés.

```js
import { defineConfig, globalIgnores } from "eslint/config";
import nextConfig from "eslint-config-next/core-web-vitals";

const eslintConfig = defineConfig([
    ...nextConfig,

    globalIgnores(["node_modules/**", ".next/**", "out/**", "build/**", "next-env.d.ts"]),
]);

export default eslintConfig;
```

Ligne à ligne : les deux `import` chargent des outils d'ESLint (`defineConfig`, `globalIgnores`) et le jeu de règles de Next.js. `defineConfig([...])` assemble la configuration : `...nextConfig` y déverse toutes les règles de Next.js, `globalIgnores([...])` liste les dossiers à ne jamais examiner (dépendances et fichiers générés). `export default` rend la configuration à ESLint.

Le `.prettierrc` fixe le style de l'équipe :

```json
{
    "singleQuote": false,
    "trailingComma": "all",
    "printWidth": 100,
    "tabWidth": 4,
    "semi": true,
    "endOfLine": "lf"
}
```

Ligne à ligne : `singleQuote: false` impose les guillemets doubles ; `trailingComma: "all"` ajoute une virgule après le dernier élément d'une liste multiligne (les différences entre deux versions restent plus courtes) ; `printWidth` est la largeur maximale d'une ligne ; `tabWidth` le nombre d'espaces par niveau d'indentation ; `semi` impose le point-virgule ; `endOfLine: "lf"` fixe le saut de ligne de type Unix.

Concrètement : guillemets doubles, points-virgules, virgule finale partout, lignes de 100 caractères au plus, indentation de 4 espaces. Peu importe le style choisi : l'important est que **tout le monde ait le même**. (Le vrai fichier contient aussi quelques autres options et le plugin `prettier-plugin-tailwindcss`.)

## Voir la différence sur un exemple

Prenons un fichier qui est correct pour `tsc`, mais pas pour les deux autres :

```ts
export function total(prix: any, quantite: number) {
  const inutilisee = 1;
  return prix * quantite
}
```

```bash
npx tsc --noEmit
npx eslint .
npx prettier --check src
```

```console
$ npx tsc --noEmit
(aucune sortie : les types sont acceptés)

$ npx eslint .
src/asso.ts
  1:29  error  Unexpected any. Specify a different type         @typescript-eslint/no-explicit-any
  2:9   error  'inutilisee' is assigned a value but never used  @typescript-eslint/no-unused-vars

✖ 2 problems (2 errors, 0 warnings)

$ npx prettier --check src
Checking formatting...
[warn] src/asso.ts
[warn] Code style issues found in the above file. Run Prettier with --write to fix.
```

Dans cet essai, ESLint était configuré avec `typescript-eslint` (règles « recommended »), ce qui est un choix de configuration, pas la config de MiniShop. Après `npx prettier --write src`, la ligne `return prix * quantite` reçoit son point-virgule et le fichier est ré-indenté selon le `.prettierrc`.

## Dans l'éditeur et dans le flux de travail

- Installe les extensions **ESLint** et **Prettier** de VS Code : les fautes se soulignent pendant que tu écris, et le formatage peut se faire à l'enregistrement.
- Lance `npm run prettier` avant de **committer** (enregistrer tes modifications dans l'historique Git) pour ne pas polluer la merge request avec des changements de mise en forme.
- Sur un projet d'équipe, ces contrôles tournent aussi dans la **CI** (l'intégration continue : un serveur lance automatiquement les vérifications à chaque envoi de code, et refuse la merge request si l'une échoue). Les passer en local évite de découvrir l'échec plus tard.
- Une règle ESLint qui te gêne ? Ne la désactive pas en silence : discute-en avec l'équipe, puis modifie la configuration partagée.

:::warning `--write` modifie tes fichiers
`prettier --write .` reformate **tout** le projet. Sur un dépôt qui n'a jamais été formaté, cela touche des centaines de lignes. Vérifie `git status` et `git diff` avant de committer, et fais ce reformatage dans un commit séparé.
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  Ton dossier de travail contient un mini-projet avec `src/asso.ts`, le fichier de la leçon : il passe `tsc`, mais ESLint et Prettier ont des remarques. ESLint, `typescript-eslint` et Prettier sont déjà installés (pas de réseau ici, donc pas de `npm install`). Tu vas écrire la configuration d'ESLint et celle de Prettier, corriger le fichier, puis ajouter les scripts `lint` et `prettier-check`. Utilise `nano` pour éditer. Le portail contrôle ton travail sur une copie propre : un commentaire `eslint-disable` ne corrige rien, il faut corriger le code.
commands:
  - cp -R /opt/exercices/06-lint-formatage/. .
  - lier-outils
steps:
  - text: 'Crée la configuration d''ESLint, `eslint.config.mjs`, avec les règles recommandées d''ESLint et de `typescript-eslint` : `npx eslint .` doit alors signaler `no-explicit-any`'
    hint: 'Importe `defineConfig` depuis `"eslint/config"`, `eslint` depuis `"@eslint/js"` et `tseslint` depuis `"typescript-eslint"`, puis exporte `defineConfig(eslint.configs.recommended, tseslint.configs.recommended)`.'
    checks:
      - command-succeeds: 'verifier-ts 06 config'
    solution:
      - |
        cat > eslint.config.mjs <<'EOF'
        import eslint from "@eslint/js";
        import { defineConfig } from "eslint/config";
        import tseslint from "typescript-eslint";

        export default defineConfig(eslint.configs.recommended, tseslint.configs.recommended);
        EOF
  - text: 'Corrige `src/asso.ts` pour que `npx eslint . --max-warnings 0` ne signale plus rien : plus de `any`, plus de variable inutilisée'
    hint: 'Remplace `any` par `number` et supprime la constante `inutilisee`.'
    after: [1]
    checks:
      - command-succeeds: 'verifier-ts 06 asso'
    solution:
      - |
        cat > src/asso.ts <<'EOF'
        export function total(prix: number, quantite: number): number {
          return prix * quantite;
        }
        EOF
  - text: 'Crée `.prettierrc` pour fixer le style de l''équipe : lignes de 100 caractères au plus (`printWidth`) et indentation de 4 espaces (`tabWidth`)'
    hint: 'Le fichier est du JSON : `{ "printWidth": 100, "tabWidth": 4 }`.'
    checks:
      - command-succeeds: 'verifier-ts 06 prettierrc'
    solution:
      - |
        cat > .prettierrc <<'EOF'
        {
            "printWidth": 100,
            "tabWidth": 4
        }
        EOF
  - text: 'Mets le dossier `src` en forme avec Prettier : `npx prettier --check src` doit réussir'
    hint: 'Lance `npx prettier --write src` : Prettier réécrit les fichiers selon `.prettierrc`.'
    after: [3]
    checks:
      - command-succeeds: 'verifier-ts 06 format'
    solution:
      - npx prettier --write src
  - text: 'Ajoute dans `package.json` les scripts `lint` (qui lance `eslint`) et `prettier-check` (qui lance `prettier --check src`), puis vérifie que `npm run lint && npm run prettier-check` réussit'
    hint: 'Comme pour `ts-check`, ajoute deux lignes dans la section `scripts` : `"lint": "eslint"` et `"prettier-check": "prettier --check src"`.'
    after: [2, 4]
    checks:
      - command-succeeds: 'verifier-ts 06 scripts'
    solution:
      - |-
        sed -i 's|"ts-check": "tsc --noEmit"|"ts-check": "tsc --noEmit",\n        "lint": "eslint",\n        "prettier-check": "prettier --check src"|' package.json
:::

## Vérifie tes acquis

:::quiz
Quel outil signale une variable déclarée mais jamais utilisée ?

- [ ] Prettier
- [ ] Le navigateur
- [x] ESLint
- [ ] `tsc --noEmit` dans tous les cas

> C'est le rôle d'ESLint, ici via la règle `no-unused-vars`. `tsc` ne le signale que si on active des options dédiées.
:::

:::quiz
Que fait `npm run prettier-check` dans MiniShop ?

- [x] Il vérifie la mise en forme sans modifier les fichiers
- [ ] Il reformate tous les fichiers
- [ ] Il lance les tests unitaires
- [ ] Il corrige les erreurs de type

> `prettier --check` signale les fichiers mal formatés ; `--write` les réécrit.
:::

:::quiz
Un fichier passe `tsc --noEmit` sans erreur. Peut-il quand même échouer à ESLint ?

- [ ] Non : si les types sont bons, le code est bon
- [x] Oui : ESLint contrôle d'autres choses, comme les `any` explicites
- [ ] Non : ESLint ne lit pas le TypeScript
- [ ] Oui, mais seulement si le fichier n'est pas formaté

> `tsc` et ESLint ne vérifient pas les mêmes règles. Ils se complètent.
:::

:::quiz
Dans le `.prettierrc` de MiniShop, que fixe `"printWidth": 100` ?

- [ ] Le nombre maximal de fichiers par dossier
- [ ] La largeur d'une tabulation
- [x] La longueur de ligne au-delà de laquelle Prettier renvoie à la ligne
- [ ] Le nombre de caractères d'un nom de variable

> Prettier coupe les lignes qui dépassent cette largeur ; la tabulation se règle avec `tabWidth`.
:::
