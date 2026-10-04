'use strict';
// Aide pour les tests des exercices : exécute le script d'une personne dans un « bac à sable » et récupère les
// valeurs dont le test a besoin (fonctions, variables), SANS exiger de `module.exports` ni de `export`.
// Ce qu'a affiché le script avec console.log est renvoyé dans `sortie`.
const fs = require('node:fs');
const vm = require('node:vm');

function charger(fichier, noms) {
  const code = fs.readFileSync(fichier, 'utf8');
  const sortie = [];
  const bac = { console: { log: (...a) => sortie.push(a.join(' ')), error() {}, warn() {} } };
  vm.createContext(bac);
  const lecture = `;({ ${noms.map((n) => `${n}: typeof ${n} === 'undefined' ? undefined : ${n}`).join(', ')} })`;
  const valeurs = vm.runInContext(code + '\n' + lecture, bac, { filename: fichier, timeout: 2000 });
  return { ...valeurs, sortie };
}

// Les tableaux et objets créés dans le bac à sable ne sont pas « du même monde » que ceux du test : on les copie.
const copie = (valeur) => JSON.parse(JSON.stringify(valeur));

module.exports = { charger, copie };
