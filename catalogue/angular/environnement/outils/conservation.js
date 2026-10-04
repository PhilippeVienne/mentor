// Usage : node conservation.js original.ts modifie.ts
// Un fichier de test « à compléter » doit garder toutes les lignes de code d'origine (commentaires exceptés), dans l'ordre :
// on a le droit d'y AJOUTER du code, pas d'en retirer, ni de désactiver un test (skip, only, todo...).
const fs = require("node:fs");
const { sansCommentaires } = require("./sans-commentaires.js");

const [, , original, modifie] = process.argv;
const lignes = (chemin) =>
  sansCommentaires(fs.readFileSync(chemin, "utf8"), chemin)
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l !== "");
const attendues = lignes(original);
const actuelles = lignes(modifie);

let i = 0;
for (const ligne of actuelles) {
  if (i < attendues.length && ligne === attendues[i]) i++;
}
if (i < attendues.length) {
  console.log(`Le fichier ${modifie.replace(/^.*?\/(src\/)/, "$1")} ne doit rien perdre de ce qui y était déjà écrit (hors commentaires).`);
  console.log(`Ligne manquante ou modifiée : ${attendues[i]}`);
  process.exit(1);
}
const interdit = /\b(?:xit|xdescribe|fit|fdescribe)\s*\(|\.(?:skip|only|todo|skipIf|runIf)\b|\b(?:ts-nocheck)\b/;
const fautif = actuelles.find((l) => interdit.test(l));
if (fautif) {
  console.log(`Ce test ne doit pas être désactivé ni isolé : ${fautif}`);
  process.exit(1);
}
