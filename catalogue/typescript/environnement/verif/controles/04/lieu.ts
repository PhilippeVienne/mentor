import assert from "node:assert/strict";
import { lieuEnMajuscules } from "./src/lieu";

let n = 0;
const verifier = (f: () => void) => { f(); n++; };

verifier(() => assert.equal(lieuEnMajuscules({ titre: "Gala", lieu: null }), "Lieu à confirmer"));
verifier(() => assert.equal(lieuEnMajuscules({ titre: "Gala", lieu: "Amphi Chappe" }), "AMPHI CHAPPE"));
verifier(() => assert.equal(lieuEnMajuscules({ titre: "Soirée", lieu: "hall" }), "HALL"));
console.log(`CONTROLES-REUSSIS ${n}`);
