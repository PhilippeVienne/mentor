---
id: donnees-deploiement
titre: "Base de données, migrations et déploiement"
resume: "Lire une base avec Kysely, la faire évoluer par migrations, accélérer avec Redis et livrer l'application en conteneur."
duree: 50
objectifs:
  - Écrire une requête typée avec Kysely et comprendre à quoi sert le fichier de types généré
  - Créer une migration réversible (`up` et `down`) et expliquer quand elle s'exécute
  - Expliquer le principe d'un cache et de son invalidation avec Redis
  - Lire et écrire le `Dockerfile` d'une application Next.js de l'équipe, et lire sa CI (ses vérifications automatiques)
---

Ton application affiche des produits, mais où sont-ils enregistrés ? Et le jour où tu dois ajouter une colonne « seuil d'alerte de stock », comment la créer sur ton ordinateur, sur celui de ta coéquipière et sur le serveur, sans rien casser ? Cette dernière leçon répond à ces deux questions, puis explique comment l'application part en production.

## À quoi servent une base de données et Kysely

Une **base de données** (ici **PostgreSQL**) range les informations dans des **tables**, comme des feuilles de tableur : la table `product` a une ligne par produit et une colonne par information (`name`, `price`…). On l'interroge avec **SQL**, un langage de requêtes (« donne-moi le nom des produits actifs »).

**Kysely** est une bibliothèque qui permet d'écrire ces requêtes **en TypeScript**. Son atout : le compilateur connaît les tables et leurs colonnes. Une faute de frappe sur `"price"` ou une comparaison avec le mauvais type est signalée dans l'éditeur, avant même d'exécuter le code. Pour cela, Kysely a besoin d'une description des tables : MiniShop la **génère** avec la commande `npm run generate-types`, qui lance `kysely-codegen` et écrit `src/db/types.ts` (le fichier porte en tête « Please do not edit it manually »).

## Une requête, ligne à ligne

Voici une requête dans le style de `src/app/actions/stock.ts` de MiniShop : les stocks des produits actifs.

```ts
import { getDb } from "@/db/database";

const lignes = await getDb()
  .selectFrom("product_version")
  .innerJoin("product", "product.id", "product_version.product_id")
  .where("product.enabled", "=", true)
  .select(["product.name", "product_version.stock"])
  .orderBy("product.id", "asc")
  .execute();
```

- `import { getDb } from "@/db/database"` : l'alias `@/` désigne le dossier `src`. `await` attend le résultat de la requête (elle est asynchrone, comme `fetch` à la leçon 3).
- `getDb()` renvoie la connexion à la base. Dans `src/db/database.ts`, c'est un **singleton** (un objet créé une seule fois et partagé) : la connexion est créée à la première utilisation (`_db ??= createDatabase()`, où `??=` n'affecte la valeur que si la variable est encore vide), puis réutilisée. Ouvrir une connexion à chaque requête serait lent.
- `selectFrom("product_version")` : « dans la table `product_version` ». Si le nom n'existe pas dans le fichier de types, TypeScript refuse.
- `innerJoin("product", "product.id", "product_version.product_id")` : une **jointure** relie deux tables : on associe chaque version à son produit quand les deux identifiants correspondent.
- `where("product.enabled", "=", true)` : un filtre. Les valeurs sont transmises à la base comme **paramètres**, jamais collées dans le texte SQL : c'est ce qui protège contre l'injection SQL (une attaque qui glisse du SQL dans un champ de formulaire).
- `select([...])` choisit les colonnes, `orderBy` trie.
- `execute()` lance la requête et renvoie un tableau de lignes, **typées** : `lignes[0].stock` est un `number`. Pour une seule ligne, MiniShop utilise `executeTakeFirst()`, qui renvoie la ligne ou `undefined`.

Pour plusieurs écritures qui doivent réussir **ensemble ou pas du tout**, on utilise une **transaction** : `db.transaction().execute(async (trx) => { … })`. Dans `updateProductVersion`, MiniShop modifie le stock et écrit une ligne d'historique dans la même transaction.

## Les migrations : faire évoluer la base proprement

Une **migration** est un petit fichier qui décrit **un changement de structure** de la base. Au lieu de modifier la base à la main, on versionne les changements dans Git, comme le code. Le dossier `src/migrations/` de MiniShop contient `001_initial_schema.ts` à `014_deliveries.ts`, appliquées dans l'ordre.

Chaque migration exporte deux fonctions : `up` (appliquer) et `down` (annuler). Exemple fictif, une colonne de plus :

```ts
import { Kysely } from "kysely";
import { DB } from "@/db/types";

export async function up(db: Kysely<DB>): Promise<void> {
  await db.schema
    .alterTable("product")
    .addColumn("seuil_alerte", "integer", (col) => col.notNull().defaultTo(0))
    .execute();
}

export async function down(db: Kysely<DB>): Promise<void> {
  await db.schema.alterTable("product").dropColumn("seuil_alerte").execute();
}
```

- `up(db: Kysely<DB>): Promise<void>` : la fonction reçoit la connexion `db`, dont le type `Kysely<DB>` connaît toutes les tables ; elle ne renvoie rien (`void`) mais est asynchrone, d'où `Promise`. `.execute()` envoie réellement la commande.
- `alterTable("product").addColumn(…)` : modifier la table, y ajouter une colonne entière.
- `notNull().defaultTo(0)` : la colonne ne peut pas être vide, et les lignes existantes reçoivent `0`. Sans valeur par défaut, l'ajout d'une colonne obligatoire échouerait sur une table déjà remplie.
- `down` fait exactement le contraire : on doit pouvoir revenir en arrière.

Kysely garde dans la base la liste des migrations déjà passées et n'applique que les nouvelles. Voici comment ça s'enchaîne dans MiniShop :

| Commande ou moment | Effet |
| --- | --- |
| `npm run migrate` | applique les migrations manquantes |
| `npm run dev` | lance `migrate` puis le serveur de développement |
| `npm run migrate:down` | annule la dernière migration |
| démarrage du serveur (`instrumentation.ts`) | applique aussi les migrations avant de servir |
| `npm run generate-types` | régénère `src/db/types.ts` à partir de la base |

La séquence correcte quand tu ajoutes une colonne : écrire la migration, la lancer, **régénérer les types**, puis utiliser la colonne dans le code.

:::danger Ne modifie jamais une migration déjà déployée
Une migration appliquée en production est figée. Si elle comporte une erreur, écris une **nouvelle** migration qui la corrige. Modifier l'ancienne ferait diverger les bases des développeur·se·s et celle de production, sans que Kysely s'en aperçoive.
:::

## Redis : éviter de refaire le même calcul

Un **cache** garde en mémoire le résultat d'une opération coûteuse pour la ressortir immédiatement la fois suivante. **Redis** est un petit serveur de cache très rapide ; MiniShop s'en sert aussi pour les sessions de better-auth. Dans `src/app/actions/product.ts`, les produits mis en avant sont gardés **300 secondes** (`PRODUCT_CACHE_TTL_SECONDS`, TTL signifie *time to live*, durée de vie).

Le motif porte un nom : *cache-aside* (« le cache à côté » : le code consulte d'abord le cache, et ne va à la source que s'il n'y trouve rien). En voici une version autonome, avec un simple dictionnaire en mémoire (un `Map`, qui associe une clé à une valeur) à la place de Redis :

```ts
type Entree<T> = { valeur: T; expireA: number };
const cache = new Map<string, Entree<unknown>>();
const TTL_SECONDES = 300;

export async function lireOuCalculer<T>(cle: string, charger: () => Promise<T>): Promise<T> {
  const trouve = cache.get(cle);
  if (trouve && trouve.expireA > Date.now()) {
    return trouve.valeur as T;
  }
  const valeur = await charger();
  cache.set(cle, { valeur, expireA: Date.now() + TTL_SECONDES * 1000 });
  return valeur;
}

export function invalider(cle: string): void {
  cache.delete(cle);
}
```

- `Entree<T>` est un type **générique** : `T` est un type à choisir à l'usage (une liste de goodies, un nombre…), et la fonction `lireOuCalculer<T>` renvoie ce même type. `trouve.valeur as T` affirme à TypeScript le type de la valeur retrouvée.
- `Date.now()` donne l'heure actuelle en millisecondes, d'où `TTL_SECONDES * 1000` pour ajouter 300 secondes.
- On cherche d'abord dans le cache ; si l'entrée existe et n'a pas expiré, on la renvoie sans toucher à la base.
- Sinon, `charger()` va chercher la donnée, on l'enregistre avec une date d'expiration, puis on la renvoie.
- `invalider` supprime l'entrée. MiniShop l'appelle à chaque modification d'un produit (`invalidateProductCachesByProductId`), sinon les visiteurs verraient l'ancienne version pendant 5 minutes.

Avec deux lectures consécutives, puis une invalidation et une troisième lecture, cette version affiche :

```console
appels à la base : 1
appels à la base après invalidation : 2
```

La clé `goodies:bde` suit l'esprit de celles de MiniShop (par exemple `shop:<id>:products:<actif>` dans `product.ts`) : un nom lisible, préfixé par ce qu'elle concerne. Le client Redis de MiniShop ajoute en plus un préfixe (`REDIS_PREFIX`) et, s'il ne parvient pas à se connecter, **renvoie `null` au lieu de planter** : le cache est une optimisation, l'application doit continuer à marcher sans lui.

## Livrer l'application

Pour faire tourner l'application à l'identique partout, on la met dans une **image Docker** : un paquet qui contient le programme et tout ce dont il a besoin (le parcours *Docker* détaille la notion). Une image se décrit dans un fichier texte, le `Dockerfile`, qui liste des instructions. Le `Dockerfile` de MiniShop procède en plusieurs étapes (*multi-stage* : une base commune, puis les trois ci-dessous) :

| Étape | Rôle |
| --- | --- |
| `deps` | installe les dépendances avec `npm ci` (installation exacte selon `package-lock.json`) |
| `builder` | copie le code et lance `npm run build` : Next.js compile l'application |
| `runner` | image finale, minimale : ne garde que le résultat de la compilation |

Voici un `Dockerfile` minimal pour une application Next.js, à deux étapes, que tu écriras dans le labo :

```dockerfile
FROM node:24-bookworm-slim AS builder
WORKDIR /app
COPY . .
RUN npm ci && npm run build

FROM node:24-bookworm-slim AS runner
WORKDIR /app
ENV NODE_ENV=production HOSTNAME="0.0.0.0"
RUN useradd --create-home --uid 1001 nextjs
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static
USER nextjs
EXPOSE 3000
CMD ["node", "server.js"]
```

- `FROM node:24-bookworm-slim AS builder` : part d'une image qui contient Node.js et nomme cette étape `builder`.
- `WORKDIR /app` : se place dans le dossier `/app` (créé au besoin) pour les instructions suivantes.
- `COPY . .` : copie le code du projet dans l'image. `RUN npm ci && npm run build` installe les dépendances puis compile l'application.
- Le second `FROM` démarre l'étape finale : on repart d'une image propre, sans les outils de compilation.
- `ENV` fixe des variables d'environnement ; `HOSTNAME="0.0.0.0"` fait écouter le serveur sur toutes les interfaces.
- `RUN useradd …` crée un utilisateur ordinaire `nextjs`, et `USER nextjs` fait tourner la suite sous cette identité.
- `COPY --from=builder …` ne rapporte que le résultat de la compilation depuis la première étape.
- `EXPOSE 3000` documente le port utilisé ; `CMD` est la commande lancée au démarrage du conteneur.

Trois détails importants :

- `next.config.ts` contient `output: "standalone"`. Next.js produit alors un dossier autonome avec uniquement les fichiers nécessaires, d'où une image finale légère, lancée par `node server.js`.
- L'image finale crée un utilisateur `nextjs` et passe à `USER nextjs` : l'application ne tourne **pas en administrateur** du conteneur.
- `ENV HOSTNAME="0.0.0.0"` et `EXPOSE 3000` : le serveur écoute sur toutes les interfaces, port 3000, pour que le conteneur soit joignable de l'extérieur.

La **CI** (*continuous integration*, « intégration continue ») est un robot de GitLab qui exécute automatiquement une série de tâches, décrites dans le fichier `.gitlab-ci.yml`, à chaque **merge request** (la demande de fusion par laquelle tu proposes tes modifications à l'équipe, qui les relit). Voici les vérifications de MiniShop :

| Job | Commande | Vérifie |
| --- | --- | --- |
| `ts-check` | `npm run ts-check` (`tsc --noEmit`) | que le TypeScript compile |
| `lint` | `npm run lint` | les règles ESLint |
| `prettier` | `npm run prettier-check` | le formatage |
| `build` | `npm run build` | que Next.js compile |
| `build-docker` | `docker build` puis `docker push` | construit et publie l'image (branche `main`) |

Un job planifié, `sync-shop-purchases`, appelle la route `/api/cron_jobs` vue à la leçon 4 avec le secret `CRON_SECRET`.

D'après les fichiers lus, MiniShop **n'a pas de suite de tests automatisés** : le filet de sécurité est le typage, le lint et le build. Si tu ajoutes de la logique non triviale (calcul de prix, de stock), propose des tests pour les fonctions pures comme celles de la leçon 2 : c'est la contribution la plus utile.

:::tip Démarrer en local
1. Copie `.env.example` vers `.env` et remplis les valeurs (génère `BETTER_AUTH_SECRET` avec `openssl rand -base64 32`, vu à la leçon 6).
2. Lance les services avec `docker-compose up -d`. `docker-compose` est la commande qui démarre ensemble plusieurs conteneurs décrits dans un fichier (ici `compose.yaml`) ; `up` les démarre et `-d` les laisse tourner en arrière-plan. Le fichier démarre PostgreSQL 17, Redis et RustFS (un stockage de fichiers).
3. Lance `npm install` puis `npm run dev` : les migrations s'appliquent, puis le site démarre sur `http://localhost:3000`.
:::

## Entraîne-toi

:::labo
moteur: reel
intro: |
  Démarre ton environnement. Il n'y a pas de base de données PostgreSQL dans l'environnement, ni de Redis, ni de Docker : tu travailles quand même avec de vrais outils. **Kysely** écrit le SQL (le langage de la base) comme d'habitude, et une « fausse » connexion (`src/faux-db.ts`) note chaque requête au lieu de l'envoyer : les **tests** (de petits programmes qui vérifient ton travail) lisent ce SQL. `npx vitest run requete` lance les tests dont le nom de fichier contient `requete`.

  Tu écriras dans `src` une requête (`stocks.ts`), une migration (`migrations/015_seuil_alerte.ts`) et un cache (`cache.ts`), puis deux fichiers de livraison : `next.config.ts` et un `Dockerfile`. Utilise `nano` pour éditer (`Ctrl+O` puis `Entrée` enregistre, `Ctrl+X` quitte). Le `Dockerfile` n'est pas construit ici (pas de Docker dans l'environnement) : le serveur vérifie qu'il contient ce qu'il faut.
commandes:
  - cp -R /opt/exercices/commun/. .
  - cp -R /opt/exercices/07-donnees-deploiement/. .
  - lier-outils
etapes:
  - texte: >-
      Dans `src/stocks.ts`, la fonction `stocksActifs` lit toute la table `product`. Réécris-la pour qu'elle renvoie le **nom** et le **stock** des produits **actifs** : pars de `product_version`, joins `product` (`product.id` égale `product_version.product_id`), filtre sur `product.enabled`, choisis ces deux colonnes seulement et trie par `product.id` croissant. Vérifie avec `npx vitest run requete`.
    indice: >-
      `db.selectFrom("product_version").innerJoin("product", "product.id", "product_version.product_id").where("product.enabled", "=", true).select(["product.name", "product_version.stock"]).orderBy("product.id", "asc").execute()`.
    verif:
      - commande-reussit: controler tests 07-donnees-deploiement requete
    solution:
      - ecrire:
          'src/stocks.ts': |
            import type { Kysely } from "kysely";
            import type { DB } from "./types";

            export async function stocksActifs(db: Kysely<DB>) {
              return db
                .selectFrom("product_version")
                .innerJoin("product", "product.id", "product_version.product_id")
                .where("product.enabled", "=", true)
                .select(["product.name", "product_version.stock"])
                .orderBy("product.id", "asc")
                .execute();
            }
  - texte: >-
      Dans `src/migrations/015_seuil_alerte.ts`, écris la fonction `up` : elle modifie la table `product` pour y ajouter la colonne entière `seuil_alerte`, obligatoire (`notNull()`) et valant `0` par défaut (`defaultTo(0)`). Vérifie avec `npx vitest run migration-up`.
    indice: >-
      `await db.schema.alterTable("product").addColumn("seuil_alerte", "integer", (col) => col.notNull().defaultTo(0)).execute();`.
    verif:
      - commande-reussit: controler tests 07-donnees-deploiement migration-up
    solution:
      - ecrire:
          'src/migrations/015_seuil_alerte.ts': |
            import type { Kysely } from "kysely";
            import type { DB } from "../types";

            export async function up(db: Kysely<DB>): Promise<void> {
              await db.schema
                .alterTable("product")
                .addColumn("seuil_alerte", "integer", (col) => col.notNull().defaultTo(0))
                .execute();
            }

            export async function down(db: Kysely<DB>): Promise<void> {}
  - texte: >-
      Une migration doit pouvoir être annulée. Écris la fonction `down` du même fichier : elle supprime la colonne `seuil_alerte` de `product`. Vérifie avec `npx vitest run migration-down`.
    indice: >-
      `await db.schema.alterTable("product").dropColumn("seuil_alerte").execute();`.
    verif:
      - commande-reussit: controler tests 07-donnees-deploiement migration-down
    apres: [2]
    solution:
      - ecrire:
          'src/migrations/015_seuil_alerte.ts': |
            import type { Kysely } from "kysely";
            import type { DB } from "../types";

            export async function up(db: Kysely<DB>): Promise<void> {
              await db.schema
                .alterTable("product")
                .addColumn("seuil_alerte", "integer", (col) => col.notNull().defaultTo(0))
                .execute();
            }

            export async function down(db: Kysely<DB>): Promise<void> {
              await db.schema.alterTable("product").dropColumn("seuil_alerte").execute();
            }
  - texte: >-
      Dans `src/cache.ts`, écris `lireOuCalculer(cle, charger)` selon le motif *cache-aside*. Cherche la clé dans un `Map`. Si l'entrée existe et n'a pas expiré (`expireA > Date.now()`), renvoie sa valeur sans appeler `charger`. Sinon appelle `charger()`, mémorise le résultat avec une date d'expiration dans `TTL_SECONDES` (300) secondes, puis renvoie-le. Vérifie avec `npx vitest run cache-lecture`.
    indice: >-
      Déclare `const cache = new Map<string, { valeur: unknown; expireA: number }>();`. Une durée en secondes se convertit en millisecondes (`Date.now()` compte en millisecondes) avec `TTL_SECONDES * 1000`.
    verif:
      - commande-reussit: controler tests 07-donnees-deploiement cache-lecture
    solution:
      - ecrire:
          'src/cache.ts': |
            type Entree<T> = { valeur: T; expireA: number };
            const cache = new Map<string, Entree<unknown>>();
            const TTL_SECONDES = 300;

            export async function lireOuCalculer<T>(cle: string, charger: () => Promise<T>): Promise<T> {
              const trouve = cache.get(cle);
              if (trouve && trouve.expireA > Date.now()) {
                return trouve.valeur as T;
              }
              const valeur = await charger();
              cache.set(cle, { valeur, expireA: Date.now() + TTL_SECONDES * 1000 });
              return valeur;
            }

            export function invalider(cle: string): void {}
  - texte: >-
      Un cache qui ne s'efface jamais sert de vieilles données. Écris `invalider(cle)` : elle supprime l'entrée de cette clé, sans toucher aux autres. Vérifie avec `npx vitest run cache-invalidation`.
    indice: >-
      `cache.delete(cle);` supprime une entrée d'un `Map`.
    verif:
      - commande-reussit: controler tests 07-donnees-deploiement cache-invalidation
    apres: [4]
    solution:
      - ecrire:
          'src/cache.ts': |
            type Entree<T> = { valeur: T; expireA: number };
            const cache = new Map<string, Entree<unknown>>();
            const TTL_SECONDES = 300;

            export async function lireOuCalculer<T>(cle: string, charger: () => Promise<T>): Promise<T> {
              const trouve = cache.get(cle);
              if (trouve && trouve.expireA > Date.now()) {
                return trouve.valeur as T;
              }
              const valeur = await charger();
              cache.set(cle, { valeur, expireA: Date.now() + TTL_SECONDES * 1000 });
              return valeur;
            }

            export function invalider(cle: string): void {
              cache.delete(cle);
            }
  - texte: >-
      Prépare la livraison. Dans `next.config.ts`, ajoute `output: "standalone"` à la configuration (Next.js produira un dossier autonome, d'où une image plus légère). Puis crée un `Dockerfile` à plusieurs étapes : une étape `builder` qui copie le code et lance `npm ci && npm run build`, et une étape finale qui copie `.next/standalone`, **passe à un utilisateur qui n'est pas root** avec `USER`, déclare `EXPOSE 3000` et lance `node server.js`.
    indice: >-
      Crée l'utilisateur avec `RUN useradd --create-home --uid 1001 nextjs`, puis `USER nextjs` avant `EXPOSE 3000` et `CMD ["node", "server.js"]`. Chaque étape commence par `FROM … AS nom`.
    verif:
      - commande-reussit: contient next.config.ts 'output\s*:\s*[\x27\"]standalone[\x27\"]'
      - commande-reussit: contient Dockerfile '^USER\s+(?!root\b)\S+'
      - commande-reussit: contient -v Dockerfile '^USER\s+root\b'
      - commande-reussit: contient Dockerfile '^EXPOSE\s+3000'
      - commande-reussit: contient Dockerfile '^FROM\s[^\n]*\n[\s\S]*^FROM\s'
      - commande-reussit: contient Dockerfile '^FROM\s[^\n]*\s+AS\s+builder\b'
      - commande-reussit: contient Dockerfile 'npm\s+ci\b[\s\S]*npm\s+run\s+build'
      - commande-reussit: contient Dockerfile '\.next/standalone'
      - commande-reussit: contient Dockerfile 'node[\s\x27\",\[]+server\.js'
    solution:
      - ecrire:
          'next.config.ts': |
            import type { NextConfig } from "next";

            const config: NextConfig = {
              output: "standalone",
            };

            export default config;
          'Dockerfile': |
            FROM node:24-bookworm-slim AS builder
            WORKDIR /app
            COPY . .
            RUN npm ci && npm run build

            FROM node:24-bookworm-slim AS runner
            WORKDIR /app
            ENV NODE_ENV=production HOSTNAME="0.0.0.0"
            RUN useradd --create-home --uid 1001 nextjs
            COPY --from=builder /app/.next/standalone ./
            COPY --from=builder /app/.next/static ./.next/static
            USER nextjs
            EXPOSE 3000
            CMD ["node", "server.js"]
:::

## Vérifie tes acquis

:::quiz
Tu viens d'ajouter une colonne par une migration. Que faire avant de l'utiliser dans le code TypeScript ?

- [ ] Redémarrer l'ordinateur
- [x] Régénérer `src/db/types.ts` avec `npm run generate-types`
- [ ] Modifier la migration précédente
- [ ] Rien, Kysely devine la colonne

> Kysely se fie au fichier de types : tant qu'il n'est pas régénéré, la colonne n'existe pas pour le compilateur.
:::

:::quiz
Une migration déjà déployée contient une erreur. Que fais-tu ?

- [ ] Je la modifie directement
- [ ] Je la supprime du dépôt
- [x] J'écris une nouvelle migration qui la corrige
- [ ] Je modifie la base à la main en production

> Une migration appliquée ne doit plus bouger, sinon les bases divergent.
:::

:::quiz
Pourquoi invalider le cache Redis quand un produit est modifié ?

- [ ] Pour libérer de la mémoire
- [ ] Pour accélérer la base de données
- [x] Pour que les visiteurs ne voient pas l'ancienne version jusqu'à l'expiration
- [ ] Parce que Redis refuse les mises à jour

> Sans invalidation, la valeur en cache reste servie jusqu'à la fin de son TTL (300 secondes).
:::

:::quiz
Quel est l'intérêt de `output: "standalone"` dans `next.config.ts` ?

- [ ] Désactiver le rendu côté serveur
- [ ] Supprimer la nécessité de Node.js
- [ ] Rendre le site hors ligne
- [x] Produire un dossier autonome, d'où une image Docker finale plus légère

> Le dossier `.next/standalone` contient seulement les fichiers nécessaires pour lancer `node server.js`.
:::
