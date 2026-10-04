import type { ReactNode } from "react";

// La grille des catégories de la page d'accueil. Pour l'instant : une simple <div>, les cartes s'empilent.
// À compléter (étape 2) : une grille adaptative.
export function GrilleCategories({ children }: Readonly<{ children: ReactNode }>) {
  return <div>{children}</div>;
}
