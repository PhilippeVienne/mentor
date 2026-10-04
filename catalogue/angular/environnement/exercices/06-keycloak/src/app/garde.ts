import { inject } from '@angular/core';
import { ActivatedRouteSnapshot, CanActivateFn, RouterStateSnapshot, UrlTree } from '@angular/router';
import { AuthGuardData, createAuthGuard } from 'keycloak-angular';
import Keycloak from 'keycloak-js';
import { accesAutorise } from './acces';

// La garde d'Adhésion, simplifiée (fichier complet : tu n'as rien à y changer).
const verifier = async (
  route: ActivatedRouteSnapshot,
  state: RouterStateSnapshot,
  authData: AuthGuardData,
): Promise<boolean | UrlTree> => {
  if (!authData.authenticated) {
    // Pas connecté·e : on envoie la personne sur la page de connexion de Keycloak, puis elle revient ici.
    await inject(Keycloak).login({ redirectUri: window.location.origin + state.url });
  }
  const roles = authData.grantedRoles.resourceRoles['adhesion-api'] ?? [];
  return accesAutorise(route.data['roles'], roles);
};

export const authGuard = createAuthGuard<CanActivateFn>(verifier);
