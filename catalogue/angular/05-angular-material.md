---
id: angular-material
title: "Angular Material"
summary: "Utiliser les composants Material (champs, cartes, dialogues, messages) dans l'interface d'administration."
minutes: 40
objectives:
  - Importer les composants Material nécessaires dans un composant ou un module
  - Construire un champ de saisie avec `mat-form-field` et `matInput`
  - Ouvrir une boîte de dialogue, lui passer des données et récupérer sa réponse
  - Afficher un message temporaire avec `MatSnackBar`
---

Une interface d'administration a besoin de beaucoup d'éléments : champs, boutons, tableaux, fenêtres. Écrire à la main un menu déroulant accessible, un sélecteur de date ou une boîte de dialogue qui piège le focus (le clavier reste dans la fenêtre ouverte, sans « s'échapper » vers la page derrière), c'est des semaines de travail. **Angular Material** fournit ces composants prêts à l'emploi, accessibles et cohérents visuellement. L'administration d'Adhésion les utilise partout.

## Importer ce dont on a besoin

Chaque famille de composants vit dans son propre module : `MatButtonModule`, `MatCardModule`, `MatInputModule`… Un **module** est ici un lot de composants prêts à l'emploi : on n'importe que ceux qu'on utilise, pour ne pas alourdir l'application.

Dans un composant autonome, on les liste dans `imports` :

```ts
import { Component } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatCardModule } from '@angular/material/card';

@Component({
  selector: 'app-carte-bienvenue',
  imports: [MatCardModule, MatButtonModule],
  template: `
    <mat-card appearance="outlined">
      <mat-card-header>
        <mat-card-title>Bienvenue</mat-card-title>
      </mat-card-header>
      <mat-card-content>Prête pour la prochaine sortie photo ?</mat-card-content>
      <mat-card-actions>
        <button mat-flat-button type="button">Je m'inscris</button>
      </mat-card-actions>
    </mat-card>
  `,
})
export class CarteBienvenueComponent {}
```

Ligne par ligne : `mat-card` est la carte ; `mat-flat-button` est un attribut qui donne au bouton le style Material « plein » ; `appearance="outlined"` choisit le contour.

Adhésion, organisée en modules, a regroupé tout cela dans un `MaterialModule` (`src/app/material/material.module.ts`) qui **réexporte** une trentaine de modules Material (`MatButtonModule`, `MatCardModule`, `MatDialogModule`, `MatTableModule`…). `AppModule` importe `MaterialModule` et tous les composants déclarés y ont alors accès :

```ts
@NgModule({
  exports: [MatButtonModule, MatCardModule, MatDialogModule /* … */],
})
export class MaterialModule {}
```

- `@NgModule({ … })` déclare un module (le « classeur » de la leçon 1).
- `exports` liste ce que ce module met à disposition des modules qui l'importent : ici, les modules Material.
- La classe `MaterialModule` est vide : elle n'a pas de code, c'est seulement un point d'entrée pour regrouper les `exports`.

Si tu ajoutes un composant Material qui n'est pas encore dans cette liste, ajoute son module à `exports` (et à l'import en tête de fichier).

## Les champs de formulaire

Un champ de formulaire (leçon 3) peut s'afficher avec Material. Un champ Material s'écrit avec `mat-form-field` autour d'un `<input matInput>` :

```html
<mat-form-field appearance="outline">
  <mat-label>Adresse e-mail</mat-label>
  <mat-icon matPrefix>email</mat-icon>
  <input matInput formControlName="email" type="email" />
  <mat-hint>Utilise ton adresse de l'école</mat-hint>
  <mat-error>Adresse invalide</mat-error>
</mat-form-field>
```

- `mat-label` flotte au-dessus du champ une fois rempli.
- `matPrefix` place une icône avant la saisie (les icônes viennent de `<mat-icon>nom</mat-icon>`).
- `mat-error` ne s'affiche **que** lorsque le contrôle est invalide et touché : plus besoin d'écrire le `@if` de la leçon 3.
- Pour une liste de choix, on utilise `<mat-select formControlName="categorie">` avec des `<mat-option value="…">`, comme dans la page de création d'adhérent·e.

Le champ de recherche d'Adhésion combine `mat-form-field`, `mat-label`, `mat-icon matPrefix` et `matInput`, puis un `mat-spinner` pendant le chargement et des `mat-card` pour chaque résultat (`members/list/list.component.html`).

## Boîtes de dialogue

Un **dialogue** est une petite fenêtre qui s'affiche au-dessus de la page pour demander une confirmation ou une saisie. Un dialogue Material est un **composant** que l'on ouvre avec le service `MatDialog`. Il reçoit des données par `MAT_DIALOG_DATA` et renvoie un résultat en se fermant. Voici une version complète, sur un exemple fictif de confirmation :

```ts
import { Component, inject } from '@angular/core';
import { MAT_DIALOG_DATA, MatDialog, MatDialogModule, MatDialogRef } from '@angular/material/dialog';

export interface ConfirmationData {
  nom: string;
}

@Component({
  selector: 'app-confirmation-dialog',
  imports: [MatDialogModule],
  template: `
    <h2 mat-dialog-title>Supprimer {{ data.nom }} ?</h2>
    <mat-dialog-actions>
      <button type="button" (click)="ref.close(false)">Annuler</button>
      <button type="button" (click)="ref.close(true)">Supprimer</button>
    </mat-dialog-actions>
  `,
})
export class ConfirmationDialogComponent {
  readonly data = inject<ConfirmationData>(MAT_DIALOG_DATA);
  readonly ref = inject(MatDialogRef<ConfirmationDialogComponent, boolean>);
}

// Dans le composant qui l'ouvre :
// private readonly dialog = inject(MatDialog);
//
// supprimer(nom: string) {
//   this.dialog
//     .open(ConfirmationDialogComponent, { width: '420px', data: { nom } })
//     .afterClosed()
//     .subscribe((confirme) => { if (confirme) { /* appeler l'API */ } });
// }
```

Ligne par ligne : `mat-dialog-title` et `mat-dialog-actions` sont des zones prévues par Material ; `inject(MAT_DIALOG_DATA)` récupère les données passées à l'ouverture ; `ref.close(true)` ferme la fenêtre et renvoie `true` ; `afterClosed()` est un Observable (leçon 4) qui émet ce résultat.

Le `BanDialogComponent` d'Adhésion fonctionne selon ce principe, avec l'écriture à base de constructeur : `@Inject(MAT_DIALOG_DATA) public data: BanDialogData` pour recevoir le nom du membre, et `dialogRef.close({ confirmed: true, reason })` pour répondre. `MemberViewComponent` ouvre ses dialogues avec `this.dialog.open(UnbanDialogComponent, { width: '420px', … })`.

## Messages temporaires

Pour un retour discret après une action (« Adhérent effacé ! »), `MatSnackBar` affiche un bandeau qui disparaît seul :

```ts
private readonly snackBar = inject(MatSnackBar);

this.snackBar.open('Adhérent effacé !', 'OK', { duration: 2500 });
```

Ligne par ligne : le premier argument est le message ; le deuxième est le texte du bouton d'action (facultatif) ; `duration` est en millisecondes. Adhésion s'en sert aussi pour demander une confirmation légère : `open('Voulez-vous vraiment supprimer cet adhérent ?', 'oui', …).onAction().subscribe(…)` déclenche la suppression si la personne clique sur « oui ».

:::tip Traduire les composants
Les textes intégrés à Material sont en anglais (« Items per page »). Adhésion les traduit en fournissant une sous-classe de `MatPaginatorIntl` (une classe qui étend celle d'origine en remplaçant ce qu'elle veut : `FrenchMatPaginatorIntl`) via `{ provide: MatPaginatorIntl, useClass: … }`, comme vu à la leçon 2.
:::

:::warning « 'mat-card' is not a known element »
C'est l'erreur la plus fréquente avec Material : le module du composant n'est pas importé là où le gabarit l'utilise. Ajoute-le dans `imports` du composant autonome, ou dans `MaterialModule` pour Adhésion. Adhésion importe aussi `BrowserAnimationsModule` dans `AppModule` (le module qui active les animations, comme l'ouverture douce d'une fenêtre) : si tu crées une application Angular neuve avec Material, pense à activer les animations (`provideAnimations()` ou `provideAnimationsAsync()`) selon la version de Material utilisée.
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  Le projet de `/workspace` contient quatre composants ou services Material à compléter, chacun avec son fichier de test (`*.spec.ts`, à ne pas modifier : `tester` remet de toute façon les originaux). Les tests créent vraiment les composants Angular Material dans un faux navigateur : si un module Material manque, le compilateur te le dit avec une erreur `NG8001 … is not a known element`.

  Tu lances un test avec `tester` suivi d'une partie du nom du fichier (`tester carte-bienvenue`).
commands:
  - cp -R /opt/exercices/05-angular-material/. .
  - /opt/angular/preparer
steps:
  - text: >-
      Lance `tester carte-bienvenue` et lis l'erreur : `'mat-card' is not a known element`. Dans `src/app/carte-bienvenue.ts`, importe `MatCardModule` (de `@angular/material/card`) et `MatButtonModule` (de `@angular/material/button`) et liste-les dans `imports`. Relance : le test doit passer.
    hint: >-
      `imports: [MatCardModule, MatButtonModule],` dans le décorateur `@Component`, plus les deux lignes `import` en haut du fichier.
    checks:
      - command-succeeds: tester carte-bienvenue
      - command-succeeds: contient src/app/carte-bienvenue.ts 'imports\s*:\s*\[[^\]]*MatCardModule'
      - command-succeeds: contient src/app/carte-bienvenue.ts 'imports\s*:\s*\[[^\]]*MatButtonModule'
    solution:
      - write:
          src/app/carte-bienvenue.ts: |
            import { Component } from '@angular/core';
            import { MatButtonModule } from '@angular/material/button';
            import { MatCardModule } from '@angular/material/card';

            @Component({
              selector: 'app-carte-bienvenue',
              imports: [MatCardModule, MatButtonModule],
              template: `
                <mat-card appearance="outlined">
                  <mat-card-header>
                    <mat-card-title>Bienvenue</mat-card-title>
                  </mat-card-header>
                  <mat-card-content>Prête pour la prochaine sortie photo ?</mat-card-content>
                  <mat-card-actions>
                    <button mat-flat-button type="button">Je m'inscris</button>
                  </mat-card-actions>
                </mat-card>
              `,
            })
            export class CarteBienvenueComponent {}
  - text: >-
      Dans `src/app/champ-email.ts`, habille le champ de saisie : un `<mat-form-field appearance="outline">` qui contient `<mat-label>Adresse e-mail</mat-label>`, l'`<input matInput [formControl]="email" type="email" />`, `<mat-hint>Utilise ton adresse de l'école</mat-hint>` et `<mat-error>Adresse invalide</mat-error>`. Ajoute `MatFormFieldModule` et `MatInputModule` à `imports` (de `@angular/material/form-field` et `@angular/material/input`). Vérifie avec `tester champ-email`.
    hint: >-
      Le champ garde `[formControl]="email"` : c'est `matInput` qui le rend Material. `mat-error` ne s'affiche que si le contrôle est invalide **et** touché.
    after: [1]
    checks:
      - command-succeeds: tester champ-email
      - command-succeeds: contient src/app/champ-email.ts '<mat-form-field'
      - command-succeeds: contient src/app/champ-email.ts 'matInput'
    solution:
      - write:
          src/app/champ-email.ts: |
            import { Component } from '@angular/core';
            import { FormControl, ReactiveFormsModule, Validators } from '@angular/forms';
            import { MatFormFieldModule } from '@angular/material/form-field';
            import { MatInputModule } from '@angular/material/input';

            @Component({
              selector: 'app-champ-email',
              imports: [ReactiveFormsModule, MatFormFieldModule, MatInputModule],
              template: `
                <mat-form-field appearance="outline">
                  <mat-label>Adresse e-mail</mat-label>
                  <input matInput [formControl]="email" type="email" />
                  <mat-hint>Utilise ton adresse de l'école</mat-hint>
                  <mat-error>Adresse invalide</mat-error>
                </mat-form-field>
              `,
            })
            export class ChampEmailComponent {
              readonly email = new FormControl('', { nonNullable: true, validators: [Validators.required, Validators.email] });
            }
  - text: >-
      Dans `src/app/confirmation-dialog.ts`, fais fonctionner le dialogue : récupère les données avec `readonly data = inject<ConfirmationData>(MAT_DIALOG_DATA)` et la référence du dialogue avec `readonly ref = inject(MatDialogRef<ConfirmationDialogComponent, boolean>)`. Le titre devient `Supprimer {{ data.nom }} ?`, le bouton « Annuler » appelle `ref.close(false)` et « Supprimer » appelle `ref.close(true)`. Vérifie avec `tester confirmation-dialog`.
    hint: >-
      Importe `inject` (de `@angular/core`), `MAT_DIALOG_DATA` et `MatDialogRef` (de `@angular/material/dialog`). Dans le gabarit : `(click)="ref.close(true)"`.
    after: [2]
    checks:
      - command-succeeds: tester confirmation-dialog
      - command-succeeds: contient src/app/confirmation-dialog.ts 'MAT_DIALOG_DATA'
      - command-succeeds: contient src/app/confirmation-dialog.ts '\.close\s*\(\s*true\s*\)'
    solution:
      - write:
          src/app/confirmation-dialog.ts: |
            import { Component, inject } from '@angular/core';
            import { MAT_DIALOG_DATA, MatDialogModule, MatDialogRef } from '@angular/material/dialog';

            export interface ConfirmationData {
              nom: string;
            }

            @Component({
              selector: 'app-confirmation-dialog',
              imports: [MatDialogModule],
              template: `
                <h2 mat-dialog-title>Supprimer {{ data.nom }} ?</h2>
                <mat-dialog-actions>
                  <button type="button" (click)="ref.close(false)">Annuler</button>
                  <button type="button" (click)="ref.close(true)">Supprimer</button>
                </mat-dialog-actions>
              `,
            })
            export class ConfirmationDialogComponent {
              readonly data = inject<ConfirmationData>(MAT_DIALOG_DATA);
              readonly ref = inject(MatDialogRef<ConfirmationDialogComponent, boolean>);
            }
  - text: >-
      Dans `src/app/notifier.ts`, écris `adherentEfface()` : elle affiche un message temporaire avec `this.snackBar.open('Adhérent effacé !', 'OK', { duration: 2500 })`. Vérifie avec `tester notifier`.
    hint: >-
      Le service `MatSnackBar` est déjà injecté dans `snackBar` : il ne reste qu'à appeler `open` avec le message, le texte du bouton et la durée en millisecondes.
    after: [3]
    checks:
      - command-succeeds: tester notifier
      - command-succeeds: contient src/app/notifier.ts 'snackBar\.open\s*\('
    solution:
      - write:
          src/app/notifier.ts: |
            import { inject, Injectable } from '@angular/core';
            import { MatSnackBar } from '@angular/material/snack-bar';

            @Injectable({ providedIn: 'root' })
            export class Notifier {
              private readonly snackBar = inject(MatSnackBar);

              adherentEfface() {
                this.snackBar.open('Adhérent effacé !', 'OK', { duration: 2500 });
              }
            }
  - text: >-
      Traduis le paginateur. Dans `src/app/french-paginator-intl.ts`, donne à la classe `FrenchMatPaginatorIntl` deux propriétés : `override itemsPerPageLabel = 'Éléments par page :';` et `override nextPageLabel = 'Page suivante';`. Dans `src/app/fournisseurs.ts`, ajoute la recette `{ provide: MatPaginatorIntl, useClass: FrenchMatPaginatorIntl }`. Puis lance `tester` : tout doit être vert, et `ngc -p tsconfig.json --noEmit` doit compiler sans erreur.
    hint: >-
      `override` est obligatoire : ces propriétés existent déjà dans `MatPaginatorIntl`, que tu étends. Importe `MatPaginatorIntl` de `@angular/material/paginator` dans `fournisseurs.ts`.
    after: [4]
    checks:
      - command-succeeds: tester
      - command-succeeds: tester --compile-seul
      - command-succeeds: contient src/app/french-paginator-intl.ts 'itemsPerPageLabel\s*='
      - command-succeeds: contient src/app/fournisseurs.ts 'provide\s*:\s*MatPaginatorIntl'
    solution:
      - write:
          src/app/french-paginator-intl.ts: |
            import { Injectable } from '@angular/core';
            import { MatPaginatorIntl } from '@angular/material/paginator';

            @Injectable()
            export class FrenchMatPaginatorIntl extends MatPaginatorIntl {
              override itemsPerPageLabel = 'Éléments par page :';
              override nextPageLabel = 'Page suivante';
            }
      - write:
          src/app/fournisseurs.ts: |-
            import { Provider } from '@angular/core';
            import { MatPaginatorIntl } from '@angular/material/paginator';
            import { FrenchMatPaginatorIntl } from './french-paginator-intl';

            export const fournisseurs: Provider[] = [{ provide: MatPaginatorIntl, useClass: FrenchMatPaginatorIntl }];
:::

## Vérifie tes acquis

:::quiz
Dans un composant autonome, où déclares-tu que tu utilises `<mat-card>` ?

- [ ] Dans `providers`
- [ ] Dans `declarations`
- [x] Dans `imports`, avec `MatCardModule`
- [ ] Nulle part : Material est global

> Chaque famille de composants Material a son module, à importer là où son gabarit en a besoin.
:::

:::quiz
À quoi sert le `MaterialModule` d'Adhésion ?

- [ ] À remplacer le thème de Material par celui de l'équipe
- [x] À regrouper et réexporter les modules Material pour que tous les composants d'`AppModule` y aient accès
- [ ] À déclarer les composants Material dans `declarations`
- [ ] À fournir les services HTTP

> Le module n'a que des `exports`. `AppModule` n'importe que lui, au lieu de trente modules Material séparés.
:::

:::quiz
Comment une boîte de dialogue Material reçoit-elle ses données et renvoie-t-elle sa réponse ?

- [ ] Par des `@Input()` et `@Output()` du dialogue
- [ ] Par des variables globales dans `window`
- [x] Par `MAT_DIALOG_DATA` pour les données entrantes, et par `MatDialogRef.close(résultat)` récupéré via `afterClosed()`
- [ ] Par l'URL, avec des paramètres de requête

> `dialog.open(Composant, { data })` transmet `data` au dialogue ; `close(valeur)` ferme le dialogue et émet `valeur` dans `afterClosed()`.
:::

:::quiz
Que fait `this.snackBar.open('Adhérent effacé !', '', { duration: 2500 })` ?

- [ ] Elle ouvre une boîte de dialogue bloquante pendant 2500 ms
- [ ] Elle écrit le message dans la console du navigateur
- [x] Elle affiche un bandeau qui disparaît tout seul après 2,5 secondes
- [ ] Elle attend 2500 ms puis demande confirmation

> `MatSnackBar` affiche une notification temporaire. `duration` est exprimée en millisecondes.
:::
