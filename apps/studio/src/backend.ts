import { invoke } from "@tauri-apps/api/core";

import type { LanguageId } from "./projectCatalog";

export type Project = { name: string; language: LanguageId; template: string; runtimeVersion?: string; kernelMode: false };
export type EnvironmentStatus = { pspdevReady: boolean; pspdevVersion?: string; ppssppReady: boolean };
const inTauri = "__TAURI_INTERNALS__" in window;
const demoProjects = new Map<string, string>();
demoProjects.set("PremierJeu", `#include <pspkernel.h>\n#include <pspdebug.h>\n\nPSP_MODULE_INFO("PremierJeu", PSP_MODULE_USER, 1, 0);\n\nint main() {\n    pspDebugScreenInit();\n    pspDebugScreenPrintf("Bonjour PSP !\\n");\n    sceKernelSleepThread();\n    return 0;\n}\n`);

export async function listProjects(): Promise<Project[]> {
  return inTauri ? invoke("list_projects") : [...demoProjects.keys()].map(name => ({ name, language: "cpp", template: "minimal", kernelMode: false }));
}
export async function getEnvironmentStatus(): Promise<EnvironmentStatus> {
  return inTauri
    ? invoke("environment_status")
    : { pspdevReady: true, pspdevVersion: "Mode aperçu", ppssppReady: false };
}
export async function createProject(name: string, language: LanguageId, template: string, runtimeVersion?: string): Promise<Project> {
  if (inTauri) return invoke("create_project", { name, language, template, runtimeVersion });
  if (demoProjects.has(name)) throw new Error("Ce projet existe déjà");
  demoProjects.set(name, demoProjects.get("PremierJeu")!.replaceAll("PremierJeu", name));
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
export async function buildProject(project: string): Promise<string> { return invoke("build_project", { project }); }
export async function runInPpsspp(project: string): Promise<string> { return invoke("run_project_ppsspp", { project }); }
export async function deployProject(project: string, pspRoot: string, overwrite: boolean): Promise<string> { return invoke("deploy_project", { project, pspRoot, overwrite }); }
