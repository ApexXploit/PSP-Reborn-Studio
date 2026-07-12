import { PBP_SECTIONS, buildPbp, formatSize, parsePbp, parseSfo } from "./pbp.js";

const state = { version: 0x10000, sections: Object.fromEntries(PBP_SECTIONS.map(n => [n, new Uint8Array()])) };
const rows = document.querySelector("#sections");
const status = document.querySelector("#status");

function download(data, name) {
  const url = URL.createObjectURL(new Blob([data]));
  const a = Object.assign(document.createElement("a"), { href: url, download: name });
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

function preview(name, data) {
  const panel = document.querySelector("#preview");
  panel.replaceChildren();
  if (!data.length) return panel.textContent = "Cette section est vide.";
  if (name.endsWith(".PNG")) {
    const img = document.createElement("img");
    img.src = URL.createObjectURL(new Blob([data], { type: "image/png" }));
    panel.append(img);
  } else if (name === "PARAM.SFO") {
    const values = parseSfo(data);
    panel.innerHTML = values.length
      ? `<dl>${values.map(x => `<dt>${escapeHtml(x.key)}</dt><dd>${escapeHtml(String(x.value))}</dd>`).join("")}</dl>`
      : "PARAM.SFO vide ou non reconnu.";
  } else {
    panel.textContent = `${name} — ${formatSize(data.length)}\nAperçu binaire indisponible.`;
  }
}

function escapeHtml(value) {
  return value.replace(/[&<>"']/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
}

function render() {
  rows.replaceChildren(...PBP_SECTIONS.map(name => {
    const data = state.sections[name];
    const row = document.createElement("tr");
    row.innerHTML = `<td><strong>${name}</strong></td><td>${formatSize(data.length)}</td><td class="actions"></td>`;
    const actions = row.querySelector(".actions");
    const show = Object.assign(document.createElement("button"), { textContent: "Aperçu" });
    show.onclick = () => preview(name, data);
    const replace = Object.assign(document.createElement("button"), { textContent: "Remplacer" });
    replace.onclick = () => chooseFile(file => { state.sections[name] = file; render(); preview(name, file); });
    const save = Object.assign(document.createElement("button"), { textContent: "Extraire", disabled: !data.length });
    save.onclick = () => download(data, name);
    const clear = Object.assign(document.createElement("button"), { textContent: "Vider", disabled: !data.length });
    clear.onclick = () => { state.sections[name] = new Uint8Array(); render(); };
    actions.append(show, replace, save, clear);
    return row;
  }));
}

function chooseFile(callback, accept = "") {
  const input = Object.assign(document.createElement("input"), { type: "file", accept });
  input.onchange = async () => input.files[0] && callback(new Uint8Array(await input.files[0].arrayBuffer()), input.files[0]);
  input.click();
}

document.querySelector("#open").onclick = () => chooseFile((bytes, file) => {
  try {
    const pbp = parsePbp(bytes);
    state.version = pbp.version;
    pbp.sections.forEach(section => state.sections[section.name] = section.data);
    status.textContent = `${file.name} ouvert — ${formatSize(bytes.length)}`;
    render(); preview("PARAM.SFO", state.sections["PARAM.SFO"]);
  } catch (error) { status.textContent = error.message; }
}, ".pbp");

document.querySelector("#new").onclick = () => {
  PBP_SECTIONS.forEach(name => state.sections[name] = new Uint8Array());
  status.textContent = "Nouveau PBP"; render(); preview("PARAM.SFO", new Uint8Array());
};
document.querySelector("#save").onclick = () => download(buildPbp(state.sections, state.version), "EBOOT.PBP");
document.querySelector("#extract-all").onclick = () => {
  const nonEmpty = PBP_SECTIONS.filter(name => state.sections[name].length);
  nonEmpty.forEach((name, index) => setTimeout(() => download(state.sections[name], name), index * 120));
  status.textContent = `${nonEmpty.length} section(s) envoyée(s) au dossier de téléchargements.`;
};

render();
