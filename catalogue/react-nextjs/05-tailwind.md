---
id: tailwind
titre: "Mettre en forme avec Tailwind CSS"
resume: "Styler un composant en combinant de petites classes utilitaires, adaptées à tous les écrans."
duree: 30
objectifs:
  - Expliquer ce qu'est une classe utilitaire et pourquoi Tailwind en propose autant
  - Lire et écrire des classes de mise en page, d'espacement et de couleur
  - Rendre une interface adaptative avec les préfixes `sm:`, `md:`, `lg:` et `hover:`
  - Éviter les classes construites dynamiquement
---

Un composant affiche les bonnes données, mais il est brut : texte collé au bord, pas de couleur, une seule colonne même sur grand écran. Il faut le mettre en forme avec du **CSS** (le langage qui décrit l'apparence d'une page : couleurs, marges, disposition). Tailwind propose une manière de le faire sans quitter ton fichier `.tsx`.

## À quoi sert Tailwind

Avec le CSS classique, tu inventes un nom de classe (`.carte-produit`), tu l'écris dans un autre fichier et tu y décris tout. **Tailwind CSS** retourne l'approche : il fournit des milliers de petites classes qui font **une seule chose chacune**, et tu les assembles dans l'attribut de ton élément.

| Classe | Effet CSS |
| --- | --- |
| `px-6` | marge intérieure horizontale de 1,5 rem (`padding-left` et `padding-right`) ; le **rem** est une unité de longueur proportionnelle à la taille du texte de la page, 1 rem valant en général 16 pixels |
| `text-sm` | petite taille de texte |
| `font-semibold` | gras moyen |
| `text-slate-600` | texte gris-bleu foncé |
| `rounded-lg` | coins arrondis |
| `shadow-sm` | légère ombre |

En JSX l'attribut HTML `class` s'appelle `className` (car `class` est un mot réservé de JavaScript).

```tsx
<article className="rounded-lg bg-white p-4 shadow-sm">
  <h3 className="text-lg font-semibold text-slate-900">Gourde Éco</h3>
  <p className="mt-2 text-sm text-slate-600">Inox, 50 cl.</p>
</article>
```

Lis-le classe par classe : `rounded-lg` arrondit les coins, `bg-white` met un fond blanc, `p-4` ajoute 1 rem de marge intérieure sur les quatre côtés, `shadow-sm` ajoute l'ombre. Dans le `<p>`, `mt-2` est une marge **extérieure** au-dessus (`m` = *margin*, `t` = *top*). Les nombres suivent une échelle : 1 unité = 0,25 rem, donc `p-4` = 1 rem.

Les deux projets de l'équipe déclarent Tailwind en version 4. Dans MiniShop, l'installation tient en deux fichiers : `postcss.config.mjs` déclare le plugin `@tailwindcss/postcss` (PostCSS est l'outil qui transforme le CSS avant de l'envoyer au navigateur), et `globals.css` commence par `@import "tailwindcss";`, la ligne CSS qui charge tout le framework.

## Disposer les éléments : flex et grille

Deux outils de CSS reviennent partout :

- **Flexbox** (`flex`) aligne des éléments sur une ligne ou une colonne. `flex-col` passe en colonne, `items-center` centre sur l'axe transversal, `justify-between` écarte les éléments, `gap-4` met un espace entre eux.
- **Grille** (`grid`) découpe l'espace en colonnes. `grid-cols-3` fait trois colonnes égales.

La page d'accueil de MiniShop affiche ses catégories avec :

```tsx
<div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
  {categories.map((cat) => (
    <Card key={cat.id}>{cat.name}</Card>
  ))}
</div>
```

## Adapter à la taille de l'écran : les préfixes

Un **préfixe** conditionne une classe. Les préfixes de taille sont **mobile d'abord** : une classe sans préfixe s'applique partout, et `sm:`, `md:`, `lg:` ne s'appliquent qu'à partir d'une largeur minimale (640, 768 et 1024 pixels).

Lis donc `grid gap-4 sm:grid-cols-2 lg:grid-cols-3` ainsi : une grille avec un espace entre les cases ; sur un petit écran, une colonne (valeur par défaut) ; dès 640 px, deux colonnes ; dès 1024 px, trois colonnes.

D'autres préfixes décrivent un **état** : `hover:` (survol de la souris), `focus:`, `disabled:`. La barre de navigation de MiniShop cache ses liens sur téléphone et les montre sur écran moyen avec `hidden … md:flex`, et les cartes se soulèvent au survol avec `transition hover:-translate-y-2 hover:shadow-md`.

Pour vérifier que ces classes produisent bien le CSS attendu, on a compilé un échantillon avec Tailwind 4 (la version de l'environnement du labo). Extrait du résultat :

```console
@media (hover: hover) { .hover\:-translate-y-2:hover { … } }
@media (width >= 40rem) { .sm\:grid-cols-2 { … } }
@media (width >= 48rem) { .md\:flex { … } }
@media (width >= 64rem) { .lg\:grid-cols-3 { … } }
```

Les préfixes se traduisent bien en **media queries** (règles CSS conditionnées, ici par la largeur d'écran) ou en pseudo-classes comme `:hover`. Les largeurs sont écrites en `rem` : 40 rem × 16 pixels = 640 pixels, et ainsi de suite. Le `\` devant le `:` est un échappement CSS, car le deux-points est un caractère spécial dans un sélecteur.

## Piège : les classes ne se construisent pas à la volée

Tailwind ne fabrique que les classes qu'il **lit telles quelles** dans tes fichiers. Si tu assembles le nom avec une variable, il ne le voit pas et le style n'existe pas.

```tsx
// ✗ Ne marchera pas : "bg-red-500" n'apparaît nulle part en entier
<p className={`bg-${couleur}-500`}>…</p>

// ✓ Écris les noms complets
const FONDS = { rouge: "bg-red-500", bleu: "bg-blue-500" } as const;
<p className={FONDS[couleur]}>…</p>
```

Dans ce code, `FONDS` est un objet qui associe chaque couleur à sa classe écrite en entier, `as const` dit à TypeScript que ces textes ne changeront pas, et `FONDS[couleur]` va chercher la bonne classe.

Quand la couleur vient de la base de données et peut être n'importe quelle valeur, MiniShop ne passe pas par une classe mais par l'attribut `style`, valable pour une valeur libre : `style={{ backgroundColor: shopSettings?.accent_color ?? "#104e64" }}`. Les doubles accolades ne sont pas une syntaxe spéciale : les premières ouvrent du JavaScript dans le JSX, les secondes forment l'objet de styles. L'opérateur `??` choisit la valeur de droite quand celle de gauche est absente, et `?.` lit un champ sans planter si l'objet est absent.

:::info Tailwind et les bibliothèques de composants
MiniShop combine Tailwind avec **Ant Design** (des composants prêts à l'emploi : `Button`, `Card`, `Form`), et le front d'Adhésion avec **HeroUI**. Quand Tailwind et la bibliothèque se contredisent, on force Tailwind avec le modificateur `!` : `className="!mb-0"` signifie « marge basse nulle, même si le composant en imposait une ». À utiliser avec modération.
:::

:::tip Laisse Prettier trier les classes
Les deux projets déclarent `prettier-plugin-tailwindcss`. **Prettier** est un outil qui met en forme le code automatiquement (indentation, guillemets…) ; ce plugin lui apprend à trier les classes Tailwind. Au formatage (`npm run prettier`), les classes sont réordonnées dans un ordre standard. Les relectures de code y gagnent, et la CI de MiniShop vérifie le formatage avec `prettier-check`.
:::

## Entraîne-toi

:::labo
moteur: reel
intro: |
  Démarre ton environnement. Tu vas mettre en forme cinq petits composants de `src` avec **Tailwind CSS** (version 4). Le fichier `globals.css` (déjà écrit) charge Tailwind avec `@import "tailwindcss";`. Modifie les composants avec `nano` (`Ctrl+O` puis `Entrée` enregistre, `Ctrl+X` quitte).

  Il n'y a pas de navigateur pour juger l'apparence : deux outils vérifient ton travail. Les **tests** (`npx vitest run carte` lance ceux dont le nom de fichier contient `carte`) regardent les classes posées sur chaque élément. La commande `classes-generees` compile `globals.css` avec Tailwind, comme le ferait le build du site, et vérifie que chaque classe citée a bien reçu une règle CSS. Ainsi, une classe mal écrite ou construite à la volée est détectée, même si le test, lui, est content.
commandes:
  - cp -R /opt/exercices/commun/. .
  - cp -R /opt/exercices/05-tailwind/. .
  - lier-outils
etapes:
  - texte: >-
      Dans `src/Carte.tsx`, habille la carte avec des classes utilitaires dans l'attribut `className` : l'`<article>` reçoit `rounded-lg bg-white p-4 shadow-sm`, le titre `<h3>` reçoit `text-lg font-semibold text-slate-900` et le paragraphe `mt-2 text-sm text-slate-600`. Vérifie avec `npx vitest run carte`, puis avec `classes-generees rounded-lg bg-white p-4 shadow-sm text-lg font-semibold text-slate-900 mt-2 text-sm text-slate-600`.
    indice: >-
      En JSX, l'attribut HTML `class` s'écrit `className`. Exemple : `<article className="rounded-lg bg-white p-4 shadow-sm">`.
    verif:
      - commande-reussit: controler tests 05-tailwind carte
      - commande-reussit: classes-generees rounded-lg bg-white p-4 shadow-sm text-lg font-semibold text-slate-900 mt-2 text-sm text-slate-600
    solution:
      - ecrire:
          'src/Carte.tsx': |
            export function Carte({ nom, description }: Readonly<{ nom: string; description: string }>) {
              return (
                <article className="rounded-lg bg-white p-4 shadow-sm">
                  <h3 className="text-lg font-semibold text-slate-900">{nom}</h3>
                  <p className="mt-2 text-sm text-slate-600">{description}</p>
                </article>
              );
            }
  - texte: >-
      Dans `src/GrilleCategories.tsx`, transforme la `<div>` en grille **adaptative**, « mobile d'abord » : `grid gap-4` partout, une seule colonne par défaut, deux colonnes dès l'écran `sm` (640 pixels) avec `sm:grid-cols-2`, trois dès `lg` (1024 pixels) avec `lg:grid-cols-3`. Vérifie avec `npx vitest run grille` et `classes-generees grid gap-4 sm:grid-cols-2 lg:grid-cols-3`.
    indice: >-
      Les classes sans préfixe valent pour toutes les largeurs ; `sm:` et `lg:` ne s'appliquent qu'à partir d'une largeur minimale : `className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3"`.
    verif:
      - commande-reussit: controler tests 05-tailwind grille
      - commande-reussit: classes-generees grid gap-4 sm:grid-cols-2 lg:grid-cols-3
    solution:
      - ecrire:
          'src/GrilleCategories.tsx': |
            import type { ReactNode } from "react";

            export function GrilleCategories({ children }: Readonly<{ children: ReactNode }>) {
              return <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">{children}</div>;
            }
  - texte: >-
      Dans `src/Bouton.tsx`, donne au bouton un fond `bg-sky-600` qui devient `hover:bg-sky-700` au survol de la souris, et une opacité réduite `disabled:opacity-50` quand il est désactivé. Ajoute aussi `rounded-md px-4 py-2 text-white`. Vérifie avec `npx vitest run bouton` et `classes-generees rounded-md px-4 py-2 text-white bg-sky-600 hover:bg-sky-700 disabled:opacity-50`.
    indice: >-
      Un préfixe d'état se colle devant la classe, avec deux-points : `hover:bg-sky-700`, `disabled:opacity-50`.
    verif:
      - commande-reussit: controler tests 05-tailwind bouton
      - commande-reussit: classes-generees rounded-md px-4 py-2 text-white bg-sky-600 hover:bg-sky-700 disabled:opacity-50
    solution:
      - ecrire:
          'src/Bouton.tsx': |
            import type { ReactNode } from "react";

            export function Bouton({ children, disabled }: Readonly<{ children: ReactNode; disabled?: boolean }>) {
              return (
                <button
                  type="button"
                  disabled={disabled}
                  className="rounded-md bg-sky-600 px-4 py-2 text-white hover:bg-sky-700 disabled:opacity-50"
                >
                  {children}
                </button>
              );
            }
  - texte: >-
      `src/Pastille.tsx` assemble le nom de sa classe à la volée avec `` `bg-${nom}-500` `` : à l'écran du test, tout va bien, mais Tailwind ne voit jamais `bg-red-500` écrit en entier et ne génère pas son CSS. Lance `classes-generees bg-red-500 bg-blue-500` pour le constater, puis corrige le composant avec un objet qui associe chaque couleur à sa classe **complète**. Vérifie avec `npx vitest run pastille` et `classes-generees bg-red-500 bg-blue-500`.
    indice: >-
      `const FONDS = { rouge: "bg-red-500", bleu: "bg-blue-500" } as const;` puis `` className={`${FONDS[couleur]} inline-block size-4 rounded-full`} ``.
    verif:
      - commande-reussit: controler tests 05-tailwind pastille
      - commande-reussit: classes-generees bg-red-500 bg-blue-500
      - commande-reussit: contient -v src/Pastille.tsx 'bg-\$\{'
    solution:
      - ecrire:
          'src/Pastille.tsx': |
            export type Couleur = "rouge" | "bleu";

            const FONDS = { rouge: "bg-red-500", bleu: "bg-blue-500" } as const;

            export function Pastille({ couleur }: Readonly<{ couleur: Couleur }>) {
              return <span className={`${FONDS[couleur]} inline-block size-4 rounded-full`} />;
            }
  - texte: >-
      Quand la couleur vient de la base de données, elle peut être n'importe quelle valeur : une classe ne convient plus. Dans `src/Badge.tsx`, applique la couleur reçue avec l'attribut `style` (`backgroundColor`), et `#104e64` quand elle est absente. Vérifie avec `npx vitest run badge`.
    indice: >-
      En JSX, `style` reçoit un objet : `style={{ backgroundColor: couleur ?? "#104e64" }}`. L'opérateur `??` choisit la valeur de droite quand celle de gauche est absente.
    verif:
      - commande-reussit: controler tests 05-tailwind badge
      - commande-reussit: contient src/Badge.tsx 'backgroundColor'
    solution:
      - ecrire:
          'src/Badge.tsx': |
            export function Badge({ texte, couleur }: Readonly<{ texte: string; couleur?: string }>) {
              return (
                <span className="rounded px-2 text-white" style={{ backgroundColor: couleur ?? "#104e64" }}>
                  {texte}
                </span>
              );
            }
:::

## Vérifie tes acquis

:::quiz
Que signifie `className="grid gap-4 sm:grid-cols-2"` sur un téléphone de 400 px de large ?

- [ ] Deux colonnes
- [x] Une seule colonne, car `sm:` ne s'applique qu'à partir de 640 px
- [ ] Aucune grille
- [ ] Une erreur de compilation

> Les préfixes de taille sont « mobile d'abord » : sans préfixe, la classe vaut pour toutes les largeurs.
:::

:::quiz
Quelle classe ajoute une marge intérieure de 1 rem sur les quatre côtés ?

- [ ] `m-4`
- [ ] `gap-4`
- [x] `p-4`
- [ ] `px-4`

> `p` est le padding (intérieur), `m` le margin (extérieur), `gap` l'espace entre enfants et `px` seulement l'horizontal.
:::

:::quiz
Pourquoi `className={`bg-${couleur}-500`}` ne fonctionne-t-il pas ?

- [ ] Parce que les accolades sont interdites dans `className`
- [ ] Parce que Tailwind refuse les couleurs en variable
- [x] Parce que Tailwind ne génère que les classes écrites en entier dans le code
- [ ] Parce que `500` n'est pas une valeur valide

> Il faut écrire `bg-red-500` en entier quelque part, ou utiliser `style` pour une valeur libre.
:::

:::quiz
Quel préfixe applique un style seulement quand la souris survole l'élément ?

- [ ] `md:`
- [x] `hover:`
- [ ] `active-only:`
- [ ] `over:`

> `hover:` est le préfixe d'état de survol.
:::
