interface Evenement {
  id: number;
  titre: string;
  brouillon: boolean;
}

function modifier(e: Evenement, changements: Evenement): Evenement {
  return { ...e, ...changements };
}

const gala: Evenement = { id: 1, titre: "Gala", brouillon: true };

console.log(modifier(gala, { titre: "Gala 2027" }));

// Une faute de frappe dans les changements DOIT rester une erreur.
// @ts-expect-error
modifier(gala, { titer: "Gala" });
