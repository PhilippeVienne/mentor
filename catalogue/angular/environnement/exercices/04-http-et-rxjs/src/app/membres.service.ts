import { HttpClient, HttpParams } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { Observable, of } from 'rxjs';

export interface Membre {
  id: number;
  prenom: string;
  nom: string;
}

export interface Page<T> {
  count: number;
  results: T[];
}

@Injectable({ providedIn: 'root' })
export class MembresService {
  private readonly http = inject(HttpClient);
  private readonly urlApi = 'https://example.org/api';

  // À FAIRE (étape 1) : appelle `GET {urlApi}/membres/?search=texte` avec HttpParams et renvoie la réponse typée.
  rechercher(texte: string): Observable<Page<Membre>> {
    return of({ count: 0, results: [] });
  }

  // À FAIRE (étape 2) : appelle `POST {urlApi}/membres/` avec le membre dans le corps de la requête.
  creer(membre: Omit<Membre, 'id'>): Observable<Membre> {
    return of({ id: 0, ...membre });
  }
}
