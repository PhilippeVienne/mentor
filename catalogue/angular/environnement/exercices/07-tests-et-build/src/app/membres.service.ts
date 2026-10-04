import { HttpClient, HttpParams } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { Observable } from 'rxjs';

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

  rechercher(texte: string): Observable<Page<Membre>> {
    const params = new HttpParams({ fromObject: { search: texte } });
    return this.http.get<Page<Membre>>(`${this.urlApi}/membres/`, { params });
  }
}
