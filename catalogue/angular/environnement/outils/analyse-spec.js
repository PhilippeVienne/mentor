// Usage : node analyse-spec.js fichier.spec.ts
// Contrôle statique d'un test écrit à la main : de vraies vérifications, aucune désactivation.
const fs = require("node:fs");
const { sansCommentaires } = require("./sans-commentaires.js");

const fichier = process.argv[2];
const brut = fs.readFileSync(fichier, "utf8");
const code = sansCommentaires(brut, fichier);
const problemes = [];
if (/@ts-(nocheck|ignore|expect-error)/.test(brut)) problemes.push("@ts-nocheck / @ts-ignore / @ts-expect-error masquent les erreurs de TypeScript : retire-les.");
if (/\b(?:xit|xtest|xdescribe|fit|fdescribe)\s*\(|\b(?:it|test|describe)\.(?:skip|only|todo|skipIf|runIf|fails)\b/.test(code)) {
  problemes.push("Un test est désactivé ou isolé (skip, only, todo, xit...).");
}
const tests = (code.match(/\b(?:it|test)\s*\(/g) ?? []).length;
const attentes = (code.match(/\bexpect\s*\([^;]*?\)\s*\.\s*(?:not\s*\.\s*)?(?:to[A-Za-z]+|resolves|rejects)\b/g) ?? []).length;
if (tests < 2) problemes.push(`Il faut au moins 2 tests it(...) : il y en a ${tests}.`);
if (attentes < 2) problemes.push(`Il faut au moins 2 vérifications expect(...).toBe(...) (ou toEqual…) : il y en a ${attentes}.`);
if (/\.(?:toBeDefined|toBeTruthy)\s*\(\s*\)|expect\.anything|toBeTypeOf|expect\s*\(\s*(?:true|1|"[^"]*"|'[^']*')\s*\)/.test(code)) {
  problemes.push("Une vérification est trop faible (toBeDefined, toBeTruthy, expect(true)…) : compare avec une valeur précise.");
}
if (problemes.length) {
  console.log("Ton test n'est pas valable :\n- " + problemes.join("\n- "));
  process.exit(1);
}
