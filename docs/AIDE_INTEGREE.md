# Aide intégrée de PSP Reborn Studio

Le centre d'aide est accessible avec **Aide & exemples** dans la barre latérale.
Il fonctionne entièrement hors ligne et s'appuie sur les en-têtes de PSPDEV
v20260701 ainsi que sur la documentation archivée de LuaPlayer Plus r163.

## Projets prêts à créer

| Langage | Modèles |
| --- | --- |
| C++ | Bonjour PSP, contrôles, graphismes 2D, boucle de jeu, audio PCM, sauvegarde |
| Lua | Bonjour Lua, contrôles, image, sprite, timer, musique et effets |

Les six modèles C++ sont compilés automatiquement pendant les tests avec la
toolchain PSPDEV gérée. Le projet audio Lua reçoit automatiquement un fichier
`assets/sound.wav` PCM stéréo afin de fonctionner sans ressource à ajouter.

## Articles C++ / PSPSDK

1. Structure d'un homebrew et module utilisateur.
2. Écran texte de débogage.
3. Boutons et stick analogique.
4. PSP GU et double buffering.
5. Delta time et horloge RTC.
6. Sortie audio PCM stéréo.
7. Lecture et écriture d'une sauvegarde.
8. Initialisation de la pile réseau.

## Articles LuaPlayer Plus

1. Écran, couleurs et formes.
2. Contrôles PSP.
3. Images et sprites.
4. Timer et animation.
5. Effets sonores WAV.
6. Musique MP3 ou OGG.
7. Dossiers, mémoire et batterie.
8. Polices TrueType.
9. Lecture et écriture XML.
10. Extraction d'archives.
11. Caméra PSP.
12. GU et matrices GUM pour la 3D.

Chaque article fournit une description, les fonctions principales, un exemple
copiable, la provenance locale de l'API et une note lorsque du matériel ou une
bibliothèque supplémentaire est nécessaire.

## Limites assumées

- Les exemples documentent LuaPlayer Plus r163, seul runtime Lua actuellement
  embarqué et validé.
- Les anciennes variantes LuaPlayer HM restent cataloguées mais leurs
  différences d'API ne sont pas présentées comme compatibles sans leur binaire.
- L'exemple réseau C++ montre l'initialisation commune. Une connexion complète
  demande aussi les modules `pspnet_inet` et `pspnet_apctl`.
