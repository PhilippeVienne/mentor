// Mutant 3 : la liste vide donne 1.
export function calculerTotal(prix: number[]): number {
  return prix.reduce((somme, valeur) => somme + valeur, prix.length === 0 ? 1 : 0);
}
