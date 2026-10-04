// Mutant 2 : oublie le premier prix.
export function calculerTotal(prix: number[]): number {
  return prix.slice(1).reduce((somme, valeur) => somme + valeur, 0);
}
