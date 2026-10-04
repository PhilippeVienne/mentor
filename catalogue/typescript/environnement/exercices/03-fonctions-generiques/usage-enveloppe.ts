import { envelopper } from "./enveloppe";

const rep = envelopper({ nom: "BdE", adherents: 800 });

console.log(rep.donnees.adherents.toFixed(0), rep.recu instanceof Date);

// `donnees` garde le type exact de l'objet reçu : cette ligne DOIT être une erreur.
// @ts-expect-error
console.log(rep.donnees.inconnu);
