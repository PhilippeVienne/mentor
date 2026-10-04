import { Component } from '@angular/core';
import { FormControl, ReactiveFormsModule, Validators } from '@angular/forms';

@Component({
  selector: 'app-champ-email',
  // À FAIRE (étape 2) : importe ce qu'il faut pour utiliser `mat-form-field` et `matInput`.
  imports: [ReactiveFormsModule],
  // À FAIRE (étape 2) : entoure le champ d'un `mat-form-field` avec un `mat-label`, un `mat-hint` et un `mat-error`.
  template: `<input [formControl]="email" type="email" />`,
})
export class ChampEmailComponent {
  readonly email = new FormControl('', { nonNullable: true, validators: [Validators.required, Validators.email] });
}
