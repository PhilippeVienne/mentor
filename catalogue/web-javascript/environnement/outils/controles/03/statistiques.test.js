'use strict';
// Tests d'origine de l'exercice (copie non modifiable par l'apprenant·e), lancés par `verifier-web 03 stats`.
const test = require('node:test');
const assert = require('node:assert/strict');
const path = require('node:path');
const { charger, copie } = require(path.join(process.env.MENTOR_OUTILS || '/opt/outils', 'charger.js'));

const { moyenne, titresDisponibles } = charger('statistiques.js', ['moyenne', 'titresDisponibles']);

test('moyenne de plusieurs nombres', () => {
  assert.equal(typeof moyenne, 'function', 'moyenne doit être une fonction');
  assert.equal(moyenne([2, 4, 9]), 5);
  assert.equal(moyenne([10]), 10);
});

test('moyenne avec des décimales et des négatifs', () => {
  assert.equal(moyenne([1, 2]), 1.5);
  assert.equal(moyenne([-2, 2]), 0);
});

test('moyenne d\'un tableau vide', () => {
  assert.equal(moyenne([]), 0);
});

test('titresDisponibles garde seulement les événements avec des places', () => {
  assert.equal(typeof titresDisponibles, 'function', 'titresDisponibles doit être une fonction');
  const donnees = [
    { titre: 'Sortie photo', places: 20 },
    { titre: 'Atelier retouche', places: 0 },
    { titre: 'Exposition', places: 50 }
  ];
  assert.deepEqual(copie(titresDisponibles(donnees)), ['Sortie photo', 'Exposition']);
  assert.deepEqual(copie(titresDisponibles([])), []);
});

test('titresDisponibles avec d\'autres données', () => {
  assert.deepEqual(copie(titresDisponibles([{ titre: 'A', places: 0 }, { titre: 'B', places: 1 }])), ['B']);
  assert.deepEqual(copie(titresDisponibles([{ titre: 'A', places: 0 }])), []);
});

test('titresDisponibles ne modifie pas le tableau reçu', () => {
  const donnees = [{ titre: 'A', places: 3 }, { titre: 'B', places: 0 }];
  titresDisponibles(donnees);
  assert.equal(donnees.length, 2);
});
