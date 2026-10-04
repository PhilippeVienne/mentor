// Une pastille de couleur choisie parmi quelques couleurs connues.
// BUG (étape 4) : le nom de la classe est assemblé à la volée, donc Tailwind ne le voit jamais « en entier ».
export type Couleur = "rouge" | "bleu";

export function Pastille({ couleur }: Readonly<{ couleur: Couleur }>) {
  const nom = couleur === "rouge" ? "red" : "blue";
  return <span className={`bg-${nom}-500 inline-block size-4 rounded-full`} />;
}
