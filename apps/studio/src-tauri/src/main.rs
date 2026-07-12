use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};
use tauri::Manager;

const MAX_SOURCE_SIZE: usize = 2_000_000;
const CONFIG_NAME: &str = "psp-project.json";

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Project {
    name: String,
    language: String,
    template: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    runtime_version: Option<String>,
    kernel_mode: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnvironmentStatus {
    pspdev_ready: bool,
    pspdev_version: Option<String>,
    ppsspp_ready: bool,
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 32
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn projects_root(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .document_dir()
        .map(|path| path.join("PSP Reborn").join("Games"))
        .map_err(|error| error.to_string())
}

fn managed_pspdev(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .home_dir()
        .map(|path| path.join(".pspdev").join("v20260701"))
        .map_err(|error| error.to_string())
}

fn toolchain_bin(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let root = managed_pspdev(app)?;
    if !root.join("bin/psp-g++").is_file() || !root.join("bin/psp-config").is_file() {
        return Err("La toolchain PSPDEV gérée est absente".into());
    }
    Ok(root.join("bin"))
}

#[tauri::command]
fn environment_status(app: tauri::AppHandle) -> Result<EnvironmentStatus, String> {
    let pspdev_ready = toolchain_bin(&app).is_ok();
    let ppsspp_ready = Path::new("/Applications/PPSSPP.app").is_dir()
        || Path::new("/Applications/PPSSPPSDL.app").is_dir();
    Ok(EnvironmentStatus {
        pspdev_ready,
        pspdev_version: pspdev_ready.then(|| "v20260701 / GCC 15.2.0".into()),
        ppsspp_ready,
    })
}

fn project_dir(app: &tauri::AppHandle, name: &str) -> Result<PathBuf, String> {
    if !valid_name(name) {
        return Err("Nom de projet invalide".into());
    }
    Ok(projects_root(app)?.join(name))
}

fn require_safe_project(app: &tauri::AppHandle, name: &str) -> Result<PathBuf, String> {
    let root = projects_root(app)?;
    let directory = project_dir(app, name)?;
    let root = root
        .canonicalize()
        .map_err(|_| "Le dossier des projets est inaccessible".to_string())?;
    let directory = directory
        .canonicalize()
        .map_err(|_| "Projet introuvable".to_string())?;
    if !directory.starts_with(&root) || directory == root {
        return Err("Le projet sort du dossier autorisé".into());
    }
    let metadata = fs::symlink_metadata(&directory).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("Projet non sûr ou invalide".into());
    }
    Ok(directory)
}

fn write_atomic(path: &Path, contents: &[u8]) -> Result<(), String> {
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, contents).map_err(|error| error.to_string())?;
    fs::rename(&temporary, path).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        error.to_string()
    })
}

fn validate_pbp(path: &Path) -> Result<(), String> {
    let length = fs::metadata(path).map_err(|error| error.to_string())?.len();
    if length < 40 {
        return Err("EBOOT.PBP trop petit".into());
    }
    let mut header = [0_u8; 40];
    File::open(path)
        .and_then(|mut file| file.read_exact(&mut header))
        .map_err(|error| error.to_string())?;
    if &header[0..4] != b"\0PBP" {
        return Err("Signature EBOOT.PBP invalide".into());
    }
    let mut previous = 40_u32;
    for index in 0..8 {
        let position = 8 + index * 4;
        let offset = u32::from_le_bytes(header[position..position + 4].try_into().unwrap());
        if offset < previous || u64::from(offset) > length {
            return Err("Table des sections EBOOT.PBP invalide".into());
        }
        previous = offset;
    }
    Ok(())
}

fn sha256(path: &Path) -> Result<[u8; 32], String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(digest.finalize().into())
}

fn copy_verified(source: &Path, destination: &Path, overwrite: bool) -> Result<(), String> {
    if destination.exists() && !overwrite {
        return Err(format!(
            "{} existe déjà. Confirme son remplacement.",
            destination
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        ));
    }
    let temporary = destination.with_extension("copying");
    fs::copy(source, &temporary).map_err(|error| error.to_string())?;
    if sha256(source)? != sha256(&temporary)? {
        let _ = fs::remove_file(&temporary);
        return Err("La vérification de la copie PSP a échoué".into());
    }
    if destination.exists() {
        fs::remove_file(destination).map_err(|error| error.to_string())?;
    }
    fs::rename(&temporary, destination).map_err(|error| error.to_string())
}

fn copy_safe_tree(source: &Path, destination: &Path, overwrite: bool) -> Result<(), String> {
    if !source.exists() {
        return Ok(());
    }
    fs::create_dir_all(destination).map_err(|error| error.to_string())?;
    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let kind = entry.file_type().map_err(|error| error.to_string())?;
        if kind.is_symlink() {
            return Err("Les liens symboliques sont interdits dans les ressources".into());
        }
        let target = destination.join(entry.file_name());
        if kind.is_dir() {
            copy_safe_tree(&entry.path(), &target, overwrite)?;
        } else if kind.is_file() {
            copy_verified(&entry.path(), &target, overwrite)?;
        }
    }
    Ok(())
}

fn valid_project_choice(language: &str, template: &str, runtime: Option<&str>) -> bool {
    const CPP_TEMPLATES: &[&str] = &["hello", "controls", "graphics", "timer"];
    const LUA_TEMPLATES: &[&str] = &["hello", "controls", "image", "sprite"];
    const LUA_RUNTIMES: &[&str] = &[
        "hm-v1.7",
        "hm-v2.0.4",
        "hm-v3",
        "hm-v4.0",
        "hm-v6",
        "hm-v6.6",
        "hm-v6.9-beta",
        "hm-v7-rc1",
        "hm-v8",
        "hm-v8.1-beta",
        "lpp-r163",
    ];
    match language {
        "cpp" => CPP_TEMPLATES.contains(&template) && runtime.is_none(),
        "lua" => {
            LUA_TEMPLATES.contains(&template)
                && runtime.is_some_and(|id| LUA_RUNTIMES.contains(&id))
        }
        _ => false,
    }
}

fn default_project(
    name: String,
    language: String,
    template: String,
    runtime_version: Option<String>,
) -> Project {
    Project {
        name,
        language,
        template,
        runtime_version,
        kernel_mode: false,
    }
}

fn cpp_source_template(name: &str, template: &str) -> String {
    let body = match template {
        "controls" => {
            r#"    pspDebugScreenPrintf("Utilise les boutons de la PSP.\n");
    while (true) {
        SceCtrlData pad{};
        sceCtrlPeekBufferPositive(&pad, 1);
        pspDebugScreenSetXY(0, 3);
        pspDebugScreenPrintf("X:%s  O:%s  Analogique:%3d,%3d   ",
            pad.Buttons & PSP_CTRL_CROSS ? "oui" : "non",
            pad.Buttons & PSP_CTRL_CIRCLE ? "oui" : "non", pad.Lx, pad.Ly);
        sceDisplayWaitVblankStart();
    }"#
        }
        "graphics" => {
            r#"    sceGuInit();
    sceGuStart(GU_DIRECT, displayList);
    sceGuDrawBuffer(GU_PSM_8888, (void*)0, 512);
    sceGuDispBuffer(480, 272, (void*)0x88000, 512);
    sceGuDepthBuffer((void*)0x110000, 512);
    sceGuFinish(); sceGuSync(0, 0); sceDisplayWaitVblankStart(); sceGuDisplay(GU_TRUE);
    while (true) {
        sceGuStart(GU_DIRECT, displayList);
        sceGuClearColor(0xff301810); sceGuClear(GU_COLOR_BUFFER_BIT);
        sceGuFinish(); sceGuSync(0, 0); sceDisplayWaitVblankStart(); sceGuSwapBuffers();
    }"#
        }
        "timer" => {
            r#"    u64 tickResolution = sceRtcGetTickResolution();
    u64 previous = 0; sceRtcGetCurrentTick(&previous);
    unsigned frames = 0;
    while (true) {
        u64 current = 0; sceRtcGetCurrentTick(&current);
        float delta = float(current - previous) / float(tickResolution); previous = current;
        pspDebugScreenSetXY(0, 3);
        pspDebugScreenPrintf("Image: %u  delta: %.4f s   ", frames++, delta);
        sceDisplayWaitVblankStart();
    }"#
        }
        _ => {
            r#"    pspDebugScreenPrintf("Bonjour depuis PSP Reborn Studio !\n");
    pspDebugScreenPrintf("Ce code fonctionne dans PPSSPP et sur PSP.\n");
    while (true) sceDisplayWaitVblankStart();"#
        }
    };
    let includes = match template {
        "controls" => "#include <pspctrl.h>\n#include <pspdisplay.h>",
        "graphics" => "#include <pspdisplay.h>\n#include <pspgu.h>\n\nstatic unsigned int __attribute__((aligned(16))) displayList[262144];",
        "timer" => "#include <pspdisplay.h>\n#include <psprtc.h>",
        _ => "#include <pspdisplay.h>",
    };
    format!("#include <pspkernel.h>\n#include <pspdebug.h>\n{includes}\n\nPSP_MODULE_INFO(\"{name}\", PSP_MODULE_USER, 1, 0);\nPSP_MAIN_THREAD_ATTR(PSP_THREAD_ATTR_USER | PSP_THREAD_ATTR_VFPU);\n\nint main() {{\n    pspDebugScreenInit();\n{body}\n    return 0;\n}}\n")
}

fn lua_source_template(template: &str) -> &'static str {
    match template {
        "controls" => "x = 20\nwhile true do\n    pad = Controls.read()\n    if pad:left() then x = x - 2 end\n    if pad:right() then x = x + 2 end\n    screen:clear(Color.new(20, 24, 40))\n    screen:fillRect(x, 110, 32, 32, Color.new(130, 90, 255))\n    screen.flip()\n    screen.waitVblankStart()\nend\n",
        "image" => "image = Image.createEmpty(160, 100)\nimage:clear(Color.new(120, 70, 230))\nwhile true do\n    screen:clear()\n    screen:blit(160, 86, image)\n    screen:print(175, 125, \"Image Lua\", Color.new(255,255,255))\n    screen.flip()\n    screen.waitVblankStart()\nend\n",
        "sprite" => "sprite = Image.createEmpty(32, 32)\nsprite:clear(Color.new(130, 90, 255))\nx, y = 220, 120\nwhile true do\n    pad = Controls.read()\n    if pad:left() then x = x - 2 end\n    if pad:right() then x = x + 2 end\n    if pad:up() then y = y - 2 end\n    if pad:down() then y = y + 2 end\n    screen:clear()\n    screen:blit(x, y, sprite)\n    screen.flip()\n    screen.waitVblankStart()\nend\n",
        _ => "while true do\n    screen:clear(Color.new(24, 18, 62))\n    screen:fillRect(105, 70, 270, 132, Color.new(105, 70, 215))\n    screen:fillRect(125, 90, 230, 92, Color.new(18, 22, 36))\n    screen:fillRect(145, 115, 190, 42, Color.new(145, 105, 255))\n    screen.flip()\n    screen.waitVblankStart()\nend\n",
    }
}

fn makefile_template(name: &str, template: &str) -> String {
    let libraries = match template {
        "graphics" => "LIBS = -lpspgu\n",
        "timer" => "LIBS = -lpsprtc\n",
        _ => "",
    };
    format!(
        "TARGET = game\nOBJS = src/main.o\nCFLAGS = -O2 -G0 -Wall\nCXXFLAGS = $(CFLAGS) -std=c++17 -fno-exceptions -fno-rtti\nASFLAGS = $(CFLAGS)\n{libraries}EXTRA_TARGETS = EBOOT.PBP\nPSP_EBOOT_TITLE = {name}\nPSPSDK := $(shell psp-config --pspsdk-path)\ninclude $(PSPSDK)/lib/build.mak\n"
    )
}

fn prepare_lua_project(resources: &Path, directory: &Path, runtime: &str) -> Result<(), String> {
    if runtime != "lpp-r163" {
        return Err(format!(
            "Le runtime {runtime} est référencé mais son binaire original reste à récupérer"
        ));
    }
    let runtime_dir = resources.join("resources/runtimes/lpp-r163");
    for file in ["EBOOT.PBP", "lpp.ini", "License.txt"] {
        let source = runtime_dir.join(file);
        if !source.is_file() {
            return Err("Les fichiers LuaPlayer Plus r163 sont absents de l’application".into());
        }
        fs::copy(source, directory.join(file)).map_err(|error| error.to_string())?;
    }
    validate_pbp(&directory.join("EBOOT.PBP"))
}

#[tauri::command]
fn list_projects(app: tauri::AppHandle) -> Result<Vec<Project>, String> {
    let root = projects_root(&app)?;
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let mut projects = Vec::new();
    for entry in fs::read_dir(root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_dir()
            || !valid_name(&name)
        {
            continue;
        }
        let config = entry.path().join(CONFIG_NAME);
        if let Ok(contents) = fs::read_to_string(config) {
            if let Ok(project) = serde_json::from_str::<Project>(&contents) {
                if project.name == name
                    && !project.kernel_mode
                    && valid_project_choice(
                        &project.language,
                        &project.template,
                        project.runtime_version.as_deref(),
                    )
                {
                    projects.push(project);
                }
            }
        }
    }
    projects.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(projects)
}

#[tauri::command]
fn create_project(
    app: tauri::AppHandle,
    name: String,
    language: String,
    template: String,
    runtime_version: Option<String>,
) -> Result<Project, String> {
    if !valid_project_choice(&language, &template, runtime_version.as_deref()) {
        return Err("Combinaison de langage, runtime et modèle non autorisée".into());
    }
    let root = projects_root(&app)?;
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let directory = project_dir(&app, &name)?;
    if directory.exists() {
        return Err("Ce projet existe déjà".into());
    }

    let temporary = root.join(format!(".{name}.creating"));
    if temporary.exists() {
        fs::remove_dir_all(&temporary).map_err(|error| error.to_string())?;
    }
    let result = (|| -> Result<Project, String> {
        fs::create_dir(&temporary).map_err(|error| error.to_string())?;
        fs::create_dir(temporary.join("src")).map_err(|error| error.to_string())?;
        fs::create_dir(temporary.join("assets")).map_err(|error| error.to_string())?;
        if language == "cpp" {
            fs::write(
                temporary.join("src/main.cpp"),
                cpp_source_template(&name, &template),
            )
            .map_err(|error| error.to_string())?;
            fs::write(
                temporary.join("Makefile"),
                makefile_template(&name, &template),
            )
            .map_err(|error| error.to_string())?;
        } else {
            fs::write(temporary.join("script.lua"), lua_source_template(&template))
                .map_err(|error| error.to_string())?;
        }
        let project = default_project(
            name.clone(),
            language.clone(),
            template.clone(),
            runtime_version.clone(),
        );
        let config = serde_json::to_vec_pretty(&project).map_err(|error| error.to_string())?;
        fs::write(temporary.join(CONFIG_NAME), config).map_err(|error| error.to_string())?;
        fs::rename(&temporary, &directory).map_err(|error| error.to_string())?;
        Ok(project)
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(temporary);
    }
    result
}

#[tauri::command]
fn read_main_source(app: tauri::AppHandle, project: String) -> Result<String, String> {
    let directory = require_safe_project(&app, &project)?;
    let config: Project = serde_json::from_str(
        &fs::read_to_string(directory.join(CONFIG_NAME)).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let path = if config.language == "lua" {
        directory.join("script.lua")
    } else {
        directory.join("src/main.cpp")
    };
    fs::read_to_string(path).map_err(|error| error.to_string())
}

#[tauri::command]
fn save_main_source(app: tauri::AppHandle, project: String, source: String) -> Result<(), String> {
    if source.len() > MAX_SOURCE_SIZE {
        return Err("Source trop volumineuse".into());
    }
    let directory = require_safe_project(&app, &project)?;
    let config: Project = serde_json::from_str(
        &fs::read_to_string(directory.join(CONFIG_NAME)).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let path = if config.language == "lua" {
        directory.join("script.lua")
    } else {
        directory.join("src/main.cpp")
    };
    if fs::symlink_metadata(&path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Err("Le fichier source ne peut pas être un lien symbolique".into());
    }
    write_atomic(&path, source.as_bytes())
}

#[tauri::command]
fn build_project(app: tauri::AppHandle, project: String) -> Result<String, String> {
    let directory = require_safe_project(&app, &project)?;
    let config: Project = serde_json::from_str(
        &fs::read_to_string(directory.join(CONFIG_NAME)).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    if config.language == "lua" {
        let runtime = config.runtime_version.as_deref().unwrap_or("");
        let resources = app
            .path()
            .resource_dir()
            .map_err(|error| error.to_string())?;
        prepare_lua_project(&resources, &directory, runtime)?;
        return Ok("Projet LuaPlayer Plus r163 prêt".into());
    }
    let toolchain = toolchain_bin(&app)?;
    let pspdev = managed_pspdev(&app)?;
    if !directory.join("Makefile").is_file() {
        return Err("Configuration de build gérée absente".into());
    }
    let output = Command::new("make")
        .args(["clean", "all"])
        .current_dir(&directory)
        .env("PSPDEV", &pspdev)
        .env(
            "PATH",
            format!(
                "{}:/usr/bin:/bin:/usr/sbin:/sbin",
                toolchain.to_string_lossy()
            ),
        )
        .output()
        .map_err(|_| "PSPDEV n’est pas installé".to_string())?;
    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if message.is_empty() {
            "La compilation a échoué".into()
        } else {
            message
        });
    }
    let eboot = directory.join("EBOOT.PBP");
    validate_pbp(&eboot)
        .map_err(|error| format!("La compilation a produit un PBP invalide : {error}"))?;
    Ok("EBOOT.PBP prêt".into())
}

#[tauri::command]
fn run_project_ppsspp(app: tauri::AppHandle, project: String) -> Result<String, String> {
    let directory = require_safe_project(&app, &project)?;
    let eboot = directory.join("EBOOT.PBP");
    if !eboot.is_file() {
        return Err("Compile le jeu avant de le tester".into());
    }
    validate_pbp(&eboot)?;
    let executable = Path::new("/Applications/PPSSPPSDL.app/Contents/MacOS/PPSSPPSDL");
    if !executable.is_file() {
        return Err("PPSSPP n’est pas installé".into());
    }
    Command::new(executable)
        .arg(&eboot)
        .current_dir(&directory)
        .spawn()
        .map_err(|error| format!("Impossible de lancer PPSSPP : {error}"))?;
    Ok("Jeu lancé dans PPSSPP".into())
}

#[tauri::command]
fn deploy_project(
    app: tauri::AppHandle,
    project: String,
    psp_root: String,
    overwrite: bool,
) -> Result<String, String> {
    let project_directory = require_safe_project(&app, &project)?;
    let source = project_directory.join("EBOOT.PBP");
    if !source.is_file() {
        return Err("Compile le jeu avant de l’installer".into());
    }
    validate_pbp(&source)?;
    let game = Path::new(&psp_root).join("PSP").join("GAME");
    let game = game
        .canonicalize()
        .map_err(|_| "Le volume sélectionné ne contient pas PSP/GAME".to_string())?;
    if !game.is_dir() {
        return Err("Le volume sélectionné ne contient pas PSP/GAME".into());
    }
    let destination = game.join(&project);
    if destination.exists() {
        let canonical = destination
            .canonicalize()
            .map_err(|error| error.to_string())?;
        if !canonical.starts_with(&game)
            || fs::symlink_metadata(&destination)
                .map_err(|error| error.to_string())?
                .file_type()
                .is_symlink()
        {
            return Err("Destination PSP non sûre".into());
        }
    } else {
        fs::create_dir(&destination).map_err(|error| error.to_string())?;
    }
    copy_verified(&source, &destination.join("EBOOT.PBP"), overwrite)?;
    let config: Project = serde_json::from_str(
        &fs::read_to_string(project_directory.join(CONFIG_NAME))
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    if config.language == "lua" {
        for file in ["script.lua", "lpp.ini"] {
            copy_verified(
                &project_directory.join(file),
                &destination.join(file),
                overwrite,
            )?;
        }
    }
    copy_safe_tree(
        &project_directory.join("assets"),
        &destination.join("assets"),
        overwrite,
    )?;
    Ok(format!("Installé dans PSP/GAME/{project}"))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            list_projects,
            environment_status,
            create_project,
            read_main_source,
            save_main_source,
            build_project,
            run_project_ppsspp,
            deploy_project
        ])
        .run(tauri::generate_context!())
        .expect("Tauri failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_names_are_strictly_limited() {
        for valid in ["Jeu", "Jeu_2", "psp-game", "A123"] {
            assert!(valid_name(valid), "{valid} should be valid");
        }
        for invalid in ["", "../Jeu", "Jeu/Deux", "Jeu Deux", ".cache", "évasion"] {
            assert!(!valid_name(invalid), "{invalid} should be rejected");
        }
        assert!(!valid_name(&"a".repeat(33)));
    }

    #[test]
    fn generated_project_never_enables_kernel_mode() {
        let project = default_project("Test".into(), "cpp".into(), "hello".into(), None);
        assert!(!project.kernel_mode);
        assert_eq!(project.language, "cpp");
        assert_eq!(project.template, "hello");
    }

    #[test]
    fn makefile_has_fixed_toolchain_and_no_user_command() {
        let makefile = makefile_template("Test", "hello");
        assert!(makefile.contains("psp-config --pspsdk-path"));
        assert!(!makefile.contains("sh -c"));
        assert!(!makefile.contains("sudo"));
    }

    #[test]
    fn project_catalog_rejects_unknown_or_mixed_choices() {
        assert!(valid_project_choice("cpp", "graphics", None));
        assert!(!valid_project_choice("cpp", "sprite", None));
        assert!(!valid_project_choice("cpp", "hello", Some("hm-v7-rc1")));
        assert!(valid_project_choice("lua", "sprite", Some("hm-v7-rc1")));
        assert!(!valid_project_choice("lua", "sprite", Some("unknown")));
    }

    #[test]
    fn cpp_examples_compile_with_managed_toolchain_when_available() {
        let home = match std::env::var("HOME") {
            Ok(home) => PathBuf::from(home),
            Err(_) => return,
        };
        let pspdev = home.join(".pspdev/v20260701");
        if !pspdev.join("bin/psp-g++").is_file() {
            return;
        }
        for template in ["hello", "controls", "graphics", "timer"] {
            let directory = std::env::temp_dir().join(format!(
                "psp-reborn-example-{}-{template}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&directory);
            fs::create_dir_all(directory.join("src")).unwrap();
            fs::write(
                directory.join("src/main.cpp"),
                cpp_source_template("Test", template),
            )
            .unwrap();
            fs::write(
                directory.join("Makefile"),
                makefile_template("Test", template),
            )
            .unwrap();
            let output = Command::new("make")
                .arg("all")
                .current_dir(&directory)
                .env("PSPDEV", &pspdev)
                .env(
                    "PATH",
                    format!("{}:/usr/bin:/bin", pspdev.join("bin").display()),
                )
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "template {template}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(validate_pbp(&directory.join("EBOOT.PBP")).is_ok());
            let _ = fs::remove_dir_all(directory);
        }
    }

    #[test]
    fn bundled_lua_player_plus_runtime_builds_a_valid_project() {
        let resources = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let directory =
            std::env::temp_dir().join(format!("psp-reborn-lua-runtime-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("script.lua"), lua_source_template("hello")).unwrap();
        prepare_lua_project(&resources, &directory, "lpp-r163").unwrap();
        assert!(validate_pbp(&directory.join("EBOOT.PBP")).is_ok());
        assert!(directory.join("lpp.ini").is_file());
        assert!(directory.join("License.txt").is_file());
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn pbp_validator_accepts_a_minimal_header_and_rejects_bad_magic() {
        let path = std::env::temp_dir().join(format!("psp-reborn-{}.pbp", std::process::id()));
        let mut header = vec![0_u8; 40];
        header[0..4].copy_from_slice(b"\0PBP");
        header[4..8].copy_from_slice(&0x10000_u32.to_le_bytes());
        for index in 0..8 {
            let position = 8 + index * 4;
            header[position..position + 4].copy_from_slice(&40_u32.to_le_bytes());
        }
        fs::write(&path, &header).unwrap();
        assert!(validate_pbp(&path).is_ok());
        header[0] = 1;
        fs::write(&path, &header).unwrap();
        assert!(validate_pbp(&path).is_err());
        let _ = fs::remove_file(path);
    }
}
