// Une carte de produit, sans aucun style pour l'instant.
// À compléter (étape 1) avec les classes Tailwind demandées dans la consigne.
export function Carte({ nom, description }: Readonly<{ nom: string; description: string }>) {
  return (
    <article>
      <h3>{nom}</h3>
      <p>{description}</p>
    </article>
  );
}
