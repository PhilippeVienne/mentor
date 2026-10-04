import { environment } from './environment';
import { configKeycloak, fournisseursKeycloak, optionsInit } from './keycloak.config';

describe('configuration de Keycloak', () => {
  it('pointe vers le serveur, le realm et le client de l’environnement', () => {
    expect(configKeycloak).toEqual({
      url: environment.keycloak_url,
      realm: environment.keycloak_realm,
      clientId: environment.keycloak_client_id,
    });
  });

  it('vérifie discrètement la connexion au démarrage', () => {
    expect(optionsInit.onLoad).toBe('check-sso');
    expect(optionsInit.flow).toBe('standard');
    expect(optionsInit.silentCheckSsoRedirectUri).toMatch(/\/assets\/silent-check-sso\.html$/);
  });

  it('fabrique les fournisseurs Angular', () => {
    expect(fournisseursKeycloak).toBeDefined();
  });
});
