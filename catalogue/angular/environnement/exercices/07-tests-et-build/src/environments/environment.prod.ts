// Valeurs de production : des MARQUEURS, remplacés au démarrage du conteneur par start.sh. Jamais de secret ici.
export const environment = {
  adhesion_api_url: '__ADHESION_API_URL__',
  keycloak_url: '__KEYCLOAK_URL__',
  keycloak_realm: '__KEYCLOAK_REALM__',
  keycloak_client_id: '__KEYCLOAK_CLIENT_ID__',
};
