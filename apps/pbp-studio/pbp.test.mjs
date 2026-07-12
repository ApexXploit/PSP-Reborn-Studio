import assert from "node:assert/strict";
import { buildPbp, parsePbp } from "./pbp.js";

const sections = { "PARAM.SFO": Uint8Array.of(1, 2, 3), "ICON0.PNG": Uint8Array.of(4, 5) };
const built = buildPbp(sections);
const parsed = parsePbp(built);
assert.equal(parsed.sections.length, 8);
assert.deepEqual([...parsed.sections[0].data], [1, 2, 3]);
assert.deepEqual([...parsed.sections[1].data], [4, 5]);
assert.equal(parsed.sections[7].size, 0);
assert.throws(() => parsePbp(Uint8Array.of(1, 2, 3)), /trop petit/);
console.log("PBP parser/build tests: OK");
