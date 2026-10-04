// Contrôle d'origine : importe prix.js de l'apprenant·e et teste ses exports avec d'autres valeurs.
import assert from 'node:assert/strict';

const module = await import('./prix.js');
let n = 0;
assert.equal(module.prixTotal(2, 3), 6); n++;
assert.equal(module.TVA, 0.2); n++;
assert.equal(typeof module.default, 'function'); n++;
assert.equal(module.default(13.5), '13.50 €'); n++;
console.log(`CONTROLES-REUSSIS ${n}`);
