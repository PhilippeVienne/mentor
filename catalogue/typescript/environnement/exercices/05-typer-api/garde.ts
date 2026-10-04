import type { Evenement } from "./types";

export function estEvenement(valeur: unknown): valeur is Evenement {
  // À toi : vérifier réellement chaque champ (pour l'instant, tout est accepté).
  return true;
}
