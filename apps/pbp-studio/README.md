# PBP Studio

Première brique autonome de PSP Reborn Studio. Elle reproduit les fonctions
essentielles de PBP Unpacker : ouverture et validation d'un PBP, liste et aperçu
des huit sections, extraction individuelle ou globale, remplacement des
sections et construction d'un nouvel `EBOOT.PBP`. `PARAM.SFO` est décodé en
lecture et les PNG sont prévisualisés.

```sh
npm test
npm start
```

Puis ouvrir http://localhost:4173. Aucune donnée n'est envoyée sur Internet :
les fichiers sont traités intégralement dans le navigateur.
