#!/usr/bin/env node
import { access, cp, mkdir, readFile, stat, writeFile } from "node:fs/promises";
import { constants } from "node:fs";
import { spawnSync } from "node:child_process";
import { delimiter, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const template = join(root, "templates", "cpp-minimal");
const managedPspdev = join(process.env.HOME ?? "", ".pspdev", "v20260701");
const [command = "help", ...args] = process.argv.slice(2);
const exists = async path => access(path, constants.F_OK).then(() => true, () => false);
const has = name => spawnSync("sh", ["-c", `command -v ${name}`], { stdio: "ignore" }).status === 0;

function run(program, params, options = {}) {
  const result = spawnSync(program, params, { stdio: "inherit", ...options });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${program} a échoué (${result.status}).`);
}

async function doctor() {
  const managed = await exists(join(managedPspdev, "bin", "psp-g++"));
  const checks = [
    ["PSPDEV géré v20260701", managed], ["psp-g++ dans PATH", has("psp-g++")],
    ["make", has("make")], ["docker", has("docker")], ["podman", has("podman")],
  ];
  console.log("Environnement PSP Reborn\n");
  checks.forEach(([name, ok]) => console.log(`${ok ? "✓" : "✗"} ${name}`));
  if (!checks[0][1] && !checks[1][1] && !checks[3][1] && !checks[4][1]) {
    console.log("\nCompilation indisponible : installe PSPDEV ou Docker/Podman.");
  } else console.log("\nUne méthode de compilation est disponible.");
}

async function createProject(name) {
  if (!name || !/^[a-zA-Z0-9_-]+$/.test(name)) throw new Error("Nom requis : lettres, chiffres, _ ou -.");
  const destination = resolve("games", name);
  if (await exists(destination)) throw new Error(`Le projet existe déjà : ${destination}`);
  await mkdir(dirname(destination), { recursive: true });
  await cp(template, destination, { recursive: true });
  for (const file of ["Makefile", "psp-project.json"]) {
    const path = join(destination, file);
    await writeFile(path, (await readFile(path, "utf8")).replaceAll("{{PROJECT_NAME}}", name));
  }
  console.log(`Projet créé : ${destination}`);
}

async function build(projectArg = ".") {
  const project = resolve(projectArg);
  if (!(await exists(join(project, "Makefile")))) throw new Error("Makefile PSP introuvable.");
  if (await exists(join(managedPspdev, "bin", "psp-g++"))) {
    run("make", ["clean", "all"], { cwd: project, env: { ...process.env, PSPDEV: managedPspdev, PATH: `${join(managedPspdev, "bin")}${delimiter}${process.env.PATH ?? ""}` } });
  } else if (has("psp-g++") && has("psp-config")) run("make", ["clean", "all"], { cwd: project });
  else {
    const engine = has("docker") ? "docker" : has("podman") ? "podman" : null;
    if (!engine) throw new Error("Installe PSPDEV ou Docker/Podman pour compiler.");
    run(engine, ["run", "--rm", "-v", `${project}:/src`, "-w", "/src", "pspdev/pspdev:v20260701", "make", "clean", "all"]);
  }
  const eboot = join(project, "EBOOT.PBP");
  if (!(await exists(eboot)) || (await stat(eboot)).size < 40) throw new Error("La compilation n’a pas produit d’EBOOT.PBP valide.");
  console.log(`EBOOT prêt : ${eboot}`);
}

async function deploy(projectArg, pspRootArg) {
  if (!projectArg || !pspRootArg) throw new Error("Usage : deploy <projet> <racine-PSP>");
  const project = resolve(projectArg);
  const pspRoot = resolve(pspRootArg);
  const gameRoot = join(pspRoot, "PSP", "GAME");
  if (!(await exists(gameRoot))) throw new Error(`Ce volume ne ressemble pas à une PSP : ${gameRoot} est absent.`);
  const config = JSON.parse(await readFile(join(project, "psp-project.json"), "utf8"));
  const folder = config.deployFolder;
  if (!/^[A-Z0-9_-]{1,32}$/i.test(folder)) throw new Error("deployFolder invalide.");
  const eboot = join(project, "EBOOT.PBP");
  if (!(await exists(eboot))) throw new Error("EBOOT.PBP absent : compile le projet d’abord.");
  const destination = join(gameRoot, folder);
  await mkdir(destination, { recursive: true });
  await cp(eboot, join(destination, "EBOOT.PBP"));
  if (await exists(join(project, "assets"))) await cp(join(project, "assets"), join(destination, "assets"), { recursive: true });
  console.log(`Installé sur PSP : ${destination}`);
}

function help() {
  console.log(`PSP Reborn CLI

  doctor                         Vérifier la chaîne PSP
  create <nom>                   Créer un jeu C++
  build <dossier>                Produire EBOOT.PBP
  deploy <dossier> <racine-PSP> Copier dans PSP/GAME
  run <dossier> <racine-PSP>    Compiler puis installer`);
}

try {
  if (command === "doctor") await doctor();
  else if (command === "create") await createProject(args[0]);
  else if (command === "build") await build(args[0]);
  else if (command === "deploy") await deploy(args[0], args[1]);
  else if (command === "run") { await build(args[0]); await deploy(args[0], args[1]); }
  else help();
} catch (error) { console.error(`Erreur : ${error.message}`); process.exitCode = 1; }
