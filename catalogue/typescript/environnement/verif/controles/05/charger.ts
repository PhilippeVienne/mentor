import assert from "node:assert/strict";
import { chargerEvenements } from "./charger";
import { reponseCassee, reponseCorrecte } from "./serveur";

let n = 0;
const compter = () => { n++; };

const evenements = await chargerEvenements(reponseCorrecte);
assert.equal(evenements[0].titre, "Gala"); compter();
assert.equal(evenements.length, 1); compter();
await assert.rejects(() => chargerEvenements(reponseCassee)); compter();
await assert.rejects(() => chargerEvenements(async () => ({ pas: "un tableau" }))); compter();
await assert.rejects(() => chargerEvenements(async () => [{ id: 1, titre: "Gala", places: 5, lieu: null }, { id: "x" }])); compter();
const autre = await chargerEvenements(async () => [{ id: 2, titre: "Soirée", places: 10, lieu: "Hall" }]);
assert.equal(autre[0].titre, "Soirée"); compter();
console.log(`CONTROLES-REUSSIS ${n}`);
