function premier(liste: number[]): number | undefined {
  return liste[0];
}

const n = premier([3, 4, 5]);
const s = premier(["a", "b"]);

console.log(n, s);

// `s` est un texte (ou `undefined`) : `toFixed` n'existe pas, cette ligne DOIT être une erreur.
// @ts-expect-error
console.log(s.toFixed(1));
