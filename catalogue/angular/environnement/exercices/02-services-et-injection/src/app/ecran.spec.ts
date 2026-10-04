import { TestBed } from '@angular/core/testing';
import { EcranMembresComponent } from './ecran';
import { UiService } from './ui.service';

describe('EcranMembresComponent', () => {
  it('envoie le titre de la page au service', () => {
    const titres: string[] = [];
    TestBed.inject(UiService).getTitle().subscribe((titre) => titres.push(titre));

    TestBed.createComponent(EcranMembresComponent);

    expect(titres).toEqual(['Membres']);
  });
});
