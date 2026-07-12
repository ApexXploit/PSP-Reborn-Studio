# Cahier des charges — PSP Reborn Studio

**Version :** 0.1
**Date :** 13 juillet 2026
**Statut :** document de cadrage initial

## 1. Présentation du projet

PSP Reborn Studio est un environnement de développement intégré dédié à la
création de jeux et d'applications homebrew pour Sony PlayStation Portable.

Le logiciel doit permettre à un utilisateur de créer, programmer, compiler,
tester, empaqueter et installer un projet sur une véritable PSP sans avoir à
configurer manuellement PSPDEV, PSPSDK, les Makefiles ou la structure interne
d'un fichier `EBOOT.PBP`.

L'application cible en priorité les développeurs débutants et intermédiaires,
tout en produisant des projets C++ standards et exploitables en dehors de l'IDE.

## 2. Objectifs

### 2.1 Objectif principal

Fournir une boucle de développement simple et fiable :

```text
Créer → Coder → Compiler → Tester → Installer sur PSP
```

### 2.2 Objectifs secondaires

- Faciliter l'apprentissage du développement PSP en C++.
- Proposer ultérieurement le prototypage en Lua.
- Intégrer des bibliothèques PSP validées et documentées.
- Remplacer les anciens outils séparés par une expérience cohérente.
- Permettre l'inspection, l'extraction et la reconstruction des PBP.
- Garantir des builds reproductibles et compatibles avec le matériel réel.
- Préserver les connaissances et les sources de l'écosystème PSP historique.

## 3. Public cible

- Débutants souhaitant créer leur premier homebrew PSP.
- Développeurs C/C++ intéressés par une plateforme rétro.
- Créateurs de jeux 2D indépendants.
- Développeurs souhaitant restaurer un ancien projet LuaPlayer.
- Utilisateurs de PSP sous custom firmware.

## 4. Plateformes

### 4.1 Plateformes de développement

- macOS Apple Silicon et Intel.
- Windows 10 et 11 64 bits.
- Linux 64 bits.

### 4.2 Cibles d'exécution

- PSP-1000.
- PSP-2000.
- PSP-3000.
- PSP Go.
- PPSSPP pour les tests rapides.

La première version cible les PSP capables d'exécuter des homebrews. Le support
des exécutables signés pour firmware officiel pourra être étudié ultérieurement.

## 5. Choix techniques

| Composant | Technologie |
|---|---|
| Application de bureau | Tauri 2 |
| Interface | React et TypeScript |
| Éditeur de code | Monaco Editor |
| Backend natif | Rust |
| Langage cible principal | C++17 |
| Chaîne PSP | PSPDEV et PSPSDK |
| Compilation | `psp-g++` et Make PSPSDK |
| Émulateur | PPSSPP |
| Format exécutable | `EBOOT.PBP` |
| Configuration projet | JSON versionné |

## 6. Principes de conception

### 6.1 Simplicité

L'utilisateur ne doit pas avoir besoin de connaître le fonctionnement de
`psp-g++`, `psp-prxgen`, `mksfoex` ou `pack-pbp` pour créer un jeu.

### 6.2 Sécurité par défaut

Le frontend ne doit jamais recevoir une commande permettant d'exécuter du shell
arbitraire. Chaque opération native doit correspondre à une commande Tauri
nommée, limitée et validée.

### 6.3 Compatibilité réelle

Un projet déclaré compatible ne doit pas seulement fonctionner dans PPSSPP. Il
doit produire un EBOOT testable sur une vraie PSP prise en charge.

### 6.4 Progressivité

Les fonctions avancées doivent être masquées tant qu'elles ne sont pas utiles.
Le mode standard reste l'expérience de référence.

## 7. Périmètre fonctionnel du MVP

### 7.1 Accueil et projets

Le logiciel doit permettre de :

- créer un projet C++ minimal ;
- afficher les projets existants ;
- ouvrir et supprimer un projet après confirmation ;
- renommer un projet sans casser son build ;
- conserver tous les projets dans un répertoire contrôlé ;
- afficher la cible et l'état du dernier build.

Le nom technique d'un projet est limité aux lettres ASCII, chiffres, tirets et
traits de soulignement, sur 32 caractères maximum.

### 7.2 Éditeur C++

Le logiciel doit proposer :

- coloration syntaxique C/C++ ;
- numéros de ligne ;
- indentation automatique ;
- recherche et remplacement ;
- annulation et rétablissement ;
- sauvegarde manuelle et automatique ;
- affichage des erreurs de compilation avec fichier et ligne ;
- ouverture de plusieurs fichiers du projet.

L'autocomplétion PSP via `clangd` est souhaitée pour la version suivant le MVP.

### 7.3 Modèle de projet minimal

Un projet nouvellement créé doit contenir :

```text
MonJeu/
├── psp-project.json
├── src/
│   └── main.cpp
├── assets/
└── build/
```

Le Makefile et les fichiers intermédiaires peuvent être générés dans un dossier
interne géré. Ils ne constituent pas la source de configuration publique.

### 7.4 Compilation PSP

Le bouton **Compiler** doit :

1. sauvegarder les fichiers ouverts ;
2. valider la configuration du projet ;
3. générer la configuration de build contrôlée ;
4. invoquer uniquement les outils PSPDEV autorisés ;
5. capturer les sorties standard et d'erreur ;
6. produire un véritable `EBOOT.PBP` ;
7. vérifier la signature et les sections du PBP ;
8. afficher un résultat compréhensible.

Le build doit échouer explicitement si PSPDEV est absent. Aucun faux EBOOT ne
doit être présenté comme un build réussi.

### 7.5 Gestion de PSPDEV

L'IDE doit détecter :

- `psp-g++` ;
- `psp-config` ;
- `make` ;
- les outils de création PBP ;
- la version de PSPSDK.

La première version peut guider l'installation. Une version ultérieure pourra
gérer une chaîne PSPDEV isolée et versionnée par l'application.

### 7.6 Test dans PPSSPP

Le logiciel doit permettre de :

- détecter PPSSPP ;
- sélectionner son emplacement si nécessaire ;
- lancer l'EBOOT du build courant ;
- arrêter et redémarrer la session de test ;
- afficher les logs disponibles ;
- distinguer clairement un test PPSSPP d'un test matériel.

### 7.7 Installation sur PSP

Le bouton **Installer sur ma PSP** doit :

- demander ou détecter le volume PSP ;
- vérifier la présence du répertoire `PSP/GAME` ;
- refuser toute autre destination ;
- installer uniquement dans `PSP/GAME/<nom-du-projet>` ;
- copier l'EBOOT et les ressources nécessaires ;
- demander confirmation avant d'écraser un projet existant ;
- ne jamais toucher aux sauvegardes sans action explicite ;
- vérifier la copie finale ;
- indiquer comment lancer le jeu depuis le XMB.

### 7.8 PBP Studio

Une section dédiée doit reproduire les fonctions utiles de PBP Unpacker :

- ouvrir et valider un PBP ;
- afficher ses huit sections ;
- afficher les offsets et tailles ;
- prévisualiser les PNG ;
- lire `PARAM.SFO` ;
- extraire une section ;
- extraire toutes les sections ;
- remplacer ou vider une section ;
- construire un nouvel `EBOOT.PBP` ;
- personnaliser l'icône et les images du projet.

Dans le parcours standard, PBP Studio travaille sur une copie ou sur le build du
projet. Il ne modifie jamais silencieusement un fichier original.

## 8. Configuration d'un projet

Exemple de configuration :

```json
{
  "schemaVersion": 1,
  "name": "MonJeu",
  "title": "Mon premier jeu",
  "language": "cpp",
  "template": "minimal",
  "target": "psp-homebrew",
  "kernelMode": false,
  "libraries": [],
  "deployFolder": "MonJeu"
}
```

Les propriétés inconnues ou interdites doivent être rejetées. Le mode standard
ne permet pas d'activer `kernelMode`.

## 9. Exigences de sécurité

### 9.1 Interdictions du MVP

- Pas de terminal intégré.
- Pas d'exécution de commande utilisateur.
- Pas de paramètres libres transmis au compilateur.
- Pas d'accès au flash PSP.
- Pas d'opérations UMD, ISO, firmware, downgrade ou jailbreak.
- Pas d'écriture hors du projet, du dossier de build et de la destination PSP.
- Pas de bibliothèque téléchargée depuis une URL saisie par l'utilisateur.
- Pas de plugin Tauri shell.
- Pas de permission HTTP générique.
- Pas de suppression récursive hors des racines approuvées.

### 9.2 Validation des chemins

- Canonicaliser les chemins avant toute opération sensible.
- Refuser les traversées `..` et les liens symboliques sortant d'une racine.
- Vérifier la destination après résolution des liens.
- Ne jamais concaténer un chemin utilisateur sans validation.
- Protéger les fichiers existants par confirmation et sauvegarde.

### 9.3 Processus externes

Les exécutables autorisés sont listés dans le backend. Les arguments doivent être
construits par l'application à partir de propriétés validées. Aucun appel ne doit
passer par `sh -c`, `cmd /c` ou une chaîne équivalente.

## 10. Bibliothèques

### 10.1 Bibliothèques initiales

- PSPSDK standard ;
- PSP GU/GUM ;
- contrôles PSP ;
- affichage et synchronisation verticale ;
- audio standard PSPSDK ;
- fonctions de fichiers utilisateur.

### 10.2 Gestion future

Chaque bibliothèque intégrée doit posséder une recette versionnée contenant :

- nom et version ;
- origine ;
- licence ;
- empreinte cryptographique ;
- options de compilation ;
- dépendances ;
- plateformes prises en charge ;
- exemple minimal validé sur PPSSPP et, si possible, sur PSP.

Les premières candidates sont intraFont, libpng, zlib, OSLib et des bibliothèques
audio compatibles.

## 11. Lua et projets historiques

Le support Lua est hors du MVP initial mais fait partie de la vision du produit.
Il devra permettre :

- la création d'un projet Lua ;
- la sélection d'un runtime approuvé ;
- l'autocomplétion des fonctions prises en charge ;
- la validation de compatibilité HM, Euphoria, LuaPlayer Plus ou OneLua ;
- l'empaquetage du runtime et des scripts ;
- l'exécution sur PPSSPP et PSP.

Une couche de compatibilité ne doit pas prétendre supporter une API sans test.

## 12. Ergonomie

### 12.1 Navigation principale

L'interface standard contient au maximum :

- Projets ;
- Code ;
- Ressources ;
- PBP Studio ;
- Tester ;
- Installer.

### 12.2 États visibles

L'utilisateur doit toujours pouvoir identifier :

- le projet actif ;
- les modifications non sauvegardées ;
- la cible active ;
- l'état de PSPDEV ;
- l'état de PPSSPP ;
- la présence d'une PSP ;
- le résultat du dernier build ;
- la différence entre avertissement et erreur.

### 12.3 Messages d'erreur

Les erreurs techniques doivent être traduites en actions compréhensibles. Le log
complet reste accessible dans un panneau secondaire, sans être la seule source
d'information.

## 13. Exigences non fonctionnelles

- Démarrage de l'interface en moins de trois secondes sur une machine moderne.
- Interface utilisable à partir de 900 × 600 pixels.
- Absence de blocage de l'interface pendant une compilation.
- Builds déterministes avec une même version de toolchain.
- Conservation atomique des fichiers de projet.
- Journalisation des actions de build et de déploiement.
- Zéro vulnérabilité critique ou élevée connue dans une version publiée.
- Fonctionnement hors ligne après installation des composants nécessaires.

## 14. Critères d'acceptation du MVP

Le MVP est accepté lorsque les scénarios suivants sont validés :

### Scénario A — Premier jeu

1. L'utilisateur crée `PremierJeu`.
2. L'IDE génère un exemple C++.
3. L'utilisateur modifie un message.
4. Le projet compile sans commande manuelle.
5. Un `EBOOT.PBP` valide est produit.
6. Le programme affiche le message dans PPSSPP.
7. Le même build démarre sur une PSP compatible.

### Scénario B — Erreur de code

1. Une erreur C++ est introduite.
2. Le build échoue sans produire de faux succès.
3. L'IDE affiche le fichier, la ligne et un message lisible.
4. Un clic conduit à la ligne concernée.

### Scénario C — Déploiement sécurisé

1. Un dossier arbitraire est sélectionné : l'installation est refusée.
2. Un volume contenant `PSP/GAME` est sélectionné : il est accepté.
3. Le jeu est copié uniquement dans son propre dossier.
4. L'intégrité de l'EBOOT copié est vérifiée.

### Scénario D — PBP Studio

1. Un EBOOT valide est ouvert.
2. Ses huit sections sont correctement listées.
3. `PARAM.SFO` et les PNG sont prévisualisés.
4. Le PBP est reconstruit.
5. Le PBP reconstruit est relu sans erreur et conserve ses données.

## 15. Tests

- Tests unitaires du parseur et constructeur PBP.
- Tests unitaires de validation des noms et chemins.
- Tests de traversée de répertoires et de liens symboliques.
- Tests de configuration projet valide et invalide.
- Tests d'intégration du build avec une version PSPDEV figée.
- Tests d'installation sur une arborescence PSP simulée.
- Tests manuels sous PPSSPP.
- Tests matériels sur au moins une PSP, puis élargissement aux différents modèles.

## 16. Livrables

### MVP

- Application Tauri macOS, Windows et Linux.
- Éditeur C++.
- Création et gestion de projets.
- Détection PSPDEV.
- Compilation en EBOOT.
- Intégration PPSSPP.
- Installation USB sur PSP.
- PBP Studio.
- Documentation de prise en main.
- Projet d'exemple.

### Après le MVP

- Autocomplétion `clangd`.
- Gestionnaire de ressources.
- Bibliothèques validées.
- Modèle de jeu 2D.
- Support Lua.
- Déploiement FTP.
- Débogage distant et logs réseau.
- Éditeur de scènes et de tilemaps.

## 17. Découpage prévisionnel

### Phase 1 — Fondations

- Stabiliser la structure Tauri.
- Finaliser le modèle de projet.
- Renforcer le confinement des chemins.
- Détecter et installer PSPDEV.

### Phase 2 — Build réel

- Générer le build contrôlé.
- Compiler le premier projet C++.
- Valider l'EBOOT produit.
- Corréler les erreurs GCC avec Monaco.

### Phase 3 — Exécution et matériel

- Intégrer PPSSPP.
- Détecter les volumes PSP.
- Installer et vérifier un build.
- Réaliser le premier test matériel.

### Phase 4 — Expérience de création

- Gérer plusieurs fichiers et ressources.
- Intégrer PBP Studio dans la navigation principale.
- Ajouter les métadonnées et images XMB.
- Améliorer les modèles de jeux.

### Phase 5 — Écosystème

- Ajouter les bibliothèques validées.
- Intégrer Lua.
- Construire la documentation interactive.

## 18. Risques

| Risque | Réponse prévue |
|---|---|
| Installation PSPDEV complexe | Toolchain isolée ou procédure guidée |
| Différences PPSSPP/PSP | Tests matériels obligatoires pour les jalons |
| Bibliothèques anciennes | Versions figées, correctifs et recettes contrôlées |
| Accès disque dangereux | Backend Rust, racines et chemins canonicalisés |
| Interface trop complexe | Mode standard réduit, fonctions progressives |
| Incompatibilités de modèles PSP | Matrice de tests et profils matériels |
| Dépendances vulnérables | Audit automatisé et versions verrouillées |

## 19. Définition de « terminé »

Une fonctionnalité est terminée uniquement si :

- son interface est utilisable sans terminal ;
- ses entrées et chemins sont validés ;
- les erreurs sont gérées ;
- elle possède des tests adaptés ;
- elle est documentée ;
- elle ne contourne pas le modèle de sécurité ;
- elle a été vérifiée sur la cible appropriée ;
- elle ne présente pas un résultat simulé comme un résultat réel.

## 20. État actuel

Les éléments suivants existent déjà dans le dépôt :

- socle Tauri 2, React, TypeScript et Rust ;
- frontend compilable ;
- backend Rust compilable ;
- Monaco Editor ;
- commandes natives limitées ;
- prototype de création, build et déploiement ;
- moteur PBP avec tests ;
- archive de recherche LuaPlayer ;
- Rust/Cargo installés sur la machine de développement.

Les prochaines priorités sont l'installation de PSPDEV, le premier build C++
réel et le lancement du même EBOOT dans PPSSPP puis sur une PSP physique.
