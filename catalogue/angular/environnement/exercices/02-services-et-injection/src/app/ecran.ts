import { Component } from '@angular/core';
import { UiService } from './ui.service';

@Component({
  selector: 'app-ecran-membres',
  template: `<h2>Membres</h2>`,
})
export class EcranMembresComponent {
  // Écriture historique : le service est demandé par un paramètre du constructeur.
  constructor(private ui: UiService) {
    this.ui.setTitle('Membres');
  }
}
