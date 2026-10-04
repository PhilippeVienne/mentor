import assert from "node:assert/strict";
import { Association } from "./src/association";

let n = 0;
const verifier = (f: () => void) => { f(); n++; };

const a = new Association();
verifier(() => assert.equal(typeof a.nom, "string"));
verifier(() => assert.equal(typeof a.adherents, "number"));
verifier(() => assert.ok(a instanceof Association));
console.log(`CONTROLES-REUSSIS ${n}`);
