import { Component } from '@angular/core';
import { MatDialogModule } from '@angular/material/dialog';

export interface ConfirmationData {
  nom: string;
}

@Component({
  selector: 'app-confirmation-dialog',
  imports: [MatDialogModule],
  // À FAIRE (étape 3) : affiche le nom reçu et fais répondre `false` (Annuler) ou `true` (Supprimer).
  template: `
    <h2 mat-dialog-title>Supprimer ?</h2>
    <mat-dialog-actions>
      <button type="button">Annuler</button>
      <button type="button">Supprimer</button>
    </mat-dialog-actions>
  `,
})
export class ConfirmationDialogComponent {}
