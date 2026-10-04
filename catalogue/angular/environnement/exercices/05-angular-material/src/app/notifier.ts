import { inject, Injectable } from '@angular/core';
import { MatSnackBar } from '@angular/material/snack-bar';

@Injectable({ providedIn: 'root' })
export class Notifier {
  private readonly snackBar = inject(MatSnackBar);

  // À FAIRE (étape 4) : affiche « Adhérent effacé ! » avec le bouton « OK », pendant 2,5 secondes.
  adherentEfface() {}
}
