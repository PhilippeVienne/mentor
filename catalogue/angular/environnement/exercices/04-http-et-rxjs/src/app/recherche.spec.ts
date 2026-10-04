import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { RechercheComponent } from './recherche';

describe('RechercheComponent', () => {
  let fixture: ComponentFixture<RechercheComponent>;
  let composant: RechercheComponent;
  let http: HttpTestingController;

  beforeEach(() => {
    vi.useFakeTimers();
    TestBed.configureTestingModule({ providers: [provideHttpClient(), provideHttpClientTesting()] });
    http = TestBed.inject(HttpTestingController);
    fixture = TestBed.createComponent(RechercheComponent);
    composant = fixture.componentInstance;
    fixture.detectChanges();
  });

  afterEach(() => vi.useRealTimers());

  describe('recherche réactive', () => {
    // Le test s'abonne lui-même au flux ; si le gabarit s'y abonne aussi (tube async), chaque requête part deux fois :
    // on compare donc des adresses distinctes.
    const adresses = () => [...new Set(http.match(() => true).map((r) => r.request.urlWithParams))];

    it('n’envoie qu’une requête pour une rafale de frappes, avec le dernier terme', () => {
      composant.resultats$.subscribe();
      composant.terme$.next('c');
      composant.terme$.next('ca');
      composant.terme$.next('cam');
      vi.advanceTimersByTime(300);

      expect(adresses()).toEqual(['https://example.org/api/membres/?search=cam']);
    });

    it('ignore un terme identique au précédent', () => {
      composant.resultats$.subscribe();
      composant.terme$.next('cam');
      vi.advanceTimersByTime(300);
      expect(adresses()).toEqual(['https://example.org/api/membres/?search=cam']);

      composant.terme$.next('cam');
      vi.advanceTimersByTime(300);
      expect(adresses()).toEqual([]);
    });
  });

  describe('affichage', () => {
    it('affiche le nombre de résultats et un élément par membre', () => {
      const saisie = (fixture.nativeElement as HTMLElement).querySelector<HTMLInputElement>('input')!;
      saisie.value = 'cam';
      saisie.dispatchEvent(new Event('input'));
      vi.advanceTimersByTime(300);
      http.expectOne('https://example.org/api/membres/?search=cam').flush({
        count: 2,
        results: [
          { id: 1, prenom: 'Camille', nom: 'Durand' },
          { id: 2, prenom: 'Camélia', nom: 'Petit' },
        ],
      });
      fixture.detectChanges();

      const page = fixture.nativeElement as HTMLElement;
      expect(page.querySelector('p')?.textContent).toBe('2 résultat(s)');
      expect(page.querySelectorAll('li').length).toBe(2);
      expect(page.querySelector('li')?.textContent).toContain('Camille Durand');
    });
  });

  describe('fin des abonnements', () => {
    it('coupe l’abonnement à la destruction du composant', () => {
      expect(composant.terme$.observed).toBe(true);
      fixture.destroy();
      expect(composant.terme$.observed).toBe(false);
    });
  });
});
