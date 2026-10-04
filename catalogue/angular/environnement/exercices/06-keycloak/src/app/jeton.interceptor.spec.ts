import { provideHttpClient, withInterceptors, HttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import Keycloak from 'keycloak-js';
import { jetonInterceptor } from './jeton.interceptor';

describe('jetonInterceptor', () => {
  let http: HttpClient;
  let faux: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        provideHttpClient(withInterceptors([jetonInterceptor])),
        provideHttpClientTesting(),
        // Un faux Keycloak : aucune connexion réelle, juste un jeton de test.
        { provide: Keycloak, useValue: { token: 'jeton-de-test' } },
      ],
    });
    http = TestBed.inject(HttpClient);
    faux = TestBed.inject(HttpTestingController);
  });

  describe('journaux', () => {
    it('signe les requêtes vers l’API sans écrire le jeton dans la console', () => {
      const console_ = vi.spyOn(console, 'log').mockImplementation(() => undefined);
      http.get('https://example.org/api/membres/').subscribe();

      const requete = faux.expectOne('https://example.org/api/membres/');
      expect(requete.request.headers.get('Authorization')).toBe('Bearer jeton-de-test');
      expect(console_).not.toHaveBeenCalled();
      console_.mockRestore();
    });
  });

  describe('destination', () => {
    it('ne montre pas le jeton à un autre serveur', () => {
      vi.spyOn(console, 'log').mockImplementation(() => undefined);
      http.get('https://autre.example.net/stats').subscribe();

      const requete = faux.expectOne('https://autre.example.net/stats');
      expect(requete.request.headers.has('Authorization')).toBe(false);
    });
  });
});
