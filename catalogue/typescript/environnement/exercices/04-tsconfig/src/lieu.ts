export interface Evenement {
  titre: string;
  lieu: string | null;
}

export function lieuEnMajuscules(evenement: Evenement): string {
  return evenement.lieu.toUpperCase();
}
