import { describe, expect, it } from "vitest";
import { buildPbp, parsePbp, PBP_SECTIONS, type PbpSectionName } from "./pbp";

const emptySections = () => Object.fromEntries(
  PBP_SECTIONS.map(name => [name, new Uint8Array()]),
) as Record<PbpSectionName, Uint8Array>;

describe("PBP Studio", () => {
  it("reconstruit sans perte les huit sections", () => {
    const sections = emptySections();
    sections["PARAM.SFO"] = Uint8Array.of(1, 2, 3);
    sections["ICON0.PNG"] = Uint8Array.of(4, 5);
    sections["DATA.PSP"] = Uint8Array.of(6, 7, 8, 9);
    const parsed = parsePbp(buildPbp(sections));
    expect(parsed.sections).toHaveLength(8);
    expect([...parsed.sections[0].data]).toEqual([1, 2, 3]);
    expect([...parsed.sections[1].data]).toEqual([4, 5]);
    expect([...parsed.sections[6].data]).toEqual([6, 7, 8, 9]);
  });

  it("refuse les fichiers courts ou sans signature PBP", () => {
    expect(() => parsePbp(Uint8Array.of(1, 2, 3))).toThrow(/trop petit/);
    expect(() => parsePbp(new Uint8Array(40))).toThrow(/Signature/);
  });
});
