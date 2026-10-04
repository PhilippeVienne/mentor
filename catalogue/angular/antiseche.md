## Composant et gabarit

| Besoin | Syntaxe |
| --- | --- |
| Déclarer un composant | `@Component({ selector: 'app-x', templateUrl: './x.component.html' })` |
| Afficher une valeur | `{{ valeur }}` |
| Lier une propriété | `[disabled]="enCours"` |
| Réagir à un évènement | `(click)="enregistrer()"` |
| Liaison dans les deux sens | `[(ngModel)]="champ"` |
| Condition | `@if (x) { … } @else { … }` |
| Boucle | `@for (m of liste; track m.id) { … }` |
| Entrée / sortie | `prenom = input.required<string>();` `choisi = output<string>();` |
| Composant d'un NgModule (Adhésion) | `standalone: false` + ajout dans `declarations` |

## Services et injection

| Besoin | Syntaxe |
| --- | --- |
| Service fourni partout | `@Injectable({ providedIn: 'root' })` |
| Service fourni par un module | `@Injectable()` + ligne dans `providers` |
| Injecter | `private readonly s = inject(MonService);` ou `constructor(private s: MonService)` |
| Remplacer une classe | `{ provide: Ancienne, useClass: Nouvelle }` |

## Formulaires réactifs

| Besoin | Syntaxe |
| --- | --- |
| Champ | `new FormControl('', { nonNullable: true, validators: [Validators.required] })` |
| Groupe | `new FormGroup({ email: new FormControl('') })` |
| Relier au gabarit | `[formGroup]="form"` `formControlName="email"` |
| Lire la valeur | `form.getRawValue()` |
| Erreur d'un champ | `form.controls.email.hasError('required')` |
| Erreur renvoyée par l'API | `form.get('email')?.setErrors({ serveur: true })` |

## HTTP et RxJS

| Besoin | Syntaxe |
| --- | --- |
| Appeler une API | `this.http.get<Type>(url, { params })` |
| Déclencher la requête | `.subscribe({ next: …, error: … })` |
| Transformer | `.pipe(map(…))` |
| Recherche instantanée | `debounceTime(300)`, `distinctUntilChanged()`, `switchMap(…)` |
| Afficher dans le gabarit | `@if (flux$ \| async; as valeur) { … }` |
| Fin d'abonnement automatique | `takeUntilDestroyed()` |

## Material

| Besoin | Syntaxe |
| --- | --- |
| Importer | `imports: [MatCardModule, MatButtonModule]` |
| Champ | `<mat-form-field><mat-label>…</mat-label><input matInput /></mat-form-field>` |
| Ouvrir un dialogue | `dialog.open(Composant, { width: '420px', data })` |
| Message temporaire | `snackBar.open('Fait !', 'OK', { duration: 2500 })` |

## Tests

| Besoin | Syntaxe |
| --- | --- |
| Regrouper, décrire | `describe('Nom', () => { it('fait ceci', () => { … }); });` |
| Vérifier | `expect(valeur).toBe(attendu)` |
| Préparer avant chaque test | `beforeEach(async () => { await TestBed.configureTestingModule({ imports: [MonComposant] }).compileComponents(); });` |
| Calculer le gabarit | `fixture.detectChanges()` |
| Simuler le réseau | `provideHttpClient(), provideHttpClientTesting()` puis `http.expectOne(url).flush(réponse)` |
| Lancer les tests | `ng test` (Karma et Jasmine dans Adhésion), `tester` dans les labos (Vitest) |
| Vérifier la compilation stricte | `ngc -p tsconfig.json --noEmit` (dans les labos) |

## Authentification et déploiement

| Besoin | Syntaxe |
| --- | --- |
| Protéger une route | `canActivate: [AuthGuard], data: { roles: ['staff'] }` |
| Jeton dans les requêtes | intercepteur : `req.clone({ setHeaders: { Authorization: 'Bearer ' + jeton } })` |
| Lancer en local | `ng serve` (http://localhost:4200/) |
| Créer un composant | `ng generate component nom` |
| Version de production | `npm run build` (`ng build --configuration production`) |
| Valeurs d'environnement | marqueurs `__NOM__` remplacés par `start.sh` au démarrage du conteneur |
