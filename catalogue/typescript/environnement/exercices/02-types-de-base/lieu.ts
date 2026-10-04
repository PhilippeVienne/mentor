interface Evenement {
  id: number;
  titre: string;
  lieu: string | null;
  places: number;
}

function afficherLieu(evenement: Evenement): string {
  return evenement.lieu.toUpperCase();
}

console.log(afficherLieu({ id: 1, titre: "Gala", lieu: null, places: 200 }));
console.log(afficherLieu({ id: 2, titre: "Soirée", lieu: "Amphi Chappe", places: 120 }));
