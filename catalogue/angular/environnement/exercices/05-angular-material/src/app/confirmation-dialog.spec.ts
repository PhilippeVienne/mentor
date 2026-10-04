import { ApplicationRef } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { MatDialog } from '@angular/material/dialog';
import { firstValueFrom } from 'rxjs';
import { ConfirmationDialogComponent } from './confirmation-dialog';

describe('ConfirmationDialogComponent', () => {
  const ouvrir = () => {
    const ref = TestBed.inject(MatDialog).open(ConfirmationDialogComponent, { data: { nom: 'Camille Durand' } });
    TestBed.inject(ApplicationRef).tick(); // Angular calcule le gabarit du dialogue
    return ref;
  };
  const bouton = (libelle: string) =>
    Array.from(document.querySelectorAll('button')).find((b) => b.textContent?.trim() === libelle);

  afterEach(() => TestBed.inject(MatDialog).closeAll());

  it('affiche le nom reçu dans le titre', () => {
    ouvrir();
    expect(document.querySelector('h2')?.textContent).toBe('Supprimer Camille Durand ?');
  });

  it('répond true avec « Supprimer »', async () => {
    const reponse = firstValueFrom(ouvrir().afterClosed());
    bouton('Supprimer')?.click();
    expect(await reponse).toBe(true);
  });

  it('répond false avec « Annuler »', async () => {
    const reponse = firstValueFrom(ouvrir().afterClosed());
    bouton('Annuler')?.click();
    expect(await reponse).toBe(false);
  });
});
