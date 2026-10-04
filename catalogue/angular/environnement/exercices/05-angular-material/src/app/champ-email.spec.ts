import { TestBed } from '@angular/core/testing';
import { ChampEmailComponent } from './champ-email';

describe('ChampEmailComponent', () => {
  it('affiche un champ Material avec son étiquette et son aide', () => {
    const fixture = TestBed.createComponent(ChampEmailComponent);
    fixture.detectChanges();
    const page = fixture.nativeElement as HTMLElement;

    expect(page.querySelector('mat-form-field')).not.toBeNull();
    expect(page.querySelector('mat-label')?.textContent?.trim()).toBe('Adresse e-mail');
    expect(page.querySelector('mat-hint')?.textContent?.trim()).toBe("Utilise ton adresse de l'école");
    expect(page.querySelector('input')?.classList).toContain('mat-mdc-input-element');
  });

  it('affiche l’erreur seulement quand le champ est invalide et touché', () => {
    const fixture = TestBed.createComponent(ChampEmailComponent);
    fixture.detectChanges();
    const page = fixture.nativeElement as HTMLElement;
    expect(page.querySelector('mat-error')).toBeNull();

    fixture.componentInstance.email.setValue('pas-un-mail');
    fixture.componentInstance.email.markAsTouched();
    fixture.detectChanges();
    expect(page.querySelector('mat-error')?.textContent?.trim()).toBe('Adresse invalide');
  });
});
