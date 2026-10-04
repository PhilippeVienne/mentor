// Un sélecteur de quantité pour le bouton d'ajout au panier.
// Pour l'instant il est figé : il affiche toujours 1 et ses boutons ne font rien.
// À compléter (étapes 3 à 5) avec un état (useState), des gestionnaires d'événements et des boutons désactivés.
export function ChoixQuantite({
  stock,
  onAjouter,
}: Readonly<{ stock: number; onAjouter: (quantite: number) => void }>) {
  return (
    <div>
      <button type="button">−</button>
      <span>1</span>
      <button type="button">+</button>
      <button type="button">Ajouter au panier</button>
    </div>
  );
}
