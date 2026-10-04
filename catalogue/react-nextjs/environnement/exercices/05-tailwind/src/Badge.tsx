// Un badge dont la couleur vient de la base de données : n'importe quel code hexadécimal (« #104e64 »…).
// Une classe Tailwind ne peut pas exprimer une valeur libre. À compléter (étape 5).
export function Badge({ texte, couleur }: Readonly<{ texte: string; couleur?: string }>) {
  return <span>{texte}</span>;
}
