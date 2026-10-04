import { HttpInterceptorFn } from '@angular/common/http';
import { inject } from '@angular/core';
import Keycloak from 'keycloak-js';

// Cet intercepteur reprend le comportement de celui du dépôt d'Adhésion, défauts compris :
// il écrit le jeton dans la console et l'ajoute à TOUTES les requêtes, quel que soit le serveur visé.
export const jetonInterceptor: HttpInterceptorFn = (req, next) => {
  const token = inject(Keycloak).token;
  console.log('Request at', req.url, 'with token', token);
  const requeteSignee = req.clone({ setHeaders: { Authorization: `Bearer ${token}` } });
  return next(requeteSignee);
};
