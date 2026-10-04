interface Association {
  nom: string;
  adherents: number;
}

function ajouterAdherents(asso, nombre, motif) {
  console.log(motif ?? "sans motif");
  return { ...asso, adherents: asso.adherents + nombre };
}

const bde: Association = { nom: "BdE", adherents: 800 };
console.log(ajouterAdherents(bde, 25).adherents);
