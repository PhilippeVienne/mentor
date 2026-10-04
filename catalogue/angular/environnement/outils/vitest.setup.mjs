// Prépare Angular pour les tests : Zone.js, puis l'environnement de test d'un navigateur.
import 'zone.js';
import 'zone.js/testing';
import { getTestBed } from '@angular/core/testing';
import { BrowserTestingModule, platformBrowserTesting } from '@angular/platform-browser/testing';

// Tous les fichiers de test tournent dans le même processus : on repart d'un environnement propre à chaque fois.
getTestBed().resetTestEnvironment();
getTestBed().initTestEnvironment(BrowserTestingModule, platformBrowserTesting());

// Après chaque test : on détruit les composants créés et on oublie la configuration du TestBed (Angular le fait
// tout seul avec Jasmine ; avec Vitest on le demande explicitement pour que les fichiers de test ne s'influencent pas).
afterEach(() => {
  getTestBed().resetTestingModule();
});
