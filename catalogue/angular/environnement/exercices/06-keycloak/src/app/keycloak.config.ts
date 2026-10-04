import { provideKeycloak } from 'keycloak-angular';
import { KeycloakConfig, KeycloakInitOptions } from 'keycloak-js';
import { environment } from './environment';

// À FAIRE (étape 5) : où est Keycloak ? Remplis `url`, `realm` et `clientId` avec les valeurs de `environment`.
export const configKeycloak: KeycloakConfig = {
  url: '',
  realm: '',
  clientId: '',
};

// À FAIRE (étape 5) : au démarrage, vérifie discrètement si la personne est déjà connectée (`check-sso`), avec le
// déroulé classique (`standard`) et la page `/assets/silent-check-sso.html` pour cette vérification en arrière-plan.
export const optionsInit: KeycloakInitOptions = {};

// Les « recettes » (providers) à ajouter à l'application.
export const fournisseursKeycloak = provideKeycloak({ config: configKeycloak, initOptions: optionsInit });
