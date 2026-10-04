// Les données de l'exercice : un goodie est un objet de la boutique de l'association (prix en centimes).
export type Goodie = { id: number; nom: string; prixCents: number; stock: number };

export const goodies: Goodie[] = [
  { id: 1, nom: "Gourde Éco", prixCents: 1250, stock: 8 },
  { id: 2, nom: "Tote bag", prixCents: 600, stock: 0 },
];

// Écrit un prix à la française : 1250 devient « 12,50 € ».
export function formatEur(cents: number): string {
  return new Intl.NumberFormat("fr-FR", { style: "currency", currency: "EUR" }).format(cents / 100);
}
