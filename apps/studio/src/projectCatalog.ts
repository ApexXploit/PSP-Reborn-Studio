export type LanguageId = "cpp" | "lua";

export const languages = [
  { id: "cpp" as const, name: "C++", description: "Jeu natif compilé avec PSPDEV/PSPSDK", badge: "Recommandé" },
  { id: "lua" as const, name: "Lua", description: "Scripts pour la famille LuaPlayer HM", badge: "Historique" },
];

export const luaRuntimes = [
  { id: "hm-v1.7", name: "LuaPlayer HM v1.7", status: "archive" },
  { id: "hm-v2.0.4", name: "LuaPlayer HM v2.0.4", status: "archive" },
  { id: "hm-v3", name: "LuaPlayer HM v3 / LuaMin", status: "source" },
  { id: "hm-v4.0", name: "LuaPlayer HM v4.0 WiFi", status: "archive" },
  { id: "hm-v6", name: "LuaPlayer HM v6", status: "archive" },
  { id: "hm-v6.6", name: "LuaPlayer HM v6.6", status: "archive" },
  { id: "hm-v6.9-beta", name: "LuaPlayer HM v6.9 Beta", status: "archive" },
  { id: "hm-v7-rc1", name: "LuaPlayer HM 7 RC1", status: "documented" },
  { id: "hm-v8", name: "LuaPlayer HM v8", status: "archive" },
  { id: "hm-v8.1-beta", name: "LuaPlayer HM v8.1 Beta", status: "archive" },
  { id: "lpp-r163", name: "LuaPlayer Plus r163 (HM7 + Euphoria)", status: "available" },
] as const;

export const runtimeStatusLabel: Record<string, string> = {
  available: "prêt à exécuter",
  source: "sources récupérées",
  documented: "documenté",
  archive: "binaire à récupérer",
};

export const templates = {
  cpp: [
    { id: "hello", name: "Bonjour PSP", description: "Affichage texte et structure minimale", features: ["Écran debug", "Boucle principale"] },
    { id: "controls", name: "Contrôles", description: "Lire la croix, les boutons et le stick analogique", features: ["Boutons", "Stick analogique"] },
    { id: "graphics", name: "Graphismes 2D", description: "Initialiser le GU et dessiner des formes colorées", features: ["PSP GU", "Double buffering"] },
    { id: "timer", name: "Boucle de jeu", description: "Delta time, compteur FPS et mise à jour régulière", features: ["Timer", "VBlank", "FPS"] },
    { id: "audio", name: "Audio natif", description: "Générer et jouer un son stéréo sans ressource externe", features: ["sceAudio", "PCM", "Boucle audio"] },
    { id: "filesystem", name: "Sauvegarde", description: "Écrire et relire un fichier de progression", features: ["sceIoOpen", "sceIoWrite", "Memory Stick"] },
  ],
  lua: [
    { id: "hello", name: "Bonjour Lua", description: "Dessiner une première scène avec LuaPlayer", features: ["screen:fillRect", "Couleurs", "Boucle Lua"] },
    { id: "controls", name: "Contrôles", description: "Lire les boutons PSP et déplacer un carré", features: ["Controls.read", "Croix directionnelle"] },
    { id: "image", name: "Image en mémoire", description: "Créer une image colorée et l’afficher à l’écran", features: ["Image.createEmpty", "screen:blit"] },
    { id: "sprite", name: "Sprite mobile", description: "Déplacer un sprite avec les commandes PSP", features: ["Image.createEmpty", "Contrôles", "Animation"] },
    { id: "timer", name: "Timer et animation", description: "Mesurer le temps et animer une scène", features: ["Timer.new", "Timer:time", "VBlank"] },
    { id: "audio", name: "Musique et effets", description: "Charger un fichier WAV depuis assets", features: ["Wav.load", "Wav.play", "Contrôles"] },
  ],
} as const;
