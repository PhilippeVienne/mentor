import { TestBed } from '@angular/core/testing';
import { CompteurVisites } from './compteur-visites.service';
import { EnteteComponent } from './entete';

describe('EnteteComponent', () => {
  const texte = (f: { nativeElement: unknown }): string => (f.nativeElement as HTMLElement).textContent?.trim() ?? '';

  it('enregistre une visite à sa création et l’affiche', () => {
    const fixture = TestBed.createComponent(EnteteComponent);
    fixture.detectChanges();
    expect(texte(fixture)).toBe('Visites : 1');
    expect(TestBed.inject(CompteurVisites).visites).toBe(1);
  });

  it('partage le compteur entre deux composants', () => {
    TestBed.createComponent(EnteteComponent);
    const second = TestBed.createComponent(EnteteComponent);
    second.detectChanges();
    expect(texte(second)).toBe('Visites : 2');
  });
});
