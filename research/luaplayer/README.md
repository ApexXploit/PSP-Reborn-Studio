# Archive de recherche LuaPlayer PSP / Windows

Collecte locale effectuée le 13 juillet 2026. Les anciens exécutables ne sont
pas lancés. Les ZIP sont conservés intacts, puis extraits séparément.

## Pièces les plus importantes

- `source-snapshots/FrankBuss__LuaPlayer` : dépôt officiel de LuaPlayer,
  archivé sans ses objets Git internes.
- `extracted/LuaPlayer_v0.20_source` : archive source officielle v0.20.
- `extracted/luaplayerwindows-0.20` : distribution Windows officielle v0.20.
- `source-snapshots/Klozz__LuaPlayer` : port Windows annoncé compatible avec
  LuaPlayer HM v3, LuaDEV et OneLua. Contient `Makefile.windows`, le coeur C/C++,
  les modules, les exemples et `doc/functions.txt`.
- `source-snapshots/Rinnegatamante__lua-player-plus` : LuaPlayer Plus PSP/PSP Go,
  descendant de HM7 et Euphoria.
- `source-snapshots/CurryGuy__lua-player-plus` et
  `source-snapshots/pavel-demin__lua-player-plus` : autres copies historiques de LPP.
- `source-snapshots/antim0118__LuaPlayer-by-YuliaTeam` : autre fork PSP.
- `source-snapshots/OrkPork__my-project-big-luaplayer-for-psp` : Big LuaPlayer.
- `documentation/` : présentation et notes EuroOSCON de LuaPlayer.
- `repositories/*.git` : miroirs Git complets conservés localement. Ils sont
  exclus du dépôt public pour éviter de republier leurs objets Git ; leur code
  est publié sous forme d'instantanés autonomes dans `source-snapshots/`.

## Archives originales conservées

- `downloads/LuaPlayer_v0.20_source.zip`
- `downloads/LuaPlayer_v0.20_firmware10.zip`
- `downloads/LuaPlayer_v0.20_firmware15.zip`
- `downloads/luaplayerwindows-0.20.zip`

Les sommes SHA-256 se trouvent dans `metadata/SHA256SUMS`.

## Élément encore manquant

L'installeur exact de **LuaPlayer HM7 RC1 Windows** était publié sous le nom
`setup-luaplayer_1232308224.exe` (35,67 Mo) à l'adresse historique :

`https://dls.pspgen.com/S/setup-luaplayer_1232308224.exe`

Le serveur ne répond plus. Aucun code source propre à cette RC1 n'a encore été
retrouvé. Le dépôt Klozz est actuellement la pièce Windows/HM la plus proche et
la plus exploitable, mais son identité avec la RC1 de lordvisaris n'est pas
établie.

## Provenance Web principale

- https://luaplayer.org/
- https://github.com/FrankBuss/LuaPlayer
- https://github.com/Klozz/LuaPlayer
- https://github.com/Rinnegatamante/lua-player-plus
- https://gamergen.com/actualites/lua-player-hm-7-pc-windows-linux-mac-37736-1
- https://gamergen.com/actualites/luaplayerhm-code-source-42375-1

Chaque dépôt conserve sa propre licence. L'absence éventuelle de licence dans
un dépôt ne signifie pas que son contenu appartient au domaine public.
