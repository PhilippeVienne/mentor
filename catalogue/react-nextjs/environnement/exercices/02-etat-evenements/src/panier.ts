// La logique du panier, écrite comme des fonctions pures : elles ne modifient jamais leurs arguments.
export type Ligne = { id: number; nom: string; prixCents: number; stock: number; quantite: number };
export type LigneEntree = Omit<Ligne, "quantite">;

// À écrire (étape 1) : ajouter une unité de `entree` au panier et renvoyer un NOUVEAU tableau.
//  - stock nul ou négatif : le panier ne change pas ;
//  - article absent du panier : on ajoute une ligne de quantité 1 ;
//  - article déjà présent : sa quantité augmente de 1, sans jamais dépasser son stock.
export function ajouter(panier: Ligne[], entree: LigneEntree): Ligne[] {
  return panier;
}

// À écrire (étape 2) : renvoyer un nouveau tableau sans la ligne dont l'identifiant est `id`.
export function retirer(panier: Ligne[], id: number): Ligne[] {
  return panier;
}

// À écrire (étape 2) : le nombre total d'articles et le prix total en centimes, CALCULÉS à partir du panier.
export const totalArticles = (panier: Ligne[]): number => 0;
export const totalCents = (panier: Ligne[]): number => 0;
