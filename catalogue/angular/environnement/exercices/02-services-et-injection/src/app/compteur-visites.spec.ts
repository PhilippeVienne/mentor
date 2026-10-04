import { TestBed } from '@angular/core/testing';
import { CompteurVisites } from './compteur-visites.service';

describe('CompteurVisites', () => {
  it('compte les visites', () => {
    const compteur = TestBed.inject(CompteurVisites);
    expect(compteur.enregistrer()).toBe(1);
    expect(compteur.enregistrer()).toBe(2);
  });

  it('est partagé : deux demandes donnent la même instance', () => {
    expect(TestBed.inject(CompteurVisites)).toBe(TestBed.inject(CompteurVisites));
  });
});
