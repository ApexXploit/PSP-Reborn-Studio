# PSP Reborn Studio

<p align="center">
  <img src="assets/branding/psp-reborn-logo.png" alt="Logo PSP Reborn Studio" width="300">
</p>

IDE sécurisé consacré au développement de véritables homebrews PSP en C++ et
Lua. PSP Reborn Studio associe Tauri, Monaco Editor, PSPDEV/PSPSDK, LuaPlayer et
PPSSPP pour créer, coder, compiler, tester et installer un jeu dans une même
application.

## Identité du projet

<table>
  <tr>
    <th>Logo de l'application</th>
    <th>Logo de l'auteur</th>
  </tr>
  <tr>
    <td align="center"><img src="assets/branding/psp-reborn-logo.png" alt="PSP Reborn Studio" width="260"></td>
    <td align="center"><img src="assets/branding/apexploit-author-logo.png" alt="ApeXploit" width="260"></td>
  </tr>
  <tr>
    <td align="center"><strong>PSP Reborn Studio</strong></td>
    <td align="center"><strong>ApeXploit</strong><br><a href="https://github.com/Yami-droid-dotcom">@Yami-droid-dotcom</a></td>
  </tr>
</table>

Le logo de l'application reprend l'univers cybernétique vert de l'auteur avec
une mascotte singe originale tenant une console portable. Il reste
volontairement distinct du logo personnel ApeXploit.

## Fonctionnalités

- assistant de projet inspiré de Visual Studio ;
- projets C++17 natifs avec quatre exemples PSPDEV ;
- projets Lua avec catalogue LuaPlayer HM et LuaPlayer Plus ;
- LuaPlayer Plus r163 embarqué et exécutable ;
- éditeur Monaco C++/Lua entièrement local ;
- compilation d'un véritable `EBOOT.PBP` MIPS ;
- lancement direct dans PPSSPP ;
- installation confinée à `PSP/GAME/<projet>` ;
- PBP Studio pour inspecter, extraire, modifier et reconstruire les PBP ;
- aucun terminal ni chemin arbitraire exposé dans l'interface.

## Tester l'application macOS

Une application Apple Silicon et un installateur DMG sont générés dans :

```text
apps/studio/src-tauri/target/release/bundle/macos/PSP Reborn Studio.app
apps/studio/src-tauri/target/release/bundle/dmg/PSP Reborn Studio_0.1.0_aarch64.dmg
```

Le parcours opérationnel est : créer un projet C++ ou Lua, modifier sa source,
préparer ou compiler son EBOOT, puis le lancer dans PPSSPP. PSPDEV v20260701
peut être isolé dans `~/.pspdev/v20260701` sans modifier le PATH global.

```sh
npm run doctor
npm run psp -- create MonJeu
npm run psp -- build games/MonJeu
npm run psp -- deploy games/MonJeu /Volumes/NOM_DE_LA_PSP
```

La commande `run` combine compilation et installation :

```sh
npm run psp -- run games/MonJeu /Volumes/NOM_DE_LA_PSP
```

Le déploiement refuse un volume qui ne contient pas déjà `PSP/GAME`. Le jeu est
installé dans `PSP/GAME/<deployFolder>/EBOOT.PBP`. Une PSP sous CFW peut ensuite
le lancer depuis **Jeu > Memory Stick**.

## Compilateur

Le CLI utilise PSPDEV installé localement si `psp-g++` et `psp-config` sont
présents. Sinon, il utilise Docker ou Podman avec l'image officielle versionnée
`pspdev/pspdev:v20260601`.

## Auteur

Créé par **ApeXploit** — [Yami-droid-dotcom](https://github.com/Yami-droid-dotcom).

PSP, PlayStation et Sony sont des marques de leurs propriétaires respectifs. Ce
projet homebrew indépendant n'est ni affilié ni approuvé par Sony.

## Vérifications

```sh
cd apps/studio
npm test
npm run build
npm audit

cd src-tauri
cargo fmt --check
cargo test
cargo check
```
