import assert from "node:assert/strict";
import { total } from "./src/asso";

let n = 0;
const verifier = (f: () => void) => { f(); n++; };

verifier(() => assert.equal(total(3, 4), 12));
verifier(() => assert.equal(total(2.5, 2), 5));
verifier(() => assert.equal(total(10, 0), 0));
console.log(`CONTROLES-REUSSIS ${n}`);
