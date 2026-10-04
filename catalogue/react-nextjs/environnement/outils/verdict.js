// Usage : node verdict.js resultat.json fichiers-de-test-d-origine-séparés-par-des-virgules [filtre...]
// Contrôle le compte rendu JSON de Vitest : aucun échec ni test ignoré, au moins un test réussi, et chaque fichier de test
// d'origine concerné par les filtres a bien été joué et a réussi au moins un test.
const fs = require("node:fs");

const [, , json, liste, ...filtres] = process.argv;
const rapport = JSON.parse(fs.readFileSync(json, "utf8"));
const originaux = liste ? liste.split(",") : [];
const concernes = originaux.filter((o) => filtres.length === 0 || filtres.some((f) => o.includes(f)));
const echecs = [];
if (concernes.length === 0) echecs.push(`Aucun fichier de test ne correspond à : ${filtres.join(" ")}.`);
for (const o of concernes) {
  const f = rapport.testResults.find((r) => r.name.endsWith("/" + o));
  const reussis = f ? f.assertionResults.filter((a) => a.status === "passed").length : 0;
  if (!f) echecs.push(`Le fichier de test ${o} n'a pas été joué.`);
  else if (reussis === 0) echecs.push(`Aucun test de ${o} n'a réussi.`);
}
if (rapport.numFailedTests > 0) echecs.push(`${rapport.numFailedTests} test(s) en échec.`);
if (rapport.numPassedTests === 0) echecs.push("Aucun test n'a réussi : il faut au moins un test exécuté ET réussi.");
if (rapport.numPendingTests > 0 || rapport.numTodoTests > 0) echecs.push("Des tests sont ignorés (skip, todo) : ils doivent tous être joués.");
if (echecs.length > 0) {
  console.log("Vérification stricte non satisfaite :\n- " + echecs.join("\n- "));
  process.exit(1);
}
