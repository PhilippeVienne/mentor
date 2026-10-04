import { Injectable } from '@angular/core';
import { Observable, of } from 'rxjs';

export interface NouveauMembre {
  prenom: string;
  email: string;
  telephone: string;
}

export interface Membre extends NouveauMembre {
  id: number;
}

// Version SIMULÉE du service (la vraie appelle l'API avec HttpClient : leçon 4). Ici elle répond toujours « bien ».
@Injectable({ providedIn: 'root' })
export class MembresService {
  creer(membre: NouveauMembre): Observable<Membre> {
    return of({ id: 1, ...membre });
  }
}
