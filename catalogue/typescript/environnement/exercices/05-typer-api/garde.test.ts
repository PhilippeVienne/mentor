import assert from "node:assert/strict";
import { estEvenement } from "./garde";

assert.equal(estEvenement({ id: 1, titre: "Gala", places: 200, lieu: null }), true);
assert.equal(estEvenement({ id: 1, titre: "Gala", places: 200, lieu: "Amphi" }), true);
assert.equal(estEvenement({ id: 1, titre: "Gala", lieu: null }), false, "places manque");
assert.equal(estEvenement({ id: "1", titre: "Gala", places: 200, lieu: null }), false, "id doit être un nombre");
assert.equal(estEvenement({ id: 1, titre: "Gala", places: 200 }), false, "lieu manque");
assert.equal(estEvenement(null), false);
assert.equal(estEvenement("gala"), false);
console.log("garde.test.ts : ok");
