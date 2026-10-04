import { Component } from '@angular/core';
import { Membre } from './membre';

@Component({
  selector: 'app-liste-membres',
  // À FAIRE (étape 2) : le tube `uppercase` doit être importé ici pour être utilisable dans le gabarit.
  imports: [],
  templateUrl: './liste-membres.html',
})
export class ListeMembresComponent {
  membres: Membre[] = [
    { id: 1, prenom: 'Camille', nom: 'Durand', cotisationPayee: true },
    { id: 2, prenom: 'Sam', nom: 'Martin', cotisationPayee: false },
  ];

  ajouter() {
    const id = this.membres.length + 1;
    this.membres = [...this.membres, { id, prenom: 'Nouveau', nom: 'Membre', cotisationPayee: false }];
  }
}
