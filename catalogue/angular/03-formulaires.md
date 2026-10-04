---
id: formulaires
titre: "Formulaires et validation"
resume: "Construire un formulaire réactif typé, le valider et afficher les erreurs, y compris celles de l'API."
duree: 40
objectifs:
  - Construire un `FormGroup` typé avec ses `FormControl`
  - Brancher le formulaire sur un gabarit avec `[formGroup]` et `formControlName`
  - Utiliser les validateurs intégrés et lire l'état d'un champ
  - Reporter une erreur renvoyée par l'API sur le champ concerné
---

Un formulaire sert à saisir des données : créer un adhérent, modifier une fiche. Un formulaire d'inscription qui accepte une adresse e-mail vide ou un numéro de téléphone mal écrit, c'est des données inutilisables en base. Angular propose un système de formulaires **réactifs** : le formulaire est décrit dans la classe TypeScript, le gabarit n'a plus qu'à s'y accrocher.

## Les briques : FormControl, FormGroup, validateurs

Pense à une fiche papier : chaque case à remplir est un `FormControl`, la fiche entière un `FormGroup`, et les consignes du type « obligatoire » ou « doit contenir un @ » des validateurs.

- Un `FormControl` représente **un champ** (valeur, état de validité, erreurs).
- Un `FormGroup` regroupe des champs, et peut en contenir d'autres (groupe imbriqué).
- Un **validateur** est une règle (`Validators.required`, `Validators.email`, `Validators.pattern(…)`…) rattachée à un champ.

```ts
import { Component } from '@angular/core';
import { FormControl, FormGroup, ReactiveFormsModule, Validators } from '@angular/forms';

@Component({
  selector: 'app-inscription',
  imports: [ReactiveFormsModule],
  templateUrl: './inscription.component.html',
})
export class InscriptionComponent {
  form = new FormGroup({
    prenom: new FormControl('', { nonNullable: true, validators: [Validators.required] }),
    email: new FormControl('', { nonNullable: true, validators: [Validators.required, Validators.email] }),
    telephone: new FormControl('', { nonNullable: true, validators: [Validators.pattern(/^\+[0-9]*$/)] }),
  });

  enregistrer() {
    if (this.form.invalid) {
      return;
    }
    // this.form.getRawValue() est typé : { prenom: string; email: string; telephone: string }
    console.log(this.form.getRawValue());
  }
}
```

Lecture du code :

- `imports: [ReactiveFormsModule]` donne au gabarit les outils des formulaires réactifs.
- `new FormControl('', {…})` crée un champ dont la valeur de départ est la chaîne vide, avec ses validateurs.
- `Validators.required` refuse une valeur vide ; `Validators.email` vérifie la forme d'une adresse ; `Validators.pattern(/^\+[0-9]*$/)` vérifie une **expression régulière** (une description de forme de texte : ici un `+` puis des chiffres).
- `this.form.invalid` est vrai si au moins une règle est violée.

Quelques précisions :

- Depuis Angular 14, les formulaires sont **typés** : `this.form.controls.prenom.value` est un `string`, pas un `any`.
- `nonNullable: true` évite que `reset()` remette la valeur à `null` et garde le type `string`.
- Le validateur `pattern` ci-dessus reprend l'idée du champ téléphone d'Adhésion (`pattern="\+[0-9]*"`) : un `+` suivi de chiffres.

:::info Dans Adhésion
Le dépôt a été migré de versions plus anciennes d'Angular : `members/create/create.component.ts` utilise `UntypedFormGroup` et `UntypedFormControl`. Ce sont les versions « sans types » de `FormGroup` et `FormControl`, pratiques pour une migration. Pour un nouvel écran, écris des formulaires typés comme ci-dessus.
:::

## Le gabarit

Le gabarit relie chaque champ à son contrôle avec `formControlName` :

```html
<form [formGroup]="form" (ngSubmit)="enregistrer()">
  <label>
    Prénom
    <input formControlName="prenom" type="text" />
  </label>
  @if (form.controls.prenom.touched && form.controls.prenom.hasError('required')) {
    <p class="erreur">Le prénom est obligatoire.</p>
  }

  <label>
    E-mail
    <input formControlName="email" type="email" />
  </label>
  @if (form.controls.email.touched && form.controls.email.invalid) {
    <p class="erreur">Saisis une adresse e-mail valide.</p>
  }

  <button type="submit" [disabled]="form.invalid">Enregistrer</button>
</form>
```

Ligne par ligne :

- `[formGroup]` désigne le groupe, `(ngSubmit)` est appelé à la validation du formulaire.
- `formControlName="prenom"` relie ce champ HTML au contrôle `prenom`.
- `form.controls.prenom.touched` est vrai quand la personne est passée sur le champ puis en est ressortie.
- `[disabled]="form.invalid"` grise le bouton tant que le formulaire est invalide.

Points d'attention :

- Sans `ReactiveFormsModule` dans les `imports` du composant (ou du module), `[formGroup]` provoque une erreur de compilation.
- Un champ a des états utiles : `valid` / `invalid`, `touched` (la personne est passée dessus), `dirty` (elle a modifié la valeur).
- On n'affiche l'erreur qu'après `touched`, pour ne pas accueillir la personne avec un écran rouge.

Le formulaire de création d'adhérent·e d'Adhésion fonctionne de la même façon, avec Angular Material autour (leçon 5) :

```html
<form (ngSubmit)="save(form.value)" [formGroup]="form">
  ...
  <input formControlName="first_name" matInput placeholder="Prénom" required />
  ...
  <div *ngIf="form.get('category').value === 'student'" formGroupName="student_profile">
```

- `(ngSubmit)="save(form.value)"` appelle la méthode `save` avec les valeurs du formulaire quand on le valide ; `form.value` est un objet qui regroupe toutes les saisies.
- `matInput` habille le champ avec Angular Material (leçon 5) et `required` est l'attribut HTML qui marque le champ comme obligatoire.
- `*ngIf="…"` est l'ancienne écriture de `@if` (leçon 1) : ce bloc n'existe que pour la catégorie `student`.
- Le `formGroupName="student_profile"` correspond à un `FormGroup` **imbriqué** dans le groupe principal.

## Modifier le formulaire depuis le code

| Méthode | Rôle |
| --- | --- |
| `form.patchValue({ email: 'a@b.fr' })` | Change une partie des valeurs |
| `form.setValue({...})` | Change **toutes** les valeurs (erreur s'il en manque) |
| `form.reset()` | Remet à l'état initial |
| `form.get('email')?.setErrors({ incorrect: true })` | Marque un champ en erreur à la main |

Adhésion utilise `patchValue` pour préremplir l'adresse e-mail pendant la saisie du nom (`autoFillEmail`), et `setErrors` pour une raison plus importante : reporter l'erreur renvoyée par le serveur.

## Les erreurs de l'API

Le navigateur peut valider une adresse e-mail, mais seul le serveur sait qu'elle existe déjà. Quand l'API répond `400` (le code HTTP qui signifie « requête invalide ») avec un corps de réponse en **JSON** (un format texte pour échanger des données, que tu as déjà croisé avec `fetch`) du type `{ "email": ["Cette adresse existe déjà."] }`, il faut l'afficher sur le bon champ :

```ts
this.membres.creer(this.form.getRawValue()).subscribe({
  next: () => this.router.navigateByUrl('/membres'),
  error: (reponse) => {
    for (const champ of Object.keys(reponse.error)) {
      this.form.get(champ)?.setErrors({ serveur: reponse.error[champ][0] });
    }
  },
});
```

Ligne par ligne : `creer(...)` appelle l'API (leçon 4) ; `subscribe` déclenche l'appel ; `next` s'exécute en cas de succès et `error` en cas d'échec ; la boucle parcourt chaque champ cité dans la réponse et y attache le message du serveur.

L'idée est celle du `save()` d'Adhésion, qui parcourt `response.error` et appelle `this.form.get(field).setErrors({ incorrect: true })`. Dans le gabarit, `form.controls.email.hasError('serveur')` permet d'afficher le message.

:::warning Un formulaire invalide ne doit pas partir
Vérifie `form.invalid` dans la méthode d'envoi, pas seulement avec `[disabled]` sur le bouton : on peut valider un formulaire avec la touche Entrée. Et rappelle-toi que la validation du navigateur ne remplace **jamais** celle du serveur.
:::

## Entraîne-toi

:::labo
moteur: reel
intro: |
  Le projet de `/workspace` contient un composant d'inscription `InscriptionComponent` à moitié écrit : sa classe est dans `src/app/inscription.ts`, son gabarit dans `src/app/inscription.html`. Le fichier `src/app/inscription.spec.ts` décrit tout ce qu'il doit faire : ne le modifie pas (`tester` remet de toute façon la version d'origine), fais-le passer groupe par groupe avec `tester inscription -t "mot-du-groupe"`.

  Le `MembresService` fourni est une version simulée (elle répond toujours « bien ») : tu n'as pas besoin de réseau, ni de serveur.
commandes:
  - cp -R /opt/exercices/03-formulaires/. .
  - /opt/angular/preparer
etapes:
  - texte: >-
      Dans `src/app/inscription.ts`, remplace `new FormGroup({})` par un `FormGroup` de trois `FormControl` typés, tous avec `{ nonNullable: true }` : `prenom` (obligatoire), `email` (obligatoire et de forme e-mail) et `telephone` (un `+` suivi de chiffres : `Validators.pattern(/^\+[0-9]*$/)`). Importe `FormControl`, `FormGroup` et `Validators` depuis `@angular/forms`. Vérifie avec `tester inscription -t validation`.
    indice: >-
      `prenom: new FormControl('', { nonNullable: true, validators: [Validators.required] })`, et pareil pour `email` (avec `Validators.required, Validators.email`) et `telephone`.
    verif:
      - commande-reussit: tester inscription -t validation
      - commande-reussit: contient src/app/inscription.ts 'nonNullable\s*:\s*true'
      - commande-reussit: contient src/app/inscription.ts 'Validators\.email'
    solution:
      - ecrire:
          src/app/inscription.ts: |
            import { Component, inject } from '@angular/core';
            import { FormControl, FormGroup, ReactiveFormsModule, Validators } from '@angular/forms';
            import { MembresService } from './membres.service';

            @Component({
              selector: 'app-inscription',
              imports: [ReactiveFormsModule],
              templateUrl: './inscription.html',
            })
            export class InscriptionComponent {
              private readonly membres = inject(MembresService);

              reussi = false;

              form = new FormGroup({
                prenom: new FormControl('', { nonNullable: true, validators: [Validators.required] }),
                email: new FormControl('', { nonNullable: true, validators: [Validators.required, Validators.email] }),
                telephone: new FormControl('', { nonNullable: true, validators: [Validators.pattern(/^\+[0-9]*$/)] }),
              });

              enregistrer() {
                // À FAIRE (étapes 4 et 5)
              }
            }
  - texte: >-
      Dans `src/app/inscription.html`, relie le formulaire : `<form [formGroup]="form" (ngSubmit)="enregistrer()">`, puis un `<input formControlName="…">` pour chacun des trois champs (`prenom`, `email`, `telephone`) et un bouton `type="submit"` grisé tant que le formulaire est invalide (`[disabled]="form.invalid"`). Garde le bloc `@if (reussi)` en bas. Vérifie avec `tester inscription -t liaison`.
    indice: >-
      `<input formControlName="prenom" type="text" />` : la valeur de `formControlName` est le nom du contrôle dans le `FormGroup`.
    apres: [1]
    verif:
      - commande-reussit: tester inscription -t liaison
      - commande-reussit: contient src/app/inscription.html '\[formGroup\]\s*=\s*.form.'
      - commande-reussit: contient src/app/inscription.html '\[disabled\]\s*=\s*.form\.invalid.'
    solution:
      - ecrire:
          src/app/inscription.html: |
            <form [formGroup]="form" (ngSubmit)="enregistrer()">
              <label>
                Prénom
                <input formControlName="prenom" type="text" />
              </label>

              <label>
                E-mail
                <input formControlName="email" type="email" />
              </label>

              <label>
                Téléphone
                <input formControlName="telephone" type="tel" />
              </label>

              <button type="submit" [disabled]="form.invalid">Enregistrer</button>
            </form>
            @if (reussi) {
              <p class="succes">Membre enregistré.</p>
            }
  - texte: >-
      Affiche les messages d'erreur, mais seulement quand la personne a quitté le champ (`touched`) : sous le prénom `<p class="erreur">Le prénom est obligatoire.</p>` (si l'erreur `required` est présente), sous l'e-mail `<p class="erreur">Saisis une adresse e-mail valide.</p>` (si le champ est invalide). Vérifie avec `tester inscription -t messages`.
    indice: >-
      `@if (form.controls.prenom.touched && form.controls.prenom.hasError('required')) { <p class="erreur">…</p> }` juste après le `<label>` du prénom ; même idée avec `form.controls.email.invalid` pour l'e-mail.
    apres: [2]
    verif:
      - commande-reussit: tester inscription -t messages
      - commande-reussit: contient src/app/inscription.html 'touched'
    solution:
      - ecrire:
          src/app/inscription.html: |
            <form [formGroup]="form" (ngSubmit)="enregistrer()">
              <label>
                Prénom
                <input formControlName="prenom" type="text" />
              </label>
              @if (form.controls.prenom.touched && form.controls.prenom.hasError('required')) {
                <p class="erreur">Le prénom est obligatoire.</p>
              }

              <label>
                E-mail
                <input formControlName="email" type="email" />
              </label>
              @if (form.controls.email.touched && form.controls.email.invalid) {
                <p class="erreur">Saisis une adresse e-mail valide.</p>
              }

              <label>
                Téléphone
                <input formControlName="telephone" type="tel" />
              </label>

              <button type="submit" [disabled]="form.invalid">Enregistrer</button>
            </form>
            @if (reussi) {
              <p class="succes">Membre enregistré.</p>
            }
  - texte: >-
      Écris la méthode `enregistrer()` : si `this.form.invalid`, elle ne fait rien (`return`) ; sinon elle appelle `this.membres.creer(this.form.getRawValue())` et, dans `subscribe({ next: … })`, passe `this.reussi` à `true`. Vérifie avec `tester inscription -t envoi`.
    indice: >-
      `if (this.form.invalid) { return; }` puis `this.membres.creer(this.form.getRawValue()).subscribe({ next: () => { this.reussi = true; } });`
    apres: [3]
    verif:
      - commande-reussit: tester inscription -t envoi
      - commande-reussit: contient src/app/inscription.ts 'this\.membres\.creer\s*\(\s*this\.form\.getRawValue\(\)\s*\)'
    solution:
      - ecrire:
          src/app/inscription.ts: |
            import { Component, inject } from '@angular/core';
            import { FormControl, FormGroup, ReactiveFormsModule, Validators } from '@angular/forms';
            import { MembresService } from './membres.service';

            @Component({
              selector: 'app-inscription',
              imports: [ReactiveFormsModule],
              templateUrl: './inscription.html',
            })
            export class InscriptionComponent {
              private readonly membres = inject(MembresService);

              reussi = false;

              form = new FormGroup({
                prenom: new FormControl('', { nonNullable: true, validators: [Validators.required] }),
                email: new FormControl('', { nonNullable: true, validators: [Validators.required, Validators.email] }),
                telephone: new FormControl('', { nonNullable: true, validators: [Validators.pattern(/^\+[0-9]*$/)] }),
              });

              enregistrer() {
                if (this.form.invalid) {
                  return;
                }
                this.membres.creer(this.form.getRawValue()).subscribe({
                  next: () => {
                    this.reussi = true;
                  },
                });
              }
            }
  - texte: >-
      Gère l'échec de l'API : ajoute à `subscribe` une clause `error: (reponse) => { … }` qui parcourt `Object.keys(reponse.error)` et, pour chaque champ cité, appelle `this.form.get(champ)?.setErrors({ serveur: reponse.error[champ][0] })`. Dans le gabarit, après le champ e-mail, affiche `form.controls.email.getError('serveur')` dans un `<p class="erreur">` quand `hasError('serveur')` est vrai. Puis lance `tester inscription` (tous les groupes).
    indice: >-
      La clause `error` s'écrit comme la clause `next`, dans le même objet. Dans le gabarit : `@if (form.controls.email.hasError('serveur')) { <p class="erreur">{{ form.controls.email.getError('serveur') }}</p> }`.
    apres: [4]
    verif:
      - commande-reussit: tester inscription
      - commande-reussit: tester --compile-seul
      - commande-reussit: contient src/app/inscription.ts 'setErrors\s*\('
      - commande-reussit: contient src/app/inscription.html 'getError\s*\(\s*[\x27\"]serveur'
    solution:
      - ecrire:
          src/app/inscription.ts: |
            import { Component, inject } from '@angular/core';
            import { FormControl, FormGroup, ReactiveFormsModule, Validators } from '@angular/forms';
            import { MembresService } from './membres.service';

            @Component({
              selector: 'app-inscription',
              imports: [ReactiveFormsModule],
              templateUrl: './inscription.html',
            })
            export class InscriptionComponent {
              private readonly membres = inject(MembresService);

              reussi = false;

              form = new FormGroup({
                prenom: new FormControl('', { nonNullable: true, validators: [Validators.required] }),
                email: new FormControl('', { nonNullable: true, validators: [Validators.required, Validators.email] }),
                telephone: new FormControl('', { nonNullable: true, validators: [Validators.pattern(/^\+[0-9]*$/)] }),
              });

              enregistrer() {
                if (this.form.invalid) {
                  return;
                }
                this.membres.creer(this.form.getRawValue()).subscribe({
                  next: () => {
                    this.reussi = true;
                  },
                  error: (reponse) => {
                    for (const champ of Object.keys(reponse.error)) {
                      this.form.get(champ)?.setErrors({ serveur: reponse.error[champ][0] });
                    }
                  },
                });
              }
            }
      - ecrire:
          src/app/inscription.html: |
            <form [formGroup]="form" (ngSubmit)="enregistrer()">
              <label>
                Prénom
                <input formControlName="prenom" type="text" />
              </label>
              @if (form.controls.prenom.touched && form.controls.prenom.hasError('required')) {
                <p class="erreur">Le prénom est obligatoire.</p>
              }

              <label>
                E-mail
                <input formControlName="email" type="email" />
              </label>
              @if (form.controls.email.touched && form.controls.email.invalid) {
                <p class="erreur">Saisis une adresse e-mail valide.</p>
              }
              @if (form.controls.email.hasError('serveur')) {
                <p class="erreur">{{ form.controls.email.getError('serveur') }}</p>
              }

              <label>
                Téléphone
                <input formControlName="telephone" type="tel" />
              </label>

              <button type="submit" [disabled]="form.invalid">Enregistrer</button>
            </form>
            @if (reussi) {
              <p class="succes">Membre enregistré.</p>
            }
:::

## Vérifie tes acquis

:::quiz
Que fait `new FormControl('', { nonNullable: true })` de plus que `new FormControl('')` ?

- [ ] Il rend le champ obligatoire
- [x] Il garde le type `string` (et la valeur `''` après un `reset()`) au lieu de `string | null`
- [ ] Il empêche la saisie d'espaces
- [ ] Il désactive le champ

> Sans `nonNullable`, un `reset()` remet `null` et le type de la valeur devient `string | null`. Pour rendre un champ obligatoire, il faut `Validators.required`.
:::

:::quiz
Quel import faut-il pour utiliser `[formGroup]` et `formControlName` dans un gabarit ?

- [ ] `FormsModule`
- [ ] `MatInputModule`
- [x] `ReactiveFormsModule`
- [ ] `HttpClientModule`

> `ReactiveFormsModule` fournit les directives des formulaires réactifs. `FormsModule` sert aux formulaires pilotés par le gabarit (`ngModel`). Adhésion importe les deux dans `AppModule`.
:::

:::quiz
Tu veux n'afficher « Le prénom est obligatoire. » qu'une fois que la personne a quitté le champ. Quelle condition utilises-tu ?

- [ ] `form.controls.prenom.valid`
- [ ] `form.controls.prenom.dirty && form.valid`
- [x] `form.controls.prenom.touched && form.controls.prenom.hasError('required')`
- [ ] `form.pristine`

> `touched` devient vrai quand le champ perd le focus ; `hasError('required')` cible la règle précise.
:::

:::quiz
L'API répond `400` : l'adresse e-mail existe déjà. Comment l'afficher sur le champ ?

- [ ] On ne peut pas : seules les règles du navigateur ont des erreurs
- [x] On appelle `setErrors` sur le contrôle `email` et on teste `hasError` dans le gabarit
- [ ] On recrée entièrement le `FormGroup`
- [ ] On affiche une alerte JavaScript avec `alert()`

> `setErrors` ajoute une erreur personnalisée à un contrôle, que le gabarit peut afficher comme n'importe quelle autre. C'est la technique employée par la création d'adhérent·e dans Adhésion.
:::
