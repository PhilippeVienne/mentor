import { authGuard } from './garde';
import { routes } from './routes';

describe('routes', () => {
  it('protège la route racine avec la garde', () => {
    expect(routes[0].canActivate).toEqual([authGuard]);
  });

  it('exige le rôle staff', () => {
    expect(routes[0].data).toEqual({ roles: ['staff'] });
  });

  it('garde les deux pages sous la route protégée', () => {
    expect(routes[0].children?.map((r) => r.path)).toEqual(['', 'members']);
  });
});
