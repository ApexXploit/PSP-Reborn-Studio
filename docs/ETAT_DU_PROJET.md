# État du projet

Dernière mise à jour : 13 juillet 2026.

## Fonctionnel

- Application native Tauri sur macOS Apple Silicon.
- Assistant de création de projet inspiré de Visual Studio : langage, runtime et
  modèle d'exemple.
- Projets C++ natifs avec quatre modèles : démarrage, contrôles, graphismes GU et
  boucle de jeu.
- Projets Lua avec quatre modèles : dessin, contrôles, image en mémoire et sprite.
- Catalogue de onze versions LuaPlayer HM/LuaPlayer Plus. LuaPlayer Plus r163 est
  embarqué et exécutable ; les versions historiques non récupérées sont clairement
  signalées comme telles.
- Monaco Editor C++/Lua entièrement local et sauvegarde atomique.
- Explorateur de projet multi-fichiers : dossiers repliables, création,
  ouverture, renommage et suppression avec confinement et fichiers requis
  protégés.
- PSPDEV officiel v20260701 avec GCC PSP 15.2.0.
- Compilation d'un véritable exécutable MIPS et d'un `EBOOT.PBP`.
- PPSSPP 1.20.4 détecté et lancement du projet actif C++ ou Lua.
- Déploiement limité à `PSP/GAME/<projet>`.
- Sélection native du volume PSP.
- PBP Studio intégré : ouverture, extraction, remplacement et reconstruction.
- Audit npm sans vulnérabilité connue.

## Validation réelle effectuée

Les quatre modèles C++ sont compilés par les tests avec le véritable toolchain
PSPDEV, puis leurs PBP sont relus et validés. Le runtime LuaPlayer Plus r163
embarqué est lui aussi vérifié comme PBP valide. Les projets de démonstration C++
et Lua ont démarré dans PPSSPP ; la scène Lua graphique a été contrôlée visuellement.

Empreinte SHA-256 du runtime LuaPlayer Plus r163 embarqué :

```text
8f1cc1d78bfcc10c493299145ac58f6f8f12979380898afccc74154383d4f8ad
```

## Reste à valider

- Copie et lancement sur une PSP physique de l'utilisateur.
- Récupération et validation des binaires originaux des versions LuaPlayer HM
  historiques actuellement cataloguées seulement.
- Signature et notarisation macOS pour une distribution publique.
- Builds Windows et Linux.
- Autocomplétion PSPSDK via clangd.
- Gestion multi-fichiers et ressources du projet.
