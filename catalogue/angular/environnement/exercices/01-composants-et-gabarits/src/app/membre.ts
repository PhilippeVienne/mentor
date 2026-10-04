// La forme d'un membre du club. Une interface n'existe que pour TypeScript : elle ne produit aucun code.
export interface Membre {
  id: number;
  prenom: string;
  nom: string;
  cotisationPayee: boolean;
}
