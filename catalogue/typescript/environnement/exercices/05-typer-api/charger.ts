import type { Evenement } from "./types";

export async function chargerEvenements(lire: () => Promise<unknown>): Promise<Evenement[]> {
  const donnees = await lire();
  return donnees as Evenement[];
}
