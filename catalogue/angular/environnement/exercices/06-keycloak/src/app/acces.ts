// Décide si la personne peut voir une page. `rolesExiges` vient de la route (`data: { roles: ['staff'] }`) : on ne sait
// pas ce qu'il contient, d'où le type `unknown`. `rolesAccordes` est la liste des rôles que Keycloak lui a donnés.
export function accesAutorise(rolesExiges: unknown, rolesAccordes: string[]): boolean {
  // À FAIRE (étape 3) : sans rôle exigé, l'accès est libre ; sinon la personne doit avoir TOUS les rôles exigés.
  return true;
}
