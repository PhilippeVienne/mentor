// Affiche les goodies d'une boutique, chargés depuis le serveur.
// Pour l'instant le composant n'appelle jamais le serveur : il reste sur « Chargement… ».
// À compléter (étapes 1 à 3) avec useState et useEffect.
type Goodie = { id: number; nom: string };

export function ListeGoodies({ boutique }: Readonly<{ boutique: string }>) {
  return <p data-boutique={boutique}>Chargement…</p>;
}
