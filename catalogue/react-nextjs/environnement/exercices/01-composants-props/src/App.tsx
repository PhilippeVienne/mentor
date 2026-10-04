import { GoodieCard } from "./GoodieCard";
import { goodies } from "./goodies";

// À corriger (étapes 4 et 5) : cette page contient une erreur de type, et elle n'affiche qu'une seule carte.
// Elle doit afficher tous les goodies, dans un Panneau de titre « Nos goodies ».
export function App() {
  return <GoodieCard nom={goodies[0].nom} prixCents="1250" stock={goodies[0].stock} />;
}
