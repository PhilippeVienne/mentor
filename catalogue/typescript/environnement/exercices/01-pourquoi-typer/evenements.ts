const evenement = { titre: "Soirée d'intégration", lieu: "Amphi Chappe", places: 120 };

console.log(evenement.titer.toUpperCase());

function reserver(places: number, demandees: number): number {
  return places - demandees;
}

reserver(120, "4");

let compteur = 3;
compteur = "trois";
