import { Routes } from '@angular/router';
import { AccueilComponent, ListeMembresComponent } from './pages';

// À FAIRE (étape 4) : protège toutes les pages avec la garde `authGuard` (fichier `garde.ts`) en exigeant le rôle `staff`.
export const routes: Routes = [
  {
    path: '',
    children: [
      { path: '', component: AccueilComponent },
      { path: 'members', component: ListeMembresComponent },
    ],
  },
];
