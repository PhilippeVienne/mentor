import { ComponentFixture, TestBed } from '@angular/core/testing';
import { throwError } from 'rxjs';
import { InscriptionComponent } from './inscription';
import { MembresService } from './membres.service';

describe('InscriptionComponent', () => {
  let fixture: ComponentFixture<InscriptionComponent>;
  let composant: InscriptionComponent;
  let page: HTMLElement;

  const saisir = (nom: string, valeur: string) => {
    const champ = page.querySelector<HTMLInputElement>(`input[formControlName="${nom}"]`)!;
    champ.value = valeur;
    champ.dispatchEvent(new Event('input'));
    fixture.detectChanges();
  };
  const quitter = (nom: string) => {
    page.querySelector<HTMLInputElement>(`input[formControlName="${nom}"]`)!.dispatchEvent(new Event('blur'));
    fixture.detectChanges();
  };
  const erreurs = () => Array.from(page.querySelectorAll('.erreur')).map((p) => p.textContent?.trim());

  beforeEach(() => {
    fixture = TestBed.createComponent(InscriptionComponent);
    composant = fixture.componentInstance;
    page = fixture.nativeElement as HTMLElement;
    fixture.detectChanges();
  });

  describe('validation des champs', () => {
    it('refuse un formulaire vide', () => {
      expect(composant.form.invalid).toBe(true);
      expect(composant.form.controls.prenom.hasError('required')).toBe(true);
    });

    it('refuse une adresse e-mail mal formée', () => {
      composant.form.controls.email.setValue('pas-un-mail');
      expect(composant.form.controls.email.hasError('email')).toBe(true);
    });

    it('accepte un téléphone qui commence par +, et refuse les autres', () => {
      composant.form.controls.telephone.setValue('+33612345678');
      expect(composant.form.controls.telephone.valid).toBe(true);
      composant.form.controls.telephone.setValue('0612345678');
      expect(composant.form.controls.telephone.valid).toBe(false);
    });

    it('donne des valeurs typées, jamais null, même après reset()', () => {
      composant.form.controls.prenom.setValue('Camille');
      composant.form.reset();
      const valeurs: { prenom: string; email: string; telephone: string } = composant.form.getRawValue();
      expect(valeurs.prenom).toBe('');
    });
  });

  describe('liaison du gabarit', () => {
    it('affiche un champ de saisie par contrôle', () => {
      expect(page.querySelectorAll('input[formControlName]').length).toBe(3);
    });

    it('recopie la saisie dans le formulaire', () => {
      saisir('prenom', 'Camille');
      expect(composant.form.controls.prenom.value).toBe('Camille');
    });

    it('grise le bouton tant que le formulaire est invalide', () => {
      expect(page.querySelector<HTMLButtonElement>('button[type="submit"]')?.disabled).toBe(true);
    });
  });

  describe('messages d’erreur', () => {
    it('n’affiche aucune erreur avant que la personne ait quitté le champ', () => {
      expect(erreurs()).toEqual([]);
    });

    it('affiche l’erreur du prénom après être passé sur le champ', () => {
      quitter('prenom');
      expect(erreurs()).toEqual(['Le prénom est obligatoire.']);
    });

    it('affiche l’erreur de l’e-mail seulement si elle est invalide et touchée', () => {
      saisir('email', 'pas-un-mail');
      quitter('email');
      expect(erreurs()).toContain('Saisis une adresse e-mail valide.');
    });
  });

  describe('envoi', () => {
    const remplir = () => {
      composant.form.setValue({ prenom: 'Camille', email: 'camille@example.org', telephone: '+33612345678' });
    };

    it('n’appelle pas l’API quand le formulaire est invalide', () => {
      const espion = vi.spyOn(TestBed.inject(MembresService), 'creer');
      composant.enregistrer();
      expect(espion).not.toHaveBeenCalled();
    });

    it('envoie les valeurs saisies quand le formulaire est valide', () => {
      const espion = vi.spyOn(TestBed.inject(MembresService), 'creer');
      remplir();
      composant.enregistrer();
      expect(espion).toHaveBeenCalledWith({ prenom: 'Camille', email: 'camille@example.org', telephone: '+33612345678' });
      expect(composant.reussi).toBe(true);
    });
  });

  describe('erreurs renvoyées par l’API', () => {
    it('reporte l’erreur du serveur sur le champ concerné et l’affiche', () => {
      vi.spyOn(TestBed.inject(MembresService), 'creer').mockReturnValue(
        throwError(() => ({ error: { email: ['Cette adresse existe déjà.'] } })),
      );
      composant.form.setValue({ prenom: 'Camille', email: 'camille@example.org', telephone: '+33612345678' });
      composant.enregistrer();
      fixture.detectChanges();

      expect(composant.form.controls.email.hasError('serveur')).toBe(true);
      expect(composant.reussi).toBe(false);
      expect(erreurs()).toContain('Cette adresse existe déjà.');
    });
  });
});
