// Mutant 4 : prend le plus grand prix au lieu de la somme.
export function calculerTotal(prix: number[]): number {
  return prix.length === 0 ? 0 : Math.max(...prix);
}
