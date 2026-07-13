import { useEffect, useRef, useState } from "react";
import type { editor as MonacoEditor } from "monaco-editor";
import Editor from "@monaco-editor/react";
import EnvironmentPanel from "./EnvironmentPanel";
import Help from "./Help";
import PbpStudio from "./PbpStudio";
import { open } from "@tauri-apps/plugin-dialog";
import { languages, luaRuntimes, runtimeStatusLabel, templates, type LanguageId } from "./projectCatalog";
import { buildProject, createProject, createProjectItem, deleteProjectItem, deployProject, ejectPsp, getEnvironmentStatus, listProjectFiles, listProjects, readProjectFile, renameProjectItem, runInPpsspp, saveProjectFile, type BuildDiagnostic, type BuildReport, type DeploymentReport, type EnvironmentStatus, type Project, type ProjectFileEntry } from "./backend";

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
  const [activeFile, setActiveFile] = useState("");
  const [selectedPath, setSelectedPath] = useState("");
  const [fileEntries, setFileEntries] = useState<ProjectFileEntry[]>([]);
  const [collapsedFolders, setCollapsedFolders] = useState<Set<string>>(new Set());
  const [fileDialog, setFileDialog] = useState<"file" | "folder" | "rename" | null>(null);
  const [fileDialogName, setFileDialogName] = useState("");
  const [environment, setEnvironment] = useState<EnvironmentStatus>();
  const [environmentRefreshing, setEnvironmentRefreshing] = useState(false);
  const [buildReport, setBuildReport] = useState<BuildReport>();
  const [deploymentReport, setDeploymentReport] = useState<DeploymentReport>();
  const [consoleOpen, setConsoleOpen] = useState(true);
  const editorRef = useRef<MonacoEditor.IStandaloneCodeEditor | null>(null);
  const pendingPosition = useRef<{ lineNumber: number; column: number } | undefined>(undefined);
  const [view, setView] = useState<"code" | "pbp" | "help" | "environment">("code");
  useEffect(() => { listProjects().then(items => { setProjects(items); if (items[0]) setActive(items[0].name); }); }, []);
  const refreshEnvironment = async () => {
    setEnvironmentRefreshing(true);
    try {
      const result = await getEnvironmentStatus();
      setEnvironment(result);
      if (!pspRoot && result.pspMounts[0]) setPspRoot(result.pspMounts[0]);
    } catch (error) {
      setStatus(`Erreur diagnostic : ${error instanceof Error ? error.message : error}`);
    } finally { setEnvironmentRefreshing(false); }
  };
  useEffect(() => { refreshEnvironment(); }, []);
  useEffect(() => {
    const project = projects.find(item => item.name === active);
    if (!project) { setFileEntries([]); setActiveFile(""); setSelectedPath(""); return; }
    const initialFile = project.language === "lua" ? "script.lua" : "src/main.cpp";
    let cancelled = false;
    Promise.all([listProjectFiles(active), readProjectFile(active, initialFile)]).then(([entries, value]) => {
      if (cancelled) return;
      setFileEntries(entries); setActiveFile(initialFile); setSelectedPath(initialFile);
      setSource(value); setDirty(false); setCollapsedFolders(new Set());
    }).catch(error => { if (!cancelled) setStatus(`Erreur : ${error instanceof Error ? error.message : error}`); });
    return () => { cancelled = true; };
  }, [active, projects]);
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
  const installOnPsp = async () => {
    const overwrite = confirm(`Installer ${active} sur la PSP ? Un ancien EBOOT du même projet sera remplacé.`);
    if (!overwrite) return;
    setStatus("Compilation et installation…"); setConsoleOpen(true); setBuildReport(undefined); setDeploymentReport(undefined);
    try {
      if (dirty) { await saveProjectFile(active, activeFile, source); setDirty(false); }
      const outcome = await deployProject(active, pspRoot, true);
      setBuildReport(outcome.build);
      setDeploymentReport(outcome.deployment ?? undefined);
      setStatus(outcome.deployment?.summary ?? outcome.build.summary);
    } catch (error) { setStatus(`Erreur : ${error instanceof Error ? error.message : error}`); }
  };
  const safelyEjectPsp = async () => {
    if (!confirm("Éjecter cette PSP ? Attends la confirmation avant de débrancher le câble USB.")) return;
    await action("Éjection", async () => {
      const result = await ejectPsp(pspRoot);
      setPspRoot(""); setDeploymentReport(undefined);
      return result;
    });
  };
  const refreshFiles = async () => setFileEntries(await listProjectFiles(active));
  const openFile = async (entry: ProjectFileEntry) => {
    setSelectedPath(entry.path);
    if (entry.isDir) {
      setCollapsedFolders(current => {
        const next = new Set(current);
        if (next.has(entry.path)) next.delete(entry.path); else next.add(entry.path);
        return next;
      });
      return;
    }
    if (entry.path === activeFile) return;
    if (dirty && !confirm("Abandonner les modifications non enregistrées ?")) return;
    await action("Ouverture", async () => {
      const value = await readProjectFile(active, entry.path);
      setActiveFile(entry.path); setSource(value); setDirty(false);
      return entry.path;
    });
  };
  const selectedEntry = fileEntries.find(entry => entry.path === selectedPath);
  const creationParent = selectedEntry?.isDir
    ? selectedEntry.path
    : selectedPath.includes("/") ? selectedPath.slice(0, selectedPath.lastIndexOf("/")) : "";
  const addItem = async (isDir: boolean, name: string) => {
    await action(isDir ? "Création du dossier" : "Création du fichier", async () => {
      const path = await createProjectItem(active, creationParent, name, isDir);
      await refreshFiles();
      setSelectedPath(path);
      if (!isDir) {
        setActiveFile(path); setSource(""); setDirty(false);
      }
      return `${isDir ? "Dossier" : "Fichier"} ${path} créé`;
    });
  };
  const renameSelected = async (newName: string) => {
    if (!selectedEntry) return;
    if (!newName || newName === selectedEntry.name) return;
    if (dirty && (activeFile === selectedPath || activeFile.startsWith(`${selectedPath}/`)) && !confirm("Abandonner les modifications non enregistrées ?")) return;
    await action("Renommage", async () => {
      const destination = await renameProjectItem(active, selectedPath, newName);
      if (activeFile === selectedPath || activeFile.startsWith(`${selectedPath}/`)) {
        const nextFile = destination + activeFile.slice(selectedPath.length);
        setActiveFile(nextFile); setSource(await readProjectFile(active, nextFile)); setDirty(false);
      }
      setSelectedPath(destination); await refreshFiles();
      return `${destination} renommé`;
    });
  };
  const deleteSelected = async () => {
    if (!selectedEntry || !confirm(`Supprimer définitivement ${selectedEntry.path} ?`)) return;
    if (dirty && (activeFile === selectedPath || activeFile.startsWith(`${selectedPath}/`)) && !confirm("Les modifications non enregistrées seront perdues. Continuer ?")) return;
    await action("Suppression", async () => {
      const removedOpenFile = activeFile === selectedPath || activeFile.startsWith(`${selectedPath}/`);
      await deleteProjectItem(active, selectedPath); await refreshFiles();
      if (removedOpenFile) {
        const fallback = activeProject?.language === "lua" ? "script.lua" : "src/main.cpp";
        setActiveFile(fallback); setSelectedPath(fallback);
        setSource(await readProjectFile(active, fallback)); setDirty(false);
      } else setSelectedPath("");
      return `${selectedEntry.path} supprimé`;
    });
  };
  const visibleFileEntries = fileEntries.filter(entry => {
    const parts = entry.path.split("/");
    return parts.slice(0, -1).every((_, index) => !collapsedFolders.has(parts.slice(0, index + 1).join("/")));
  });
  const editorLanguage = (() => {
    const extension = activeFile.split(".").at(-1)?.toLowerCase();
    if (extension === "lua") return "lua";
    if (["c", "cc", "cpp", "h", "hpp"].includes(extension ?? "")) return "cpp";
    if (extension === "json") return "json";
    if (extension === "md") return "markdown";
    if (extension === "xml") return "xml";
    return "plaintext";
  })();
  const activeFileReadOnly = fileEntries.find(entry => entry.path === activeFile)?.readOnly ?? false;
  const activeProject = projects.find(project => project.name === active);
  const isLuaProject = activeProject?.language === "lua";
  const buildLabel = isLuaProject ? "Préparer" : "Compiler";
  const buildAction = isLuaProject ? "Préparation" : "Compilation";
  const canBuild = Boolean(active && (isLuaProject || environment?.pspdevReady));
  const compileActiveProject = async () => {
    setStatus(`${buildAction}…`);
    setBuildReport(undefined);
    setDeploymentReport(undefined);
    setConsoleOpen(true);
    try {
      if (dirty) {
        await saveProjectFile(active, activeFile, source);
        setDirty(false);
      }
      const report = await buildProject(active);
      setBuildReport(report);
      setStatus(report.summary);
    } catch (error) {
      setStatus(`Erreur : ${error instanceof Error ? error.message : error}`);
    }
  };
  const openDiagnostic = async (diagnostic: BuildDiagnostic) => {
    if (dirty && diagnostic.file !== activeFile && !confirm("Abandonner les modifications non enregistrées ?")) return;
    pendingPosition.current = { lineNumber: diagnostic.line, column: diagnostic.column };
    if (diagnostic.file !== activeFile) {
      const value = await readProjectFile(active, diagnostic.file);
      setActiveFile(diagnostic.file); setSelectedPath(diagnostic.file); setSource(value); setDirty(false);
    } else if (editorRef.current) {
      editorRef.current.setPosition(pendingPosition.current);
      editorRef.current.revealPositionInCenter(pendingPosition.current);
      editorRef.current.focus();
      pendingPosition.current = undefined;
    }
  };
  const mountEditor = (instance: MonacoEditor.IStandaloneCodeEditor) => {
    editorRef.current = instance;
    if (pendingPosition.current) {
      instance.setPosition(pendingPosition.current);
      instance.revealPositionInCenter(pendingPosition.current);
      instance.focus();
      pendingPosition.current = undefined;
    }
  };
  return <div className="shell">
    <aside className="sidebar">
      <div className="brand"><img src="/psp-reborn-logo.png" alt=""/><span><b>PSP</b> Reborn</span></div>
      <button className="new" onClick={() => setShowCreate(true)}>＋ Nouveau jeu</button>
      <button className={view === "code" ? "nav active" : "nav"} onClick={() => setView("code")}>⌨ Code</button>
      <button className={view === "pbp" ? "nav active" : "nav"} onClick={() => setView("pbp")}>📦 PBP Studio</button>
      <button className={view === "help" ? "nav active" : "nav"} onClick={() => setView("help")}>❔ Aide & exemples</button>
      <button className={view === "environment" ? "nav active" : "nav"} onClick={() => setView("environment")}>⚙ Environnement</button>
      <h3>PROJETS</h3>
      <div className="project-list">{projects.map(p => <button className={p.name === active ? "project active" : "project"} onClick={() => { if (!dirty || confirm("Abandonner les modifications non enregistrées ?")) { setActive(p.name); setView("code"); } }} key={p.name}><span>{p.language === "lua" ? "🌙" : "🎮"} {p.name}</span><small>{p.language === "lua" ? p.runtimeVersion : "C++17"}</small></button>)}</div>
      {active && <section className="explorer">
        <div className="explorer-head"><h3>EXPLORATEUR</h3><div>
          <button title="Nouveau fichier" onClick={() => { setFileDialogName(""); setFileDialog("file"); }}>＋F</button>
          <button title="Nouveau dossier" onClick={() => { setFileDialogName(""); setFileDialog("folder"); }}>＋D</button>
          <button title="Renommer" disabled={!selectedEntry} onClick={() => { setFileDialogName(selectedEntry?.name ?? ""); setFileDialog("rename"); }}>✎</button>
          <button title="Supprimer" disabled={!selectedEntry} onClick={deleteSelected}>⌫</button>
        </div></div>
        <div className="file-tree">{visibleFileEntries.map(entry => <button
          className={`file-entry ${entry.path === selectedPath ? "selected" : ""} ${entry.path === activeFile ? "open" : ""}`}
          style={{ paddingLeft: 8 + entry.depth * 14 }}
          title={entry.path}
          onClick={() => openFile(entry)}
          key={entry.path}
        ><span>{entry.isDir ? (collapsedFolders.has(entry.path) ? "▸ 📁" : "▾ 📂") : entry.name.endsWith(".lua") ? "◐" : "◇"}</span><em>{entry.name}</em></button>)}</div>
      </section>}
      <div className="locked">🔒 Mode sécurisé<br/><small>Fichiers confinés au projet actif</small></div>
    </aside>
    <main className={view === "code" ? (consoleOpen ? "code-main" : "code-main console-closed") : "tool-main"}>{view === "code" ? <>
      <header><div><strong>{active || "Aucun projet"}</strong><span>{dirty ? `● ${activeFile} modifié` : `${activeFile}${activeFileReadOnly ? " · lecture seule" : ""}`}</span></div><button className="environment" onClick={() => setView("environment")}><span className={environment?.pspdevReady ? "ok" : "missing"}>● PSPDEV</span><span className={environment?.ppssppReady ? "ok" : "missing"}>● PPSSPP</span></button><div className="toolbar"><button disabled={!activeFile || !dirty || activeFileReadOnly} onClick={() => action("Sauvegarde", async () => { await saveProjectFile(active, activeFile, source); setDirty(false); })}>Enregistrer</button><button disabled={!active || !environment?.ppssppReady} onClick={() => action("Test", () => runInPpsspp(active))}>Tester</button><button className="build" disabled={!canBuild} title={isLuaProject ? activeProject?.runtimeVersion : environment?.pspdevVersion} onClick={compileActiveProject}>{buildLabel}</button></div></header>
      {activeFile ? <div className="editor-host"><Editor key={activeFile} height="100%" language={editorLanguage} theme="vs-dark" value={source} onMount={mountEditor} onChange={v => { if (!activeFileReadOnly) { setSource(v ?? ""); setDirty(true); } }} options={{ minimap:{enabled:false}, fontSize:14, automaticLayout:true, tabSize:4, readOnly:activeFileReadOnly }}/></div>
      : <div className="welcome"><div><span>🎮</span><h1>Crée ton premier jeu PSP</h1><p>Un projet C++ prêt à compiler et à lancer sur PPSSPP en quelques secondes.</p><button className="build" onClick={() => setShowCreate(true)}>Créer un jeu</button></div></div>}
      <footer><div className="deploy-bar"><b>Installation PSP</b><input readOnly value={pspRoot} placeholder="Aucun volume sélectionné"/><button onClick={choosePsp}>Choisir</button><button disabled={!pspRoot || !active} onClick={installOnPsp}>Compiler & installer</button><button disabled={!pspRoot} onClick={safelyEjectPsp}>Éjecter</button><button className="console-toggle" onClick={() => setConsoleOpen(open => !open)}>{consoleOpen ? "Masquer la console" : "Afficher la console"}</button></div>
        {consoleOpen && <section className="build-console"><div className="console-title"><b>Sortie</b><span className={buildReport ? (buildReport.success ? "success" : "failure") : ""}>{status}</span>{buildReport && <small>{buildReport.sourceCount} source(s)</small>}</div>{deploymentReport && <div className="deployment-result"><b>✓ Copie vérifiée</b><span>{deploymentReport.files} fichier(s) · {(deploymentReport.bytes / 1024).toFixed(1)} Kio</span><code title={deploymentReport.ebootSha256}>SHA-256 {deploymentReport.ebootSha256.slice(0, 16)}…</code></div>}{buildReport?.diagnostics.length ? <div className="diagnostic-list">{buildReport.diagnostics.map((diagnostic, index) => <button className={diagnostic.severity} onClick={() => openDiagnostic(diagnostic)} key={`${diagnostic.file}:${diagnostic.line}:${index}`}><b>{diagnostic.severity === "error" ? "Erreur" : diagnostic.severity === "warning" ? "Avertissement" : "Note"}</b><span>{diagnostic.file}:{diagnostic.line}:{diagnostic.column}</span><em>{diagnostic.message}</em></button>)}</div> : null}<pre>{buildReport?.output ?? status}</pre></section>}
      </footer></> : view === "pbp" ? <PbpStudio/> : view === "help" ? <Help onNewProject={language => { setNewLanguage(language); setNewTemplate(templates[language][0].id); if (language === "lua") setNewRuntime("lpp-r163"); setShowCreate(true); }}/> : <EnvironmentPanel
        environment={environment}
        refreshing={environmentRefreshing}
        onRefresh={refreshEnvironment}
        onUsePsp={path => { setPspRoot(path); setStatus(`PSP sélectionnée : ${path}`); setView("code"); }}
      />}
    </main>
    {fileDialog && <div className="modal-backdrop" onMouseDown={() => setFileDialog(null)}><form className="modal file-dialog" onSubmit={e => { e.preventDefault(); const name = fileDialogName.trim(); if (!name) return; const operation = fileDialog; setFileDialog(null); if (operation === "rename") renameSelected(name); else addItem(operation === "folder", name); }} onMouseDown={e => e.stopPropagation()}><h2>{fileDialog === "rename" ? "Renommer" : fileDialog === "folder" ? "Nouveau dossier" : "Nouveau fichier"}</h2><label>Nom<input autoFocus value={fileDialogName} maxLength={80} onChange={e => setFileDialogName(e.target.value)} placeholder={fileDialog === "folder" ? "scripts" : "player.cpp"}/></label><p>Emplacement : {creationParent || "racine du projet"}</p><div className="modal-actions"><button type="button" onClick={() => setFileDialog(null)}>Annuler</button><button className="build" type="submit">{fileDialog === "rename" ? "Renommer" : "Créer"}</button></div></form></div>}
    {showCreate && <div className="modal-backdrop" onMouseDown={() => setShowCreate(false)}><form className="modal project-wizard" onSubmit={e => { e.preventDefault(); create(); }} onMouseDown={e => e.stopPropagation()}><h2>Nouveau projet PSP</h2><label>Nom du projet<input autoFocus value={newName} maxLength={32} onChange={e => setNewName(e.target.value)} placeholder="MonJeu"/></label><h3>Langage</h3><div className="choice-grid">{languages.map(language => <button type="button" className={newLanguage === language.id ? "choice selected" : "choice"} onClick={() => { setNewLanguage(language.id); setNewTemplate(templates[language.id][0].id); if (language.id === "lua") setNewRuntime("lpp-r163"); }} key={language.id}><b>{language.name}</b><small>{language.description}</small><em>{language.badge}</em></button>)}</div>{newLanguage === "lua" && <label>Version LuaPlayer<select value={newRuntime} onChange={e => setNewRuntime(e.target.value)}>{luaRuntimes.map(runtime => <option value={runtime.id} key={runtime.id}>{runtime.name} — {runtimeStatusLabel[runtime.status]}</option>)}</select></label>}<h3>Modèle d’exemple</h3><div className="template-list">{templates[newLanguage].map(item => <button type="button" className={newTemplate === item.id ? "template-choice selected" : "template-choice"} onClick={() => setNewTemplate(item.id)} key={item.id}><b>{item.name}</b><small>{item.description}</small><span>{item.features.join(" · ")}</span></button>)}</div><p>{newLanguage === "lua" && newRuntime !== "lpp-r163" ? "Cette version est cataloguée, mais son binaire original doit encore être récupéré avant l’exécution." : "Le projet utilisera uniquement les composants validés pour ce modèle."}</p><div className="modal-actions"><button type="button" onClick={() => setShowCreate(false)}>Annuler</button><button className="build" type="submit">Créer le projet</button></div></form></div>}
  </div>;
}
