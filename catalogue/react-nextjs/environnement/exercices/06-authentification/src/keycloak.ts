// Le jeton remis par Keycloak accompagne chaque appel à l'API d'Adhésion, dans l'en-tête Authorization.
// À écrire (étape 5) : entetesAuthorization(token) renvoie { Authorization: "Bearer <jeton>" },
// ou un objet vide quand il n'y a pas (encore) de jeton.
export function entetesAuthorization(token: string | null): Record<string, string> {
  return {};
}
