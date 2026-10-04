import { TestBed } from '@angular/core/testing';
import { CarteBienvenueComponent } from './carte-bienvenue';

describe('CarteBienvenueComponent', () => {
  it('affiche une vraie carte Material avec un bouton Material', () => {
    const fixture = TestBed.createComponent(CarteBienvenueComponent);
    fixture.detectChanges();
    const page = fixture.nativeElement as HTMLElement;

    // Quand le module Material est importé, Material ajoute ses propres classes CSS aux éléments.
    expect(page.querySelector('mat-card')?.classList).toContain('mat-mdc-card');
    expect(page.querySelector('mat-card-title')?.textContent).toBe('Bienvenue');
    expect(page.querySelector('button')?.classList).toContain('mat-mdc-unelevated-button');
  });
});
