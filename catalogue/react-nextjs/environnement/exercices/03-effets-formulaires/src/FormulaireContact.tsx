// Un formulaire de contact. Pour l'instant ses champs ne sont pas « contrôlés » : React ne sait pas ce qu'ils contiennent.
// À compléter (étapes 4 et 5) : état de chaque champ, validation de l'e-mail, état d'envoi et message d'erreur.
export type Message = { email: string; texte: string };

export function FormulaireContact({ onEnvoyer }: Readonly<{ onEnvoyer: (message: Message) => Promise<void> }>) {
  return (
    <form>
      <label htmlFor="email">Ton e-mail</label>
      <input id="email" type="email" />
      <label htmlFor="texte">Ton message</label>
      <textarea id="texte" />
      <button type="submit">Envoyer</button>
    </form>
  );
}
