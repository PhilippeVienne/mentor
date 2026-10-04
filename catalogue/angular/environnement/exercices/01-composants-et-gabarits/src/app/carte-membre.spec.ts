import { TestBed } from '@angular/core/testing';
import { CarteMembreComponent } from './carte-membre';

describe('CarteMembreComponent', () => {
  it('affiche le prénom reçu du parent', () => {
    const fixture = TestBed.createComponent(CarteMembreComponent);
    fixture.componentRef.setInput('prenom', 'Camille');
    fixture.detectChanges();
    expect((fixture.nativeElement as HTMLElement).querySelector('h3')?.textContent).toBe('Camille');
  });

  it('émet le prénom quand on clique sur « Voir la fiche »', () => {
    const fixture = TestBed.createComponent(CarteMembreComponent);
    fixture.componentRef.setInput('prenom', 'Sam');
    fixture.detectChanges();

    const recus: string[] = [];
    fixture.componentInstance.selectionne.subscribe((prenom: string) => recus.push(prenom));
    (fixture.nativeElement as HTMLElement).querySelector('button')?.click();

    expect(recus).toEqual(['Sam']);
  });
});
