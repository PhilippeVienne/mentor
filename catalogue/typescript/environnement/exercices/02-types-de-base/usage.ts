import type { Inscription } from "./inscription";

const ins: Inscription = { id: 1, statut: "en_attente" };

// Le statut est une liste fermée : cette ligne DOIT être une erreur (sinon le commentaire ci-dessous se plaint).
// @ts-expect-error
const mauvaise: Inscription = { id: 2, statut: "confirmé" };

console.log(ins.commentaire?.length ?? 0, mauvaise.id);
