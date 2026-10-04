function decrire(valeur: string | number): string {
  return valeur.toFixed(2);
}

console.log(decrire("gala"));
console.log(decrire(3.5));
