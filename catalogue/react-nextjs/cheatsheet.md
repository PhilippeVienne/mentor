## Composants et props

| Besoin | Écriture |
| --- | --- |
| Composant | `function Carte({ nom }: Readonly<{ nom: string }>) { return <p>{nom}</p>; }` (majuscule obligatoire) |
| Utiliser un composant | `<Carte nom="Gourde" />` (chaîne entre guillemets, le reste entre `{ }`) |
| Liste | `{items.map((i) => <Carte key={i.id} nom={i.nom} />)}` |
| Condition | `{ok ? <A /> : <B />}` ou `{ok && <A />}` |
| Contenu imbriqué | prop `children: ReactNode` |

## État, effets, formulaires

| Besoin | Écriture |
| --- | --- |
| État | `const [n, setN] = useState(1);` |
| Mise à jour depuis l'ancienne valeur | `setN((x) => x + 1)` |
| Ajouter à un tableau | `setListe([...liste, element])` (jamais `push`) |
| Modifier un objet | `setObjet({ ...objet, champ: valeur })` |
| Clic | `onClick={() => faire()}` (jamais `onClick={faire()}`) |
| Effet | `useEffect(() => { …; return () => nettoyage; }, [dependances]);` |
| Champ contrôlé | `value={v} onChange={(e) => setV(e.target.value)}` |
| Envoi de formulaire | `onSubmit={(e) => { e.preventDefault(); … }}` |

Dépendances : `[x]` à chaque changement de `x` ; `[]` une seule fois ; absent : à chaque affichage.

## Next.js (App Router)

| Fichier ou directive | Rôle |
| --- | --- |
| `app/page.tsx` | page de l'adresse `/` |
| `app/[boutique]/page.tsx` | segment dynamique : `/bde`, `/asso`… |
| `layout.tsx` | cadre commun, avec `children` |
| `route.ts` | route d'API : fonctions `GET`, `POST`… |
| `"use client"` | composant exécuté aussi dans le navigateur (hooks, événements) |
| `"use server"` | fonctions appelables depuis le navigateur, exécutées sur le serveur |
| `params` | `Promise<{ boutique: string }>` : à attendre avec `await` |
| `generateMetadata` | titre et description de la page |
| `notFound()` | affiche la page 404 |
| `redirect("/x")` | redirige vers `/x` |

Par défaut, un composant de `app/` est un composant serveur.

## Tailwind

| Classe | Effet |
| --- | --- |
| `p-4`, `px-6`, `py-2` | marge intérieure (tous côtés, horizontal, vertical) |
| `m-4`, `mt-2`, `mx-auto` | marge extérieure |
| `flex`, `flex-col`, `items-center`, `justify-between`, `gap-4` | disposition en ligne ou colonne |
| `grid grid-cols-3` | grille de trois colonnes |
| `text-sm`, `font-semibold`, `text-slate-600` | texte |
| `bg-white`, `rounded-lg`, `shadow-sm` | fond, coins, ombre |
| `sm:` `md:` `lg:` | à partir de 640 / 768 / 1024 px |
| `hover:` `focus:` `disabled:` | selon l'état |

Écris toujours les noms de classes en entier : `bg-${couleur}-500` ne fonctionne pas.

## Tests et outils du labo

| Commande | Rôle |
| --- | --- |
| `npx vitest run` | lance tous les tests (`vitest` est l'outil de test, `run` : une seule passe) |
| `npx vitest run carte` | lance les tests dont le nom de fichier contient `carte` |
| `npx tsc --noEmit` | vérifie les types de tout le projet, sans produire de fichier |
| `npx tsx fichier.tsx` | exécute un fichier TypeScript |
| `npx next build --webpack` | compile une application Next.js (leçon 4) |
| `classes-generees p-4 sm:flex` | vérifie que Tailwind génère bien ces classes (leçon 5) |

## Authentification

| Besoin | Où |
| --- | --- |
| Qui est connecté·e ? | `auth.api.getSession({ headers: await headers() })` côté serveur |
| Connexion côté navigateur | `authClient.signIn.email({ email, password })` |
| Protéger une page ou une action | garde (`requireAuth`) en première ligne, avant tout traitement |
| Filtrer une zone d'URL | `proxy.ts` avec un `matcher` |
| Secret | variable d'environnement sans `NEXT_PUBLIC_` |

Un bouton caché ne protège rien : les droits se vérifient côté serveur.

## Données, migrations, déploiement

```ts
const rows = await getDb()
  .selectFrom("product")
  .select(["id", "name"])
  .where("enabled", "=", true)
  .orderBy("id", "asc")
  .execute();
```

```bash
npm run migrate          # applique les migrations manquantes
npm run migrate:down     # annule la dernière migration
npm run generate-types   # régénère src/db/types.ts
npm run ts-check         # tsc --noEmit
npm run lint             # ESLint
npm run prettier-check   # formatage
npm run build            # compilation Next.js
docker-compose up -d     # démarre les conteneurs de compose.yaml : PostgreSQL, Redis, stockage de fichiers
```

Une migration exporte `up` et `down`. On ne modifie jamais une migration déjà déployée : on en écrit une nouvelle.
