import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { MembresService } from './membres.service';

describe('MembresService', () => {
  it('appelle la recherche avec le terme', () => {
    TestBed.configureTestingModule({
      providers: [provideHttpClient(), provideHttpClientTesting()],
    });
    const service = TestBed.inject(MembresService);
    const http = TestBed.inject(HttpTestingController);

    let recu: unknown;
    service.rechercher('camille').subscribe((page) => (recu = page));

    const requete = http.expectOne('https://example.org/api/membres/?search=camille');
    expect(requete.request.method).toBe('GET');
    // À FAIRE (étape 3) : simule la réponse du serveur avec `requete.flush(…)` (une page vide : 0 résultat).

    expect(recu).toEqual({ count: 0, results: [] });
    http.verify(); // aucune requête imprévue
  });
});
