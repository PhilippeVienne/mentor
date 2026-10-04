import { TestBed } from '@angular/core/testing';
import { fournisseurs } from './fournisseurs';
import { Notifications, NotificationsSilencieuses } from './notifications';

describe('Notifications', () => {
  beforeEach(() => TestBed.configureTestingModule({ providers: fournisseurs }));

  it('donne la version silencieuse quand on demande Notifications', () => {
    expect(TestBed.inject(Notifications)).toBeInstanceOf(NotificationsSilencieuses);
  });

  it('n’écrit rien dans la console', () => {
    const espion = vi.spyOn(console, 'log').mockImplementation(() => undefined);
    TestBed.inject(Notifications).prevenir('Bonjour');
    expect(espion).not.toHaveBeenCalled();
    espion.mockRestore();
  });
});
