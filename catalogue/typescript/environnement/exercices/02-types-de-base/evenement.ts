interface Evenement {
  id: number;
  titre: string;
  lieu: string | null;
  places: number;
  description?: string;
}

const soiree: Evenement = { id: 1, titre: "Soirée d'intégration", lieu: null };

console.log(soiree.titre, soiree.places);
