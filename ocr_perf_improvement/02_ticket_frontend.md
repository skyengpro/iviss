# Ticket frontend — pipeline de scan de plaques

Aucun de ces correctifs n'est encore appliqué au code. Chaque section = un
défaut diagnostiqué + le correctif conçu, validé par tests unitaires et par
mesure sur photo réelle lors d'une itération antérieure. Fichiers/lignes
ci-dessous réfèrent à l'état actuel de `dev`.

> **Lire d'abord [`06_validation_documentaire.md`](06_validation_documentaire.md) §3.**
> Ce ticket a été rédigé contre une version antérieure du code : **trois noms de
> symboles et deux valeurs numériques y sont faux**. Les corrections sont
> reportées en ligne ci-dessous, marquées « ⚠️ Correction ».

## 1. Capture — résolution native et vrai still (pas une frame vidéo)

**Défaut.** `ScanViewfinder.tsx` ne passe ni résolution ni
`forceScreenshotSourceSize` à `<Webcam>` : le navigateur sert son défaut
(souvent 640×480), et `react-webcam.getScreenshot()` dimensionne son canvas à
`video.clientWidth` (pixels CSS, ~390px), pas `videoWidth`. La ROI réellement
découpée tombe à ~135×38px réels — sous le plancher Tesseract de 30-33px de
hauteur de capitale.

**Correctif.**
- `videoConstraints: { facingMode, width: {ideal: 1920}, height: {ideal: 1080} }`
- `forceScreenshotSourceSize` sur `<Webcam>`
- Nouveau module `utils/captureFrame.ts` avec échelle de capture :
  1. `ImageCapture.grabFrame()` — frame vidéo à résolution native de la piste,
     retour en un temps de frame.
  2. `ImageCapture.takePhoto()` — vraie capture *still*, mais sous **timeout
     de 1,5s** (voir §2).
  3. `getScreenshot()` avec `forceScreenshotSourceSize` — repli Safari/iOS.

  `getImageCaptureCtor()` détecte la capacité via
  `window.ImageCapture`, jamais supposée présente.

- **Mesuré** : hauteur de capitale ~130px après ce correctif (voir
  `04_mesures_verifiees.md`). Ne pas monter plus haut sans nouvelle preuve.

> **⚠️ Précision (vérifiée le 2026-07-31).** `ImageCapture` n'est supporté **ni
> sur Safari macOS ni sur Safari iOS**, et est désactivé par défaut dans Firefox
> (`takePhoto()` seul). Sur iOS, **seul le 3ᵉ barreau de l'échelle s'exécute** :
> tout le correctif de résolution y repose sur `width/height: {ideal: 1920/1080}`
> **et** `forceScreenshotSourceSize`. Ces deux réglages ne sont donc pas des
> compléments de l'échelle `ImageCapture` — sur iOS ils *sont* le correctif.
> `ideal` (et non `exact`) est le bon choix : `exact` échoue en
> `OverconstrainedError` si la définition n'est pas disponible.
>
> **La résolution de capture 1920×1080 est actée et ne doit pas être baissée.**
> Ce qui reste à mesurer, c'est la **largeur du crop OCR** (800px), pas la
> capture — voir `06_validation_documentaire.md` §4.

## 2. Capture — latence perçue du bouton (`takePhoto` en tête d'échelle)

**Défaut.** Si l'échelle ci-dessus place `takePhoto()` en premier essai (au
lieu de `grabFrame` en premier), sur Chrome/Android l'appel **reconfigure le
capteur en mode photo** : re-focus, re-cadrage, retour en plusieurs secondes.
C'est la latence perçue à l'appui du bouton.

**Correctif.**
- Ordre **`grabFrame` → `takePhoto` → `getScreenshot`**, pas l'inverse.
  `grabFrame` rend déjà plusieurs fois le plancher de 30px requis (voir
  `04_mesures_verifiees.md`) — les pixels capteur additionnels de `takePhoto`
  ne valent pas le coût.
- `takePhoto()` sous un timeout de 1,5s (`withTimeout` générique) : si le
  capteur ne répond pas assez vite, on tombe au rang suivant plutôt que de
  bloquer l'obturateur.
- **Garder `focusMode: 'continuous'`** dans `focusOnViewfinder` (ne PAS le
  changer en single-shot) : puisque la capture lit maintenant la frame
  *vidéo* plutôt que de déclencher un `takePhoto`, un focus continu pendant
  le cadrage est ce qui rend cette frame utilisable. Un verrou avant
  l'obturateur réintroduirait l'attente qu'on vient de supprimer. **Ce point
  a été vérifié à tort comme "défaut" en première passe de relecture — ne
  pas y retoucher.**

> **⚠️ Correction.** `focusOnViewfinder` **n'existe pas sur `dev`** :
> `useCamera.ts` fait 55 lignes et n'accède jamais aux `MediaStreamTrack`.
> La consigne ci-dessus porte donc sur du code **à écrire**, pas sur un
> correctif à appliquer. Idem pour le métrage `pointsOfInterest` (audit §4.4) :
> tout l'accès aux capacités de la piste est à créer, sous garde
> `track.getCapabilities()`, sans échec si non supporté.
>
> Le fondement de l'ordre `grabFrame` → `takePhoto` est confirmé **verbatim**
> par le README ImageCapture de Chromium : « takePhoto() interrupts the
> MediaStream, reconfigures the camera […] whereas grabFrame() just takes the
> next available VideoFrame […] inside the renderer process ». Contrepartie
> documentée et acceptée : `grabFrame` est plafonné à la définition de la
> piste vidéo, d'où l'importance des contraintes `ideal` du §1.

## 3. Viseur — géométrie et aspect

**Défaut.** `VIEWFINDER_ASPECT = 3.5` contre une plaque mesurée à 4.60 →
+31% de débord vertical, qui tombe sur le cadre de concessionnaire (voir
`04_mesures_verifiees.md`). Par ailleurs le cadre dessiné à l'écran et le
crop réellement envoyé étaient calculés indépendamment (overlay en
`min(100vw-64px, 384px)`, crop en `0.92 × window.innerWidth`), désaccord de
~10% sur téléphone et ~145% sur tablette.

**Correctif.**
- `VIEWFINDER_ASPECT` : 3.5 → **4.5** (marge ~2% contre la norme 4.7, garde
  un peu de tolérance au tremblement de main sans réadmettre le cadre).
- Nouveau module `utils/viewfinder.ts`, source unique de vérité :
  `computeViewfinderCrop(imgW, imgH, boxW, boxH)` — inverse le mapping
  `object-cover` du `<video>` pour replacer le cadre de visée sur les pixels
  source. Utilisé à la fois par l'overlay dessiné et par le crop envoyé.
  Retourne `null` sur dimension dégénérée (repli sur image entière).
- **`cropToViewfinder` (imageProcessor.ts)** : calculer la hauteur de sortie
  depuis l'aspect **du crop lui-même** (`crop.sh / crop.sw`), pas depuis la
  constante `VIEWFINDER_ASPECT` — sur le chemin de repli (crop null) les deux
  ne coïncident plus, et diviser par la constante écrasait un cadre portrait
  entier en bande illisible.
- Ne pas upscaler ni appliquer de noyau de netteté (héritage de l'ancien
  `preprocessForPhoto`, qui blowait ~135px à 1400px et amplifiait le bruit).
  Downscale uniquement, jamais upscale.

> **⚠️ Corrections (vérifiées le 2026-07-31).**
>
> - La constante s'appelle **`ImageProcessor.VF_ASPECT`** (`imageProcessor.ts:190`),
>   pas `VIEWFINDER_ASPECT`. Sa valeur 3.5 est bien celle du ticket.
> - **`cropToViewfinder` est du code mort** : seul `cropToViewfinderFast` est
>   appelé (`useScanPlate.ts:91`). Le correctif d'aspect vaut pour les deux —
>   ou `cropToViewfinder` est supprimé avec le reste du code mort (§ audit 2.6 :
>   `preprocessForHighRes`, `preprocessForOCR`, `scaleImage`, tous sans appelant
>   hors tests).
> - **Il existe un TROISIÈME calcul de cadre, non mentionné jusqu'ici.**
>   `preprocessForPhotoCapture` (`imageProcessor.ts:125-184`) code en dur son
>   propre aspect : `const vh = vw / 2.0;` (`:154`) — ni `VF_ASPECT` (3.5), ni
>   l'overlay. C'est le chemin de repli photo (`usePhotoCapture.ts:158`). Il
>   **doit** passer par `utils/viewfinder.ts` comme les deux autres, sinon la
>   « géométrie partagée par construction » laisse un calcul indépendant en place.
>   Le test de non-régression doit couvrir les **trois** appelants.

## 4. Qualité JPEG du chemin live

**Défaut.** `LIVE_CROP_OPTIONS.quality = 0.7` — artefacts JPEG qui
s'impriment sur les traits des glyphes, alors que le chemin photo est déjà à
0.95. Voir mesure dans `04_mesures_verifiees.md` : gain réseau négligeable
(~15 Ko) pour une perte de netteté d'entrée réelle.

**Correctif.** `LIVE_CROP_OPTIONS: { maxWidth: 800, quality: 0.95 }`. Le
plafond de coût vient de `maxWidth`, pas de la qualité.

> **⚠️ Correction.** `LIVE_CROP_OPTIONS` **n'existe pas sur `dev`** : la qualité
> est codée en dur à **0.65** (`imageProcessor.ts:665`) — donc **pire** que les
> 0.7 annoncés — et la largeur à `targetWidth = 800` (`:644`). Le chemin photo
> est à 0.92 (`:292`), pas 0.95. Le correctif reste le bon : introduire
> l'objet `LIVE_CROP_OPTIONS: { maxWidth: 800, quality: 0.95 }` et aligner le
> chemin photo sur 0.95. La documentation Tesseract (*ImproveQuality*)
> déconseille les artefacts de compression en entrée.

## 5. Scan live — seuil de confiance et logique de stabilité

> **⚠️ Correction.** `MIN_LIVE_CONFIDENCE` **n'existe pas sur `dev`**. Les
> valeurs réelles sont passées en ligne : `useScanPlate.ts:36-39` appelle
> `useStabilityDetection({ requiredMatches: 2, minConfidence: 40 })`. (Les
> défauts du hook lui-même sont `3` / `75` — `useStabilityDetection.ts:18-19` —
> mais l'appelant les écrase.) Lire donc « 40 » partout où le ticket dit « 55 ».
> **Le diagnostic est inchangé** : face à un régime mesuré 0-63 de médiane
> proche de 0-16, un seuil à 40 rejette encore la quasi-totalité des lectures.

**Défaut, cause du "scan qui ne se termine jamais".** `MIN_LIVE_CONFIDENCE =
55` datait de l'époque où le backend écrasait la confiance par 0.90/0.50.
Une fois la vraie `mean_text_conf` de Tesseract exposée (ticket backend §4),
la confiance mesurée sur plaques réelles tombe à 0-63, médiane proche de
0-16 — Tesseract renvoie 0 sur une bonne part des lectures **correctes**.
Un seuil à 55 rejette donc la quasi-totalité des lectures : la fenêtre de
stabilité ne se remplit jamais, le scan tourne indéfiniment.

Second verrou combiné : `REQUIRED_STABLE_MATCHES = 2` exigeait des lectures
**consécutives identiques**. Sur un flux bruité, deux lectures correctes ne
se suivent presque jamais immédiatement — un mauvais lecture s'intercale et
remet le compteur à zéro à chaque fois.

**Correctif — réécriture de `useStabilityDetection.ts` :**
- Confiance : **filtre, pas décision**. `minConfidence` reste un réglage
  (défaut 0 en attendant un jeu de référence mesuré), mais l'agrément entre
  plusieurs frames indépendantes est le signal réel, pas la confiance brute
  d'une seule.
- **Vote majoritaire glissant** au lieu de compteur consécutif : fenêtre des
  `windowSize` dernières lectures (défaut 5, ≥ `requiredMatches`),
  confirmation dès que `requiredMatches` (défaut 3) d'entre elles portent la
  même plaque. Une lecture erronée intercalée n'efface plus l'accord déjà
  engrangé par les autres.
- La confiance rapportée sur un résultat stable est le **max** des lectures
  d'accord, pas leur moyenne (la moyenne tire vers le bas à cause des zéros
  de Tesseract sur des lectures par ailleurs correctes).
- Dans `useScanPlate.ts`, seules les lectures `format_valid === true` votent
  (`addDetection` n'est plus appelé sur les lectures non formées) — le bruit
  qui ne parse même pas comme plaque ne doit ni retarder le consensus ni
  risquer de le confirmer à tort.
- Retirer l'auto-abort mort en tête de `processFrame` : la boucle attend déjà
  la fin de chaque frame et `isProcessingRef` empêche la ré-entrée, donc il
  n'y a jamais de requête en vol à annuler à cet endroit.

## 6. Charge CPU téléphone — coaching concurrent au scan live

**Défaut.** `useCaptureCoaching` (guidage de cadrage temps réel) n'était
gardé que par `photoState === 'idle'`, qui reste vrai **pendant tout un scan
live** (`photoState` ne change qu'en mode photo). Résultat : deux boucles
tournaient en parallèle sur le téléphone — le scan live (~10Hz, crop
800×229) et le coaching (~4Hz), ce dernier encodant un JPEG **plein format
1080×1920** à chaque tick pour une mesure qui se renormalise de toute façon à
une largeur de sonde fixe.

**Correctif.**
- Restreindre l'activation du coaching à `mode === 'photo' && photoState ===
  'idle' && !isScanning`.
- Nouvelle méthode `getPreviewScreenshot` dans `useCamera.ts` : screenshot
  dédié au coaching, capé à 640px de large (`getScreenshot({ width, height })`
  — l'API `react-webcam` l'accepte), au lieu du screenshot plein format
  partagé avec le scan.

> **⚠️ Correction.** `useCaptureCoaching` **n'existe pas sur `dev`** : il n'y a
> aujourd'hui aucune boucle de coaching, seulement le gate bloquant
> post-capture (`usePhotoCapture.ts:123-128`). Cette section n'est donc pas un
> correctif à appliquer mais une **contrainte de conception** du hook à créer :
> le garder sous `mode === 'photo' && photoState === 'idle' && !isScanning`
> **dès la première écriture**, et lui donner sa propre source d'image capée à
> 640px. C'est un piège déjà rencontré, pas une hypothèse.

## Tests à ajouter/adapter

Voir `05_tests_de_non_regression.md` pour le détail. En bref : fixtures de
confiance recalibrées sur le régime réel (0-63, pas 76-92), test de tolérance
au misread intercalé (la régression qui bloquait le scan), test que la
géométrie du viseur (overlay = crop) reste garantie par construction.

## Code de référence

Le diff complet de l'itération antérieure est dans l'historique de la
conversation qui a produit ce dossier — non rejoué ici verbatim pour rester un
ticket, pas un patch. Les fichiers todo côté frontend : `utils/viewfinder.ts` (nouveau),
`utils/captureFrame.ts` (nouveau), `utils/imageProcessor.ts`,
`hooks/feature/useScanPlate.ts`, `hooks/feature/useStabilityDetection.ts`,
`hooks/feature/useCamera.ts`, `hooks/feature/useCaptureCoaching.ts` (nouveau),
`pages/mobile/MobileScan.tsx`, `components/mobile/scan/ScanViewfinder.tsx`.
