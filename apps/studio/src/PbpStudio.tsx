import { useRef, useState } from "react";
import { buildPbp, parsePbp, PBP_SECTIONS, sizeLabel, type PbpSectionName } from "./pbp";

const empty = () => Object.fromEntries(PBP_SECTIONS.map(name => [name, new Uint8Array()])) as Record<PbpSectionName, Uint8Array>;

export default function PbpStudio() {
  const [sections, setSections] = useState(empty);
  const [version, setVersion] = useState(0x10000);
  const [status, setStatus] = useState("Ouvre un EBOOT.PBP pour inspecter ses sections.");
  const [preview, setPreview] = useState<PbpSectionName>("ICON0.PNG");
  const input = useRef<HTMLInputElement>(null);

  const download = (data: Uint8Array, name: string) => {
    const copy = new Uint8Array(data);
    const url = URL.createObjectURL(new Blob([copy.buffer]));
    Object.assign(document.createElement("a"), { href: url, download: name }).click();
    setTimeout(() => URL.revokeObjectURL(url), 500);
  };
  const open = async (file?: File) => {
    if (!file) return;
    try {
      const parsed = parsePbp(await file.arrayBuffer());
      setVersion(parsed.version); setSections(Object.fromEntries(parsed.sections.map(section => [section.name, section.data])) as Record<PbpSectionName, Uint8Array>);
      setStatus(`${file.name} — ${sizeLabel(file.size)} — PBP valide`);
    } catch (error) { setStatus(`Erreur : ${error instanceof Error ? error.message : error}`); }
  };
  const replace = (name: PbpSectionName) => {
    const picker = Object.assign(document.createElement("input"), { type: "file" });
    picker.onchange = async () => {
      const file = picker.files?.[0];
      if (file) {
        const data = new Uint8Array(await file.arrayBuffer());
        setSections(current => ({ ...current, [name]: data }));
      }
    };
    picker.click();
  };
  const image = preview.endsWith(".PNG") && sections[preview].length
    ? URL.createObjectURL(new Blob([new Uint8Array(sections[preview]).buffer], { type: "image/png" })) : undefined;

  return <div className="pbp-page"><div className="pbp-head"><div><h2>PBP Studio</h2><p>Inspecter, extraire et reconstruire un EBOOT sans toucher au fichier original.</p></div><div><input ref={input} hidden type="file" accept=".pbp" onChange={e => open(e.target.files?.[0])}/><button onClick={() => input.current?.click()}>Ouvrir</button><button onClick={() => { setSections(empty()); setStatus("Nouveau PBP"); }}>Nouveau</button><button className="build" onClick={() => download(buildPbp(sections, version), "EBOOT.PBP")}>Construire</button></div></div>
    <div className="pbp-layout"><section><table><thead><tr><th>Section</th><th>Taille</th><th>Actions</th></tr></thead><tbody>{PBP_SECTIONS.map(name => <tr key={name}><td><b>{name}</b></td><td>{sizeLabel(sections[name].length)}</td><td><button onClick={() => setPreview(name)}>Aperçu</button><button onClick={() => replace(name)}>Remplacer</button><button disabled={!sections[name].length} onClick={() => download(sections[name], name)}>Extraire</button></td></tr>)}</tbody></table></section><aside><h3>{preview}</h3>{image ? <img src={image}/> : <p>{sections[preview].length ? `Section binaire de ${sizeLabel(sections[preview].length)}` : "Section vide"}</p>}</aside></div><output>{status}</output></div>;
}
