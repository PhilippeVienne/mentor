import assert from "node:assert/strict";
import { estEvenement } from "./garde";

let n = 0;
const attendre = (valeur: unknown, attendu: boolean, message: string) => {
  assert.equal(estEvenement(valeur), attendu, message);
  n++;
};

attendre({ id: 1, titre: "Gala", places: 200, lieu: null }, true, "lieu null accepté");
attendre({ id: 1, titre: "Gala", places: 200, lieu: "Amphi" }, true, "lieu texte accepté");
attendre({ id: 7, titre: "", places: 0, lieu: "" }, true, "valeurs vides mais du bon type");
attendre({ id: 1, titre: "Gala", lieu: null }, false, "places manque");
attendre({ id: "1", titre: "Gala", places: 200, lieu: null }, false, "id doit être un nombre");
attendre({ id: 1, titre: "Gala", places: 200 }, false, "lieu manque");
attendre({ id: 1, titre: 5, places: 200, lieu: null }, false, "titre doit être un texte");
attendre({ id: 1, titre: "Gala", places: "200", lieu: null }, false, "places doit être un nombre");
attendre({ id: 1, titre: "Gala", places: 200, lieu: 3 }, false, "lieu doit être un texte ou null");
attendre({ titre: "Gala", places: 200, lieu: null }, false, "id manque");
attendre(null, false, "null refusé");
attendre("gala", false, "texte refusé");
attendre(undefined, false, "undefined refusé");
attendre([], false, "tableau vide refusé");
console.log(`CONTROLES-REUSSIS ${n}`);
