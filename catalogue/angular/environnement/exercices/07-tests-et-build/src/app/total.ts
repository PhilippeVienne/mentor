// Additionne des prix. C'est la fonction que tu vas tester à l'étape 1.
export function calculerTotal(prix: number[]): number {
  return prix.reduce((somme, valeur) => somme + valeur, 0);
}
