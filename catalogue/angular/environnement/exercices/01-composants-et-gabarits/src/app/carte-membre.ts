import { Component } from '@angular/core';

// À FAIRE (étape 6) : transforme cette carte en composant qui reçoit un prénom et prévient son parent.
@Component({
  selector: 'app-carte-membre',
  template: `
    <article>
      <h3>Prénom à afficher</h3>
      <button type="button">Voir la fiche</button>
    </article>
  `,
})
export class CarteMembreComponent {}
