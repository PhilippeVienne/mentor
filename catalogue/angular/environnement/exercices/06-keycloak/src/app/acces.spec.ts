import { accesAutorise } from './acces';

describe('accesAutorise', () => {
  it('laisse passer quand la route n’exige aucun rôle', () => {
    expect(accesAutorise(undefined, [])).toBe(true);
    expect(accesAutorise([], [])).toBe(true);
  });

  it('laisse passer quand la personne a le rôle exigé', () => {
    expect(accesAutorise(['staff'], ['staff', 'autre'])).toBe(true);
  });

  it('refuse quand le rôle exigé manque', () => {
    expect(accesAutorise(['staff'], [])).toBe(false);
    expect(accesAutorise(['staff'], ['lecteur'])).toBe(false);
  });

  it('exige TOUS les rôles demandés', () => {
    expect(accesAutorise(['staff', 'admin'], ['staff'])).toBe(false);
    expect(accesAutorise(['staff', 'admin'], ['admin', 'staff'])).toBe(true);
  });
});
