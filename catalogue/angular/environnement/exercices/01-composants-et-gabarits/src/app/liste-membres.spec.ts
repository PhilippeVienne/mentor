import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ListeMembresComponent } from './liste-membres';

// Ces tests décrivent ce que l'écran doit afficher. Ne les modifie pas : fais-les passer en écrivant le gabarit.
describe('ListeMembresComponent', () => {
  let fixture: ComponentFixture<ListeMembresComponent>;

  const texte = (selecteur: string): string =>
    (fixture.nativeElement as HTMLElement).querySelector(selecteur)?.textContent?.trim() ?? '';
  const elements = (selecteur: string): number =>
    (fixture.nativeElement as HTMLElement).querySelectorAll(selecteur).length;

  beforeEach(() => {
    fixture = TestBed.createComponent(ListeMembresComponent);
    fixture.detectChanges();
  });

  it('affiche le nombre de membres dans le titre', () => {
    expect(texte('h2')).toBe('Membres (2)');
  });

  it('affiche un élément de liste par membre, avec le nom en majuscules', () => {
    expect(elements('li')).toBe(2);
    expect(texte('li')).toContain('Camille DURAND');
  });

  it('signale seulement la cotisation à régler', () => {
    const lignes = Array.from((fixture.nativeElement as HTMLElement).querySelectorAll('li'));
    expect(lignes[0].textContent).not.toContain('cotisation à régler');
    expect(lignes[1].textContent).toContain('cotisation à régler');
  });

  it('affiche un message quand la liste est vide', () => {
    fixture.componentInstance.membres = [];
    fixture.detectChanges();
    expect(elements('li')).toBe(0);
    expect(texte('p')).toBe("Aucun membre pour l'instant.");
  });

  it('ajoute un membre au clic sur le bouton', () => {
    (fixture.nativeElement as HTMLElement).querySelector('button')?.click();
    fixture.detectChanges();
    expect(elements('li')).toBe(3);
  });
});
