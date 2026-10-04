---
id: http-et-rxjs
titre: "Appels HTTP et RxJS"
resume: "Interroger une API avec HttpClient, comprendre les Observables et écrire une recherche réactive."
duree: 40
objectifs:
  - Écrire un service qui appelle une API avec `HttpClient` et des types
  - Expliquer ce qu'est un `Observable` et pourquoi il faut s'y abonner
  - Combiner `map`, `debounceTime`, `distinctUntilChanged` et `switchMap`
  - Afficher un `Observable` avec le tube `async` ou se désabonner proprement
---

Une **API** est un service web qui échange des données (ici l'API d'Adhésion, qui connaît les adhérent·e·s) ; ton application la questionne par des requêtes HTTP (**HTTP** est la convention que navigateurs et serveurs utilisent pour s'échanger des demandes et des réponses). Dans la leçon de JavaScript, tu as appelé une API avec `fetch` et `await`. Angular propose son propre client, `HttpClient`, qui s'appuie sur une autre abstraction : les **Observables** de la bibliothèque RxJS. C'est la partie qui déroute le plus au début, alors allons-y pas à pas.

## HttpClient dans un service

Les appels HTTP se mettent dans un **service** (leçon 2), jamais dans le gabarit. Le client se configure une fois pour toute l'application avec `provideHttpClient()` (voir la leçon 6 pour la version d'Adhésion).

```ts
import { HttpClient, HttpParams } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { Observable } from 'rxjs';

export interface Membre {
  id: number;
  prenom: string;
  nom: string;
}

export interface Page<T> {
  count: number;
  results: T[];
}

@Injectable({ providedIn: 'root' })
export class MembresService {
  private readonly http = inject(HttpClient);
  private readonly urlApi = 'https://example.org/api';

  rechercher(texte: string): Observable<Page<Membre>> {
    const params = new HttpParams({ fromObject: { search: texte } });
    return this.http.get<Page<Membre>>(`${this.urlApi}/membres/`, { params });
  }

  creer(membre: Omit<Membre, 'id'>): Observable<Membre> {
    return this.http.post<Membre>(`${this.urlApi}/membres/`, membre);
  }
}
```

Ligne par ligne :

- `interface Page<T>` décrit une page de résultats pour n'importe quel type `T` (c'est un type *générique*).
- `inject(HttpClient)` demande le client HTTP d'Angular (leçon 2).
- `new HttpParams({ fromObject: { search: texte } })` fabrique les paramètres d'adresse `?search=…`.
- `Observable<Page<Membre>>` annonce ce que la méthode renvoie : un flux qui livrera une `Page` de `Membre`.
- `Omit<Membre, 'id'>` signifie « un `Membre` sans son `id` » : on ne connaît pas l'identifiant avant la création.

Précisions :

- `get<Page<Membre>>` indique à TypeScript la forme de la réponse. Attention : c'est une promesse faite au compilateur, il ne vérifie pas le JSON reçu.
- Les méthodes correspondent aux verbes HTTP : `get`, `post`, `put`, `patch`, `delete`.

Le `MembersService` d'Adhésion suit exactement ce modèle : `search()` fait un `get` avec `HttpParams`, `update()` un `put`, `create()` un `post`, `delete()` un `delete`, tous sur `environment.adhesion_api_url`.

## Un Observable ne s'exécute pas tout seul

Analogie : un `Observable` est comme **l'abonnement à une chaîne vidéo**. Tant que tu ne t'abonnes pas, rien ne t'est envoyé. Une fois abonné·e, tu reçois ce qui est publié, éventuellement plusieurs fois, jusqu'à ce que la chaîne se termine ou que tu te désabonnes.

```ts
const requete$ = membres.rechercher('camille'); // rien ne part encore
requete$.subscribe({
  next: (page) => console.log(page.count, 'résultats'),
  error: (erreur) => console.error('Échec', erreur),
});                                              // la requête part ici
```

Lecture : `subscribe` s'abonne ; `next` est appelé à chaque valeur reçue ; `error` est appelé si la requête échoue.

Par convention, le nom d'une variable Observable se termine par `$`. Un `Observable` peut émettre **plusieurs valeurs** dans le temps (des frappes au clavier, des messages d'une *websocket*, une connexion qui reste ouverte entre le navigateur et un serveur pour qu'ils s'envoient des messages à tout moment) puis se terminer ou échouer. Une réponse HTTP n'en émet qu'une.

:::info Deux écritures de `subscribe`
Tu verras dans Adhésion `subscribe((valeur) => …, (erreur) => …)`, avec deux fonctions en arguments. Cette écriture est dépréciée dans RxJS 7 ; utilise l'objet `{ next, error }` pour tout nouveau code.
:::

## Transformer avec des opérateurs

Les **opérateurs** sont des fonctions qui transforment le flux, comme les étapes d'une chaîne de montage. Ils se chaînent dans `.pipe(...)`. Le plus simple, `map`, fonctionne comme sur un tableau :

```ts
import { map } from 'rxjs';

const total$ = membres.rechercher('a').pipe(map((page) => page.count));
```

Un `Subject` est un Observable dans lequel on peut aussi **pousser** des valeurs avec `.next(…)` : pratique pour y envoyer les frappes du clavier. Pour une recherche instantanée, il faut éviter d'envoyer une requête à chaque lettre et ignorer les réponses périmées. Voici le motif classique :

```ts
import { debounceTime, distinctUntilChanged, Subject, switchMap } from 'rxjs';

export class RechercheComponent {
  private readonly membres = inject(MembresService);
  readonly terme$ = new Subject<string>();

  readonly resultats$ = this.terme$.pipe(
    debounceTime(300),
    distinctUntilChanged(),
    switchMap((terme) => this.membres.rechercher(terme)),
  );
}
```

1. `debounceTime(300)` attend 300 ms de silence avant de laisser passer la valeur.
2. `distinctUntilChanged()` ignore une valeur identique à la précédente.
3. `switchMap` lance la requête et **annule la précédente** si une nouvelle valeur arrive : on ne reçoit jamais la réponse d'une ancienne frappe après celle d'une récente.

La recherche d'adhérent·e·s d'Adhésion (`ListComponent`) utilise `debounceTime(300)` et `distinctUntilChanged()` sur un `Subject`. Pour annuler la requête précédente, elle garde l'abonnement dans `_searchSubscription` et appelle `unsubscribe()` à la main : `switchMap` fait la même chose en une ligne.

## Afficher et se désabonner

Pour afficher un Observable dans le gabarit, le tube (*pipe*) `async` s'abonne **et se désabonne** à ta place :

```html
<input #saisie (input)="terme$.next(saisie.value)" type="search" placeholder="Rechercher un membre" />

@if (resultats$ | async; as page) {
  <p>{{ page.count }} résultat(s)</p>
  <ul>
    @for (m of page.results; track m.id) {
      <li>{{ m.prenom }} {{ m.nom }}</li>
    }
  </ul>
}
```

Ici `#saisie` donne un nom à l'élément pour lire sa valeur ; `@if (resultats$ | async; as page)` attend la première valeur, la range dans `page`, puis affiche le bloc.

Il faut importer `AsyncPipe` (dans `imports` du composant autonome). Dans un module, `CommonModule` (déjà présent via `BrowserModule`) le fournit.

Si tu dois t'abonner dans la classe (pour un effet de bord), prévois la fin de l'abonnement. La solution moderne :

```ts
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';

constructor() {
  this.terme$.pipe(takeUntilDestroyed()).subscribe((t) => console.log(t));
}
```

Ici `constructor()` est la fonction appelée à la création du composant. `takeUntilDestroyed()` coupe l'abonnement quand le composant est détruit. Sans cela, un `Subject` qui vit longtemps (ou un service) retient le composant en mémoire, et son code continue de s'exécuter.

:::warning Les requêtes HTTP se terminent seules, les autres flux non
Une réponse HTTP termine l'Observable : pas de fuite à craindre. En revanche, un `Subject`, un flux d'évènements du routeur (`router.events`, qui annonce chaque changement de page) ou une websocket ne se terminent jamais. Adhésion s'abonne à `router.events` dans `AppComponent.logRoute()` sans désabonnement : sur le composant racine c'est sans conséquence, mais ne copie pas ce réflexe dans un écran qu'on ouvre et ferme.
:::

## Entraîne-toi

:::labo
moteur: reel
intro: |
  Le projet de `/workspace` contient un `MembresService` qui n'appelle pas encore l'API (`src/app/membres.service.ts`) et un composant de recherche `RechercheComponent` (`src/app/recherche.ts`) à compléter. Les tests ne font **aucun vrai appel réseau** : ils remplacent `HttpClient` par un faux serveur (`HttpTestingController`) qui note les requêtes et répond ce qu'ils décident. Ne modifie pas les fichiers `*.spec.ts` (`tester` remet de toute façon les tests d'origine dans une copie : les modifier ne servirait à rien).

  Tu lances un groupe de tests avec `tester fichier -t "mot-du-groupe"`, ou un fichier entier avec `tester fichier`.
commandes:
  - cp -R /opt/exercices/04-http-et-rxjs/. .
  - /opt/angular/preparer
etapes:
  - texte: >-
      Dans `src/app/membres.service.ts`, écris `rechercher(texte)` : crée les paramètres avec `new HttpParams({ fromObject: { search: texte } })` et renvoie `this.http.get<Page<Membre>>(`${this.urlApi}/membres/`, { params })`. Vérifie avec `tester membres.service -t "rechercher envoie"`.
    indice: >-
      Le corps de la méthode a deux lignes : `const params = …;` puis `return this.http.get<Page<Membre>>(…, { params });`. `HttpParams` est déjà importé.
    verif:
      - commande-reussit: tester membres.service -t 'rechercher envoie'
      - commande-reussit: contient src/app/membres.service.ts 'new\s+HttpParams\s*\('
    solution:
      - ecrire:
          src/app/membres.service.ts: |
            import { HttpClient, HttpParams } from '@angular/common/http';
            import { inject, Injectable } from '@angular/core';
            import { Observable, of } from 'rxjs';

            export interface Membre {
              id: number;
              prenom: string;
              nom: string;
            }

            export interface Page<T> {
              count: number;
              results: T[];
            }

            @Injectable({ providedIn: 'root' })
            export class MembresService {
              private readonly http = inject(HttpClient);
              private readonly urlApi = 'https://example.org/api';

              rechercher(texte: string): Observable<Page<Membre>> {
                const params = new HttpParams({ fromObject: { search: texte } });
                return this.http.get<Page<Membre>>(`${this.urlApi}/membres/`, { params });
              }

              // À FAIRE (étape 2) : appelle `POST {urlApi}/membres/` avec le membre dans le corps de la requête.
              creer(membre: Omit<Membre, 'id'>): Observable<Membre> {
                return of({ id: 0, ...membre });
              }
            }
  - texte: >-
      Écris `creer(membre)` dans le même fichier : elle renvoie `this.http.post<Membre>(`${this.urlApi}/membres/`, membre)`. Tu peux supprimer l'import `of`, devenu inutile. Vérifie avec `tester membres.service`.
    indice: >-
      `return this.http.post<Membre>(`${this.urlApi}/membres/`, membre);` : le deuxième argument est le corps de la requête.
    apres: [1]
    verif:
      - commande-reussit: tester membres.service
      - commande-reussit: contient src/app/membres.service.ts 'this\.http\.post\s*<'
    solution:
      - ecrire:
          src/app/membres.service.ts: |
            import { HttpClient, HttpParams } from '@angular/common/http';
            import { inject, Injectable } from '@angular/core';
            import { Observable } from 'rxjs';

            export interface Membre {
              id: number;
              prenom: string;
              nom: string;
            }

            export interface Page<T> {
              count: number;
              results: T[];
            }

            @Injectable({ providedIn: 'root' })
            export class MembresService {
              private readonly http = inject(HttpClient);
              private readonly urlApi = 'https://example.org/api';

              rechercher(texte: string): Observable<Page<Membre>> {
                const params = new HttpParams({ fromObject: { search: texte } });
                return this.http.get<Page<Membre>>(`${this.urlApi}/membres/`, { params });
              }

              creer(membre: Omit<Membre, 'id'>): Observable<Membre> {
                return this.http.post<Membre>(`${this.urlApi}/membres/`, membre);
              }
            }
  - texte: >-
      Dans `src/app/recherche.ts`, rends la recherche réactive : devant le `switchMap` de `resultats$`, ajoute `debounceTime(300)` (attendre 300 ms de silence) puis `distinctUntilChanged()` (ignorer un terme identique au précédent). Importe-les depuis `rxjs`. Vérifie avec `tester recherche -t "rafale|identique"`.
    indice: >-
      Dans `.pipe(…)`, l'ordre compte : `debounceTime(300), distinctUntilChanged(), switchMap(…)`.
    apres: [2]
    verif:
      - commande-reussit: tester recherche -t 'rafale|identique'
      - commande-reussit: contient src/app/recherche.ts 'debounceTime\s*\(\s*300\s*\)'
      - commande-reussit: contient src/app/recherche.ts 'distinctUntilChanged\s*\('
    solution:
      - ecrire:
          src/app/recherche.ts: |
            import { Component, inject } from '@angular/core';
            import { debounceTime, distinctUntilChanged, Observable, Subject, switchMap } from 'rxjs';
            import { Membre, MembresService, Page } from './membres.service';

            @Component({
              selector: 'app-recherche',
              imports: [],
              template: `<input #saisie (input)="terme$.next(saisie.value)" type="search" placeholder="Rechercher un membre" />`,
            })
            export class RechercheComponent {
              private readonly membres = inject(MembresService);

              // Chaque frappe du clavier est poussée dans ce `Subject`.
              readonly terme$ = new Subject<string>();

              readonly resultats$: Observable<Page<Membre>> = this.terme$.pipe(
                debounceTime(300),
                distinctUntilChanged(),
                switchMap((terme) => this.membres.rechercher(terme)),
              );

              dernierTerme = '';

              constructor() {
                this.terme$.subscribe((terme) => (this.dernierTerme = terme));
              }
            }
  - texte: >-
      Affiche les résultats dans le gabarit de `RechercheComponent` : sous le champ, `@if (resultats$ | async; as page) { … }` avec un `<p>` qui affiche `{{ page.count }} résultat(s)` et un `<ul>` dont chaque `<li>` (bloc `@for`, `track m.id`) affiche `{{ m.prenom }} {{ m.nom }}`. Il faut importer `AsyncPipe` (de `@angular/common`) dans `imports`. Vérifie avec `tester recherche -t "nombre de r"`.
    indice: >-
      Le tube `async` s'abonne à `resultats$` et se désabonne tout seul. `@if (resultats$ | async; as page)` range la valeur reçue dans `page`.
    apres: [3]
    verif:
      - commande-reussit: tester recherche -t 'nombre de r'
      - commande-reussit: contient src/app/recherche.ts '@for\s*\('
      - commande-reussit: contient src/app/recherche.ts 'AsyncPipe'
    solution:
      - ecrire:
          src/app/recherche.ts: |
            import { AsyncPipe } from '@angular/common';
            import { Component, inject } from '@angular/core';
            import { debounceTime, distinctUntilChanged, Observable, Subject, switchMap } from 'rxjs';
            import { Membre, MembresService, Page } from './membres.service';

            @Component({
              selector: 'app-recherche',
              imports: [AsyncPipe],
              template: `
                <input #saisie (input)="terme$.next(saisie.value)" type="search" placeholder="Rechercher un membre" />

                @if (resultats$ | async; as page) {
                  <p>{{ page.count }} résultat(s)</p>
                  <ul>
                    @for (m of page.results; track m.id) {
                      <li>{{ m.prenom }} {{ m.nom }}</li>
                    }
                  </ul>
                }
              `,
            })
            export class RechercheComponent {
              private readonly membres = inject(MembresService);

              // Chaque frappe du clavier est poussée dans ce `Subject`.
              readonly terme$ = new Subject<string>();

              readonly resultats$: Observable<Page<Membre>> = this.terme$.pipe(
                debounceTime(300),
                distinctUntilChanged(),
                switchMap((terme) => this.membres.rechercher(terme)),
              );

              dernierTerme = '';

              constructor() {
                this.terme$.subscribe((terme) => (this.dernierTerme = terme));
              }
            }
  - texte: >-
      L'abonnement du constructeur (`this.terme$.subscribe(…)`) n'est jamais coupé : le test `destruction` le montre. Ajoute `takeUntilDestroyed()` (de `@angular/core/rxjs-interop`) dans un `.pipe(…)` avant le `subscribe`. Puis lance `tester` : tout le projet doit être vert.
    indice: >-
      `this.terme$.pipe(takeUntilDestroyed()).subscribe(…)` : `takeUntilDestroyed()` s'appelle dans le constructeur, qui est un contexte d'injection.
    apres: [4]
    verif:
      - commande-reussit: tester recherche -t destruction
      - commande-reussit: tester
      - commande-reussit: contient src/app/recherche.ts 'takeUntilDestroyed\s*\('
    solution:
      - ecrire:
          src/app/recherche.ts: |
            import { AsyncPipe } from '@angular/common';
            import { Component, inject } from '@angular/core';
            import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
            import { debounceTime, distinctUntilChanged, Observable, Subject, switchMap } from 'rxjs';
            import { Membre, MembresService, Page } from './membres.service';

            @Component({
              selector: 'app-recherche',
              imports: [AsyncPipe],
              template: `
                <input #saisie (input)="terme$.next(saisie.value)" type="search" placeholder="Rechercher un membre" />

                @if (resultats$ | async; as page) {
                  <p>{{ page.count }} résultat(s)</p>
                  <ul>
                    @for (m of page.results; track m.id) {
                      <li>{{ m.prenom }} {{ m.nom }}</li>
                    }
                  </ul>
                }
              `,
            })
            export class RechercheComponent {
              private readonly membres = inject(MembresService);

              // Chaque frappe du clavier est poussée dans ce `Subject`.
              readonly terme$ = new Subject<string>();

              readonly resultats$: Observable<Page<Membre>> = this.terme$.pipe(
                debounceTime(300),
                distinctUntilChanged(),
                switchMap((terme) => this.membres.rechercher(terme)),
              );

              dernierTerme = '';

              constructor() {
                this.terme$.pipe(takeUntilDestroyed()).subscribe((terme) => (this.dernierTerme = terme));
              }
            }
:::

## Vérifie tes acquis

:::quiz
Que se passe-t-il si tu écris `this.http.get<Membre[]>(url)` sans jamais appeler `subscribe` ni utiliser `async` ?

- [ ] La requête part, mais la réponse est ignorée
- [ ] Angular s'abonne automatiquement à la fin de la méthode
- [x] Aucune requête n'est envoyée : l'Observable est paresseux
- [ ] Une erreur de compilation est levée

> `HttpClient` ne déclenche la requête qu'à l'abonnement. Il en faut un par exécution souhaitée.
:::

:::quiz
Dans une recherche instantanée, à quoi sert `switchMap` ?

- [ ] À attendre 300 ms avant d'envoyer la requête
- [ ] À ignorer deux termes identiques consécutifs
- [x] À lancer la nouvelle requête et à annuler la précédente si elle n'est pas terminée
- [ ] À convertir l'Observable en promesse

> `debounceTime` temporise, `distinctUntilChanged` filtre les doublons, `switchMap` remplace la requête en cours par la nouvelle.
:::

:::quiz
Quel est l'avantage du tube `async` dans un gabarit ?

- [x] Il s'abonne à l'Observable et se désabonne à la destruction du composant
- [ ] Il transforme l'Observable en tableau synchrone
- [ ] Il envoie la requête plus vite
- [ ] Il met en cache la réponse pour tous les composants

> `async` gère le cycle de vie de l'abonnement. Il n'accélère rien et ne met rien en cache.
:::

:::quiz
`this.http.get<Membre>(url)` : que garantit le type `Membre` ?

- [ ] Que le serveur renvoie bien un objet de cette forme
- [ ] Que la réponse est validée champ par champ à l'exécution
- [x] Rien à l'exécution : c'est une indication pour le compilateur, le JSON reçu n'est pas vérifié
- [ ] Que l'API est en HTTPS

> Les types disparaissent à la compilation. Si l'API change de forme, TypeScript ne le verra pas : il faut des tests ou une validation explicite.
:::
