// Tests de l'exercice : lance-les avec « node --test statistiques.test.js ».
// Ils exécutent TON fichier statistiques.js et regardent ce que renvoient tes fonctions.
const test = require('node:test');
const assert = require('node:assert/strict');
const { charger, copie } = require('/opt/outils/charger.js');

const { moyenne, titresDisponibles } = charger('statistiques.js', ['moyenne', 'titresDisponibles']);

test('moyenne de plusieurs nombres', () => {
  assert.equal(typeof moyenne, 'function', 'moyenne doit être une fonction');
  assert.equal(moyenne([2, 4, 9]), 5);
  assert.equal(moyenne([10]), 10);
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
