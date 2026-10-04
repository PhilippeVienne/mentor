'use strict';
// Contrôle d'origine : exécute prix.js de l'apprenant·e et teste ses fonctions avec d'autres valeurs.
const assert = require('node:assert/strict');
const path = require('node:path');
const { charger } = require(path.join(process.env.MENTOR_OUTILS || '/opt/outils', 'charger.js'));

const { prixTotal, prixAvecRemise } = charger('prix.js', ['prixTotal', 'prixAvecRemise']);
let n = 0;
const proche = (a, b) => { assert.ok(Math.abs(a - b) < 1e-9, `${a} devrait valoir ${b}`); n++; };

assert.equal(typeof prixTotal, 'function'); n++;
assert.equal(typeof prixAvecRemise, 'function'); n++;
proche(prixTotal(10, 3), 30);
proche(prixAvecRemise(10, 5), 45);
proche(prixAvecRemise(10, 4), 40);
proche(prixAvecRemise(20, 6), 108);
console.log(`CONTROLES-REUSSIS ${n}`);
