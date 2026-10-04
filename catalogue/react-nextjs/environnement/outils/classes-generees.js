#!/usr/bin/env node
// Usage (via la commande `classes-generees`, qui travaille sur une copie de ton projet) : classes-generees classe...
// Compile `globals.css` (celui d'origine) avec Tailwind CSS, comme le ferait le build du site, puis vérifie que chaque
// classe donnée a bien reçu une règle CSS. Une classe que Tailwind n'a pas lue « en entier » dans tes fichiers
// n'existe pas dans le CSS final : c'est exactement ce que ce contrôle détecte. Les commentaires sont retirés des fichiers
// avant la compilation (une classe écrite dans un commentaire ne compte pas) et les fichiers de test sont ignorés.
// Code de retour : 0 si toutes les classes existent, 1 sinon, 2 si Tailwind échoue.
const { execFileSync } = require("node:child_process");
const { mkdtempSync, readFileSync, readdirSync, statSync, writeFileSync, unlinkSync } = require("node:fs");
const { sansCommentaires } = require("./sans-commentaires.js");
const { tmpdir } = require("node:os");
const { join } = require("node:path");

const classes = process.argv.slice(2);

// Prépare le dossier courant : plus de tests, plus de commentaires dans les sources.
function nettoyer(dossier) {
  for (const nom of readdirSync(dossier)) {
    if (nom === "node_modules") continue;
    const chemin = join(dossier, nom);
    if (statSync(chemin).isDirectory()) nettoyer(chemin);
    else if (/\.test\.tsx?$/.test(nom)) unlinkSync(chemin);
    else if (/\.tsx?$/.test(nom)) writeFileSync(chemin, sansCommentaires(readFileSync(chemin, "utf8"), chemin));
  }
}
nettoyer(process.cwd());
const dossier = mkdtempSync(join(tmpdir(), "tw-"));
const sortie = join(dossier, "sortie.css");
try {
  execFileSync("/opt/outils/node_modules/.bin/tailwindcss", ["-i", "globals.css", "-o", sortie], { stdio: "pipe" });
} catch (e) {
  console.error("Tailwind a échoué :", String(e.stderr || e.message).trim());
  process.exit(2);
}
const css = readFileSync(sortie, "utf8");
const absentes = classes.filter((c) => {
  const selecteur = "." + c.replace(/[^a-zA-Z0-9_-]/g, (ch) => "\\" + ch);
  const trouve = css.split(selecteur).slice(1).some((suite) => !/^[\w-]/.test(suite));
  return !trouve;
});
if (absentes.length > 0) {
  console.log("Classes absentes du CSS généré : " + absentes.join(", "));
  process.exit(1);
}
console.log("Toutes les classes sont dans le CSS généré (" + classes.length + ").");
