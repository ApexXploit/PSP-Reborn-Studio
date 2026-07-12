import { useEffect, useState } from "react";
import Editor from "@monaco-editor/react";
import PbpStudio from "./PbpStudio";
import { open } from "@tauri-apps/plugin-dialog";
import { languages, luaRuntimes, runtimeStatusLabel, templates, type LanguageId } from "./projectCatalog";
import { buildProject, createProject, deployProject, getEnvironmentStatus, listProjects, readSource, runInPpsspp, saveSource, type EnvironmentStatus, type Project } from "./backend";

export default function App() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [active, setActive] = useState("");
  const [source, setSource] = useState("");
  const [status, setStatus] = useState("Prêt");
  const [pspRoot, setPspRoot] = useState("");
  const [showCreate, setShowCreate] = useState(false);
  const [newName, setNewName] = useState("");
  const [newLanguage, setNewLanguage] = useState<LanguageId>("cpp");
  const [newTemplate, setNewTemplate] = useState("hello");
  const [newRuntime, setNewRuntime] = useState("hm-v7-rc1");
  const [dirty, setDirty] = useState(false);
  const [environment, setEnvironment] = useState<EnvironmentStatus>();
  const [view, setView] = useState<"code" | "pbp">("code");
  useEffect(() => { listProjects().then(items => { setProjects(items); if (items[0]) setActive(items[0].name); }); }, []);
  useEffect(() => { getEnvironmentStatus().then(setEnvironment); }, []);
  useEffect(() => { if (active) readSource(active).then(value => { setSource(value); setDirty(false); }); }, [active]);
  const action = async (label: string, job: () => Promise<unknown>) => {
    setStatus(`${label}…`); try { const result = await job(); setStatus(typeof result === "string" ? result : `${label} terminé`); }
    catch (error) { setStatus(`Erreur : ${error instanceof Error ? error.message : error}`); }
  };
  const create = async () => {
    if (!/^[A-Za-z0-9_-]{1,32}$/.test(newName)) return setStatus("Erreur : utilise 1 à 32 lettres, chiffres, _ ou -");
    await action("Création", async () => {
      const project = await createProject(newName, newLanguage, newTemplate, newLanguage === "lua" ? newRuntime : undefined);
      setProjects(current => [...current, project].sort((a,b) => a.name.localeCompare(b.name)));
      setActive(project.name); setNewName(""); setShowCreate(false);
      return `Projet ${project.name} créé`;
    });
  };
  const choosePsp = async () => {
    try {
      const selected = await open({ directory: true, multiple: false, title: "Choisir le volume de la PSP" });
      if (selected) setPspRoot(selected);
    } catch { setStatus("Le sélecteur de volume est disponible dans l’application native."); }
  };
  const installOnPsp = () => {
    const overwrite = confirm(`Installer ${active} sur la PSP ? Un ancien EBOOT du même projet sera remplacé.`);
    if (overwrite) action("Installation", () => deployProject(active, pspRoot, true));
  };
  const activeProject = projects.find(project => project.name === active);
  const isLuaProject = activeProject?.language === "lua";
  const buildLabel = isLuaProject ? "Préparer" : "Compiler";
  const buildAction = isLuaProject ? "Préparation" : "Compilation";
  const canBuild = Boolean(active && (isLuaProject || environment?.pspdevReady));
  return <div className="shell">
    <aside className="sidebar"><div className="brand"><img src="/psp-reborn-logo.png" alt=""/><span><b>PSP</b> Reborn</span></div><button className="new" onClick={() => setShowCreate(true)}>＋ Nouveau jeu</button><button className={view === "code" ? "nav active" : "nav"} onClick={() => setView("code")}>⌨ Code</button><button className={view === "pbp" ? "nav active" : "nav"} onClick={() => setView("pbp")}>📦 PBP Studio</button><h3>PROJETS</h3>{projects.map(p => <button className={p.name === active ? "project active" : "project"} onClick={() => { if (!dirty || confirm("Abandonner les modifications non enregistrées ?")) { setActive(p.name); setView("code"); } }} key={p.name}><span>{p.language === "lua" ? "🌙" : "🎮"} {p.name}</span><small>{p.language === "lua" ? p.runtimeVersion : "C++17"}</small></button>)}<div className="locked">🔒 Mode sécurisé<br/><small>Kernel, terminal et chemins libres désactivés</small></div></aside>
    <main className={view === "pbp" ? "pbp-main" : ""}>{view === "code" ? <><header><div><strong>{active || "Aucun projet"}</strong><span>{dirty ? "● Modifié" : activeProject?.language === "lua" ? "script.lua" : "src/main.cpp"}</span></div><div className="environment"><span className={environment?.pspdevReady ? "ok" : "missing"}>● PSPDEV</span><span className={environment?.ppssppReady ? "ok" : "missing"}>● PPSSPP</span></div><div className="toolbar"><button disabled={!active || !dirty} onClick={() => action("Sauvegarde", async () => { await saveSource(active, source); setDirty(false); })}>Enregistrer</button><button disabled={!active || !environment?.ppssppReady} onClick={() => action("Test", () => runInPpsspp(active))}>Tester</button><button className="build" disabled={!canBuild} title={isLuaProject ? activeProject?.runtimeVersion : environment?.pspdevVersion} onClick={() => action(buildAction, async () => { await saveSource(active, source); setDirty(false); return buildProject(active); })}>{buildLabel}</button></div></header>
      {active ? <Editor height="calc(100vh - 154px)" language={activeProject?.language === "lua" ? "lua" : "cpp"} theme="vs-dark" value={source} onChange={v => { setSource(v ?? ""); setDirty(true); }} options={{ minimap:{enabled:false}, fontSize:14, automaticLayout:true, tabSize:4 }}/>
      : <div className="welcome"><div><span>🎮</span><h1>Crée ton premier jeu PSP</h1><p>Un projet C++ prêt à compiler et à lancer sur PPSSPP en quelques secondes.</p><button className="build" onClick={() => setShowCreate(true)}>Créer un jeu</button></div></div>}
      <footer><div><b>Installation PSP</b><input readOnly value={pspRoot} placeholder="Aucun volume sélectionné"/><button onClick={choosePsp}>Choisir</button><button disabled={!pspRoot || !active} onClick={installOnPsp}>Installer sur ma PSP</button></div><output>{status}</output></footer></> : <PbpStudio/>}
    </main>
    {showCreate && <div className="modal-backdrop" onMouseDown={() => setShowCreate(false)}><form className="modal project-wizard" onSubmit={e => { e.preventDefault(); create(); }} onMouseDown={e => e.stopPropagation()}><h2>Nouveau projet PSP</h2><label>Nom du projet<input autoFocus value={newName} maxLength={32} onChange={e => setNewName(e.target.value)} placeholder="MonJeu"/></label><h3>Langage</h3><div className="choice-grid">{languages.map(language => <button type="button" className={newLanguage === language.id ? "choice selected" : "choice"} onClick={() => { setNewLanguage(language.id); setNewTemplate(templates[language.id][0].id); if (language.id === "lua") setNewRuntime("lpp-r163"); }} key={language.id}><b>{language.name}</b><small>{language.description}</small><em>{language.badge}</em></button>)}</div>{newLanguage === "lua" && <label>Version LuaPlayer<select value={newRuntime} onChange={e => setNewRuntime(e.target.value)}>{luaRuntimes.map(runtime => <option value={runtime.id} key={runtime.id}>{runtime.name} — {runtimeStatusLabel[runtime.status]}</option>)}</select></label>}<h3>Modèle d’exemple</h3><div className="template-list">{templates[newLanguage].map(item => <button type="button" className={newTemplate === item.id ? "template-choice selected" : "template-choice"} onClick={() => setNewTemplate(item.id)} key={item.id}><b>{item.name}</b><small>{item.description}</small><span>{item.features.join(" · ")}</span></button>)}</div><p>{newLanguage === "lua" && newRuntime !== "lpp-r163" ? "Cette version est cataloguée, mais son binaire original doit encore être récupéré avant l’exécution." : "Le projet utilisera uniquement les composants validés pour ce modèle."}</p><div className="modal-actions"><button type="button" onClick={() => setShowCreate(false)}>Annuler</button><button className="build" type="submit">Créer le projet</button></div></form></div>}
  </div>;
}
