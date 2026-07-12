export const PBP_SECTIONS = ["PARAM.SFO", "ICON0.PNG", "ICON1.PMF", "PIC0.PNG", "PIC1.PNG", "SND0.AT3", "DATA.PSP", "DATA.PSAR"] as const;
export type PbpSectionName = typeof PBP_SECTIONS[number];
export type PbpSection = { name: PbpSectionName; offset: number; size: number; data: Uint8Array };

const MAGIC = 0x50425000;

export function parsePbp(input: ArrayBuffer | Uint8Array): { version: number; sections: PbpSection[] } {
  const bytes = input instanceof Uint8Array ? input : new Uint8Array(input);
  if (bytes.byteLength < 40) throw new Error("Fichier trop petit pour être un PBP");
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  if (view.getUint32(0, true) !== MAGIC) throw new Error("Signature PBP invalide");
  const offsets = PBP_SECTIONS.map((_, index) => view.getUint32(8 + index * 4, true));
  if (offsets[0] < 40) throw new Error("En-tête PBP invalide");
  offsets.forEach((offset, index) => {
    if (offset > bytes.byteLength || (index > 0 && offset < offsets[index - 1])) throw new Error("Table des sections invalide");
  });
  return { version: view.getUint32(4, true), sections: PBP_SECTIONS.map((name, index) => {
    const offset = offsets[index]; const end = offsets[index + 1] ?? bytes.byteLength;
    return { name, offset, size: end - offset, data: bytes.slice(offset, end) };
  }) };
}

export function buildPbp(sections: Record<PbpSectionName, Uint8Array>, version = 0x10000): Uint8Array {
  const parts = PBP_SECTIONS.map(name => sections[name]);
  const output = new Uint8Array(40 + parts.reduce((sum, part) => sum + part.byteLength, 0));
  const view = new DataView(output.buffer); view.setUint32(0, MAGIC, true); view.setUint32(4, version, true);
  let offset = 40;
  parts.forEach((part, index) => { view.setUint32(8 + index * 4, offset, true); output.set(part, offset); offset += part.byteLength; });
  return output;
}

export function sizeLabel(size: number): string {
  if (size < 1024) return `${size} o`;
  if (size < 1024 ** 2) return `${(size / 1024).toFixed(1)} Ko`;
  return `${(size / 1024 ** 2).toFixed(2)} Mo`;
}
