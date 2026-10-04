---
titre: "Examen de validation — Angular"
tirage: 5
seuil: 80
duree: 10
melange: true
---

Cet examen valide les bases d'Angular : composants, services, formulaires, RxJS, Material, Keycloak et déploiement.

:::quiz
Qu'est-ce qui distingue un composant Angular d'une classe TypeScript ordinaire ?

- [ ] Il hérite obligatoirement de la classe `HTMLElement` et doit être enregistré à la main auprès du navigateur
- [x] Il est décoré par `@Component`, qui lui associe un sélecteur, un gabarit et des styles
- [ ] Il ne peut contenir aucune méthode, seulement des propriétés
- [ ] Il est exécuté uniquement sur le serveur lors du build

> Le décorateur `@Component` transforme la classe en brique d'interface : balise (`selector`), HTML et CSS.
:::

:::quiz
Dans un gabarit, quelle écriture appelle la méthode `valider()` quand on clique sur un bouton ?

- [ ] `[click]="valider()"`
- [ ] `{{ click: valider() }}`
- [x] `(click)="valider()"`
- [ ] `click="valider"`

> Les parenthèses désignent une liaison d'évènement ; les crochets lient une propriété.
:::

:::quiz
Quel bloc de contrôle répète un élément pour chaque entrée d'un tableau, dans la syntaxe moderne ?

- [ ] `*ngFor` uniquement, car `@for` n'existe pas encore
- [ ] `@if (x of liste)`
- [ ] `@switch (liste)`
- [x] `@for (x of liste; track x.id)`

> `@for` répète son bloc et exige `track` pour identifier chaque élément.
:::

:::quiz
Tu crées un composant dans Adhésion avec `standalone: false`. Que dois-tu faire pour qu'il soit utilisable ?

- [x] Le déclarer dans un `NgModule` (`declarations` d'`AppModule`)
- [ ] Écrire son sélecteur dans `main.ts`
- [ ] Le déplacer dans le dossier `assets`
- [ ] Ajouter `providedIn: 'root'` à sa classe

> Un composant non autonome appartient à un module, ici `AppModule`.
:::

:::quiz
Quelle écriture moderne déclare une donnée que le parent doit obligatoirement fournir ?

- [ ] `@Output() prenom: string;`
- [ ] `prenom = output<string>();`
- [x] `readonly prenom = input.required<string>();`
- [ ] `prenom = inject(String);`

> `input.required<T>()` est l'écriture moderne ; `@Input()` fonctionne toujours.
:::

:::quiz
Une collègue écrit `private ui = new UiService();` dans un composant. Quel est le problème principal ?

- [ ] Le mot-clé `new` est interdit dans les composants
- [ ] La classe ne compile pas
- [x] L'instance n'est pas celle gérée par Angular : elle n'est pas partagée et ne reçoit pas ses dépendances
- [ ] Il faut toujours préférer `static`

> L'injection de dépendances garantit une instance partagée et substituable, notamment en test.
:::

:::quiz
`UiService` d'Adhésion est un `@Injectable()` simple. Comment les composants peuvent-ils l'utiliser ?

- [ ] Il suffit de l'importer dans chaque composant
- [x] Il est listé dans les `providers` d'`AppModule`
- [ ] Il est fourni par le navigateur
- [ ] Il faut le déclarer dans `declarations`

> Sans `providedIn`, le service doit apparaître dans une liste `providers`.
:::

:::quiz
Tu écris `{ provide: Notifications, useClass: NotificationsSilencieuses }` dans `providers`. Quel est l'effet ?

- [ ] Les deux classes sont fusionnées
- [ ] `Notifications` est supprimée du projet
- [ ] La classe `Notifications` ne compile plus
- [x] Les composants qui demandent `Notifications` reçoivent une instance de `NotificationsSilencieuses`

> `provide` est la clé demandée, `useClass` la classe réellement instanciée.
:::

:::quiz
Quelle erreur obtient-on en injectant un service qui n'est fourni nulle part ?

- [x] `NullInjectorError: No provider for …`
- [ ] `TypeError: undefined is not a function` à la compilation
- [ ] `404 Not Found`
- [ ] Aucune : Angular crée le service automatiquement

> Angular ne sait pas fabriquer le service tant qu'il n'est pas fourni.
:::

:::quiz
Quel import faut-il pour utiliser `[formGroup]` dans un gabarit ?

- [ ] `HttpClientModule`
- [ ] `FormsModule` uniquement
- [ ] `MatFormFieldModule`
- [x] `ReactiveFormsModule`

> `ReactiveFormsModule` fournit `formGroup` et `formControlName`.
:::

:::quiz
Un champ porte `Validators.required`. Quand est-il invalide ?

- [x] Quand sa valeur est vide
- [ ] Quand il contient plus de dix caractères
- [ ] Quand il n'a pas été touché
- [ ] Quand le serveur renvoie une erreur

> `required` n'accepte pas une valeur vide. Pour l'e-mail, c'est `Validators.email`.
:::

:::quiz
L'API répond `400` : l'adresse e-mail est déjà utilisée. Quel appel permet de l'afficher sur le champ concerné ?

- [ ] `form.reset()`
- [x] `form.get('email')?.setErrors({ serveur: true })`
- [ ] `form.patchValue({ email: null })`
- [ ] `form.disable()`

> `setErrors` ajoute une erreur personnalisée que le gabarit peut tester avec `hasError`.
:::

:::quiz
Que se passe-t-il si on n'appelle jamais `subscribe` sur le résultat de `http.get(...)` ?

- [ ] La requête part quand même, en arrière-plan
- [x] La requête n'est pas envoyée
- [ ] La requête est envoyée deux fois
- [ ] Angular s'abonne à la fin de la méthode

> Un Observable est paresseux : la requête part à l'abonnement.
:::

:::quiz
Dans une recherche instantanée, quel opérateur annule la requête précédente quand une nouvelle valeur arrive ?

- [ ] `debounceTime`
- [ ] `distinctUntilChanged`
- [ ] `map`
- [x] `switchMap`

> `switchMap` remplace la requête en cours ; `debounceTime` temporise et `distinctUntilChanged` filtre les doublons.
:::

:::quiz
Dans un gabarit, tu écris `@if (resultats$ | async; as page)`. Qui se désabonne de `resultats$` ?

- [ ] Personne : il faut appeler `unsubscribe()` à la main
- [ ] Le serveur, à la fin de la réponse
- [ ] Le navigateur, quand l'onglet est fermé
- [x] Le tube `async`, à la destruction du composant

> `async` gère l'abonnement et le désabonnement à ta place.
:::

:::quiz
Que reproche-t-on à l'écriture `subscribe(fnSucces, fnErreur)` en RxJS 7 ?

- [ ] Elle ne peut pas gérer les erreurs
- [x] Elle est dépréciée au profit de l'objet `{ next, error }`
- [ ] Elle est plus lente
- [ ] Elle n'existe plus du tout

> La signature avec plusieurs fonctions est dépréciée ; l'objet observateur est l'écriture recommandée.
:::

:::quiz
Tu veux utiliser `<mat-card>` dans un composant autonome. Que dois-tu écrire ?

- [ ] `providers: [MatCard]`
- [x] `imports: [MatCardModule]`
- [ ] `declarations: [MatCardModule]`
- [ ] Rien : toutes les balises Material sont globales et disponibles dans chaque gabarit de l'application

> Chaque famille Material a son module, à importer là où le gabarit s'en sert.
:::

:::quiz
Un dialogue est ouvert avec `dialog.open(MonDialogue, { data: { nom } })`. Comment le dialogue lit-il `nom` ?

- [ ] Avec `@Input() nom`
- [ ] Dans l'adresse de la page
- [ ] Dans le jeton Keycloak
- [x] Par injection de `MAT_DIALOG_DATA`

> L'option `data` est transmise au dialogue, qui l'injecte avec `MAT_DIALOG_DATA`.
:::

:::quiz
Dans l'architecture d'Adhésion, que prouve le jeton (*token*) envoyé à l'API ?

- [ ] La clé secrète de l'application
- [x] L'identité et les rôles de la personne connectée, signés par Keycloak
- [ ] L'adresse IP du serveur
- [ ] Le mot de passe de la personne

> Le jeton est présenté à l'API à chaque requête ; il se traite comme un mot de passe temporaire.
:::

:::quiz
Que se passe-t-il quand une route protégée par `canActivate: [AuthGuard]` est demandée ?

- [ ] Le composant est affiché sans vérification
- [ ] L'API vérifie le jeton à la place du navigateur
- [ ] La page est mise en cache puis affichée sans aucune vérification de l'identité
- [x] La garde s'exécute et peut refuser la navigation

> Une garde améliore la navigation, mais la sécurité réelle reste celle de l'API.
:::

:::quiz
Un intercepteur ajoute le jeton à toutes les requêtes, même vers un site tiers. Quelle correction apporter ?

- [x] Ajouter l'en-tête seulement si l'adresse commence par celle de l'API
- [ ] Chiffrer le jeton en base 64
- [ ] Doubler la durée de vie du jeton pour limiter le nombre de renouvellements
- [ ] Supprimer l'intercepteur et saisir le jeton à chaque appel

> Un jeton envoyé à un tiers permettrait à celui-ci de se faire passer pour la personne.
:::

:::quiz
Lors d'un `ng build --configuration production`, que devient `src/environments/environment.ts` ?

- [ ] Il est supprimé du dépôt Git dès le premier build de production
- [ ] Il est chiffré
- [x] Il est remplacé dans le build par `environment.prod.ts`
- [ ] Il est copié tel quel dans `dist/`, avec ses adresses de développement

> `fileReplacements` substitue un fichier par un autre à la compilation.
:::

:::quiz
Dans `environment.prod.ts`, la valeur `'__KEYCLOAK_URL__'` est remplacée par `start.sh`. Quand ?

- [ ] Pendant `npm install`
- [x] Au démarrage du conteneur, avec les variables d'environnement
- [ ] Quand une personne se connecte
- [ ] À la relecture du code

> Une même image peut ainsi servir dans plusieurs environnements. Le résultat reste public : pas de secret.
:::

:::quiz
Quelle ligne de `nginx/default.conf` permet d'ouvrir directement l'adresse `/members/12` d'une application Angular ?

- [x] `try_files $uri $uri/ /index.html;`
- [ ] `listen 80;`
- [ ] `gzip_static on;`
- [ ] `server_name localhost;`

> L'application n'a qu'une page HTML : le routeur gère le reste dans le navigateur.
:::

:::quiz
Dans un test Jasmine, que fait `beforeEach` ?

- [ ] Il décrit un test individuel
- [ ] Il s'exécute une seule fois à la fin
- [x] Il prépare l'état avant chaque test
- [ ] Il remplace `describe`

> `beforeEach` repart d'un état propre pour chaque test.
:::

:::quiz
Comment tester un service qui appelle l'API sans accès au réseau ?

- [x] Avec `HttpTestingController`, qui permet de simuler la réponse
- [ ] En lançant la vraie API sur ta machine à chaque test
- [ ] En supprimant l'appel `subscribe`
- [ ] En remplaçant `HttpClient` par `fetch`

> `HttpTestingController` (avec `provideHttpClientTesting()`) vérifie les requêtes et simule les réponses sans réseau.
:::
