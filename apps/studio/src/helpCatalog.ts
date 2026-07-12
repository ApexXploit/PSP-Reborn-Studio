export type HelpLanguage = "cpp" | "lua";

export type HelpArticle = {
  id: string;
  language: HelpLanguage;
  category: string;
  title: string;
  summary: string;
  apis: string[];
  source: string;
  code: string;
  note?: string;
};

const snippet = (...lines: string[]) => lines.join("\n");

export const helpArticles: HelpArticle[] = [
  {
    id: "cpp-structure", language: "cpp", category: "Fondations", title: "Structure d’un homebrew",
    summary: "Déclare un module utilisateur, initialise l’écran et garde le programme actif.",
    apis: ["PSP_MODULE_INFO", "PSP_MAIN_THREAD_ATTR", "sceDisplayWaitVblankStart"],
    source: "PSPSDK v20260701 · pspkernel.h / pspdisplay.h",
    code: snippet(
      "#include <pspkernel.h>",
      "#include <pspdisplay.h>",
      "",
      "PSP_MODULE_INFO(\"MonJeu\", PSP_MODULE_USER, 1, 0);",
      "PSP_MAIN_THREAD_ATTR(PSP_THREAD_ATTR_USER | PSP_THREAD_ATTR_VFPU);",
      "",
      "int main() {",
      "    while (true) sceDisplayWaitVblankStart();",
      "    return 0;",
      "}"
    )
  },
  {
    id: "cpp-debug-screen", language: "cpp", category: "Affichage", title: "Écran texte de débogage",
    summary: "Affiche rapidement des informations, change les couleurs et positionne le curseur.",
    apis: ["pspDebugScreenInit", "pspDebugScreenPrintf", "pspDebugScreenSetXY", "pspDebugScreenSetTextColor"],
    source: "PSPSDK v20260701 · pspdebug.h",
    code: snippet(
      "#include <pspdebug.h>",
      "",
      "pspDebugScreenInit();",
      "pspDebugScreenSetTextColor(0xff80ff80);",
      "pspDebugScreenSetXY(2, 3);",
      "pspDebugScreenPrintf(\"Score : %d\\n\", score);"
    )
  },
  {
    id: "cpp-controls", language: "cpp", category: "Entrées", title: "Boutons et stick analogique",
    summary: "Lit l’état instantané des boutons, de la croix et du stick analogique.",
    apis: ["sceCtrlSetSamplingMode", "sceCtrlPeekBufferPositive", "PSP_CTRL_CROSS"],
    source: "PSPSDK v20260701 · pspctrl.h",
    code: snippet(
      "#include <pspctrl.h>",
      "",
      "sceCtrlSetSamplingMode(PSP_CTRL_MODE_ANALOG);",
      "SceCtrlData pad{};",
      "sceCtrlPeekBufferPositive(&pad, 1);",
      "if (pad.Buttons & PSP_CTRL_CROSS) tirer();",
      "float horizontal = (int(pad.Lx) - 128) / 128.0f;"
    )
  },
  {
    id: "cpp-gu", language: "cpp", category: "Graphismes", title: "PSP GU et double buffering",
    summary: "Initialise le processeur graphique, efface l’image et échange les buffers à chaque frame.",
    apis: ["sceGuInit", "sceGuStart", "sceGuClear", "sceGuSwapBuffers"],
    source: "PSPSDK v20260701 · pspgu.h",
    code: snippet(
      "#include <pspgu.h>",
      "static unsigned int __attribute__((aligned(16))) list[262144];",
      "",
      "sceGuInit();",
      "sceGuStart(GU_DIRECT, list);",
      "sceGuDrawBuffer(GU_PSM_8888, (void*)0, 512);",
      "sceGuDispBuffer(480, 272, (void*)0x88000, 512);",
      "sceGuFinish(); sceGuSync(0, 0); sceGuDisplay(GU_TRUE);",
      "",
      "sceGuStart(GU_DIRECT, list);",
      "sceGuClearColor(0xff201008);",
      "sceGuClear(GU_COLOR_BUFFER_BIT);",
      "sceGuFinish(); sceGuSync(0, 0); sceGuSwapBuffers();"
    )
  },
  {
    id: "cpp-time", language: "cpp", category: "Boucle de jeu", title: "Delta time et horloge",
    summary: "Calcule le temps écoulé entre deux images pour rendre les déplacements indépendants du FPS.",
    apis: ["sceRtcGetTickResolution", "sceRtcGetCurrentTick"],
    source: "PSPSDK v20260701 · psprtc.h",
    code: snippet(
      "#include <psprtc.h>",
      "",
      "u64 previous = 0, current = 0;",
      "sceRtcGetCurrentTick(&previous);",
      "// À chaque image :",
      "sceRtcGetCurrentTick(&current);",
      "float dt = float(current - previous) / sceRtcGetTickResolution();",
      "previous = current;",
      "position += vitesse * dt;"
    )
  },
  {
    id: "cpp-audio", language: "cpp", category: "Audio", title: "Sortie PCM stéréo",
    summary: "Réserve un canal audio et envoie un buffer de samples 16 bits.",
    apis: ["sceAudioChReserve", "sceAudioOutputBlocking", "sceAudioChRelease"],
    source: "PSPSDK v20260701 · pspaudio.h",
    code: snippet(
      "#include <pspaudio.h>",
      "constexpr int count = 1024;",
      "short pcm[count * 2] = {}; // gauche, droite",
      "",
      "int channel = sceAudioChReserve(PSP_AUDIO_NEXT_CHANNEL, count, PSP_AUDIO_FORMAT_STEREO);",
      "sceAudioOutputBlocking(channel, PSP_AUDIO_VOLUME_MAX, pcm);",
      "sceAudioChRelease(channel);"
    ),
    note: "Ajoute -lpspaudio dans les bibliothèques de l’édition de liens."
  },
  {
    id: "cpp-files", language: "cpp", category: "Données", title: "Fichier de sauvegarde",
    summary: "Écrit une progression à côté de l’EBOOT puis la relit depuis le Memory Stick.",
    apis: ["sceIoOpen", "sceIoWrite", "sceIoRead", "sceIoClose"],
    source: "PSPSDK v20260701 · pspiofilemgr.h",
    code: snippet(
      "#include <pspiofilemgr.h>",
      "",
      "const char data[] = \"score=1200\\n\";",
      "SceUID file = sceIoOpen(\"save.txt\", PSP_O_WRONLY | PSP_O_CREAT | PSP_O_TRUNC, 0777);",
      "if (file >= 0) {",
      "    sceIoWrite(file, data, sizeof(data) - 1);",
      "    sceIoClose(file);",
      "}"
    )
  },
  {
    id: "cpp-network", language: "cpp", category: "Réseau", title: "Initialisation réseau",
    summary: "Prépare la pile réseau avant d’utiliser les modules resolver, HTTP ou sockets.",
    apis: ["sceNetInit", "sceNetGetLocalEtherAddr", "sceNetTerm"],
    source: "PSPSDK v20260701 · pspnet.h",
    code: snippet(
      "#include <pspnet.h>",
      "",
      "int result = sceNetInit(128 * 1024, 42, 4 * 1024, 42, 4 * 1024);",
      "if (result >= 0) {",
      "    unsigned char mac[6];",
      "    sceNetGetLocalEtherAddr(mac);",
      "    sceNetTerm();",
      "}"
    ),
    note: "Une connexion Wi-Fi complète demande aussi pspnet_inet, pspnet_apctl et une configuration utilisateur."
  },
  {
    id: "lua-screen", language: "lua", category: "Affichage", title: "Écran, couleurs et formes",
    summary: "Dessine une scène 2D avec des rectangles, cercles, lignes et texte.",
    apis: ["Color.new", "screen:clear", "screen:fillRect", "screen:fillCircle", "screen.flip"],
    source: "LuaPlayer Plus r163 · screen / Color",
    code: snippet(
      "fond = Color.new(12, 18, 28)",
      "vert = Color.new(100, 255, 80)",
      "while true do",
      "    screen:clear(fond)",
      "    screen:fillRect(30, 30, 180, 80, vert)",
      "    screen:fillCircle(300, 136, 35, vert)",
      "    screen.flip()",
      "    screen.waitVblankStart()",
      "end"
    )
  },
  {
    id: "lua-controls", language: "lua", category: "Entrées", title: "Contrôles PSP",
    summary: "Lit les boutons, les gâchettes, la croix et les valeurs analogiques.",
    apis: ["Controls.read", "Controls:cross", "Controls:left", "Controls:analogX"],
    source: "LuaPlayer Plus r163 · Controls",
    code: snippet(
      "pad = Controls.read()",
      "if pad:cross() then action() end",
      "if pad:left() then x = x - 2 end",
      "if pad:right() then x = x + 2 end",
      "analogiqueX = pad:analogX()"
    )
  },
  {
    id: "lua-image", language: "lua", category: "Images", title: "Images et sprites",
    summary: "Charge un PNG/JPG, le place en VRAM et affiche une portion avec rotation.",
    apis: ["Image.load", "Image:toVram", "screen:blit", "Image:createEmpty"],
    source: "LuaPlayer Plus r163 · Image",
    code: snippet(
      "sprite = Image.load(\"assets/player.png\")",
      "sprite:toVram()",
      "angle = 0",
      "while true do",
      "    screen:clear()",
      "    screen:blit(220, 120, sprite, 255, angle)",
      "    angle = angle + 1",
      "    screen.flip()",
      "    screen.waitVblankStart()",
      "end"
    )
  },
  {
    id: "lua-timer", language: "lua", category: "Boucle de jeu", title: "Timer et animation",
    summary: "Démarre, mesure, arrête et remet à zéro une horloge.",
    apis: ["Timer.new", "Timer:start", "Timer:time", "Timer:reset"],
    source: "LuaPlayer Plus r163 · Timer",
    code: snippet(
      "timer = Timer.new()",
      "timer:start()",
      "while true do",
      "    secondes = timer:time() / 1000",
      "    x = 220 + math.sin(secondes * 2) * 100",
      "    screen:clear()",
      "    screen:fillCircle(x, 136, 16, Color.new(80, 255, 100))",
      "    screen.flip()",
      "    screen.waitVblankStart()",
      "end"
    )
  },
  {
    id: "lua-wav", language: "lua", category: "Audio", title: "Effets sonores WAV",
    summary: "Charge un WAV dans un canal, le joue en boucle et contrôle son volume.",
    apis: ["Wav.load", "Wav.play", "Wav.pause", "Wav.volume", "Wav.unload"],
    source: "LuaPlayer Plus r163 · Wav",
    code: snippet(
      "Wav.load(\"assets/jump.wav\", 0)",
      "Wav.volume(100, 0)",
      "Wav.play(false, 0)",
      "-- À la fermeture du niveau :",
      "Wav.stop(0)",
      "Wav.unload(0)"
    )
  },
  {
    id: "lua-stream-audio", language: "lua", category: "Audio", title: "Musique MP3 ou OGG",
    summary: "Utilise les interfaces MP3 et OGG, qui partagent les mêmes opérations de lecture.",
    apis: ["Mp3.load", "Mp3.play", "Ogg.load", "Ogg.play", "Mp3.eos"],
    source: "LuaPlayer Plus r163 · Mp3 / Ogg",
    code: snippet(
      "Mp3.load(\"assets/music.mp3\", 0)",
      "Mp3.volume(90, 0)",
      "Mp3.play(true, 0)",
      "-- Mp3.pause(0), Mp3.stop(0), Mp3.unload(0)"
    )
  },
  {
    id: "lua-system", language: "lua", category: "Système", title: "Dossiers, mémoire et batterie",
    summary: "Inspecte un dossier, crée des répertoires et lit l’état de la batterie.",
    apis: ["System.listDir", "System.createDir", "System.getFreeMemory", "System.powerGetBatteryLifePercent"],
    source: "LuaPlayer Plus r163 · System",
    code: snippet(
      "System.createDir(\"saves\")",
      "fichiers = System.listDir(\"assets\")",
      "memoire = System.getFreeMemory()",
      "batterie = System.powerGetBatteryLifePercent()",
      "screen:print(10, 10, \"Batterie : \" .. batterie .. \"%\")"
    )
  },
  {
    id: "lua-font", language: "lua", category: "Texte", title: "Polices TrueType",
    summary: "Charge une police ou crée une police intégrée puis mesure et affiche un texte.",
    apis: ["Font.load", "Font.createProportional", "Font:getTextSize", "screen:fontPrint"],
    source: "LuaPlayer Plus r163 · Font",
    code: snippet(
      "font = Font.load(\"assets/font.ttf\", 20, Font.FONT_SIZE_PIXELS)",
      "largeur, hauteur = font:getTextSize(\"PSP Reborn\")",
      "screen:fontPrint((480-largeur)/2, 30, font, \"PSP Reborn\", Color.new(255,255,255))"
    )
  },
  {
    id: "lua-xml", language: "lua", category: "Données", title: "Lire et écrire du XML",
    summary: "Construit un document, ajoute des attributs, sauvegarde puis recherche un nœud.",
    apis: ["Xml.new", "Xml:newElement", "Xml:setAttr", "Xml:save", "Xml.findNode"],
    source: "LuaPlayer Plus r163 · Xml",
    code: snippet(
      "document = Xml.new(\"1.0\")",
      "joueur = document:newElement(\"joueur\")",
      "joueur:setAttr(\"nom\", \"ApeXploit\")",
      "joueur:setInt(1200)",
      "document:save(\"save.xml\")"
    )
  },
  {
    id: "lua-archive", language: "lua", category: "Données", title: "Extraire une archive",
    summary: "Ouvre une archive et extrait un fichier précis ou tout son contenu.",
    apis: ["Archive.open", "Archive:extractFile", "Archive:extractAll"],
    source: "LuaPlayer Plus r163 · Archive",
    code: snippet(
      "archive = Archive.open(\"assets/level.zip\")",
      "archive:extractFile(\"map.json\", \"cache/map.json\")",
      "-- ou : archive:extractAll(\"cache\")"
    )
  },
  {
    id: "lua-camera", language: "lua", category: "Périphériques", title: "Caméra PSP",
    summary: "Initialise la caméra, affiche le flux et capture une photo.",
    apis: ["Camera.init", "Camera.initVideo", "Camera.render", "Camera.takePhoto", "Camera.shutdown"],
    source: "LuaPlayer Plus r163 · Camera",
    code: snippet(
      "Camera.init()",
      "Camera.initVideo(Camera.RESOLUTION_480_272)",
      "while true do",
      "    Camera.render()",
      "    if Controls.read():cross() then",
      "        Camera.takePhoto(\"photo.jpg\", 80, Camera.RESOLUTION_480_272)",
      "    end",
      "    screen.flip()",
      "end"
    ),
    note: "Nécessite une caméra PSP compatible connectée à la console."
  },
  {
    id: "lua-3d", language: "lua", category: "3D", title: "GU et matrices GUM",
    summary: "Démarre une scène 3D, configure la projection et dessine une liste de sommets.",
    apis: ["Gu.start3d", "Gum.matrixMode", "Gum.perspective", "Gum.translate", "Gum.drawArray", "Gu.end3d"],
    source: "LuaPlayer Plus r163 · Gu / Gum",
    code: snippet(
      "Gu.start3d()",
      "Gum.matrixMode(Gu.PROJECTION)",
      "Gum.loadIdentity()",
      "Gum.perspective(75, 480/272, 0.5, 1000)",
      "Gum.matrixMode(Gu.MODEL)",
      "Gum.loadIdentity()",
      "Gum.translate(0, 0, -3)",
      "Gum.drawArray(Gu.TRIANGLES, Gu.VERTEX_32BITF, sommets)",
      "Gu.end3d()"
    )
  }
];
