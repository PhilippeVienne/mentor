import assert from "node:assert/strict";
import { chargerEvenements } from "./charger";
import { reponseCassee, reponseCorrecte } from "./serveur";

const evenements = await chargerEvenements(reponseCorrecte);
assert.equal(evenements[0].titre, "Gala");
await assert.rejects(() => chargerEvenements(reponseCassee), "une réponse incomplète doit être refusée");
await assert.rejects(() => chargerEvenements(async () => ({ pas: "un tableau" })));
console.log("charger.test.ts : ok");
