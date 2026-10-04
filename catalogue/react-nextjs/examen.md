---
titre: "Examen de validation — React et Next.js"
tirage: 5
seuil: 80
duree: 10
melange: true
---

Cet examen valide les bases de React et de Next.js utiles pour contribuer aux projets de l'équipe : composants et props, état, effets et formulaires, routage et rendu serveur, Tailwind, authentification, base de données et déploiement.

:::quiz
En React, qu'est-ce qu'un composant ?

- [x] Une fonction dont le nom commence par une majuscule et qui renvoie du JSX
- [ ] Un fichier CSS associé à une page, qui décrit les couleurs de chaque élément
- [ ] Une balise HTML personnalisée que tu déclares ensuite dans le navigateur
- [ ] Un objet qui stocke les données de la page et les envoie au serveur

> Un composant est une fonction qui décrit un affichage ; la majuscule le distingue d'une balise HTML.
:::

:::quiz
Que représentent les props d'un composant ?

- [ ] Les variables internes que le composant modifie librement
- [ ] Les styles appliqués par Tailwind
- [x] Les données reçues du composant parent, en lecture seule
- [ ] Les événements envoyés au serveur

> Les props descendent du parent vers l'enfant et ne se modifient pas depuis l'enfant.
:::

:::quiz
Pourquoi fournit-on une `key` à chaque élément d'une liste affichée avec `map` ?

- [ ] Pour que le navigateur trie la liste par ordre alphabétique
- [x] Pour que React suive chaque élément quand la liste change
- [ ] Pour transmettre l'identifiant en prop au composant
- [ ] Pour chiffrer les éléments de la liste

> La clé identifie un élément de façon stable ; elle n'est pas lue par le composant lui-même.
:::

:::quiz
Quel risque présente `key={index}` sur une liste que l'on peut réordonner ?

- [ ] Une erreur de compilation TypeScript qui bloque la construction du site
- [ ] Un chargement plus lent du serveur lors de la première visite
- [ ] Un conflit entre la clé et les classes Tailwind de la liste
- [x] React peut associer de mauvaises données à de mauvais éléments

> Quand l'ordre change, l'index ne désigne plus le même élément : il faut une clé propre à la donnée.
:::

:::quiz
Quel est le type habituel de la prop `children` ?

- [ ] `string[]`
- [ ] `Function`
- [x] `ReactNode`
- [ ] `HTMLElement`

> `ReactNode` couvre tout ce que React sait afficher : texte, éléments, tableaux, `null`.
:::

:::quiz
Un composant affiche la quantité choisie et doit la modifier au clic. Qu'utilises-tu ?

- [x] Un état avec `useState`
- [ ] Une prop que le composant modifie directement
- [ ] Une variable `let` déclarée dans le composant
- [ ] Un attribut `data-quantite` sur la balise

> Seul l'état déclenche un nouvel affichage quand il change ; une variable locale est réinitialisée à chaque appel.
:::

:::quiz
Quel code met à jour correctement un état `panier` de type tableau pour y ajouter `ligne` ?

- [ ] `panier.push(ligne)`
- [x] `setPanier([...panier, ligne])`
- [ ] `panier[panier.length] = ligne`
- [ ] `setPanier(panier)`

> Il faut passer un nouveau tableau ; les trois autres solutions gardent la même référence.
:::

:::quiz
Que fait `<button onClick={handleClick()}>` ?

- [ ] Il appelle `handleClick` uniquement au clic
- [ ] Il désactive le bouton
- [ ] Il est équivalent à `onClick={handleClick}`
- [x] Il appelle `handleClick` à chaque affichage au lieu d'attendre le clic

> Avec les parenthèses, la fonction est exécutée immédiatement ; on doit passer la fonction elle-même.
:::

:::quiz
Pourquoi `useEffect` est-il le bon endroit pour appeler `fetch` au chargement d'un composant client ?

- [x] Parce que l'effet s'exécute après l'affichage, en dehors du calcul de l'affichage
- [ ] Parce que `fetch` n'existe que dans les effets et plante partout ailleurs
- [ ] Parce que l'effet s'exécute avant le premier affichage, donc plus tôt
- [ ] Parce que l'effet rend automatiquement le composant asynchrone

> Le corps du composant doit rester un calcul d'affichage ; les actions extérieures vont dans un effet.
:::

:::quiz
Dans `useEffect(() => {...}, [boutique])`, quand l'effet est-il relancé ?

- [ ] À chaque frappe dans n'importe quel champ
- [ ] Uniquement au premier affichage
- [x] Après le premier affichage, puis à chaque changement de `boutique`
- [ ] Jamais, `boutique` est ignorée

> Le tableau liste les valeurs lues par l'effet ; React compare leurs valeurs entre deux affichages.
:::

:::quiz
À quoi sert `event.preventDefault()` dans un gestionnaire `onSubmit` ?

- [ ] À annuler la validation du formulaire côté serveur
- [x] À empêcher le rechargement de la page par le navigateur
- [ ] À vider les champs du formulaire
- [ ] À empêcher l'événement de remonter au composant parent

> Par défaut, un formulaire HTML recharge la page à l'envoi ; React gère l'envoi lui-même.
:::

:::quiz
Quelle est la différence entre `page.tsx` et `layout.tsx` dans `app/` ?

- [ ] Le layout n'est utilisé que pour afficher les pages d'erreur comme la 404
- [ ] La page s'affiche sur le serveur, tandis que le layout s'affiche dans le navigateur
- [ ] Il n'y a aucune différence : ce sont deux noms pour le même fichier
- [x] Le layout entoure les pages du dossier et reste en place pendant la navigation

> `page.tsx` est le contenu propre à l'adresse ; `layout.tsx` est le cadre commun.
:::

:::quiz
Que contient `params` dans `app/[boutique]/produit/[id]/page.tsx` pour l'adresse `/bde/produit/12` ?

- [x] `boutique` vaut `"bde"` et `id` vaut `"12"`, tous deux en texte
- [ ] `boutique` vaut `"bde"` et `id` vaut le nombre `12`, converti par Next.js
- [ ] Un tableau `["bde", "produit", "12"]` avec tous les segments de l'adresse
- [ ] Rien : il faut lire l'URL à la main avec `window.location`

> Les segments d'URL sont toujours des chaînes ; il faut convertir (`Number(id)`) et vérifier le résultat.
:::

:::quiz
Une page de `app/` ne contient aucune directive. Quelle affirmation est vraie ?

- [ ] C'est un composant client : il peut utiliser les hooks
- [ ] Elle est exécutée uniquement dans le navigateur
- [x] C'est un composant serveur : il ne peut pas utiliser `useState`
- [ ] Elle ne peut pas afficher de JSX

> Par défaut, les composants de `app/` sont des composants serveur.
:::

:::quiz
Que faut-il vérifier au début d'une action serveur qui modifie des données ?

- [ ] Que le bouton qui l'appelle est visible
- [x] Que la personne connectée a le droit d'effectuer cette action
- [ ] Que la page est servie en HTTPS
- [ ] Que le composant parent est un composant client

> Une action est appelable directement : le contrôle des droits doit se faire dans l'action elle-même.
:::

:::quiz
Dans `className="grid gap-4 md:grid-cols-3"`, à quel moment y a-t-il trois colonnes ?

- [ ] Toujours, quelle que soit la largeur
- [ ] Uniquement sur les téléphones
- [ ] Uniquement quand la souris survole la grille
- [x] À partir d'une largeur d'écran de 768 pixels

> Le préfixe `md:` conditionne la classe à une largeur minimale ; sans préfixe, elle s'applique partout.
:::

:::quiz
Pourquoi une variable d'environnement `NEXT_PUBLIC_SECRET` est-elle une mauvaise idée ?

- [x] Parce que les variables `NEXT_PUBLIC_` sont envoyées au navigateur et donc visibles de tous
- [ ] Parce qu'elle ne fonctionne qu'en développement et disparaît en production
- [ ] Parce que Next.js interdit les variables dont le nom est écrit en majuscules, sauf en mode strict
- [ ] Parce qu'elle ralentit la compilation de toutes les pages du site

> Seules les variables sans ce préfixe restent côté serveur.
:::

:::quiz
Dans MiniShop, que fait le fichier `src/proxy.ts` pour l'adresse `/admin/stock` d'une personne non connectée ?

- [ ] Il affiche une erreur 500
- [ ] Il laisse passer, car la protection se fait uniquement dans les pages
- [x] Il la redirige vers la page de connexion de l'administration
- [ ] Il crée automatiquement un compte

> Le filtre d'URL redirige vers `/admin/login` ; les gardes dans les pages et actions restent nécessaires.
:::

:::quiz
Pourquoi ne pas coller directement une valeur saisie par l'utilisateur dans un texte SQL ?

- [ ] Parce que SQL n'accepte pas de texte
- [x] Pour éviter l'injection SQL, en passant les valeurs comme paramètres
- [ ] Parce que cela empêche l'utilisation de jointures
- [ ] Parce que PostgreSQL ne gère que les nombres

> Kysely transmet les valeurs séparément de la requête : elles ne peuvent pas être interprétées comme du SQL.
:::

:::quiz
À quoi sert la fonction `down` d'une migration ?

- [ ] À supprimer toutes les données de la base
- [ ] À vérifier que `up` s'est bien passé
- [ ] À générer le fichier de types
- [x] À annuler les changements de `up` pour revenir à l'état précédent

> Une migration doit être réversible, par exemple pour revenir en arrière après une erreur.
:::

:::quiz
Pourquoi le cache Redis de MiniShop ne fait-il pas planter l'application quand Redis est injoignable ?

- [x] Parce que le cache est une optimisation : en cas d'échec, on lit directement la base
- [ ] Parce que Redis redémarre tout seul à chaque requête et n'est donc jamais vraiment injoignable
- [ ] Parce que Next.js ignore toutes les erreurs réseau, quelle que soit leur origine
- [ ] Parce que les sessions n'utilisent jamais Redis, seulement la base de données

> Les fonctions du client Redis capturent les erreurs et renvoient `null`, ce qui revient à un « cache vide ».
:::

:::quiz
Quelle étape du `Dockerfile` de MiniShop produit l'image qui est réellement déployée ?

- [ ] `deps`, qui installe les dépendances
- [ ] `builder`, qui contient tout le code source
- [x] `runner`, qui ne garde que le résultat de la compilation
- [ ] Les trois, empilées dans l'image finale

> Les étapes précédentes servent à construire ; seule la dernière devient l'image publiée.
:::

:::quiz
Quelle commande de la CI de MiniShop vérifie que le TypeScript compile sans produire de fichiers ?

- [ ] `npm run prettier-check`, qui compare le formatage au style attendu
- [x] `npm run ts-check`, qui lance `tsc --noEmit` sur le projet
- [ ] `npm run migrate`, qui applique les migrations manquantes
- [ ] `npm run generate-types`, qui régénère le fichier de types

> `--noEmit` demande à TypeScript de vérifier les types sans écrire de JavaScript.
:::
