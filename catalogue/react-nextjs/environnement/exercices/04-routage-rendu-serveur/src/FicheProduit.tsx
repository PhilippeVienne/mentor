import type { Produit } from "./produits";

// À transformer en composant CLIENT (étape 4) : il doit garder en mémoire une quantité (useState, départ à 1),
// afficher le nom dans un <h1>, la quantité dans un <output>, et un bouton « Une de plus » qui l'augmente.
export function FicheProduit({ initial }: Readonly<{ initial: Produit }>) {
  return <h1>{initial.nom}</h1>;
}
