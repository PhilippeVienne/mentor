## Types de base et unions

| Écriture | Sens |
| --- | --- |
| `let n: number = 3` | Variable annotée (souvent inutile : le type est déduit) |
| `string[]` | Tableau de textes |
| `string \| null` | Un texte ou `null` : à tester avant usage |
| `"a" \| "b"` | Union de valeurs littérales : liste fermée |
| `type Statut = …` | Donne un nom à un type |
| `interface Evenement { … }` | Forme d'un objet ; `champ?: type` = facultatif |
| `unknown` | Valeur inconnue : à vérifier avant usage (préférable à `any`) |

## Réduire un type

```ts
if (valeur === null) return;      // après : plus de null
if (typeof valeur === "string") { /* ici : string */ }
const taille = e.description?.length ?? 0;
```

## Fonctions et génériques

```ts
function f(a: number, b?: string): void {}
function premier<T>(liste: T[]): T | undefined { return liste[0]; }
function estX(v: unknown): v is X { return true; }
async function charger(): Promise<X[]> { /* … */ }
```

| Type fourni | Sens |
| --- | --- |
| `Partial<T>` | Toutes les propriétés facultatives |
| `Record<K, V>` | Objet clés `K` → valeurs `V` |
| `Promise<T>` | Valeur `T` disponible plus tard |

## Vérifier un projet

```bash
npm install --save-dev typescript
npx tsc --init          # crée tsconfig.json
npx tsc --noEmit        # vérifie les types, sans écrire de fichier
npm run ts-check        # script de MiniShop
npm run lint            # ESLint
npm run prettier-check  # mise en forme (sans modifier)
npm run prettier        # reformate les fichiers
```

## Options de tsconfig.json à connaître

`strict` · `noEmit` · `target` · `lib` · `module` / `moduleResolution` · `paths` (alias `@/`) · `include` / `exclude` · `skipLibCheck`

## Réflexes

1. Lis la première erreur de `tsc`, en commençant par la fin du message.
2. `as` ne vérifie rien : valide les données venues de l'extérieur (`unknown` + type guard).
3. Évite `any` ; ne désactive pas `strict` pour faire taire une erreur.
4. `tsc` = types, ESLint = bonnes pratiques, Prettier = mise en forme.
