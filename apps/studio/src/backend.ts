import { invoke } from "@tauri-apps/api/core";

import type { LanguageId } from "./projectCatalog";

export type Project = { name: string; language: LanguageId; template: string; runtimeVersion?: string; kernelMode: false };
export type EnvironmentCheck = { id: string; label: string; ready: boolean; required: boolean; detail: string; path?: string };
export type EnvironmentStatus = { pspdevReady: boolean; pspdevVersion?: string; ppssppReady: boolean; pspMounts: string[]; checks: EnvironmentCheck[] };
export type BuildDiagnostic = { severity: "error" | "warning" | "note"; file: string; line: number; column: number; message: string };
export type BuildReport = { success: boolean; summary: string; output: string; diagnostics: BuildDiagnostic[]; sourceCount: number };
export type ProjectFileEntry = { path: string; name: string; isDir: boolean; depth: number; readOnly: boolean };
const inTauri = "__TAURI_INTERNALS__" in window;
const demoProjects = new Map<string, string>();
const demoFiles = new Map<string, Map<string, string>>();
demoProjects.set("PremierJeu", `#include <pspkernel.h>\n#include <pspdebug.h>\n\nPSP_MODULE_INFO("PremierJeu", PSP_MODULE_USER, 1, 0);\n\nint main() {\n    pspDebugScreenInit();\n    pspDebugScreenPrintf("Bonjour PSP !\\n");\n    sceKernelSleepThread();\n    return 0;\n}\n`);
demoFiles.set("PremierJeu", new Map([["src/main.cpp", demoProjects.get("PremierJeu")!], ["Makefile", "TARGET = game\n"]]));

export async function listProjects(): Promise<Project[]> {
  return inTauri ? invoke("list_projects") : [...demoProjects.keys()].map(name => ({ name, language: "cpp", template: "minimal", kernelMode: false }));
}
export async function getEnvironmentStatus(): Promise<EnvironmentStatus> {
  return inTauri
    ? invoke("environment_status")
    : { pspdevReady: true, pspdevVersion: "Mode aperçu", ppssppReady: false, pspMounts: [], checks: [
      { id: "pspdev", label: "PSPDEV", ready: true, required: true, detail: "Toolchain simulée pour l’aperçu" },
      { id: "make", label: "Moteur de build", ready: true, required: true, detail: "Make disponible" },
      { id: "ppsspp", label: "PPSSPP", ready: false, required: false, detail: "Application native requise" },
      { id: "psp", label: "PSP USB", ready: false, required: false, detail: "Aucun volume détecté" },
    ] };
}
export async function createProject(name: string, language: LanguageId, template: string, runtimeVersion?: string): Promise<Project> {
  if (inTauri) return invoke("create_project", { name, language, template, runtimeVersion });
  if (demoProjects.has(name)) throw new Error("Ce projet existe déjà");
  demoProjects.set(name, demoProjects.get("PremierJeu")!.replaceAll("PremierJeu", name));
  const mainPath = language === "lua" ? "script.lua" : "src/main.cpp";
  demoFiles.set(name, new Map([[mainPath, demoProjects.get(name)!]]));
  return { name, language, template, runtimeVersion, kernelMode: false };
}
export async function readSource(project: string): Promise<string> {
  if (!inTauri) return demoProjects.get(project) ?? "";
  return invoke("read_main_source", { project });
}
export async function saveSource(project: string, source: string): Promise<void> {
  if (!inTauri) { demoProjects.set(project, source); return; }
  return invoke("save_main_source", { project, source });
}
export async function listProjectFiles(project: string): Promise<ProjectFileEntry[]> {
  if (inTauri) return invoke("list_project_files", { project });
  const files = [...(demoFiles.get(project)?.keys() ?? [])].sort();
  const directories = new Set<string>();
  for (const path of files) {
    const parts = path.split("/");
    for (let index = 1; index < parts.length; index++) directories.add(parts.slice(0, index).join("/"));
  }
  return [...directories].map(path => ({ path, name: path.split("/").at(-1)!, isDir: true, depth: path.split("/").length - 1, readOnly: false }))
    .concat(files.map(path => ({ path, name: path.split("/").at(-1)!, isDir: false, depth: path.split("/").length - 1, readOnly: path.endsWith("Makefile") })))
    .sort((left, right) => left.path.localeCompare(right.path));
}
export async function readProjectFile(project: string, path: string): Promise<string> {
  if (inTauri) return invoke("read_project_file", { project, path });
  const value = demoFiles.get(project)?.get(path);
  if (value === undefined) throw new Error("Fichier introuvable");
  return value;
}
export async function saveProjectFile(project: string, path: string, source: string): Promise<void> {
  if (inTauri) return invoke("save_project_file", { project, path, source });
  demoFiles.get(project)?.set(path, source);
}
export async function createProjectItem(project: string, parent: string, name: string, isDir: boolean): Promise<string> {
  if (inTauri) return invoke("create_project_item", { project, parent, name, isDir });
  const path = parent ? `${parent}/${name}` : name;
  if (!isDir) demoFiles.get(project)?.set(path, "");
  return path;
}
export async function renameProjectItem(project: string, path: string, newName: string): Promise<string> {
  if (inTauri) return invoke("rename_project_item", { project, path, newName });
  const files = demoFiles.get(project)!;
  const parent = path.includes("/") ? path.slice(0, path.lastIndexOf("/")) : "";
  const destination = parent ? `${parent}/${newName}` : newName;
  for (const [key, value] of [...files]) {
    if (key === path || key.startsWith(`${path}/`)) {
      files.delete(key);
      files.set(destination + key.slice(path.length), value);
    }
  }
  return destination;
}
export async function deleteProjectItem(project: string, path: string): Promise<void> {
  if (inTauri) return invoke("delete_project_item", { project, path });
  const files = demoFiles.get(project)!;
  for (const key of [...files.keys()]) if (key === path || key.startsWith(`${path}/`)) files.delete(key);
}
export async function buildProject(project: string): Promise<BuildReport> {
  if (inTauri) return invoke("build_project", { project });
  return { success: true, summary: "EBOOT.PBP prêt · aperçu", output: "Compilation simulée dans le navigateur.", diagnostics: [], sourceCount: 1 };
}
export async function runInPpsspp(project: string): Promise<string> { return invoke("run_project_ppsspp", { project }); }
export async function deployProject(project: string, pspRoot: string, overwrite: boolean): Promise<string> { return invoke("deploy_project", { project, pspRoot, overwrite }); }
