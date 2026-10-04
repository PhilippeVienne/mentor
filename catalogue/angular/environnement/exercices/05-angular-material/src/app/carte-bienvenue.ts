import { Component } from '@angular/core';

@Component({
  selector: 'app-carte-bienvenue',
  // À FAIRE (étape 1) : le gabarit utilise des composants Material que ce composant autonome ne connaît pas encore.
  imports: [],
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
