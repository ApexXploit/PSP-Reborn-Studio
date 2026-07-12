export const PBP_SECTIONS = [
  "PARAM.SFO", "ICON0.PNG", "ICON1.PMF", "PIC0.PNG",
  "PIC1.PNG", "SND0.AT3", "DATA.PSP", "DATA.PSAR",
];

const MAGIC = 0x50425000;

export function parsePbp(input) {
  const bytes = input instanceof Uint8Array ? input : new Uint8Array(input);
  if (bytes.byteLength < 40) throw new Error("Fichier trop petit pour être un PBP.");
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  if (view.getUint32(0, true) !== MAGIC) throw new Error("Signature PBP invalide.");
  const version = view.getUint32(4, true);
  const offsets = PBP_SECTIONS.map((_, index) => view.getUint32(8 + index * 4, true));
  if (offsets[0] < 40) throw new Error("En-tête PBP invalide.");
  for (let i = 0; i < offsets.length; i++) {
    if (offsets[i] > bytes.byteLength || (i && offsets[i] < offsets[i - 1])) {
      throw new Error("Table des sections PBP invalide.");
    }
  }
  return {
    version,
    sections: PBP_SECTIONS.map((name, index) => {
      const start = offsets[index];
      const end = offsets[index + 1] ?? bytes.byteLength;
      return { name, offset: start, size: end - start, data: bytes.slice(start, end) };
    }),
  };
}

export function buildPbp(sectionMap, version = 0x00010000) {
  const parts = PBP_SECTIONS.map(name => sectionMap[name] ?? new Uint8Array());
  const total = 40 + parts.reduce((sum, part) => sum + part.byteLength, 0);
  const output = new Uint8Array(total);
  const view = new DataView(output.buffer);
  view.setUint32(0, MAGIC, true);
  view.setUint32(4, version, true);
  let offset = 40;
  parts.forEach((part, index) => {
    view.setUint32(8 + index * 4, offset, true);
    output.set(part, offset);
    offset += part.byteLength;
  });
  return output;
}

export function parseSfo(input) {
  const bytes = input instanceof Uint8Array ? input : new Uint8Array(input);
  if (bytes.byteLength < 20) return [];
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  if (view.getUint32(0, true) !== 0x46535000) return [];
  const keysOffset = view.getUint32(8, true);
  const valuesOffset = view.getUint32(12, true);
  const count = view.getUint32(16, true);
  if (count > 1024 || 20 + count * 16 > bytes.byteLength) return [];
  const decoder = new TextDecoder();
  const cString = start => {
    let end = start;
    while (end < bytes.length && bytes[end]) end++;
    return decoder.decode(bytes.subarray(start, end));
  };
  const result = [];
  for (let i = 0; i < count; i++) {
    const base = 20 + i * 16;
    const key = cString(keysOffset + view.getUint16(base, true));
    const format = view.getUint16(base + 2, true);
    const length = view.getUint32(base + 4, true);
    const valueAt = valuesOffset + view.getUint32(base + 12, true);
    if (valueAt + length > bytes.length) continue;
    let value;
    if (format === 0x0404 && length >= 4) value = view.getUint32(valueAt, true);
    else value = cString(valueAt);
    result.push({ key, value, format });
  }
  return result;
}

export function formatSize(size) {
  if (size < 1024) return `${size} o`;
  if (size < 1024 ** 2) return `${(size / 1024).toFixed(1)} Ko`;
  return `${(size / 1024 ** 2).toFixed(2)} Mo`;
}
