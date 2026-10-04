// Retire les commentaires d'un fichier source avant de chercher un motif dedans (un motif écrit dans un commentaire
// ne doit pas compter). TypeScript/JavaScript : analyse syntaxique avec TypeScript (les chaînes sont respectées) ;
// HTML : <!-- … --> ; CSS : /* … */ ; autres (shell, Dockerfile, nginx…) : lignes qui commencent par « # ».
const path = require("node:path");

function sansCommentaires(texte, nom) {
  const ext = path.extname(nom).toLowerCase();
  if ([".ts", ".tsx", ".js", ".jsx", ".mjs", ".mts", ".cjs"].includes(ext)) {
    const ts = require("typescript");
    const genre = ext === ".tsx" || ext === ".jsx" ? ts.ScriptKind.TSX : ts.ScriptKind.TS;
    const source = ts.createSourceFile(nom, texte, ts.ScriptTarget.Latest, true, genre);
    const plages = new Map();
    const visiter = (noeud) => {
      const enfants = noeud.getChildren(source);
      if (enfants.length === 0 && noeud.kind !== ts.SyntaxKind.JsxText) {
        for (const p of ts.getLeadingCommentRanges(texte, noeud.getFullStart()) ?? []) plages.set(p.pos, p.end);
      }
      enfants.forEach(visiter);
    };
    visiter(source);
    let sortie = texte;
    for (const [debut, fin] of [...plages].sort((x, y) => y[0] - x[0])) {
      sortie = sortie.slice(0, debut) + sortie.slice(debut, fin).replace(/[^\n]/g, "") + sortie.slice(fin);
    }
    return sortie;
  }
  if (ext === ".html") return texte.replace(/<!--[\s\S]*?-->/g, (m) => m.replace(/[^\n]/g, ""));
  if (ext === ".css") return texte.replace(/\/\*[\s\S]*?\*\//g, (m) => m.replace(/[^\n]/g, ""));
  return texte.replace(/^[ \t]*#.*$/gm, "");
}

module.exports = { sansCommentaires };
