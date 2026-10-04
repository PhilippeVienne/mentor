// Usage : node verdict.js resultat.json (tout|filtre) originaux-séparés-par-des-virgules [filtre...]
// Contrôle le compte rendu JSON de Vitest : aucun échec, au moins un test réellement réussi, et (sans filtre de titre) aucun
// test ignoré. Chaque fichier de test d'origine concerné doit avoir été joué et avoir réussi au moins un test.
const fs = require("node:fs");

const [, , json, portee, liste, ...filtres] = process.argv;
const rapport = JSON.parse(fs.readFileSync(json, "utf8"));
const originaux = liste ? liste.split(",") : [];
const base = (nom) => nom.replace(/^.*\//, "").replace(/\.spec\.js$/, ".spec");
const fichiers = rapport.testResults.map((f) => ({
  base: base(f.name),
  passes: f.assertionResults.filter((a) => a.status === "passed").length,
}));
const echecs = [];
const concernes = originaux.filter((o) => filtres.length === 0 || filtres.some((f) => o.includes(f) || f.includes(o)));
for (const o of concernes) {
  const f = fichiers.find((x) => x.base === o);
  if (!f) echecs.push(`Le fichier de test ${o}.ts n'a pas été joué.`);
  else if (f.passes === 0 && portee === "tout") echecs.push(`Aucun test de ${o}.ts n'a réussi.`);
}
const passes = rapport.numPassedTests;
if (rapport.numFailedTests > 0) echecs.push(`${rapport.numFailedTests} test(s) en échec.`);
if (passes === 0) echecs.push("Aucun test n'a réussi : il faut au moins un test exécuté ET réussi.");
if (portee === "tout" && (rapport.numPendingTests > 0 || rapport.numTodoTests > 0)) {
  echecs.push("Des tests sont ignorés (skip, todo) : ils doivent tous être joués.");
}
if (portee === "filtre" && concernes.length > 0 && !concernes.some((o) => fichiers.find((x) => x.base === o)?.passes > 0)) {
  echecs.push("Aucun test du fichier d'origine ne correspond au filtre -t, ou aucun n'a réussi.");
}
if (echecs.length > 0) {
  console.log("Vérification stricte non satisfaite :\n- " + echecs.join("\n- "));
  process.exit(1);
}
