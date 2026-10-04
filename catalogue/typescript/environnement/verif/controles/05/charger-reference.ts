import type { Evenement } from "./types";
import { estEvenement } from "./garde";

export async function chargerEvenements(lire: () => Promise<unknown>): Promise<Evenement[]> {
  const donnees = await lire();
  if (!Array.isArray(donnees) || !donnees.every(estEvenement)) {
    throw new Error("Réponse inattendue du serveur");
  }
  return donnees;
}
