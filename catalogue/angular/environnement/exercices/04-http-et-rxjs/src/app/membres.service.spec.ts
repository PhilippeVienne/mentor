import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { MembresService } from './membres.service';

describe('MembresService', () => {
  let service: MembresService;
  let http: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({ providers: [provideHttpClient(), provideHttpClientTesting()] });
    service = TestBed.inject(MembresService);
    http = TestBed.inject(HttpTestingController);
  });

  afterEach(() => http.verify());

  it('rechercher envoie un GET avec le terme et renvoie la page reçue', () => {
    let recu: unknown;
    service.rechercher('camille').subscribe((page) => (recu = page));

    const requete = http.expectOne('https://example.org/api/membres/?search=camille');
    expect(requete.request.method).toBe('GET');
    requete.flush({ count: 1, results: [{ id: 1, prenom: 'Camille', nom: 'Durand' }] });

    expect(recu).toEqual({ count: 1, results: [{ id: 1, prenom: 'Camille', nom: 'Durand' }] });
  });

  it('creer envoie un POST avec le membre dans le corps', () => {
    let recu: unknown;
    service.creer({ prenom: 'Sam', nom: 'Martin' }).subscribe((membre) => (recu = membre));

    const requete = http.expectOne('https://example.org/api/membres/');
    expect(requete.request.method).toBe('POST');
    expect(requete.request.body).toEqual({ prenom: 'Sam', nom: 'Martin' });
    requete.flush({ id: 7, prenom: 'Sam', nom: 'Martin' });

    expect(recu).toEqual({ id: 7, prenom: 'Sam', nom: 'Martin' });
  });
});
