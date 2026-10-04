interface Evenement {
  id: number;
  titre: string;
}

const brouillons: (Evenement | null)[] = [{ id: 1, titre: "Gala" }, null, { id: 0, titre: "Vide" }];

// On garde les cases non nulles dont l'identifiant est positif.
const evenements = brouillons.filter((e) => e !== null && e.id > 0);

console.log(evenements[0].titre);
