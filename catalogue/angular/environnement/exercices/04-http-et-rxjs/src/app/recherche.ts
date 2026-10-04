import { Component, inject } from '@angular/core';
import { Observable, Subject, switchMap } from 'rxjs';
import { Membre, MembresService, Page } from './membres.service';

@Component({
  selector: 'app-recherche',
  imports: [],
  // À FAIRE (étape 4) : affiche le nombre de résultats et la liste avec le tube `async` (voir la leçon).
  template: `<input #saisie (input)="terme$.next(saisie.value)" type="search" placeholder="Rechercher un membre" />`,
})
export class RechercheComponent {
  private readonly membres = inject(MembresService);

  // Chaque frappe du clavier est poussée dans ce `Subject`.
  readonly terme$ = new Subject<string>();

  // À FAIRE (étape 3) : attends 300 ms de silence, ignore un terme identique au précédent, puis lance la recherche.
  readonly resultats$: Observable<Page<Membre>> = this.terme$.pipe(
    switchMap((terme) => this.membres.rechercher(terme)),
  );

  dernierTerme = '';

  constructor() {
    // À FAIRE (étape 5) : cet abonnement n'est jamais coupé. Termine-le à la destruction du composant.
    this.terme$.subscribe((terme) => (this.dernierTerme = terme));
  }
}
