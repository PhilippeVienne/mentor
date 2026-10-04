// Une « base de données » fictive, pour l'exercice. Dans MiniShop, `chargerProduit` interroge PostgreSQL.
export type Produit = { id: number; nom: string; description: string };

const PRODUITS: Produit[] = [
  { id: 12, nom: "Gourde Éco", description: "Inox, 50 cl." },
  { id: 13, nom: "Tote bag", description: "Coton bio." },
];

export async function chargerProduit(id: number): Promise<Produit | null> {
  return PRODUITS.find((p) => p.id === id) ?? null;
}
