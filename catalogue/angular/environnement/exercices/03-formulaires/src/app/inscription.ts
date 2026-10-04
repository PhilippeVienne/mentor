import { Component, inject } from '@angular/core';
import { FormGroup, ReactiveFormsModule } from '@angular/forms';
import { MembresService } from './membres.service';

@Component({
  selector: 'app-inscription',
  imports: [ReactiveFormsModule],
  templateUrl: './inscription.html',
})
export class InscriptionComponent {
  private readonly membres = inject(MembresService);

  // Passe à vrai quand l'API a accepté l'inscription : le gabarit affiche alors une confirmation.
  reussi = false;

  // À FAIRE (étape 1) : décris les trois champs `prenom`, `email` et `telephone` (voir la leçon).
  form = new FormGroup({});

  enregistrer() {
    // À FAIRE (étapes 4 et 5) : refuse un formulaire invalide, sinon appelle `this.membres.creer(…)`
    // et reporte les erreurs de l'API sur les champs.
  }
}
