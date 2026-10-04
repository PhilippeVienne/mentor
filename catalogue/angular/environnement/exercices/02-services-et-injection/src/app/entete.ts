import { Component } from '@angular/core';

@Component({
  selector: 'app-entete',
  template: `<p>Visites : {{ visites }}</p>`,
})
export class EnteteComponent {
  // À FAIRE (étape 2) : récupère le service CompteurVisites avec inject(), enregistre une visite à la création
  // du composant (dans le constructeur) et garde le nombre de visites dans `visites`.
  visites = 0;
}
