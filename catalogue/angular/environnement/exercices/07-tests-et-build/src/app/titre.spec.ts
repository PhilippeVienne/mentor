import { ComponentFixture, TestBed } from '@angular/core/testing';
import { TitreComponent } from './titre';

describe('TitreComponent', () => {
  let fixture: ComponentFixture<TitreComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({ imports: [TitreComponent] }).compileComponents();
    fixture = TestBed.createComponent(TitreComponent);
    // À FAIRE (étape 2) : le gabarit n'est pas encore calculé ; demande à Angular de le faire ici.
  });

  it('affiche le titre', () => {
    const h1: HTMLElement = fixture.nativeElement.querySelector('h1');
    expect(h1.textContent).toBe('Mes adhérents');
  });
});
