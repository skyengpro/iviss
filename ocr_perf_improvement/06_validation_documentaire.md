# Validation du dossier contre le code réel et la documentation officielle

Session du 2026-07-31. Deux vérifications indépendantes ont été menées :

1. **Contre le code** — chaque affirmation des tickets a été relue ligne à ligne
   dans `dev` (la branche courante `perf/ocr-pipeline-resolution-and-speed`
   n'ajoute que ce dossier + une ligne dans `vehicle_data_cache.rs` : le code
   OCR est bien à l'état pré-audit).
2. **Contre la documentation officielle** — Tesseract (tessdoc), Leptonica
   (`src/binarize.c`), MDN/Chromium (`ImageCapture`), `react-webcam`, et les
   sources des crates `leptess 0.14.0` / `tesseract-plumbing 0.8.0` /
   `leptonica-sys 0.4.9` extraites du registre Cargo local.

**Verdict global : le diagnostic tient. Aucune contradiction de fond avec la
documentation.** Les corrections ci-dessous sont des précisions de nommage, un
point de mesure à trancher, et trois défauts réels que les tickets ne couvrent
pas.

---

## 1. Ce que le code confirme

| Affirmation | Emplacement vérifié | Statut |
|---|---|---|
| `videoConstraints` sans `width`/`height` | `ScanViewfinder.tsx:27-29` | ✅ |
| `forceScreenshotSourceSize` absent | `ScanViewfinder.tsx:42-51` | ✅ |
| Overlay `aspect-[7/2]` (=3.5) dans une boîte `p-8` + `max-w-sm` | `ScanViewfinder.tsx:55-56` | ✅ |
| Crop calculé indépendamment sur `window.innerWidth × 0.92` | `imageProcessor.ts:199-207` | ✅ |
| Upscale 1400×400 + noyau de netteté | `imageProcessor.ts:257-289` | ✅ |
| Gate qualité : Laplacien à 400×114, seuil fixe 80, fail-open | `imageProcessor.ts:329-416` | ✅ |
| Deux formules de luminance incohérentes | `imageProcessor.ts:351` vs `:375` | ✅ |
| Rejet post-capture | `usePhotoCapture.ts:123-128` | ✅ |
| Double requête photo | `usePhotoCapture.ts:157-161` (conditionnelle) | ✅ |
| Aucun timeout client sur `photoPlate()` | `usePhotoCapture.ts:138-141` | ✅ |
| Auto-abort mort en tête de `processFrame` | `useScanPlate.ts:82-85` | ✅ |
| Boucle live à 100 ms alors que le commentaire dit 500 ms | `useScanPlate.ts:22` vs `:178` | ✅ |
| Compteur consécutif + reset sur misread | `useStabilityDetection.ts:43-52` | ✅ |
| Confiance rapportée = moyenne | `useStabilityDetection.ts:54-55` | ✅ |
| `opt-level = "z"` | `Cargo.toml:7-8` | ✅ |
| `docker-compose` cible `development` | `docker-compose.yml:158` | ✅ |
| Ordre `deskew(gray) → contrast → threshold → morpho → border` | `ocr_service.rs:84-96` | ✅ |
| `deskew` : 15 rotations pleine résolution, remplissage `Luma([0])` | `ocr_service.rs:417-442` | ✅ |
| `morphology_open` naïf, ~36 accès bornés/pixel | `ocr_service.rs:450-490` | ✅ |
| `ADAPTIVE_RADIUS = 40` absolu | `ocr_service.rs:12` | ✅ |
| Fichiers `/tmp` + fuite sur les `return finalize(...)` anticipés | `ocr_service.rs:113-118`, `:126-131`, cleanup `:218-221` | ✅ |
| Jusqu'à 6 passes PSM (7 / 7-inv / 8 / 11 / 11-inv / 13) | `ocr_service.rs:119-209` | ✅ |
| Retour dès le premier `format_valid`, à confiance quelconque | `ocr_service.rs:124-133` etc. | ✅ |
| Confiance écrasée 0.90 / 0.50 | `ocr_service.rs:240-244` | ✅ |
| `tess_init_elapsed` jeté | `ocr_service.rs:225` | ✅ |
| `load_system_dawg` / `load_freq_dawg` / `tessedit_do_invert` absents | `ocr_service.rs:260-278` | ✅ |
| Chemin tessdata codé en dur | `ocr_service.rs:264` | ✅ |
| Repli `cleaned.len() >= 4` | `ocr_service.rs:563-566` | ✅ |
| `fuzzy_correct` : masque glissant non borné | `plate_format.rs:159-173` | ✅ |
| `^\d{7}$` et `^[A-Z]{2}\d{4}[A-Z]$` atteignables par substitution seule | `plate_format.rs:27,29-30,47` + `correct_digit` `:311-323` | ✅ |
| Aller-retour JPEG q75 entre crop et OCR | `photo_ocr_service.rs:31-36` | ✅ |
| Double passe inconditionnelle si `!format_valid` | `photo_ocr_service.rs:40-56` | ✅ |
| `color_adaptive_crop` : 5 passes plein cadre, HSV flottant par pixel | `photo_ocr_service.rs:59-152` | ✅ |
| Vote caractère par caractère dans `pick_best` | `photo_ocr_service.rs:196-225` | ✅ |
| `enhance_photo_result` force `format_valid = true` | `photo_ocr_service.rs:168-172` | ✅ |
| `handle.abort()` sur `spawn_blocking` (no-op) | `handlers/photo.rs:94` | ✅ |
| Aucun `Semaphore` | `handlers/{photo,scan}.rs` | ✅ |
| `DefaultBodyLimit` absent → limite axum 2 Mo | `routes.rs:148-157` | ✅ |
| `client_max_body_size` absent | `frontend/nginx.conf:19-30` | ✅ |
| Clés i18n `qualityToo*` absentes de `en.json` **et** `fr.json` | — | ✅ |
| Code mort : `cropToViewfinder`, `preprocessForHighRes`, `preprocessForOCR`, `scaleImage` | aucun appelant hors tests | ✅ |
| Workbox `/^\/api\/.*/i` inactif (fausse alerte, déjà rétractée) | `vite.config.ts:86` | ✅ |
| `set_image_from_mem(&mut self, img: &[u8])` existe | `leptess-0.14.0/src/lib.rs:117` | ✅ |
| `LepTess::new` sans paramètre OEM | `leptess-0.14.0/src/lib.rs:100` | ✅ |

---

## 2. Ce que la documentation officielle confirme

- **Polarité.** *ImproveQuality* : « for 4.x version use dark text on light
  background ». Valide la normalisation de polarité **avant** Tesseract, et
  `tessedit_do_invert=0`.
- **Bordures sombres.** *ImproveQuality* : les bordures de scan « can be
  erroneously picked up as extra characters ». Valide le remplissage de rotation
  en 255 plutôt qu'en 0.
- **Taille de bordure.** *ImproveQuality* recommande « a small border (e.g.
  10 px) » et prévient que les grandes bordures posent problème. Les 30 px
  actuels restent acceptables sur un crop ~1000×300, mais ne sont pas la valeur
  documentée — à ne pas augmenter.
- **`tessedit_do_invert`.** Confirmé : par défaut Tesseract tente l'OCR deux fois
  (normal + inversé), ce qui double le temps sur des pages sans texte inversé.
- **`tessedit_char_whitelist`.** Confirmé non honoré sous LSTM (issues
  tesseract-ocr#751, #998 ; PR #2294). La correction doit vivre en
  post-traitement — conforme au ticket.
- **`grabFrame` avant `takePhoto`.** Confirmé **verbatim** par le README
  ImageCapture de Chromium : « takePhoto() interrupts the MediaStream,
  reconfigures the camera, takes the photo […] and then resumes the
  MediaStreamTrack, whereas grabFrame() just takes the next available VideoFrame
  […] inside the renderer process ». L'ordre du ticket FE §2 est le bon.
- **`forceScreenshotSourceSize`.** Confirmé dans la source `react-webcam` : à
  `false` (défaut) le canvas fait `minScreenshotWidth || video.clientWidth` ; à
  `true` il fait `video.videoWidth` / `video.videoHeight`.
- **`ideal` vs `exact`.** MDN : `exact` échoue en `OverconstrainedError` si la
  résolution n'est pas disponible ; `ideal` retombe sur la plus proche. Le
  ticket utilise `ideal` — correct.
- **Sauvola (Leptonica `src/binarize.c`).** `t = m * (1 - k * (1 - s / 128))`,
  `k` typiquement 0.35 (plage 0.2–0.5), fenêtre `2*whsize+1` avec `whsize >= 7`,
  entrée 8 bpp obligatoire.

---

## 3. Corrections à apporter aux tickets

### 3.1 Écarts de nommage / de valeur (les tickets décrivent une itération antérieure du code)

| Ticket | Dit | Réalité sur `dev` |
|---|---|---|
| FE §3 | `VIEWFINDER_ASPECT` | `ImageProcessor.VF_ASPECT` — `imageProcessor.ts:190`. Valeur 3.5 correcte. |
| FE §4 | `LIVE_CROP_OPTIONS.quality = 0.7` | Cet objet n'existe pas. Qualité **codée en dur à 0.65** — `imageProcessor.ts:665` ; `maxWidth` = `targetWidth = 800` — `:644`. **Pire que ce que dit le ticket.** |
| FE §5 | `MIN_LIVE_CONFIDENCE = 55` | N'existe pas. `useScanPlate.ts:36-39` passe `minConfidence: 40, requiredMatches: 2`. (`00_audit` §2.6 a la bonne valeur.) Le diagnostic est inchangé : 40 rejette encore la quasi-totalité d'un régime médian 0-16. |
| FE §2 | « garder `focusMode: 'continuous'` dans `focusOnViewfinder` » | `focusOnViewfinder` **n'existe pas** (`useCamera.ts` fait 55 lignes, aucun accès aux pistes). C'est une consigne pour du code à écrire, pas un correctif. |
| FE §6 | « `useCaptureCoaching` n'était gardé que par `photoState === 'idle'` » | `useCaptureCoaching` **n'existe pas** sur `dev`. §6 est une consigne de conception, pas un correctif. |
| FE §3 | corriger `cropToViewfinder` | `cropToViewfinder` est **du code mort** ; seul `cropToViewfinderFast` est appelé (`useScanPlate.ts:91`). Le correctif vaut pour les deux. |
| BE §2 | « dans `init_tesseract` » | La fonction s'appelle `take_tesseract` — `ocr_service.rs:260`. |

### 3.2 Trois défauts réels qu'aucun document ne mentionne

1. **Bordure noire sur le chemin inversé.** `ocr_service.rs:137` et `:194` font
   `add_border(&invert_image(&binary), 0, 255)`. Or `binary` porte déjà une
   bordure **blanche** de 30 px (`:96`) : l'inverser la rend **noire**, et le
   `add_border(..., 0, ...)` est un no-op. Toutes les passes inversées voient
   donc un cadre noir de 30 px — exactement les « bordures sombres lues comme
   caractères parasites » que *ImproveQuality* dit de supprimer. À corriger en
   inversant **avant** d'ajouter la bordure (l'ordre corrigé du ticket BE §3 le
   règle structurellement, mais le point mérite un test dédié).

2. **La confiance est fabriquée à deux autres endroits que `finalize`.** Le
   ticket BE §7 ne vise que `ocr_service::finalize`. Or
   `photo_ocr_service.rs:175-177` (`enhance_photo_result`) et `:222`
   (`pick_best`) réécrivent aussi `confidence = 0.90`. Si on ne les traite pas,
   la recalibration du seuil frontend (ticket FE §5) reste inopérante **sur tout
   le chemin photo**. BE §6 supprime le bloc de vote (donc `:222`) ; il faut
   ajouter explicitement `:175-177`.

3. **`preprocessForPhotoCapture` a un troisième aspect, codé en dur à 2.0.**
   `imageProcessor.ts:154` : `const vh = vw / 2.0;` — ni `VF_ASPECT` (3.5), ni
   l'overlay. C'est le chemin de repli photo (`usePhotoCapture.ts:158`). Il doit
   passer par le futur `utils/viewfinder.ts` comme les deux autres, sinon la
   « géométrie partagée par construction » du ticket FE §3 laisse un troisième
   calcul indépendant en place.

### 3.3 Un point de couverture manquant (infra)

`frontend/nginx.conf` documente lui-même son `location /api/` comme un **repli
dev / même origine** : en production le frontend tape
`VITE_API_URL = https://api.iviss.skyengpro.app` en cross-origin. Ajouter
`client_max_body_size 10m` dans ce nginx **ne protège donc pas la production**.
Aucune configuration d'ingress (traefik / ALB / autre nginx) n'existe dans le
dépôt pour cet hôte. **À tracer hors dépôt avant de monter en résolution**,
sinon le 413 opaque que le ticket veut éviter reviendra par l'autre porte.

---

## 4. Le seul point de fond à trancher : la cible de hauteur de capitale

`04_mesures_verifiees.md` conclut « ~130 px mesurés contre un plancher de
30-33 px, marge ×4, ne pas ré-augmenter ». La documentation ne dit pas tout à
fait cela :

- *ImproveQuality* ne donne **aucun** chiffre de hauteur de capitale ; il
  renvoie au test « Optimal image resolution » de willus.com.
- Ce test conclut : « **there is a sweet spot for Tesseract of about 30 pixels
  for the height of a capital letter** » — un **optimum**, pas un plancher. Son
  auteur note explicitement que la précision **redescend** en haute résolution
  (« Tess v4.0.0 definitely has a consistent issue with high-res fonts »).

Autrement dit, **130 px n'est pas « ×4 de marge », c'est ×4 au-delà de
l'optimum**, du côté où l'auteur du test mesure une dégradation. Cela ne
remet pas en cause la cause racine n°1 (21-25 px était bien trop bas), mais la
conclusion « ne plus y toucher » n'est pas établie.

**Ce n'est pas bloquant et cela ne change aucun autre correctif.** Ce qu'il faut
faire : garder la capture à `1920×1080` (les pixels servent au cadrage et au
crop), et **balayer la largeur du crop OCR** (`maxWidth` 800 aujourd'hui) sur la
photo de référence pour situer l'optimum réel — le même protocole que le balayage
`ADAPTIVE_C` déjà rejoué dans `samples/binarize_replica.py`. Tant que la mesure
n'est pas faite, on garde 800 px : c'est l'état sur lequel les ~130 px ont été
mesurés.

---

## 5. Précisions sur les limites des crates (à ne pas re-chercher)

- **OEM : la conclusion du README est confirmée, et plus forte qu'énoncée.**
  `LepTess::new(Option<&str>, &str)` n'a pas de paramètre OEM, **et** le champ
  `LepTess.tess_api` est **privé** (`leptess-0.14.0/src/lib.rs:97`). Le
  `TessApi.raw` public de `tesseract-plumbing` est donc inatteignable depuis un
  `LepTess`.
- **Conséquence non relevée dans les tickets : `invert_threshold` et
  `thresholding_method` sont également inatteignables.** `set_variable` n'accepte
  que l'énum `leptess::Variable`, qui contient bien `LoadSystemDawg`,
  `LoadFreqDawg`, `TesseditDoInvert` et `TesseditCharWhitelist` (vérifié dans
  `variable.rs`), mais **ni `invert_threshold` ni `thresholding_method`**. Il n'y
  a pas de setter par chaîne libre.
- **Importance pratique :** `tessedit_do_invert` est **déprécié et sera retiré en
  Tesseract 6.0**, remplacé par `invert_threshold` (défaut 0.7 ; mettre 0.0 pour
  désactiver). L'image Docker est `debian:bookworm-slim` + `libtesseract5`, soit
  **Tesseract 5.3.0**, où `tessedit_do_invert=0` fonctionne encore. Le correctif
  est donc valable aujourd'hui, mais **c'est une dette datée** : la montée en
  Tesseract 6 exigera de passer par `tesseract-plumbing 0.8` en dépendance
  directe (`init_4` accepte un `TessOcrEngineMode`, `TessBaseApi::set_variable`
  accepte deux `&CStr`).
- **Leptonica.** `leptonica-plumbing 1.4.0` n'expose que `Pix`, `Box`, `Boxa`,
  `Pixa` — **pas** les fonctions de binarisation. `leptonica-sys 0.4.9` génère
  ses liaisons par bindgen sur les en-têtes système en ne bloquant que
  `max_align_t` : `pixSauvolaBinarizeTiled`, `pixDeskew`, `pixOtsuAdaptiveThreshold`
  y sont donc tous présents — mais ni l'un ni l'autre n'est une dépendance
  **directe** du backend aujourd'hui.
- **Sauvola vs notre binarisation — décision : on passe à Sauvola.**
  `adaptive_threshold` (`ocr_service.rs:493`) est un seuillage à **moyenne
  locale − C** (Bradley/Wellner) : il ignore la variance locale, là où Sauvola
  l'utilise (`t = m·(1 − k·(1 − s/128))`, k ≈ 0.35). C'est précisément le bon
  comportement sous éclairage inégal — le cas de terrain.
  **Voie retenue : implémentation en Rust**, par une seconde image intégrale des
  carrés pour obtenir l'écart-type en O(1) par pixel. Aucune dépendance
  nouvelle, aucun `unsafe`. Les deux voies « toutes faites » sont fermées :
  `thresholding_method` (Tesseract 5) n'est pas dans l'énum `leptess::Variable`,
  et `pixSauvolaBinarizeTiled` imposerait `leptonica-sys` en dépendance directe
  plus de la FFI `unsafe`.
  **Réserve à tenir :** le balayage `ADAPTIVE_C` a montré des glyphes
  visuellement identiques sur la photo de référence — sur *ce* corpus la
  binarisation n'est pas le facteur limitant, et le gain est **théorique**. Le
  changement est justifié par son fondement, pas par une mesure. Il doit venir
  avec une fixture à gradient d'illumination où Sauvola fait strictement mieux
  (`05_tests_de_non_regression.md`), et ne pas être présenté comme un gain
  mesuré tant que le jeu de terrain ne l'a pas montré.

---

## 6. Conclusion

Les deux tickets sont exécutables tels quels, moyennant :

- les renommages du §3.1 ;
- l'ajout des trois défauts du §3.2 (bordure noire inversée, confiance fabriquée
  dans `photo_ocr_service`, troisième aspect dans `preprocessForPhotoCapture`) ;
- le suivi hors dépôt de la limite de corps de requête en production (§3.3) ;
- une mesure — non bloquante — sur la largeur de crop OCR (§4).

**Toutes ces corrections ont été reportées dans les fichiers concernés le
2026-07-31**, marquées « ⚠️ » à l'endroit du texte qu'elles rectifient. Deux
décisions ont par ailleurs été actées et propagées :

| Décision | Où |
|---|---|
| Résolution de capture **1920×1080 conservée** — la documentation n'en prescrit aucune ; le levier mesurable est la largeur du crop OCR, pas la capture | `README.md`, `02` §1, `04` |
| Binarisation **Sauvola** en remplacement de la moyenne locale − C, implémentée en Rust | `README.md`, `03` §3 bis, `04`, `05` |
