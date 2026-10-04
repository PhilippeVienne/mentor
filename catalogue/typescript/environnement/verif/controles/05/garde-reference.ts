import type { Evenement } from "./types";

export function estEvenement(valeur: unknown): valeur is Evenement {
  if (typeof valeur !== "object" || valeur === null) {
    return false;
  }
  const candidat = valeur as Record<string, unknown>;
  return (
    typeof candidat.id === "number" &&
    typeof candidat.titre === "string" &&
    typeof candidat.places === "number" &&
    (typeof candidat.lieu === "string" || candidat.lieu === null)
  );
}
