---
id: authentification
title: "Authentification : better-auth et SSO"
summary: "Savoir qui est la personne et ce qu'elle a le droit de faire, avec better-auth (MiniShop) et Keycloak (Adhésion)."
minutes: 40
objectives:
  - Distinguer authentification et autorisation, et expliquer le rôle d'une session
  - Lire la configuration de `better-auth` et ses deux côtés (serveur et client)
  - Protéger une page, une action serveur et une zone d'URL
  - Expliquer comment un SSO comme Keycloak fournit un jeton à une application Next.js
---

Dans l'administration de MiniShop, une personne peut changer les stocks ; une cliente anonyme ne doit même pas voir la page. Pour que le site fasse la différence, il doit savoir **qui** fait la demande, puis **ce qu'elle a le droit de faire**. Ce sont deux questions distinctes, et se tromper sur l'une des deux est la faille de sécurité la plus courante des applications web.

## À quoi ça sert : deux questions, deux mots

- **Authentification** : « qui es-tu ? ». La personne le prouve (mot de passe, compte SSO…).
- **Autorisation** : « as-tu le droit de faire ça ? ». On compare son identité à ses **rôles** (`admin`, `cashier`…).

Une fois la personne authentifiée, le serveur crée une **session** : un enregistrement « cette personne est connectée » associé à un identifiant secret stocké dans un **cookie** (un petit texte que le navigateur renvoie à chaque requête). Le serveur retrouve ainsi la personne sans lui redemander son mot de passe à chaque page.

## Cas 1 : MiniShop et la bibliothèque `better-auth`

D'après son `package.json` et son fichier `src/lib/auth/auth.ts`, MiniShop gère lui-même les comptes (e-mail et mot de passe, dans sa base PostgreSQL) avec **better-auth**, une bibliothèque qui fournit l'inscription, la connexion, la vérification d'e-mail et les sessions. Voici la configuration, très simplifiée :

```ts
const auth = betterAuth({
  database: pool,
  basePath: "/api/auth",
  baseURL: process.env.BETTER_AUTH_URL,
  secret: process.env.BETTER_AUTH_SECRET,
  emailAndPassword: { enabled: true },
  emailVerification: { sendOnSignUp: true, requireEmailVerification: true },
  plugins: [nextCookies(), admin({ defaultRole: "user" })],
});
```

- `database: pool` : better-auth enregistre utilisateurs et sessions dans la même base PostgreSQL que le reste (tables `user`, `account`…).
- `basePath: "/api/auth"` : toutes ses adresses (connexion, déconnexion…) vivent sous `/api/auth`.
- `secret` : la clé qui signe les cookies de session. Elle vient d'une **variable d'environnement**, jamais du code. Le fichier `.env.example` du dépôt (un modèle de réglages, sans valeur secrète) indique comment en fabriquer une : `openssl rand -base64 32`. `openssl` est un outil en ligne de commande de cryptographie, présent sur Linux et macOS ; `rand` lui demande des octets aléatoires, `32` en demande trente-deux, et `-base64` les écrit sous forme de texte lisible (une écriture qui n'utilise que 64 caractères courants). Le résultat est ta clé secrète : copie-la dans ton fichier `.env`, jamais dans le code.
- `requireEmailVerification` : un compte doit confirmer son adresse avant de se connecter.
- `plugins` : des extensions. `nextCookies()` adapte better-auth aux cookies de Next.js, `admin(…)` ajoute la notion de rôle.

Une seule route fait le lien avec Next.js, dans `src/app/api/auth/[...all]/route.ts`. Le segment `[...all]` attrape toutes les adresses sous `/api/auth`, et deux lignes les confient à better-auth :

```ts
export const GET = authHandler.GET;
export const POST = authHandler.POST;
```

Côté navigateur, on utilise un **client** :

```ts
export const authClient = createAuthClient({ plugins: [adminClient()] });

const { error } = await authClient.signIn.email({ email, password, rememberMe: true });
if (error) {
  // afficher error.message
}
```

`signIn.email` appelle `/api/auth/sign-in/email` à ta place. Le résultat contient une `error` si les identifiants sont refusés : comme pour le chargement de la leçon 3, on traite l'échec explicitement.

## Protéger : trois barrières complémentaires

MiniShop ne se contente pas d'une protection ; il en empile trois.

**1. Le filtre d'URL (`src/proxy.ts`).** Next.js exécute ce fichier avant chaque requête. Celui de MiniShop laisse passer tout ce qui n'est pas sous `/admin`, et pour `/admin` demande la session à better-auth ; sans session, il redirige vers `/admin/login`. Sa logique de décision, isolée :

```ts
const PUBLIC = ["/admin/login", "/api/auth"];

export function estProtege(pathname: string): boolean {
  const zoneAdmin = pathname === "/admin" || pathname.startsWith("/admin/");
  return zoneAdmin && !PUBLIC.some((p) => pathname.startsWith(p));
}
```

Exécutée sur quelques chemins, elle donne :

```console
/admin true
/admin/stock true
/boutique false
/admin/login false
/administration false
```

Lis la fonction ligne à ligne : `pathname` est le chemin de l'adresse demandée (`/admin/stock`). `pathname === "/admin"` teste l'égalité exacte ; `pathname.startsWith("/admin/")` teste si le chemin commence par ce préfixe. `PUBLIC.some((p) => …)` vaut `true` si **au moins un** élément de la liste vérifie la condition. Le `&&` demande que les deux conditions soient vraies : être dans la zone admin et ne pas être public.

Remarque le dernier cas : `/administration` n'est pas protégé, car il ne commence pas par `/admin/`. Tester l'égalité exacte puis le préfixe avec `/` évite d'être trop large ou trop étroit.

**2. Le garde dans la page ou l'action (`requireAuth`).** Le filtre d'URL est pratique mais jamais suffisant. MiniShop vérifie aussi dans le code serveur, avec une fonction `requireAuth` du fichier `guardian.ts`. Version fictive équivalente :

```ts
import { headers } from "next/headers";
import { redirect } from "next/navigation";

type Role = "admin" | "caissier" | "user";

export async function exigerRole(roles: Role[]) {
  const session = await auth.api.getSession({ headers: await headers() });
  if (!session) redirect("/admin/login");
  if (!roles.includes(session.user.role)) redirect("/acces-refuse");
  return session;
}

// Dans un fichier qui commence par "use server"
export async function modifierStock(id: number, stock: number) {
  await exigerRole(["admin"]);
  // … seulement ensuite, modifier la base
}
```

- `auth.api.getSession({ headers })` lit le cookie présent dans les en-têtes de la requête et retrouve la session, ou `null`.
- `type Role = "admin" | "caissier" | "user"` : un type qui n'accepte que ces trois textes. `roles.includes(session.user.role)` vérifie que le rôle de la personne figure dans la liste autorisée.
- `redirect(…)` interrompt tout : pas de session, direction la connexion ; mauvais rôle, direction « accès refusé ». Comme `notFound()`, elle ne « revient » jamais, ce qui permet à TypeScript de savoir que `session` existe après les deux `if`.
- `exigerRole` est appelé **en première ligne** de l'action. C'est la règle d'or de la leçon 4 : une action serveur est une porte ouverte.

**3. Les rôles à deux niveaux.** MiniShop a des rôles globaux (`superadmin`, `admin`, `cashier`, `user`) et un rôle **par boutique** (table `user_shops`) : on peut être caissier d'une boutique et simple utilisatrice d'une autre. `requireAuth(roles, shopId, localRoles)` combine les deux.

:::warning Ne fais jamais confiance au navigateur
Cacher un bouton ou une page à l'interface ne protège rien : une personne peut appeler directement l'adresse d'une action ou d'une API. Chaque donnée sensible se contrôle **côté serveur**. De même, une variable `NEXT_PUBLIC_…` est envoyée à tout le monde : n'y mets jamais de secret.
:::

## Cas 2 : le SSO Keycloak du front d'Adhésion

Un **SSO** (*single sign-on*, connexion unique) permet de se connecter une fois pour accéder à plusieurs applications. Au lieu de gérer des mots de passe, l'application délègue la connexion à un serveur d'identité : dans l'équipe, **Keycloak**. Après connexion, Keycloak remet à l'application un **jeton** (*token*), une preuve signée d'identité et de durée limitée.

Le front d'Adhésion (`adhesion-front-public-next`) utilise la bibliothèque `keycloak-js`. Extrait simplifié de `MemberContext.tsx` :

```ts
const kc = new Keycloak({ url, realm, clientId });

kc.init({ onLoad: "login-required", checkLoginIframe: false }).then((auth) => {
  setToken(kc.token ?? null);
  setAuthenticated(auth);
});

kc.onTokenExpired = () => {
  kc.updateToken(5).then((refreshed) => {
    if (refreshed) setToken(kc.token ?? null);
  });
};
```

- `new Keycloak({ url, realm, clientId })` : l'adresse du serveur, le *realm* (l'espace d'identité de l'équipe, par défaut `exemple` dans le code) et l'identifiant de l'application auprès de Keycloak.
- `onLoad: "login-required"` : si la personne n'est pas connectée, elle est redirigée vers la page de connexion de Keycloak, puis revient sur le site avec un jeton.
- `kc.init(…).then((auth) => { … })` : `init` renvoie une promesse (un résultat qui arrivera plus tard) ; `.then` indique ce qu'il faut faire quand il arrive. `kc.token ?? null` prend le jeton, ou `null` s'il n'y en a pas. Les `set…` sont des mises à jour d'état, vues à la leçon 2.
- `onTokenExpired` : un jeton expire vite ; on demande à Keycloak d'en fournir un nouveau (`updateToken`) pour que la session continue.

Le jeton sert ensuite à appeler l'API d'Adhésion. Les fonctions de `app/actions/member.ts` l'ajoutent dans l'en-tête `Authorization: Bearer <jeton>` (*Bearer* signifie « porteur » : celui qui présente ce jeton est considéré comme autorisé) ; c'est l'API qui décide si la personne a le droit. Les adresses (Keycloak, API) viennent de variables d'environnement lues côté serveur par la fonction `getConfig()`, ce qui permet de changer d'environnement sans reconstruire l'image.

:::info Deux approches, un même principe
Dans MiniShop, l'application **possède** les comptes (better-auth). Dans Adhésion, elle **délègue** à Keycloak. Dans les deux cas : l'identité vient d'une source de confiance, et c'est le serveur qui vérifie les droits. Vérifie auprès de l'équipe quelle approche s'applique à un nouveau projet.
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  Démarre ton environnement. Tu vas écrire, dans `src`, les quatre barrières d'une petite application : un filtre d'URL (`proxy.ts`), un garde de rôle (`guardian.ts`), une action serveur protégée (`actions.ts`) et la lecture d'un secret (`config.ts`), puis l'en-tête d'un jeton Keycloak (`keycloak.ts`). Utilise `nano` pour éditer (`Ctrl+O` puis `Entrée` enregistre, `Ctrl+X` quitte).

  L'environnement ne contient ni better-auth ni Keycloak : `src/auth.ts` est une **fausse** bibliothèque, et les **tests** (de petits programmes qui vérifient ton travail) la remplacent par des sessions de leur choix (aucune, simple utilisateur, admin). `npx vitest run proxy` lance les tests dont le nom de fichier contient `proxy`. Aucun vrai secret n'est jamais utilisé : les tests inventent des valeurs.
commands:
  - cp -R /opt/exercices/commun/. .
  - cp -R /opt/exercices/06-authentification/. .
  - lier-outils
steps:
  - text: >-
      Dans `src/proxy.ts`, corrige `estProtege(pathname)` : une adresse est protégée si elle est exactement `/admin` ou commence par `/admin/`, **sauf** si elle commence par une des adresses publiques de la liste `PUBLIC`, qui doit contenir `/admin/login` (la page de connexion) et `/api/auth` (les adresses de better-auth). Attention à `/administration` : elle n'est pas dans la zone admin. Vérifie avec `npx vitest run proxy`.
    hint: >-
      `const zoneAdmin = pathname === "/admin" || pathname.startsWith("/admin/");` puis `return zoneAdmin && !PUBLIC.some((p) => pathname.startsWith(p));`.
    checks:
      - command-succeeds: controler tests 06-authentification proxy
    solution:
      - write:
          'src/proxy.ts': |
            // Le filtre d'URL du site : quelles adresses exigent une session ?
            const PUBLIC = ["/admin/login", "/api/auth"];

            export function estProtege(pathname: string): boolean {
              const zoneAdmin = pathname === "/admin" || pathname.startsWith("/admin/");
              return zoneAdmin && !PUBLIC.some((p) => pathname.startsWith(p));
            }
  - text: >-
      Dans `src/guardian.ts`, écris `exigerRole(roles)` : elle retrouve la session avec `auth.api.getSession({ headers: await headers() })`. Sans session, elle redirige vers `/admin/login` avec `redirect`. Si le rôle de la personne (`session.user.role`) n'est pas dans `roles`, elle redirige vers `/acces-refuse`. Sinon elle renvoie la session. Vérifie avec `npx vitest run guardian`.
    hint: >-
      `redirect(…)` interrompt tout (il lance une exception spéciale), donc après `if (!session) redirect(…);` TypeScript sait que `session` existe. Teste le rôle avec `roles.includes(session.user.role)`.
    checks:
      - command-succeeds: controler tests 06-authentification guardian
      - command-succeeds: contient src/guardian.ts '\bredirect\s*\('
    solution:
      - write:
          'src/guardian.ts': |
            import { headers } from "next/headers";
            import { redirect } from "next/navigation";
            import { auth, type Role } from "./auth";

            export async function exigerRole(roles: Role[]) {
              const session = await auth.api.getSession({ headers: await headers() });
              if (!session) redirect("/admin/login");
              if (!roles.includes(session.user.role)) redirect("/acces-refuse");
              return session;
            }
  - text: >-
      `modifierStock` dans `src/actions.ts` est une action serveur : n'importe qui peut l'appeler, même sans passer par ton interface. Ajoute `await exigerRole(["admin"]);` **en première ligne** de la fonction, avant `ecrireStock`. Vérifie avec `npx vitest run actions` : les tests contrôlent que la base n'est pas touchée sans les droits.
    hint: >-
      La fonction devient : `await exigerRole(["admin"]); await ecrireStock(id, stock);`. Si le garde lance une exception, la ligne suivante ne s'exécute jamais.
    checks:
      - command-succeeds: controler tests 06-authentification actions
      - command-succeeds: contient src/actions.ts 'modifierStock[^{]*\{\s*await\s+exigerRole\s*\(\s*\[\s*[\x27\"]admin[\x27\"]\s*\]\s*\)'
    after: [2]
    solution:
      - write:
          'src/actions.ts': |
            "use server";

            import { exigerRole } from "./guardian";
            import { ecrireStock } from "./stock-db";

            export async function modifierStock(id: number, stock: number) {
              await exigerRole(["admin"]);
              await ecrireStock(id, stock);
            }
  - text: >-
      Dans `src/config.ts`, écris `lireSecret()` : elle renvoie la variable d'environnement `BETTER_AUTH_SECRET` (`process.env.BETTER_AUTH_SECRET`) et lance une erreur « BETTER_AUTH_SECRET manquant » si elle est absente ou vide. Le secret ne doit jamais être écrit dans le code, ni dans une variable `NEXT_PUBLIC_…` (envoyée à tout le monde). Vérifie avec `npx vitest run config`.
    hint: >-
      `const secret = process.env.BETTER_AUTH_SECRET; if (!secret) { throw new Error("BETTER_AUTH_SECRET manquant"); } return secret;` : une chaîne vide est « fausse » en JavaScript, elle est donc refusée aussi.
    checks:
      - command-succeeds: controler tests 06-authentification config
      - command-fails: grep -rEq 'NEXT_PUBLIC_[A-Z_]*SECRET' src --exclude=*.test.ts
      - command-succeeds: contient src/config.ts 'process\.env\.BETTER_AUTH_SECRET'
    solution:
      - write:
          'src/config.ts': |
            export function lireSecret(): string {
              const secret = process.env.BETTER_AUTH_SECRET;
              if (!secret) {
                throw new Error("BETTER_AUTH_SECRET manquant");
              }
              return secret;
            }
  - text: >-
      Pour l'API d'Adhésion, le jeton remis par Keycloak voyage dans l'en-tête `Authorization`. Dans `src/keycloak.ts`, écris `entetesAuthorization(token)` : elle renvoie `{ Authorization: "Bearer <jeton>" }`, ou un objet vide quand il n'y a pas de jeton (`null`). Vérifie avec `npx vitest run keycloak`, puis lance toute la suite avec `npx vitest run` et `npx tsc --noEmit`.
    hint: >-
      Un opérateur ternaire suffit : `` token ? { Authorization: `Bearer ${token}` } : {} ``.
    checks:
      - command-succeeds: controler tests 06-authentification keycloak
      - command-succeeds: controler tests 06-authentification
      - command-succeeds: controler types 06-authentification
    after: [1, 2, 3, 4]
    solution:
      - write:
          'src/keycloak.ts': |-
            export function entetesAuthorization(token: string | null): Record<string, string> {
              return token ? { Authorization: `Bearer ${token}` } : {};
            }
:::

## Vérifie tes acquis

:::quiz
Quelle est la différence entre authentification et autorisation ?

- [ ] Aucune, ce sont deux noms pour la même chose
- [ ] L'authentification concerne le serveur, l'autorisation le navigateur
- [x] L'authentification établit qui est la personne, l'autorisation établit ce qu'elle peut faire
- [ ] L'autorisation se fait avant l'authentification

> On identifie d'abord la personne, puis on compare son identité à ses droits.
:::

:::quiz
Pourquoi appeler `requireAuth` au début de chaque action serveur, alors que `proxy.ts` filtre déjà `/admin` ?

- [ ] Par habitude, car c'est redondant
- [x] Parce qu'une action peut être appelée directement, sans passer par une page protégée
- [ ] Parce que `proxy.ts` ne fonctionne qu'en développement
- [ ] Parce que les actions n'ont pas accès aux cookies

> Chaque porte d'entrée se protège elle-même ; le filtre d'URL n'est qu'une première barrière.
:::

:::quiz
Où doit être stocké le `BETTER_AUTH_SECRET` ?

- [ ] Dans une variable `NEXT_PUBLIC_BETTER_AUTH_SECRET`
- [ ] En dur dans `auth.ts`
- [ ] Dans le fichier `README.md` pour l'équipe
- [x] Dans une variable d'environnement, hors du dépôt

> Une variable `NEXT_PUBLIC_` est envoyée au navigateur, et un secret commité est un secret perdu.
:::

:::quiz
Dans le front d'Adhésion, que fait `onLoad: "login-required"` ?

- [ ] Il affiche un formulaire de connexion écrit par l'application
- [ ] Il supprime le jeton à chaque chargement
- [x] Il redirige vers Keycloak si la personne n'est pas connectée
- [ ] Il crée un compte automatiquement

> Keycloak gère la page de connexion, puis renvoie la personne avec un jeton.
:::
