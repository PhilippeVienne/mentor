import { TestBed } from '@angular/core/testing';
import { MatSnackBar } from '@angular/material/snack-bar';
import { Notifier } from './notifier';

describe('Notifier', () => {
  it('affiche un bandeau temporaire de 2500 ms avec le bouton OK', () => {
    const ouvrir = vi.spyOn(TestBed.inject(MatSnackBar), 'open');
    TestBed.inject(Notifier).adherentEfface();
    expect(ouvrir).toHaveBeenCalledWith('Adhérent effacé !', 'OK', { duration: 2500 });
  });
});
