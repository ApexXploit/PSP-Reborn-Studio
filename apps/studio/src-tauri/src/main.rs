use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs,
    fs::File,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    process::Command,
};
use tauri::Manager;

const MAX_SOURCE_SIZE: usize = 2_000_000;
const MAX_TREE_ENTRIES: usize = 1_000;
const MAX_TREE_DEPTH: usize = 12;
const MAX_BUILD_SOURCES: usize = 256;
const MAX_BUILD_LOG: usize = 500_000;
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
    psp_mounts: Vec<String>,
    checks: Vec<EnvironmentCheck>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnvironmentCheck {
    id: &'static str,
    label: &'static str,
    ready: bool,
    required: bool,
    detail: String,
    path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BuildDiagnostic {
    severity: String,
    file: String,
    line: usize,
    column: usize,
    message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BuildReport {
    success: bool,
    summary: String,
    output: String,
    diagnostics: Vec<BuildDiagnostic>,
    source_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectFileEntry {
    path: String,
    name: String,
    is_dir: bool,
    depth: usize,
    read_only: bool,
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

fn ppsspp_executable() -> Option<PathBuf> {
    [
        "/Applications/PPSSPPSDL.app/Contents/MacOS/PPSSPPSDL",
        "/Applications/PPSSPP.app/Contents/MacOS/PPSSPP",
    ]
    .iter()
    .map(PathBuf::from)
    .find(|path| path.is_file())
}

fn connected_psp_mounts() -> Vec<String> {
    let mut mounts = Vec::new();
    if let Ok(volumes) = fs::read_dir("/Volumes") {
        for entry in volumes.flatten() {
            let path = entry.path();
            if path.join("PSP/GAME").is_dir()
                && fs::symlink_metadata(&path)
                    .map(|metadata| !metadata.file_type().is_symlink())
                    .unwrap_or(false)
            {
                mounts.push(path.to_string_lossy().to_string());
            }
        }
    }
    mounts.sort();
    mounts
}

#[tauri::command]
fn environment_status(app: tauri::AppHandle) -> Result<EnvironmentStatus, String> {
    let pspdev = managed_pspdev(&app)?;
    let compiler = pspdev.join("bin/psp-g++");
    let config = pspdev.join("bin/psp-config");
    let pspdev_ready = compiler.is_file() && config.is_file();
    let ppsspp = ppsspp_executable();
    let ppsspp_ready = ppsspp.is_some();
    let psp_mounts = connected_psp_mounts();
    let make = PathBuf::from("/usr/bin/make");
    let checks = vec![
        EnvironmentCheck {
            id: "pspdev",
            label: "PSPDEV",
            ready: pspdev_ready,
            required: true,
            detail: if pspdev_ready {
                "Toolchain gérée v20260701 · GCC 15.2.0".into()
            } else {
                "psp-g++ ou psp-config est absent".into()
            },
            path: Some(pspdev.to_string_lossy().to_string()),
        },
        EnvironmentCheck {
            id: "make",
            label: "Moteur de build",
            ready: make.is_file(),
            required: true,
            detail: if make.is_file() {
                "Make système disponible".into()
            } else {
                "Make est requis pour les projets C++".into()
            },
            path: Some(make.to_string_lossy().to_string()),
        },
        EnvironmentCheck {
            id: "ppsspp",
            label: "PPSSPP",
            ready: ppsspp_ready,
            required: false,
            detail: if ppsspp_ready {
                "Émulateur prêt pour les tests".into()
            } else {
                "Émulateur non détecté dans Applications".into()
            },
            path: ppsspp.map(|path| path.to_string_lossy().to_string()),
        },
        EnvironmentCheck {
            id: "psp",
            label: "PSP USB",
            ready: !psp_mounts.is_empty(),
            required: false,
            detail: match psp_mounts.len() {
                0 => "Aucun volume contenant PSP/GAME".into(),
                1 => "Une PSP prête pour le déploiement".into(),
                count => format!("{count} volumes PSP détectés"),
            },
            path: psp_mounts.first().cloned(),
        },
    ];
    Ok(EnvironmentStatus {
        pspdev_ready,
        pspdev_version: pspdev_ready.then(|| "v20260701 / GCC 15.2.0".into()),
        ppsspp_ready,
        psp_mounts,
        checks,
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

fn safe_relative_path(value: &str, allow_empty: bool) -> Result<PathBuf, String> {
    if value.is_empty() && allow_empty {
        return Ok(PathBuf::new());
    }
    if value.is_empty() || value.len() > 240 {
        return Err("Chemin de projet invalide".into());
    }
    let path = Path::new(value);
    if path.is_absolute() {
        return Err("Les chemins absolus sont interdits".into());
    }
    let mut safe = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => safe.push(part),
            _ => return Err("Le chemin ne peut pas sortir du projet".into()),
        }
    }
    if safe.as_os_str().is_empty() && !allow_empty {
        return Err("Chemin de projet invalide".into());
    }
    Ok(safe)
}

fn valid_item_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 80
        && name != "."
        && name != ".."
        && !name.starts_with('.')
        && !name
            .chars()
            .any(|character| character.is_control() || "/\\:".contains(character))
}

fn editable_project_file(path: &Path) -> bool {
    if path.file_name().is_some_and(|name| name == "Makefile") {
        return true;
    }
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "c" | "cc"
                    | "cpp"
                    | "h"
                    | "hpp"
                    | "lua"
                    | "json"
                    | "txt"
                    | "md"
                    | "ini"
                    | "xml"
                    | "csv"
                    | "vert"
                    | "frag"
                    | "obj"
                    | "mtl"
            )
        })
}

fn writable_project_file(path: &Path) -> bool {
    editable_project_file(path)
        && path.file_name().is_none_or(|name| name != "Makefile")
        && path != Path::new(CONFIG_NAME)
}

fn protected_project_path(path: &Path) -> bool {
    matches!(
        path.to_string_lossy().replace('\\', "/").as_str(),
        CONFIG_NAME | "Makefile" | "script.lua" | "src" | "src/main.cpp" | "assets"
    )
}

fn hidden_project_item(name: &str) -> bool {
    name.starts_with('.')
        || matches!(
            name,
            CONFIG_NAME
                | "EBOOT.PBP"
                | "PARAM.SFO"
                | "game.elf"
                | "Support.prx"
                | "License.txt"
                | "lpp.ini"
        )
        || name.ends_with(".o")
}

fn require_existing_project_path(
    directory: &Path,
    relative: &str,
) -> Result<(PathBuf, PathBuf), String> {
    let relative = safe_relative_path(relative, false)?;
    let candidate = directory.join(&relative);
    let metadata =
        fs::symlink_metadata(&candidate).map_err(|_| "Élément introuvable".to_string())?;
    if metadata.file_type().is_symlink() {
        return Err("Les liens symboliques sont interdits".into());
    }
    let canonical = candidate
        .canonicalize()
        .map_err(|_| "Élément introuvable".to_string())?;
    if !canonical.starts_with(directory) || canonical == directory {
        return Err("Le chemin sort du projet".into());
    }
    Ok((relative, canonical))
}

fn collect_project_files(
    directory: &Path,
    current: &Path,
    depth: usize,
    entries: &mut Vec<ProjectFileEntry>,
) -> Result<(), String> {
    if depth > MAX_TREE_DEPTH {
        return Err("L’arborescence dépasse la profondeur autorisée".into());
    }
    let mut children = fs::read_dir(current)
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    children.sort_by_key(|entry| {
        let is_file = entry.file_type().map(|kind| kind.is_file()).unwrap_or(true);
        (is_file, entry.file_name().to_string_lossy().to_lowercase())
    });
    for entry in children {
        if entries.len() >= MAX_TREE_ENTRIES {
            return Err("Le projet contient trop d’éléments à afficher".into());
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if hidden_project_item(&name) {
            continue;
        }
        let kind = entry.file_type().map_err(|error| error.to_string())?;
        if kind.is_symlink() {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(directory)
            .map_err(|_| "Chemin de projet invalide".to_string())?
            .to_path_buf();
        if kind.is_dir() {
            entries.push(ProjectFileEntry {
                path: relative.to_string_lossy().replace('\\', "/"),
                name,
                is_dir: true,
                depth,
                read_only: false,
            });
            collect_project_files(directory, &entry.path(), depth + 1, entries)?;
        } else if kind.is_file() && editable_project_file(&relative) {
            entries.push(ProjectFileEntry {
                path: relative.to_string_lossy().replace('\\', "/"),
                name,
                is_dir: false,
                depth,
                read_only: !writable_project_file(&relative),
            });
        }
    }
    Ok(())
}

fn require_no_symlinks(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() {
        return Err("Les liens symboliques sont interdits".into());
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path).map_err(|error| error.to_string())? {
            require_no_symlinks(&entry.map_err(|error| error.to_string())?.path())?;
        }
    }
    Ok(())
}

fn write_atomic(path: &Path, contents: &[u8]) -> Result<(), String> {
    let file_name = path
        .file_name()
        .ok_or_else(|| "Nom de fichier invalide".to_string())?
        .to_string_lossy();
    let temporary = path.with_file_name(format!(
        ".{file_name}.{}.psp-reborn-writing",
        std::process::id()
    ));
    let mut temporary_file = File::options()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|_| "Une autre sauvegarde est déjà en cours".to_string())?;
    temporary_file
        .write_all(contents)
        .and_then(|_| temporary_file.sync_all())
        .map_err(|error| {
            let _ = fs::remove_file(&temporary);
            error.to_string()
        })?;
    drop(temporary_file);
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
    const CPP_TEMPLATES: &[&str] = &[
        "hello",
        "controls",
        "graphics",
        "timer",
        "audio",
        "filesystem",
    ];
    const LUA_TEMPLATES: &[&str] = &["hello", "controls", "image", "sprite", "timer", "audio"];
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
        "audio" => {
            r#"    constexpr int samples = 1024;
    static short pcm[samples * 2];
    for (int index = 0; index < samples; ++index) {
        short value = (index / 32) % 2 ? 9000 : -9000;
        pcm[index * 2] = value;
        pcm[index * 2 + 1] = value;
    }
    int channel = sceAudioChReserve(PSP_AUDIO_NEXT_CHANNEL, samples, PSP_AUDIO_FORMAT_STEREO);
    if (channel < 0) {
        pspDebugScreenPrintf("Impossible de reserver un canal audio.\n");
    } else {
        pspDebugScreenPrintf("Lecture du signal PCM. Appuie sur HOME pour quitter.\n");
        while (true) sceAudioOutputBlocking(channel, PSP_AUDIO_VOLUME_MAX, pcm);
    }"#
        }
        "filesystem" => {
            r#"    const char message[] = "niveau=3\nscore=1200\n";
    SceUID file = sceIoOpen("save.txt", PSP_O_WRONLY | PSP_O_CREAT | PSP_O_TRUNC, 0777);
    if (file >= 0) {
        sceIoWrite(file, message, sizeof(message) - 1);
        sceIoClose(file);
    }
    char loaded[64] = {};
    file = sceIoOpen("save.txt", PSP_O_RDONLY, 0);
    if (file >= 0) {
        sceIoRead(file, loaded, sizeof(loaded) - 1);
        sceIoClose(file);
    }
    pspDebugScreenPrintf("Sauvegarde relue :\n%s\n", loaded);
    while (true) sceDisplayWaitVblankStart();"#
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
        "audio" => "#include <pspaudio.h>",
        "filesystem" => "#include <pspdisplay.h>\n#include <pspiofilemgr.h>",
        _ => "#include <pspdisplay.h>",
    };
    format!("#include <pspkernel.h>\n#include <pspdebug.h>\n{includes}\n\nPSP_MODULE_INFO(\"{name}\", PSP_MODULE_USER, 1, 0);\nPSP_MAIN_THREAD_ATTR(PSP_THREAD_ATTR_USER | PSP_THREAD_ATTR_VFPU);\n\nint main() {{\n    pspDebugScreenInit();\n{body}\n    return 0;\n}}\n")
}

fn lua_source_template(template: &str) -> &'static str {
    match template {
        "controls" => "x = 20\nwhile true do\n    pad = Controls.read()\n    if pad:left() then x = x - 2 end\n    if pad:right() then x = x + 2 end\n    screen:clear(Color.new(20, 24, 40))\n    screen:fillRect(x, 110, 32, 32, Color.new(130, 90, 255))\n    screen.flip()\n    screen.waitVblankStart()\nend\n",
        "image" => "image = Image.createEmpty(160, 100)\nimage:clear(Color.new(120, 70, 230))\nwhile true do\n    screen:clear()\n    screen:blit(160, 86, image)\n    screen:print(175, 125, \"Image Lua\", Color.new(255,255,255))\n    screen.flip()\n    screen.waitVblankStart()\nend\n",
        "sprite" => "sprite = Image.createEmpty(32, 32)\nsprite:clear(Color.new(130, 90, 255))\nx, y = 220, 120\nwhile true do\n    pad = Controls.read()\n    if pad:left() then x = x - 2 end\n    if pad:right() then x = x + 2 end\n    if pad:up() then y = y - 2 end\n    if pad:down() then y = y + 2 end\n    screen:clear()\n    screen:blit(x, y, sprite)\n    screen.flip()\n    screen.waitVblankStart()\nend\n",
        "timer" => "timer = Timer.new()\nwhile true do\n    elapsed = timer:time() / 1000\n    x = 220 + math.sin(elapsed * 2) * 120\n    screen:clear(Color.new(12, 18, 28))\n    screen:fillCircle(x, 136, 18, Color.new(80, 240, 120))\n    screen.flip()\n    screen.waitVblankStart()\nend\n",
        "audio" => "Wav.load(\"assets/sound.wav\", 0)\nWav.play(true, 0)\nwhile true do\n    pad = Controls.read()\n    screen:clear(Color.new(12, 18, 28))\n    screen:print(70, 120, \"CROIX: pause  ROND: reprendre\", Color.new(255,255,255))\n    if pad:cross() then Wav.pause(0) end\n    if pad:circle() then Wav.play(true, 0) end\n    screen.flip()\n    screen.waitVblankStart()\nend\n",
        _ => "while true do\n    screen:clear(Color.new(24, 18, 62))\n    screen:fillRect(105, 70, 270, 132, Color.new(105, 70, 215))\n    screen:fillRect(125, 90, 230, 92, Color.new(18, 22, 36))\n    screen:fillRect(145, 115, 190, 42, Color.new(145, 105, 255))\n    screen.flip()\n    screen.waitVblankStart()\nend\n",
    }
}

fn makefile_for_sources(name: &str, template: &str, sources: &[PathBuf]) -> String {
    let libraries = match template {
        "graphics" => "LIBS = -lpspgu\n",
        "timer" => "LIBS = -lpsprtc\n",
        "audio" => "LIBS = -lpspaudio\n",
        _ => "",
    };
    let objects = sources
        .iter()
        .map(|source| {
            let mut object = source.clone();
            object.set_extension("o");
            object.to_string_lossy().replace('\\', "/")
        })
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "# Généré par PSP Reborn Studio — les commandes utilisateur sont désactivées.\nTARGET = game\nOBJS = {objects}\nCFLAGS = -O2 -G0 -Wall -Iinclude -Isrc\nCXXFLAGS = $(CFLAGS) -std=c++17 -fno-exceptions -fno-rtti\nASFLAGS = $(CFLAGS)\n{libraries}EXTRA_TARGETS = EBOOT.PBP\nPSP_EBOOT_TITLE = {name}\nPSPSDK := $(shell psp-config --pspsdk-path)\ninclude $(PSPSDK)/lib/build.mak\n"
    )
}

fn makefile_template(name: &str, template: &str) -> String {
    makefile_for_sources(name, template, &[PathBuf::from("src/main.cpp")])
}

fn valid_build_path(path: &Path) -> bool {
    path.components().all(|component| match component {
        Component::Normal(part) => {
            let text = part.to_string_lossy();
            !text.is_empty()
                && !text.starts_with('.')
                && text
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || "_-.".contains(character))
        }
        _ => false,
    })
}

fn collect_cpp_sources(directory: &Path) -> Result<Vec<PathBuf>, String> {
    fn visit(
        root: &Path,
        current: &Path,
        depth: usize,
        sources: &mut Vec<PathBuf>,
    ) -> Result<(), String> {
        if depth > MAX_TREE_DEPTH {
            return Err("Les sources C++ dépassent la profondeur autorisée".into());
        }
        for entry in fs::read_dir(current).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let kind = entry.file_type().map_err(|error| error.to_string())?;
            if kind.is_symlink() {
                return Err("Les liens symboliques sont interdits dans src".into());
            }
            if kind.is_dir() {
                visit(root, &entry.path(), depth + 1, sources)?;
            } else if kind.is_file()
                && entry
                    .path()
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| {
                        matches!(extension.to_ascii_lowercase().as_str(), "c" | "cc" | "cpp")
                    })
            {
                let relative = entry
                    .path()
                    .strip_prefix(root)
                    .map_err(|_| "Source hors du projet".to_string())?
                    .to_path_buf();
                if !valid_build_path(&relative) {
                    return Err(format!(
                        "Chemin source non compilable : {}. Utilise lettres, chiffres, _, - et .",
                        relative.display()
                    ));
                }
                sources.push(relative);
                if sources.len() > MAX_BUILD_SOURCES {
                    return Err(format!("Maximum {MAX_BUILD_SOURCES} sources C/C++"));
                }
            }
        }
        Ok(())
    }

    let source_root = directory.join("src");
    if !source_root.is_dir() {
        return Err("Le dossier src requis est absent".into());
    }
    let mut sources = Vec::new();
    visit(directory, &source_root, 0, &mut sources)?;
    sources.sort();
    if sources.is_empty() {
        return Err("Aucun fichier C/C++ trouvé dans src".into());
    }
    if !sources
        .iter()
        .any(|source| source == Path::new("src/main.cpp"))
    {
        return Err("Le point d’entrée src/main.cpp est absent".into());
    }
    let mut objects = HashSet::new();
    for source in &sources {
        let mut object = source.clone();
        object.set_extension("o");
        if !objects.insert(object.clone()) {
            return Err(format!(
                "Deux sources produisent le même objet : {}",
                object.display()
            ));
        }
    }
    Ok(sources)
}

fn truncate_build_log(mut output: String) -> String {
    if output.len() > MAX_BUILD_LOG {
        output.truncate(MAX_BUILD_LOG);
        output.push_str("\n… journal tronqué par PSP Reborn Studio");
    }
    output
}

fn parse_build_diagnostics(output: &str, directory: &Path) -> Vec<BuildDiagnostic> {
    let mut diagnostics = Vec::new();
    for text in output.lines() {
        let parsed = ["error", "warning", "note"].iter().find_map(|severity| {
            let marker = format!(": {severity}: ");
            let (location, message) = text.split_once(&marker)?;
            let mut fields = location.rsplitn(3, ':');
            let column = fields.next()?.parse::<usize>().ok()?;
            let line = fields.next()?.parse::<usize>().ok()?;
            let raw_file = fields.next()?;
            let path = Path::new(raw_file);
            let relative = if path.is_absolute() {
                path.strip_prefix(directory).ok()?.to_path_buf()
            } else {
                safe_relative_path(raw_file, false).ok()?
            };
            if !relative.starts_with("src") && !relative.starts_with("include") {
                return None;
            }
            Some(BuildDiagnostic {
                severity: (*severity).into(),
                file: relative.to_string_lossy().replace('\\', "/"),
                line,
                column,
                message: message.trim().to_string(),
            })
        });
        if let Some(diagnostic) = parsed {
            diagnostics.push(diagnostic);
        }
    }
    diagnostics
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
            if template == "audio" {
                let sound = app
                    .path()
                    .resource_dir()
                    .map_err(|error| error.to_string())?
                    .join("resources/examples/sound.wav");
                if !sound.is_file() {
                    return Err("La ressource audio d’exemple est absente".into());
                }
                fs::copy(sound, temporary.join("assets/sound.wav"))
                    .map_err(|error| error.to_string())?;
            }
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
fn list_project_files(
    app: tauri::AppHandle,
    project: String,
) -> Result<Vec<ProjectFileEntry>, String> {
    let directory = require_safe_project(&app, &project)?;
    let mut entries = Vec::new();
    collect_project_files(&directory, &directory, 0, &mut entries)?;
    Ok(entries)
}

#[tauri::command]
fn read_project_file(
    app: tauri::AppHandle,
    project: String,
    path: String,
) -> Result<String, String> {
    let directory = require_safe_project(&app, &project)?;
    let (relative, file) = require_existing_project_path(&directory, &path)?;
    if !file.is_file() || !editable_project_file(&relative) || relative == Path::new(CONFIG_NAME) {
        return Err("Ce fichier ne peut pas être ouvert dans l’éditeur".into());
    }
    if fs::metadata(&file)
        .map_err(|error| error.to_string())?
        .len()
        > MAX_SOURCE_SIZE as u64
    {
        return Err("Fichier trop volumineux".into());
    }
    fs::read_to_string(file).map_err(|_| "Le fichier n’est pas un texte UTF-8 valide".into())
}

#[tauri::command]
fn save_project_file(
    app: tauri::AppHandle,
    project: String,
    path: String,
    source: String,
) -> Result<(), String> {
    if source.len() > MAX_SOURCE_SIZE {
        return Err("Fichier trop volumineux".into());
    }
    let directory = require_safe_project(&app, &project)?;
    let (relative, file) = require_existing_project_path(&directory, &path)?;
    if !file.is_file() || !writable_project_file(&relative) {
        return Err("Ce fichier n’est pas modifiable".into());
    }
    write_atomic(&file, source.as_bytes())
}

#[tauri::command]
fn create_project_item(
    app: tauri::AppHandle,
    project: String,
    parent: String,
    name: String,
    is_dir: bool,
) -> Result<String, String> {
    if !valid_item_name(&name) {
        return Err("Nom invalide : 1 à 80 caractères, sans /, \\, : ni nom caché".into());
    }
    let directory = require_safe_project(&app, &project)?;
    let parent_relative = safe_relative_path(&parent, true)?;
    let parent_path = if parent_relative.as_os_str().is_empty() {
        directory.clone()
    } else {
        let (_, path) =
            require_existing_project_path(&directory, &parent_relative.to_string_lossy())?;
        path
    };
    if !parent_path.is_dir() {
        return Err("Le parent doit être un dossier".into());
    }
    let target = parent_path.join(&name);
    if target.exists() {
        return Err("Un élément du même nom existe déjà".into());
    }
    let relative = target
        .strip_prefix(&directory)
        .map_err(|_| "Chemin de projet invalide".to_string())?;
    if !is_dir && !editable_project_file(relative) {
        return Err("Extension de fichier non autorisée dans l’éditeur".into());
    }
    if is_dir {
        fs::create_dir(&target).map_err(|error| error.to_string())?;
    } else {
        File::options()
            .write(true)
            .create_new(true)
            .open(&target)
            .map_err(|error| error.to_string())?;
    }
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

#[tauri::command]
fn rename_project_item(
    app: tauri::AppHandle,
    project: String,
    path: String,
    new_name: String,
) -> Result<String, String> {
    if !valid_item_name(&new_name) {
        return Err("Nouveau nom invalide".into());
    }
    let directory = require_safe_project(&app, &project)?;
    let (relative, current) = require_existing_project_path(&directory, &path)?;
    if protected_project_path(&relative) {
        return Err("Cet élément est requis par le projet et ne peut pas être renommé".into());
    }
    let destination = current
        .parent()
        .ok_or_else(|| "Chemin invalide".to_string())?
        .join(&new_name);
    if destination.exists() {
        return Err("Un élément du même nom existe déjà".into());
    }
    if current.is_file() {
        let destination_relative = destination
            .strip_prefix(&directory)
            .map_err(|_| "Chemin invalide".to_string())?;
        if !editable_project_file(destination_relative) {
            return Err("Extension de fichier non autorisée".into());
        }
    }
    fs::rename(&current, &destination).map_err(|error| error.to_string())?;
    Ok(destination
        .strip_prefix(&directory)
        .map_err(|_| "Chemin invalide".to_string())?
        .to_string_lossy()
        .replace('\\', "/"))
}

#[tauri::command]
fn delete_project_item(app: tauri::AppHandle, project: String, path: String) -> Result<(), String> {
    let directory = require_safe_project(&app, &project)?;
    let (relative, target) = require_existing_project_path(&directory, &path)?;
    if protected_project_path(&relative) {
        return Err("Cet élément est requis par le projet et ne peut pas être supprimé".into());
    }
    require_no_symlinks(&target)?;
    if target.is_dir() {
        fs::remove_dir_all(target).map_err(|error| error.to_string())
    } else {
        fs::remove_file(target).map_err(|error| error.to_string())
    }
}

#[tauri::command]
fn build_project(app: tauri::AppHandle, project: String) -> Result<BuildReport, String> {
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
        return Ok(BuildReport {
            success: true,
            summary: "Projet LuaPlayer Plus r163 prêt".into(),
            output: "Runtime et script préparés. EBOOT.PBP validé.".into(),
            diagnostics: Vec::new(),
            source_count: 1,
        });
    }
    let toolchain = toolchain_bin(&app)?;
    let pspdev = managed_pspdev(&app)?;
    let sources = collect_cpp_sources(&directory)?;
    let managed_makefile = makefile_for_sources(&config.name, &config.template, &sources);
    write_atomic(&directory.join("Makefile"), managed_makefile.as_bytes())?;
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let log = truncate_build_log(format!("{stdout}{stderr}"));
    let diagnostics = parse_build_diagnostics(&log, &directory);
    if !output.status.success() {
        let error_count = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == "error")
            .count();
        return Ok(BuildReport {
            success: false,
            summary: if error_count == 0 {
                "La compilation a échoué".into()
            } else {
                format!(
                    "Compilation échouée · {error_count} erreur{}",
                    if error_count > 1 { "s" } else { "" }
                )
            },
            output: if log.trim().is_empty() {
                "Le compilateur n’a retourné aucun détail.".into()
            } else {
                log
            },
            diagnostics,
            source_count: sources.len(),
        });
    }
    let eboot = directory.join("EBOOT.PBP");
    validate_pbp(&eboot)
        .map_err(|error| format!("La compilation a produit un PBP invalide : {error}"))?;
    let warning_count = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == "warning")
        .count();
    Ok(BuildReport {
        success: true,
        summary: format!(
            "EBOOT.PBP prêt · {} source{}{}",
            sources.len(),
            if sources.len() > 1 { "s" } else { "" },
            if warning_count > 0 {
                format!(" · {warning_count} avertissement(s)")
            } else {
                String::new()
            }
        ),
        output: if log.trim().is_empty() {
            "Compilation terminée sans message.".into()
        } else {
            log
        },
        diagnostics,
        source_count: sources.len(),
    })
}

#[tauri::command]
fn run_project_ppsspp(app: tauri::AppHandle, project: String) -> Result<String, String> {
    let directory = require_safe_project(&app, &project)?;
    let eboot = directory.join("EBOOT.PBP");
    if !eboot.is_file() {
        return Err("Compile le jeu avant de le tester".into());
    }
    validate_pbp(&eboot)?;
    let executable = ppsspp_executable().ok_or_else(|| "PPSSPP n’est pas installé".to_string())?;
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
            list_project_files,
            read_project_file,
            save_project_file,
            create_project_item,
            rename_project_item,
            delete_project_item,
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
    fn project_file_paths_cannot_escape_the_project() {
        for valid in ["src/main.cpp", "assets/maps/level1.json", "notes.txt"] {
            assert!(safe_relative_path(valid, false).is_ok(), "{valid}");
        }
        for invalid in [
            "",
            "../secret",
            "src/../../secret",
            "/tmp/secret",
            "./source.lua",
        ] {
            assert!(safe_relative_path(invalid, false).is_err(), "{invalid}");
        }
        assert!(safe_relative_path("", true).is_ok());
    }

    #[test]
    fn file_manager_protects_required_project_files() {
        for protected in [
            CONFIG_NAME,
            "Makefile",
            "script.lua",
            "src",
            "src/main.cpp",
            "assets",
        ] {
            assert!(protected_project_path(Path::new(protected)), "{protected}");
        }
        assert!(!protected_project_path(Path::new("src/player.cpp")));
        assert!(valid_item_name("player.cpp"));
        assert!(!valid_item_name("../player.cpp"));
        assert!(!valid_item_name(".git"));
        assert!(editable_project_file(Path::new("Makefile")));
        assert!(!writable_project_file(Path::new("Makefile")));
        assert!(writable_project_file(Path::new("src/player.cpp")));
    }

    #[test]
    fn file_tree_hides_generated_outputs_and_lists_sources() {
        let directory =
            std::env::temp_dir().join(format!("psp-reborn-tree-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(directory.join("src")).unwrap();
        fs::create_dir_all(directory.join("assets")).unwrap();
        fs::write(directory.join("src/main.cpp"), "int main() {}").unwrap();
        fs::write(directory.join("notes.md"), "# Notes").unwrap();
        fs::write(directory.join("EBOOT.PBP"), "generated").unwrap();
        fs::write(directory.join(CONFIG_NAME), "{}").unwrap();
        let canonical = directory.canonicalize().unwrap();
        let mut entries = Vec::new();
        collect_project_files(&canonical, &canonical, 0, &mut entries).unwrap();
        let paths = entries
            .iter()
            .map(|entry| entry.path.as_str())
            .collect::<Vec<_>>();
        assert!(paths.contains(&"src"));
        assert!(paths.contains(&"src/main.cpp"));
        assert!(paths.contains(&"notes.md"));
        assert!(!paths.contains(&"EBOOT.PBP"));
        assert!(!paths.contains(&CONFIG_NAME));
        assert!(require_existing_project_path(&canonical, "../secret").is_err());
        let _ = fs::remove_dir_all(directory);
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
    fn managed_makefile_includes_valid_recursive_sources() {
        let directory =
            std::env::temp_dir().join(format!("psp-reborn-multifile-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(directory.join("src/game")).unwrap();
        fs::write(directory.join("src/main.cpp"), "int main() { return 0; }").unwrap();
        fs::write(
            directory.join("src/game/player.cpp"),
            "int player() { return 1; }",
        )
        .unwrap();
        fs::write(directory.join("src/game/player.hpp"), "int player();").unwrap();
        let sources = collect_cpp_sources(&directory).unwrap();
        assert_eq!(
            sources,
            vec![
                PathBuf::from("src/game/player.cpp"),
                PathBuf::from("src/main.cpp")
            ]
        );
        let makefile = makefile_for_sources("Test", "hello", &sources);
        assert!(makefile.contains("OBJS = src/game/player.o src/main.o"));
        assert!(makefile.contains("-Iinclude -Isrc"));
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn compiler_messages_become_clickable_diagnostics() {
        let directory = PathBuf::from("/tmp/PSP Reborn/Games/Test");
        let log =
            "src/main.cpp:12:7: error: expected ';'\ninclude/game.hpp:4:2: warning: unused value";
        let diagnostics = parse_build_diagnostics(log, &directory);
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].severity, "error");
        assert_eq!(diagnostics[0].file, "src/main.cpp");
        assert_eq!(diagnostics[0].line, 12);
        assert_eq!(diagnostics[1].file, "include/game.hpp");
    }

    #[test]
    fn project_catalog_rejects_unknown_or_mixed_choices() {
        assert!(valid_project_choice("cpp", "graphics", None));
        assert!(valid_project_choice("cpp", "audio", None));
        assert!(valid_project_choice("cpp", "filesystem", None));
        assert!(!valid_project_choice("cpp", "sprite", None));
        assert!(!valid_project_choice("cpp", "hello", Some("hm-v7-rc1")));
        assert!(valid_project_choice("lua", "sprite", Some("hm-v7-rc1")));
        assert!(valid_project_choice("lua", "timer", Some("lpp-r163")));
        assert!(valid_project_choice("lua", "audio", Some("lpp-r163")));
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
        for template in [
            "hello",
            "controls",
            "graphics",
            "timer",
            "audio",
            "filesystem",
        ] {
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
                directory.join("src/extra.cpp"),
                "int pspRebornMultifileCheck() { return 42; }\n",
            )
            .unwrap();
            let sources = collect_cpp_sources(&directory).unwrap();
            assert_eq!(sources.len(), 2);
            fs::write(
                directory.join("Makefile"),
                makefile_for_sources("Test", template, &sources),
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
    fn lua_audio_example_bundles_a_pcm_wave() {
        let sound = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/examples/sound.wav");
        let data = fs::read(sound).unwrap();
        assert!(data.len() > 44);
        assert_eq!(&data[0..4], b"RIFF");
        assert_eq!(&data[8..12], b"WAVE");
        assert!(lua_source_template("audio").contains("assets/sound.wav"));
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
