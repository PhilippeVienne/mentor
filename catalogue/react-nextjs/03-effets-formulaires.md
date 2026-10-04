---
id: effets-formulaires
titre: "Effets et formulaires"
resume: "Charger des données au bon moment avec `useEffect` et construire un formulaire contrôlé."
duree: 40
objectifs:
  - Expliquer ce qu'est un effet de bord et quand utiliser `useEffect`
  - Lire un tableau de dépendances et écrire une fonction de nettoyage
  - Construire un formulaire contrôlé avec validation et état d'envoi
  - Gérer les trois états d'un chargement (en cours, erreur, succès)
---

Ton composant doit afficher la liste des goodies qui vient d'un serveur. Mais un composant n'est qu'une fonction qui décrit un affichage : y mettre un appel réseau directement serait une erreur, car React rappelle cette fonction très souvent. Il te faut un endroit prévu pour « faire quelque chose en dehors de l'affichage ».

## À quoi sert `useEffect`

Un **effet de bord** est toute action qui sort du simple calcul d'affichage : appeler un serveur, lire le `localStorage`, démarrer un minuteur. `useEffect` est le hook (la fonction de React qui commence par `use`) qui dit : « **après** avoir affiché, exécute ce code ».

Sa forme est toujours la même :

```tsx
useEffect(() => {
  // 1. ce qu'il faut faire après l'affichage
  return () => {
    // 2. (facultatif) le nettoyage, avant le prochain effet ou à la disparition du composant
  };
}, [dependance]); // 3. quand relancer l'effet
```

Le troisième élément, le **tableau de dépendances**, est le plus important :

| Tableau | L'effet s'exécute… |
| --- | --- |
| `[boutique]` | après le premier affichage, puis à chaque fois que `boutique` change |
| `[]` | une seule fois, après le premier affichage |
| absent | après **chaque** affichage (presque toujours une erreur) |

## Charger des données, ligne à ligne

Voici un composant qui charge les goodies d'une boutique. L'URL `/api/boutiques/…` est fictive. Deux mots à connaître avant de lire : `fetch` est la fonction du navigateur qui envoie une requête au serveur, et elle renvoie une **promesse**, c'est-à-dire un résultat qui arrivera plus tard. Le mot-clé `await` attend ce résultat, et n'est permis que dans une fonction déclarée `async`.

```tsx
import { useEffect, useState } from "react";

type Goodie = { id: number; nom: string };

export function ListeGoodies({ boutique }: Readonly<{ boutique: string }>) {
  const [goodies, setGoodies] = useState<Goodie[]>([]);
  const [chargement, setChargement] = useState(true);
  const [erreur, setErreur] = useState<string | null>(null);

  useEffect(() => {
    let annule = false;

    async function charger() {
      setChargement(true);
      setErreur(null);
      try {
        const reponse = await fetch(`/api/boutiques/${boutique}/goodies`);
        if (!reponse.ok) throw new Error(`Erreur HTTP ${reponse.status}`);
        const donnees = (await reponse.json()) as Goodie[];
        if (!annule) setGoodies(donnees);
      } catch (e) {
        if (!annule) setErreur(e instanceof Error ? e.message : "Erreur inconnue");
      } finally {
        if (!annule) setChargement(false);
      }
    }

    void charger();
    return () => {
      annule = true;
    };
  }, [boutique]);

  if (chargement) return <p>Chargement…</p>;
  if (erreur) return <p role="alert">{erreur}</p>;
  return (
    <ul>
      {goodies.map((g) => (
        <li key={g.id}>{g.nom}</li>
      ))}
    </ul>
  );
}
```

- `useState<Goodie[]>([])` : le `<Goodie[]>` précise le type de l'état (une liste de goodies), qui démarre vide. `useState<string | null>(null)` signifie « un texte, ou rien ».
- Trois états : `goodies` (le résultat), `chargement` (on attend) et `erreur` (quelque chose a échoué). Une requête réseau a **toujours** ces trois issues possibles : prévois un affichage pour chacune.
- `useEffect(…, [boutique])` : l'effet se relance quand la prop `boutique` change (la personne passe d'une boutique à l'autre).
- `async function charger()` : on déclare une fonction asynchrone **à l'intérieur** de l'effet, car la fonction passée à `useEffect` ne peut pas être `async` elle-même.
- `await fetch(…)` attend la réponse du serveur. `reponse.ok` vaut `false` pour un code HTTP 404 ou 500 : dans ce cas on lance une erreur, que le `catch` attrape.
- `(await reponse.json()) as Goodie[]` : `json()` lit le corps de la réponse comme du JSON ; `as Goodie[]` affirme à TypeScript que c'est une liste de goodies (il ne peut pas le vérifier, le serveur répond ce qu'il veut).
- `catch (e)` attrape l'erreur lancée par `throw` ou par une panne réseau. `e instanceof Error ? e.message : "Erreur inconnue"` prend le message si c'est bien une erreur, sinon un texte par défaut.
- `finally` s'exécute dans tous les cas, succès ou erreur : c'est le bon endroit pour éteindre l'indicateur de chargement. Tu retrouves ce schéma `setLoading(true)` / `try` / `finally { setLoading(false) }` dans les formulaires de connexion de MiniShop (`AuthForms.tsx`).
- `annule` et la fonction renvoyée : c'est le **nettoyage**. Si `boutique` change pendant que la première requête est encore en vol, React exécute le nettoyage (`annule = true`) avant de relancer l'effet. Ainsi la réponse en retard de l'ancienne boutique est ignorée et n'écrase pas la nouvelle. `StoreInitializer.tsx` de MiniShop utilise le même drapeau (`let cancelled = false`).
- `void charger();` lance la fonction sans attendre son résultat (le mot `void` dit « je sais que c'est une promesse, je ne l'attends pas »).
- `role="alert"` signale aux lecteurs d'écran (logiciels qui lisent la page à voix haute) qu'il faut annoncer ce message tout de suite.
- Les trois `if`/`return` finaux affichent l'écran adapté à chaque état.

:::warning Respecte le tableau de dépendances
Tout ce que l'effet **lit** parmi les props, états ou fonctions du composant doit figurer dans le tableau. Si tu l'oublies, l'effet utilise une ancienne valeur. Le linter `eslint-config-next` (présent dans les deux projets de l'équipe) te prévient avec la règle `react-hooks/exhaustive-deps` : ne la fais pas taire sans comprendre pourquoi.
:::

## Pas d'effet quand un calcul suffit

Un effet sert à **synchroniser avec l'extérieur**. Pour fabriquer une valeur à partir d'autres valeurs du composant, calcule-la directement dans le corps, comme à la leçon précédente. Écrire `useEffect(() => setTotal(a + b), [a, b])` est une erreur classique : il y a un affichage de trop avec une valeur périmée.

## Un formulaire contrôlé

Dans un formulaire **contrôlé**, la valeur de chaque champ vit dans l'état React, et le champ l'affiche. À chaque frappe, `onChange` met l'état à jour.

```tsx
import { useState, type FormEvent } from "react";

type Message = { email: string; texte: string };

export function FormulaireContact({ onEnvoyer }: Readonly<{ onEnvoyer: (message: Message) => Promise<void> }>) {
  const [email, setEmail] = useState("");
  const [texte, setTexte] = useState("");
  const [envoi, setEnvoi] = useState(false);
  const [erreur, setErreur] = useState<string | null>(null);

  const emailValide = /^\S+@\S+\.\S+$/.test(email);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!emailValide || texte.trim() === "") return;
    setEnvoi(true);
    setErreur(null);
    try {
      await onEnvoyer({ email, texte });
      setTexte("");
    } catch {
      setErreur("Envoi impossible, réessaie dans un instant.");
    } finally {
      setEnvoi(false);
    }
  }

  return (
    <form onSubmit={handleSubmit}>
      <label htmlFor="email">Ton e-mail</label>
      <input id="email" type="email" value={email} onChange={(e) => setEmail(e.target.value)} />
      {email !== "" && !emailValide && <p>Adresse invalide</p>}
      <label htmlFor="texte">Ton message</label>
      <textarea id="texte" value={texte} onChange={(e) => setTexte(e.target.value)} />
      <button type="submit" disabled={envoi || !emailValide}>
        {envoi ? "Envoi…" : "Envoyer"}
      </button>
      {erreur && <p role="alert">{erreur}</p>}
    </form>
  );
}
```

- `value={email}` + `onChange={(e) => setEmail(e.target.value)}` : le champ affiche l'état, et chaque frappe le met à jour. `e.target.value` est le texte actuellement tapé.
- `emailValide` n'est **pas** un état : c'est un calcul fait à chaque affichage avec une **expression régulière** (un motif de texte). Dans `/^\S+@\S+\.\S+$/`, `^` marque le début du texte, `\S+` un ou plusieurs caractères qui ne sont pas des espaces, `@` et `\.` un arobase et un vrai point, et `$` la fin du texte. `.test(email)` renvoie `true` si le texte correspond. C'est la règle « calculer plutôt que stocker ».
- `FormEvent<HTMLFormElement>` est le type de l'événement d'envoi d'un formulaire ; `texte.trim()` retire les espaces au début et à la fin, pour refuser un message qui ne contient que des espaces.
- `catch {` sans parenthèses : on attrape l'erreur sans avoir besoin de son message.
- `event.preventDefault()` empêche le navigateur de recharger toute la page à l'envoi, comportement par défaut d'un `<form>`.
- `htmlFor` relie le `<label>` au champ ayant le même `id` : un clic sur le texte active le champ, et les lecteurs d'écran annoncent le libellé. En JSX on écrit `htmlFor` car `for` est un mot réservé de JavaScript.
- `{condition && <p>…</p>}` : si la condition est vraie, on affiche le paragraphe, sinon rien.
- `disabled={envoi || !emailValide}` évite le double envoi pendant que la requête est en cours.

:::info Et dans MiniShop ?
Les formulaires de MiniShop utilisent le composant `Form` de la bibliothèque **Ant Design**, qui gère les champs et les règles de validation (`required`, `type: "email"`, `min: 8`). Le mécanisme est le même que ci-dessus, avec moins de code à écrire. Retiens le principe : ce que la personne voit découle de l'état.
:::

## Entraîne-toi

:::labo
moteur: reel
intro: |
  Démarre ton environnement. Deux composants t'attendent dans `src` : `ListeGoodies.tsx` (qui doit charger une liste depuis un serveur) et `FormulaireContact.tsx` (un formulaire de contact). Modifie-les avec `nano` (`Ctrl+O` puis `Entrée` enregistre, `Ctrl+X` quitte).

  Il n'y a pas de réseau dans l'environnement : les **tests** (de petits programmes qui vérifient ton travail) remplacent `fetch` par un faux serveur qui répond ce qu'ils décident (une liste, une erreur 500, une panne, une réponse très lente…). `npx vitest run liste-chargement` lance les tests dont le nom de fichier contient `liste-chargement`. Comme ils attendent des réponses, ils sont **asynchrones** : ils utilisent `findBy…`, qui patiente jusqu'à ce que l'élément apparaisse.
commandes:
  - cp -R /opt/exercices/commun/. .
  - cp -R /opt/exercices/03-effets-formulaires/. .
  - lier-outils
etapes:
  - texte: >-
      Dans `src/ListeGoodies.tsx`, charge les goodies : crée un état `goodies` (au départ un tableau vide) et un état `chargement` (au départ `true`). Dans un `useEffect` qui dépend de `boutique`, appelle `fetch` sur `/api/boutiques/<boutique>/goodies`, range le résultat dans `goodies` et passe `chargement` à `false`. Tant que ça charge, affiche « Chargement… » ; ensuite une liste `<ul>`. Vérifie avec `npx vitest run liste-chargement`.
    indice: >-
      Une fonction `async` déclarée **dans** l'effet : `async function charger() { const reponse = await fetch(…); setGoodies(await reponse.json()); setChargement(false); }`, appelée par `void charger();`. Le tableau de dépendances est `[boutique]`.
    verif:
      - commande-reussit: controler tests 03-effets-formulaires liste-chargement
      - commande-reussit: contient src/ListeGoodies.tsx 'useEffect\s*\('
      - commande-reussit: contient src/ListeGoodies.tsx '\bfetch\s*\('
    solution:
      - ecrire:
          'src/ListeGoodies.tsx': |
            import { useEffect, useState } from "react";

            type Goodie = { id: number; nom: string };

            export function ListeGoodies({ boutique }: Readonly<{ boutique: string }>) {
              const [goodies, setGoodies] = useState<Goodie[]>([]);
              const [chargement, setChargement] = useState(true);

              useEffect(() => {
                async function charger() {
                  const reponse = await fetch(`/api/boutiques/${boutique}/goodies`);
                  setGoodies((await reponse.json()) as Goodie[]);
                  setChargement(false);
                }

                void charger();
              }, [boutique]);

              if (chargement) return <p>Chargement…</p>;
              return (
                <ul>
                  {goodies.map((g) => (
                    <li key={g.id}>{g.nom}</li>
                  ))}
                </ul>
              );
            }
  - texte: >-
      Gère les échecs. Ajoute un état `erreur`. Si la réponse n'est pas `ok` (par exemple un code 500), lance une erreur avec `throw new Error("Erreur HTTP " + reponse.status)` ; entoure l'appel d'un `try … catch … finally` pour ranger le message dans `erreur` et éteindre `chargement` dans tous les cas. Affiche l'erreur dans un `<p role="alert">`. Vérifie avec `npx vitest run liste-erreur`.
    indice: >-
      `catch (e) { setErreur(e instanceof Error ? e.message : "Erreur inconnue"); } finally { setChargement(false); }`, puis `if (erreur) return <p role="alert">{erreur}</p>;` avant la liste.
    verif:
      - commande-reussit: controler tests 03-effets-formulaires liste-erreur
      - commande-reussit: contient src/ListeGoodies.tsx '\bcatch\b'
    apres: [1]
    solution:
      - ecrire:
          'src/ListeGoodies.tsx': |
            import { useEffect, useState } from "react";

            type Goodie = { id: number; nom: string };

            export function ListeGoodies({ boutique }: Readonly<{ boutique: string }>) {
              const [goodies, setGoodies] = useState<Goodie[]>([]);
              const [chargement, setChargement] = useState(true);
              const [erreur, setErreur] = useState<string | null>(null);

              useEffect(() => {
                async function charger() {
                  setChargement(true);
                  setErreur(null);
                  try {
                    const reponse = await fetch(`/api/boutiques/${boutique}/goodies`);
                    if (!reponse.ok) throw new Error(`Erreur HTTP ${reponse.status}`);
                    setGoodies((await reponse.json()) as Goodie[]);
                  } catch (e) {
                    setErreur(e instanceof Error ? e.message : "Erreur inconnue");
                  } finally {
                    setChargement(false);
                  }
                }

                void charger();
              }, [boutique]);

              if (chargement) return <p>Chargement…</p>;
              if (erreur) return <p role="alert">{erreur}</p>;
              return (
                <ul>
                  {goodies.map((g) => (
                    <li key={g.id}>{g.nom}</li>
                  ))}
                </ul>
              );
            }
  - texte: >-
      La personne change de boutique pendant qu'une requête est encore en vol : la réponse en retard de l'ancienne boutique ne doit pas écraser la nouvelle. Ajoute à l'effet une variable `annule` (au départ `false`), une fonction de **nettoyage** qui la passe à `true`, et n'appelle les fonctions `set…` que si `annule` est faux. Vérifie avec `npx vitest run liste-boutique`.
    indice: >-
      Au début de l'effet : `let annule = false;`. À la fin : `return () => { annule = true; };`. Dans le `try`, `catch` et `finally`, écris `if (!annule) setGoodies(donnees);` (et de même pour les deux autres).
    verif:
      - commande-reussit: controler tests 03-effets-formulaires liste-boutique
      - commande-reussit: contient src/ListeGoodies.tsx '\bannule\b'
    apres: [2]
    solution:
      - ecrire:
          'src/ListeGoodies.tsx': |
            import { useEffect, useState } from "react";

            type Goodie = { id: number; nom: string };

            export function ListeGoodies({ boutique }: Readonly<{ boutique: string }>) {
              const [goodies, setGoodies] = useState<Goodie[]>([]);
              const [chargement, setChargement] = useState(true);
              const [erreur, setErreur] = useState<string | null>(null);

              useEffect(() => {
                let annule = false;

                async function charger() {
                  setChargement(true);
                  setErreur(null);
                  try {
                    const reponse = await fetch(`/api/boutiques/${boutique}/goodies`);
                    if (!reponse.ok) throw new Error(`Erreur HTTP ${reponse.status}`);
                    const donnees = (await reponse.json()) as Goodie[];
                    if (!annule) setGoodies(donnees);
                  } catch (e) {
                    if (!annule) setErreur(e instanceof Error ? e.message : "Erreur inconnue");
                  } finally {
                    if (!annule) setChargement(false);
                  }
                }

                void charger();
                return () => {
                  annule = true;
                };
              }, [boutique]);

              if (chargement) return <p>Chargement…</p>;
              if (erreur) return <p role="alert">{erreur}</p>;
              return (
                <ul>
                  {goodies.map((g) => (
                    <li key={g.id}>{g.nom}</li>
                  ))}
                </ul>
              );
            }
  - texte: >-
      Passe au formulaire, dans `src/FormulaireContact.tsx`. Rends les champs **contrôlés** : un état `email` et un état `texte`, reliés à `value` et à `onChange`. Calcule `emailValide` avec l'expression régulière `/^\S+@\S+\.\S+$/` (sans le stocker dans un état), affiche « Adresse invalide » quand l'e-mail est non vide et invalide, et désactive le bouton tant que l'e-mail n'est pas valide. Vérifie avec `npx vitest run formulaire-validation`.
    indice: >-
      `value={email}` et `onChange={(e) => setEmail(e.target.value)}` sur l'`<input>` ; même chose pour la `<textarea>`. `{email !== "" && !emailValide && <p>Adresse invalide</p>}` et `disabled={!emailValide}` sur le bouton.
    verif:
      - commande-reussit: controler tests 03-effets-formulaires formulaire-validation
      - commande-reussit: contient src/FormulaireContact.tsx '\bvalue\s*='
    solution:
      - ecrire:
          'src/FormulaireContact.tsx': |
            import { useState } from "react";

            export type Message = { email: string; texte: string };

            export function FormulaireContact({ onEnvoyer }: Readonly<{ onEnvoyer: (message: Message) => Promise<void> }>) {
              const [email, setEmail] = useState("");
              const [texte, setTexte] = useState("");

              const emailValide = /^\S+@\S+\.\S+$/.test(email);

              return (
                <form>
                  <label htmlFor="email">Ton e-mail</label>
                  <input id="email" type="email" value={email} onChange={(e) => setEmail(e.target.value)} />
                  {email !== "" && !emailValide && <p>Adresse invalide</p>}
                  <label htmlFor="texte">Ton message</label>
                  <textarea id="texte" value={texte} onChange={(e) => setTexte(e.target.value)} />
                  <button type="submit" disabled={!emailValide}>
                    Envoyer
                  </button>
                </form>
              );
            }
  - texte: >-
      Gère l'envoi. Ajoute un gestionnaire `onSubmit` qui appelle `event.preventDefault()` (pour que le navigateur ne recharge pas la page), ne fait rien si le message est vide, passe un état `envoi` à `true`, attend `onEnvoyer({ email, texte })`, vide ensuite le message ; en cas d'échec, affiche « Envoi impossible, réessaie dans un instant. » dans un `<p role="alert">` ; dans tous les cas, `envoi` redevient `false`. Pendant l'envoi, le bouton affiche « Envoi… » et est désactivé. Vérifie avec `npx vitest run formulaire-envoi`.
    indice: >-
      Reprends la structure `try … catch … finally` de l'étape 2. Le bouton : `disabled={envoi || !emailValide}` et `{envoi ? "Envoi…" : "Envoyer"}`. Le formulaire : `<form onSubmit={handleSubmit}>`.
    verif:
      - commande-reussit: controler tests 03-effets-formulaires formulaire-envoi
      - commande-reussit: contient src/FormulaireContact.tsx 'preventDefault\s*\('
    apres: [4]
    solution:
      - ecrire:
          'src/FormulaireContact.tsx': |
            import { useState, type FormEvent } from "react";

            export type Message = { email: string; texte: string };

            export function FormulaireContact({ onEnvoyer }: Readonly<{ onEnvoyer: (message: Message) => Promise<void> }>) {
              const [email, setEmail] = useState("");
              const [texte, setTexte] = useState("");
              const [envoi, setEnvoi] = useState(false);
              const [erreur, setErreur] = useState<string | null>(null);

              const emailValide = /^\S+@\S+\.\S+$/.test(email);

              async function handleSubmit(event: FormEvent<HTMLFormElement>) {
                event.preventDefault();
                if (!emailValide || texte.trim() === "") return;
                setEnvoi(true);
                setErreur(null);
                try {
                  await onEnvoyer({ email, texte });
                  setTexte("");
                } catch {
                  setErreur("Envoi impossible, réessaie dans un instant.");
                } finally {
                  setEnvoi(false);
                }
              }

              return (
                <form onSubmit={handleSubmit}>
                  <label htmlFor="email">Ton e-mail</label>
                  <input id="email" type="email" value={email} onChange={(e) => setEmail(e.target.value)} />
                  {email !== "" && !emailValide && <p>Adresse invalide</p>}
                  <label htmlFor="texte">Ton message</label>
                  <textarea id="texte" value={texte} onChange={(e) => setTexte(e.target.value)} />
                  <button type="submit" disabled={envoi || !emailValide}>
                    {envoi ? "Envoi…" : "Envoyer"}
                  </button>
                  {erreur && <p role="alert">{erreur}</p>}
                </form>
              );
            }
:::

## Vérifie tes acquis

:::quiz
Que signifie le tableau de dépendances vide `[]` dans `useEffect(…, [])` ?

- [ ] L'effet ne s'exécute jamais
- [ ] L'effet s'exécute à chaque affichage
- [x] L'effet s'exécute une seule fois, après le premier affichage
- [ ] L'effet s'exécute à chaque clic

> Sans dépendance à surveiller, l'effet n'est relancé par aucun changement.
:::

:::quiz
À quoi sert la fonction renvoyée par un effet ?

- [x] À nettoyer (annuler une requête, arrêter un minuteur) avant le prochain effet ou quand le composant disparaît
- [ ] À renvoyer la valeur à afficher
- [ ] À déclencher un nouvel affichage
- [ ] À remplacer le tableau de dépendances

> Dans notre exemple, elle passe `annule` à `true` pour ignorer une réponse devenue inutile.
:::

:::quiz
Dans un formulaire contrôlé, que faut-il écrire pour qu'un champ `<input>` suive la frappe ?

- [ ] Uniquement `value={email}`
- [ ] Uniquement `onChange={…}`
- [ ] `defaultValue={email}` et rien d'autre
- [x] `value={email}` et `onChange={(e) => setEmail(e.target.value)}`

> Avec `value` seul, le champ est figé ; il faut aussi un `onChange` qui met l'état à jour.
:::

:::quiz
Une requête réseau peut avoir trois issues. Lesquelles faut-il prévoir à l'écran ?

- [ ] Rapide, lente, très lente
- [x] En cours, en erreur, réussie
- [ ] Connectée, déconnectée, inconnue
- [ ] Mise en cache, non mise en cache, expirée

> Afficher « Chargement… », un message d'erreur et le résultat évite les écrans blancs.
:::
