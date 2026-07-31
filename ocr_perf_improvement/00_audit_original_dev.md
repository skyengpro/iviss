# Audit du pipeline de capture / scan de plaques — et plan d'amélioration

> **⚠️ Statut (mis à jour le 2026-07-31) : rien de ce qui suit n'est encore
> appliqué au code.** Le pipeline OCR est à l'état **pré-audit** : famine de
> résolution, `opt-level = "z"`, aucune observabilité par étage, confiance
> fabriquée à 0.90/0.50, etc. La branche
> `perf/ocr-pipeline-resolution-and-speed` porte ce dossier de conception et
> constitue le chantier actif de l'amélioration.
>
> Ce fichier est conservé tel qu'il a été écrit, pour son diagnostic (§1, §2,
> §2.5, §2.7) — **re-vérifié ligne à ligne contre le code le 2026-07-31 et
> confirmé exact**, numéros de ligne compris (voir
> [`06_validation_documentaire.md`](06_validation_documentaire.md) §1). **Sa §4
> ("Implémentation livrée") ne décrit rien qui existe** — pour l'état réel de
> ce qui a été conçu, validé et testé, voir
> [`02_ticket_frontend.md`](02_ticket_frontend.md) et
> [`03_ticket_backend.md`](03_ticket_backend.md), qui la remplacent. Sa §4bis
> reste utile comme détail d'implémentation, sous réserve des corrections
> listées dans les tickets **et dans `06_validation_documentaire.md` §3**.

---

## 1. Contexte

Deux symptômes rapportés en test terrain, sur plusieurs modèles de téléphones :

1. **Extraction aléatoire en faible luminosité / faible contraste.** Un gate de qualité
   d'image a été ajouté côté frontend pour rejeter les captures floues ou mal exposées
   avant l'envoi. Ce gate réduit la fenêtre de traitement OCR et frustre l'utilisateur,
   qui voit sa photo rejetée à répétition.
2. **Lenteur du scan**, perçue jusqu'à ~1 minute.

L'audit couvre toute la chaîne, du choix de l'outil de capture jusqu'au post-traitement
backend. **Conclusion : aucun des deux symptômes n'est imputable à la qualité des caméras
des téléphones testés.**

### Chaîne actuelle

```
react-webcam (getUserMedia)  →  getScreenshot()  →  assessImageQuality()  →  preprocessForPhoto()
   ↓ multipart POST /api/v1/photo/plate
photo_ocr_service::photo_plate()  →  color_adaptive_crop()  →  ré-encodage JPEG  →  ocr_service::scan_plate()
   →  deskew → contrast stretch → adaptive threshold → morphology → PNG dans /tmp → jusqu'à 6 passes Tesseract
```

**Technologies en place**
- Frontend : `react-webcam@7.2.0` (enveloppe fine autour de `getUserMedia` + `<video>` +
  `canvas.toDataURL()`), Vite + React 18 en PWA. `tesseract.js@7` est déclaré dans
  `package.json` mais **jamais importé** — l'OCR est intégralement côté serveur.
- Backend : Rust / axum, OCR via `leptess 0.14` (liaisons FFI vers libtesseract + Leptonica),
  traitement d'image via `image 0.25` et `imageproc 0.25`. Modèle de langue `eng` uniquement,
  aucun modèle spécialisé plaques.

---

## 2. Diagnostic

### 2.1 Cause racine n°1 — famine de résolution

C'est l'explication réelle du « problème de luminosité ». Ce n'est pas un problème de
lumière : c'est un problème de **pixels réels**, et la lumière ne fait que révéler une marge
inexistante.

Chaîne de perte, vérifiée dans le code :

| # | Étape | Effet |
|---|---|---|
| 1 | `videoConstraints = { facingMode }` sans `width`/`height`<br>`frontend/src/components/mobile/scan/ScanViewfinder.tsx:27-29` | Le navigateur sert sa résolution par défaut (typiquement **640×480**), jamais le capteur natif |
| 2 | `forceScreenshotSourceSize` non passé<br>vérifié dans `react-webcam/dist/react-webcam.js:276-284` | Le canvas est dimensionné à `video.clientWidth` — des pixels **CSS** (~390 px), pas `videoWidth`. Screenshot ≈ **390×293** |
| 3 | `getSafeCrop` avec `imgW=390, imgH=293, W=390, H≈780`<br>`frontend/src/utils/imageProcessor.ts:198-233` | `scale = max(1.0, 2.66) = 2.66` → `sw ≈ 135`, `sh ≈ 38` |

> **La ROI réellement découpée fait ~135 × 38 pixels réels.** Elle est ensuite upscalée
> ×10 vers 1400×400, puis passée dans un noyau de netteté — qui ne fait qu'amplifier le
> bruit et les artefacts d'interpolation.

Dans 38 px de haut, les caractères de la plaque font **~21-25 px réels**. La documentation
Tesseract (*ImproveQuality*) fixe un plancher de **30-33 px de hauteur de capitale**.
Nous sommes structurellement **sous le minimum documenté**, avant même de parler d'éclairage.

D'où le caractère aléatoire des résultats : en basse lumière le gain capteur monte et le
bruit apparaît ; à 2-3 px de largeur de trait il n'existe aucune redondance permettant de
séparer le bruit du caractère. Il n'y a pas de marge à récupérer — il n'y a pas
d'information au départ.

**Preuve que c'est un oubli et non un choix :** `frontend/src/pages/mobile/MobileCarteGrise.tsx:302-317`
applique déjà `{width: 1280, height: 720}` **et** `forceScreenshotSourceSize={true}`.
Le viewfinder de scan ne l'a jamais reçu.

En demandant 1920×1080 avec `forceScreenshotSourceSize`, la ROI passe de ~135×38 à
**~600×170 px réels** : ×4 en linéaire, ×16 en surface, **à charge utile réseau identique**.

### 2.2 Ce n'est pas une photo, c'est une frame vidéo

`react-webcam.getScreenshot()` dessine la frame courante de l'élément `<video>` sur un
canvas. C'est une **capture d'écran d'un flux vidéo**, pas une prise de vue. Conséquences,
qui expliquent précisément le flou observé en plein soleil :

- L'auto-exposition **vidéo** mesure sur toute la scène. Ciel ou route brillants → la plaque
  part dans l'ombre ; ou la plaque rétroréfléchissante sature à blanc. Dans les deux cas le
  contraste des caractères s'effondre.
- L'autofocus vidéo est **continu et lent par conception** (il doit être fluide, pas net).
  Si l'on déclenche juste après avoir pointé, la convergence n'a pas eu lieu.
- Le flux vidéo subit un **débruitage temporel** de l'ISP, qui étale le détail sur les
  contenus en mouvement.
- Aucune des opérations que réalise l'application photo native : verrouillage AF/AE, HDR,
  empilement multi-frames.

**La vraie photo existe sur le web :** `ImageCapture.takePhoto()` déclenche une capture
*still* réelle à la résolution photo du capteur. Supportée sur Chrome/Android, **pas sur
Safari/iOS** — d'où la nécessité d'une échelle de repli (voir §4.4).

### 2.3 Cause racine n°2 — coût CPU backend

Le backend **ne peut pas** prendre 60 s : `OCR_TIMEOUT_MS = 9000`
(`iviss-backend/src/handlers/photo.rs:17`). La minute perçue est une **cascade** :

- Le mode photo envoie **2 requêtes** (`frontend/src/hooks/feature/usePhotoCapture.ts:157-161`)
  = 18 s de plafond ; l'utilisateur réessaie ensuite.
- Surtout, **`handle.abort()` sur une tâche `spawn_blocking` déjà démarrée est un no-op**
  (`handlers/photo.rs:94`). Le thread continue à brûler un cœur après l'envoi du 504.
- Aucun `Semaphore` ni limite de concurrence n'existe dans le code. Chaque timeout dégrade
  donc le suivant : **boucle de rétroaction positive**. C'est ce qui rend la lenteur
  intermittente.

Coût par requête photo dans le cas **courant** (échec de la passe 1 — quasi systématique
aujourd'hui, du fait de §2.1) :

| Étage | Coût | Emplacement |
|---|---|---|
| `color_adaptive_crop` | jusqu'à **5 passes plein cadre**, conversion HSV flottante par pixel | `services/photo_ocr_service.rs:59-152` |
| `deskew` | **15 rotations bilinéaires pleine résolution** + 15 scans `get_pixel` | `services/ocr_service.rs:410-446` |
| `morphology_open` | ~36 accès `get_pixel`/`put_pixel` bornés **par pixel** (naïf, non séparable) | `services/ocr_service.rs:450-490` |
| Fichiers `/tmp` | 2 encodages PNG + jusqu'à 6 décodages Leptonica **via le système de fichiers** | `services/ocr_service.rs:112-118` |
| Passes Tesseract | jusqu'à **6** (PSM 7 / 7-inv / 8 / 11 / 11-inv / 13) | `services/ocr_service.rs:119-215` |
| **× 2** | tout ce qui précède tourne **deux fois**, la seconde sur une image **2,25× plus grande** | `services/photo_ocr_service.rs:38-56` |

> **Pire cas : 12 appels `get_utf8_text()`, 30 rotations pleine résolution, 4 fichiers PNG.**

**Multiplicateur global :**

```toml
[profile.release]
opt-level = "z"     # Optimise pour la taille
```
`iviss-backend/Cargo.toml:7-8`

Un pipeline de traitement d'image CPU-bound compilé **pour la taille du binaire**.
`opt-level = "z"` désactive la vectorisation des boucles et l'inlining agressif — or
`get_pixel` n'est bon marché *que* s'il est inliné. Sur ce type de code, l'impact est
typiquement de **×2 à ×5**. C'est une ligne de configuration, et le meilleur ratio
effort/gain de tout le projet.

**Dégât collatéral :** `photo_ocr_service.rs:33` ré-encode l'image en **JPEG qualité 75**
(défaut du crate `image`) juste avant de la passer à l'OCR, qui la redécode aussitôt
(`ocr_service.rs:64`). La documentation Tesseract déconseille explicitement les artefacts
JPEG en entrée. Du détail est détruit gratuitement, entre deux étapes internes.

### 2.4 Le gate qualité frontend mesure la mauvaise chose

`frontend/src/utils/imageProcessor.ts:312-419`

- La **variance de Laplacien est calculée après un rééchantillonnage à 400×114**. Cette
  métrique est **dépendante de l'échelle** : le seuil fixe de `80` n'a donc aucune relation
  stable avec la netteté réelle. Et comme la source est déjà une image ~390 px interpolée,
  on mesure surtout des artefacts, pas la mise au point.
- La **luminance moyenne globale** (`< 40` / `> 220`) est un mauvais indicateur : une plaque
  parfaitement lisible sur une carrosserie sombre passe facilement sous 40. Ce qui compte
  est le **contraste local sur la plaque**.
- Incohérence de formule : luminance en `(R+G+B)/3` ici, en ITU-601 dans le calcul de flou.
- **Fail-open** (`:411-416`) : toute exception renvoie `isAcceptable: true` — le gate se
  désactive silencieusement.
- Il ne s'exécute **qu'en mode photo** ; le mode live n'a aucun contrôle.
- Les clés i18n `qualityTooDark` / `qualityTooBright` / `qualityTooBlurry` **n'existent ni
  dans `en.json` ni dans `fr.json`** — les utilisateurs francophones voient les messages en
  anglais.

Sur le fond : ce gate **compense** la cause racine n°1, et il intervient **après** le
déclenchement, ce qui est le pire moment en termes d'expérience utilisateur.

### 2.5 Écarts par rapport à la documentation Tesseract

| Point | État actuel | Recommandation documentaire |
|---|---|---|
| Hauteur de capitale ≥ 30-33 px | ❌ ~21-25 px | cause racine n°1 |
| Bordure autour du texte | ✅ 30 px | conforme |
| Binarisation adaptative (vs Otsu interne) | ✅ | conforme — Otsu est sous-optimal en éclairage inégal |
| **`load_system_dawg=0`, `load_freq_dawg=0`** | ❌ **absents** | indispensable pour des codes alphanumériques : sinon le modèle de langue LSTM tord `CE128BC` vers des mots anglais |
| `tessedit_do_invert=0` | ❌ absent | Tesseract retente l'inversion en interne → double le temps silencieusement |
| `tessedit_char_whitelist` | ⚠️ posé | **peu fiable avec LSTM** (support retiré en 4.x, partiel en 5.x). Ne pas en dépendre — la correction doit vivre en post-traitement |
| OEM explicite | ❌ jamais posé | dépend du défaut |
| `ADAPTIVE_RADIUS = 40` | ⚠️ pixels absolus | doit être proportionnel à la hauteur de caractère, sinon casse dès qu'on change de définition |
| Modèle | `eng` seul | voir §5 |

**Remarque de fond.** La documentation Tesseract le décrit comme optimisé pour du **texte
imprimé de document dans une langue connue**. Une plaque est une chaîne alphanumérique
courte, dans une police stylisée, sur un fond non-documentaire, souvent en perspective —
hors de son enveloppe de conception. L'ensemble de 6 modes PSM et la table de correction
floue sont les **symptômes** de ce combat contre l'outil.

### 2.6 Autres problèmes identifiés

- **Confiance fabriquée.** `ocr_service.rs:240-244` écrase la confiance réelle de Tesseract
  par `0.90` / `0.50` selon la validité regex. La valeur affichée à l'utilisateur ne signifie
  rien — et le gate de stabilité du mode live compare `minConfidence: 40`
  (`useScanPlate.ts:36-39`) à une valeur qui ne vaut jamais que 0, 50 ou 90.
- **Fuite de fichiers `/tmp`.** Le nettoyage (`ocr_service.rs:218-221`) est **inatteignable
  sur tous les chemins `return finalize(...)` anticipés**. `/tmp` grossit sans limite en
  conteneur.
- **`DefaultBodyLimit` absent** → la limite axum par défaut de **2 Mo** s'applique ; les 8 Mo
  vérifiés dans `handlers/photo.rs:10,54-60` sont du code mort.
- **nginx sans `client_max_body_size`** (`frontend/nginx.conf:19`) → défaut **1 Mo**.
  **À corriger avant toute augmentation de résolution**, sinon 413 opaques.
- ~~**Cache Workbox `NetworkFirst` sur `/^\/api\/.*/i`** (`frontend/vite.config.ts:86-97`) —
  englobe les POST de scan.~~ **Fausse alerte, corrigée après vérification :** Workbox exécute
  la regex contre `url.href` (`workbox-routing/RegExpRoute.js:47`), or l'ancre `^\/api\/` ne
  peut jamais correspondre à `https://host/api/...`. **La règle n'a jamais été active** et
  aucun POST de scan n'a été mis en cache. Un lookahead négatif a néanmoins été ajouté en
  défense, au cas où l'ancre serait « corrigée » plus tard.
- **Aucune métrique Prometheus sur `/photo/plate`** (contrairement à `/scan/plate`), aucun
  histogramme par étage. `tess_init_elapsed` est mesuré puis **jeté** (`let _ =`, ligne 225).
  *Il est aujourd'hui impossible de mesurer où passent les secondes.*
- **Aucun timeout client** sur `photoPlate()`.
- **Zéro contrôle torche / autofocus / exposition.** Le bouton flash est `disabled` en dur
  (`ScanTopControls.tsx:36-44`) — alors que c'est le levier le plus direct sur le problème
  d'éclairage.
- **Capture mono-frame.** La parade la plus efficace au bruit en basse lumière, à coût
  matériel nul, est le multi-frame.
- **Code mort** dans `imageProcessor.ts` : `cropToViewfinder`, `preprocessForHighRes`,
  `preprocessForOCR`, `scaleImage`.
- **Incohérences de documentation interne** : `useScanPlate.ts:22` annonce « 500 ms frame
  sampling » alors que la boucle tourne à 100 ms ; `useStabilityDetection.ts` documente
  3 correspondances / >75 % alors que l'appelant impose 2 / 40 %.

### 2.7 Budget de latence réaliste

L'objectif de 10 s évoqué est **trop généreux**. Une passe Tesseract LSTM sur un crop de
plaque binarisé ~1000×300 coûte typiquement **80-250 ms** sur un cœur serveur moderne.

| Étage | Cible |
|---|---|
| Décodage | ~10 ms |
| Prétraitement (correctement implémenté) | 30-60 ms |
| 1 à 3 passes Tesseract | 250-750 ms |
| **Total serveur p95** | **< 1,5 s** |

---

## 3. Décisions actées

- **Périmètre phase 1** = fondations : résolution + performance + observabilité.
- Le gate qualité devient du **coaching temps réel non bloquant**, et **le bouton de capture
  reste actif en toutes circonstances**.
- **Tesseract est conservé** et réglé conformément à sa documentation. Le remplacement du
  moteur sera réévalué sur des chiffres mesurés, pas sur des hypothèses.

### Principes directeurs

1. Un déclenchement utilisateur part **toujours** au backend. Plus aucun rejet post-capture.
2. On ne fabrique jamais de pixels : on cesse d'upscaler côté client et on va chercher la
   résolution réelle du capteur.
3. Chaque optimisation backend doit être mesurable avant/après — **l'observabilité passe en premier**.

---

## 4. Implémentation livrée

Le travail a été découpé en lots indépendants, chacun vérifiable isolément.

| Lot | Contenu | Fichiers principaux |
|---|---|---|
| **A1** | `client_max_body_size 10m` nginx, `DefaultBodyLimit` axum sur les 2 routes, lookahead Workbox | `frontend/nginx.conf`, `routes.rs`, `vite.config.ts` |
| **A2** | `opt-level = "z"` → `3` | `iviss-backend/Cargo.toml` |
| **A3** | `load_system_dawg=0`, `load_freq_dawg=0`, `tessedit_do_invert=0`, `TESSDATA_PREFIX`, préchauffage | `ocr_service.rs`, `main.rs` |
| **B1** | Instrumentation des 9 étages, `StageTimings`, `OcrBudget` | `services/ocr_timings.rs` (nouveau) |
| **B2** | `iviss_photo_scans_total` / `iviss_photo_scan_errors_total` | `handlers/photo.rs` |
| **C1** | BMP en mémoire via `set_image_from_mem`, `/tmp` et sa fuite supprimés | `ocr_service.rs` |
| **C2** | `morphology_open` séparable — ~6 accès/pixel au lieu de ~36 | `ocr_service.rs` |
| **C3** | `deskew` grossier-vers-fin sur sonde 300px — ~10 rotations minuscules au lieu de 15 pleine résolution | `ocr_service.rs` |
| **C4** | `adaptive_radius_for(height)` au lieu d'un rayon fixe de 40px | `ocr_service.rs` |
| **C5** | Échelle PSM 6 → **3 max**, polarité mesurée au lieu de deux passes à l'aveugle | `ocr_service.rs` |
| **D1** | `scan_plate_image()` — aller-retour JPEG q75 supprimé | `ocr_service.rs`, `photo_ocr_service.rs` |
| **D2** | Double passe supprimée (relance seulement si aucun texte), crop couleur sur sonde 320px + garde-fou 70 % | `photo_ocr_service.rs` |
| **D3** | `Semaphore` dimensionné au nombre de cœurs + annulation coopérative par échéance | `handlers/{photo,scan}.rs` |
| **D4** | Confiance : la mesure Tesseract n'est plus écrasée par 0.90/0.50 | `ocr_service.rs`, `photo_ocr_service.rs` |
| **E1-E2** | Contraintes 1920×1080, `forceScreenshotSourceSize`, échelle `takePhoto` → `grabFrame` → `getScreenshot`, métrage sur la ROI | `utils/captureFrame.ts` (nouveau), `useCamera.ts`, `ScanViewfinder.tsx` |
| **E3** | Géométrie partagée : le cadre dessiné et le cadre découpé dérivent des mêmes constantes | `utils/viewfinder.ts` (nouveau) |
| **E4** | Upscale ×10 et noyau de netteté supprimés, code mort retiré, timeout client | `utils/imageProcessor.ts`, `usePhotoCapture.ts` |
| **F1** | Coaching temps réel ~4 Hz, anti-rebond, **bouton toujours actif**, plus aucun rejet post-capture | `hooks/feature/useCaptureCoaching.ts` (nouveau) |
| **F2** | 4 clés i18n manquantes ajoutées en `en` **et** `fr` | `i18n/locales/{en,fr}.json` |

**Tests ajoutés.** Backend : équivalence pixel à pixel de `morphology_open` contre
l'implémentation naïve, recouvrement d'un angle de rotation connu par `deskew`,
round-trip BMP validé **contre Leptonica** (`pix_read_mem`) et non contre le
décodeur du crate `image`, bornes de `adaptive_radius_for`, détection de polarité,
`OcrBudget`, localisation couleur et rejet des bbox pleine image. Frontend :
géométrie du viewfinder (dont la non-régression window-vs-élément), `cropToViewfinder`,
`measureRoiQuality`.

État de la vérification : `cargo fmt`/`clippy` propres, 62 tests OCR backend verts,
289 tests frontend verts, eslint 0 erreur, prettier conforme. Les 82 échecs du
`cargo test --lib` complet sont **pré-existants** (tests DB qui se polluent en
parallèle) — vérifié en comparant avec la branche avant modification.

---

## 4 bis. Détail du plan (référence)

### 4.1 Observabilité (à faire en premier — sans cela on optimise à l'aveugle)

`iviss-backend/src/services/ocr_service.rs`, `iviss-backend/src/handlers/photo.rs`

- Instrumenter **chaque étage séparément** : décodage, `color_adaptive_crop`, `deskew`,
  `contrast_stretch_percentile`, `adaptive_threshold`, `morphology_open`, et **chaque passe
  Tesseract individuellement**. Aujourd'hui `process_elapsed` (`ocr_service.rs:81/101`)
  agrège tout le prétraitement, ce qui rend le coût de `deskew` invisible.
- Cesser de jeter `tess_init_elapsed` (`ocr_service.rs:225`).
- Journaliser **les dimensions de l'image reçue et de la ROI** — c'est ce qui rend le
  problème de résolution visible en production.
- Ajouter les métriques Prometheus manquantes sur `/photo/plate`, en calquant
  `handlers/scan.rs:81-107` (`iviss_scans_total`, `iviss_scan_errors_total`), plus un
  **histogramme de durée par étage** et un compteur de passes Tesseract par requête.

### 4.2 Backend — performance

**`iviss-backend/Cargo.toml`**
- `opt-level = "z"` → `opt-level = 3`. Conserver `lto = true`, `codegen-units = 1`,
  `strip = true`. **Une ligne, gain attendu ×2 à ×5 sur tout le pipeline pixel.**

**`iviss-backend/src/services/photo_ocr_service.rs`**
- **Supprimer l'aller-retour JPEG** (`:31-36`). Extraire
  `ocr_service::scan_plate_image(&DynamicImage)` et conserver `scan_plate(&[u8])` comme fin
  décodeur pour l'appelant `scan.rs`.
- **Supprimer la double passe** (`:38-56`). Ne relancer la variante rehaussée que si la
  passe 1 n'a produit **aucun texte**, jamais simplement parce que `format_valid` est faux.
- Faire tourner `color_adaptive_crop` sur une **copie réduite** (~320 px de large) et
  remettre la bbox à l'échelle. Ajouter un garde-fou : rejeter une bbox couvrant plus de
  ~70 % de l'image — un ciel blanc ou un vêtement orange fait aujourd'hui exploser la bbox
  unique et globale.

**`iviss-backend/src/services/ocr_service.rs`**
- **`deskew` (`:410-446`)** — cause dominante du prétraitement. Estimer l'angle sur une copie
  **binarisée et réduite** (~300 px de large), en recherche grossière puis fine (pas de 2°
  puis 0,5°), et n'appliquer la rotation pleine résolution **qu'une fois**, et seulement si
  `|angle| > 1°`. Passe de 15 rotations pleine résolution à ~10 rotations minuscules.
- **`morphology_open` (`:450-490`)** — réécrire en min/max **séparable** (1×3 puis 3×1) sur
  les slices brutes via `as_raw()`/`as_mut()` : ~6 accès par pixel sans vérification de
  bornes, au lieu de ~36 accès bornés.
- **`ADAPTIVE_RADIUS = 40` (`:12`)** — le dériver de la hauteur d'image
  (p. ex. `max(15, height / 8)`) au lieu d'un nombre de pixels absolu.
- **Éliminer les fichiers `/tmp`** (`:112-118`, `:137-143`, `:194-200`). `leptess 0.14` expose
  `set_image_from_mem(&[u8])` (vérifié : `leptess-0.14.0/src/lib.rs:117`). Encoder en **BMP**
  en mémoire (sans compression, coût proche d'un memcpy, sans perte) et le passer directement.
  Supprime 2 encodages PNG, jusqu'à 6 décodages, tous les accès disque, **et la fuite de fichiers**.
- **Réduire l'échelle de PSM (`:119-215`).** Avec une ROI correctement cadrée et haute
  résolution, PSM 7 puis PSM 13 doivent suffire. Choisir la variante inversée par une
  **mesure d'intensité moyenne** de la ROI plutôt que de tenter les deux à l'aveugle.
  **Cible : ≤ 3 passes.**

**`iviss-backend/src/handlers/photo.rs` et `scan.rs`**
- **Annulation coopérative.** Passer un `Arc<AtomicBool>` (ou une échéance `Instant`) vérifié
  entre chaque passe Tesseract et entre les étages de prétraitement, afin qu'une requête
  expirée cesse réellement de consommer du CPU.
- **Borner la concurrence OCR** par un `tokio::sync::Semaphore` dimensionné au nombre de
  cœurs. Sans lui, la pile de threads bloquants (défaut tokio : 512) transforme chaque
  timeout en dégradation du suivant.

### 4.3 Backend — configuration Tesseract

`iviss-backend/src/services/ocr_service.rs:260-278`, dans `take_tesseract` (posé une seule fois)

- `load_system_dawg = 0` et `load_freq_dawg = 0` — **absents aujourd'hui**. Correctif le moins
  cher et parmi les plus rentables en précision.
- `tessedit_do_invert = 0` — empêche Tesseract de retenter l'inversion en interne (on gère
  nous-mêmes l'inversion).
- Conserver `tessedit_char_whitelist` mais **ne plus en dépendre**. La correction vit en
  post-traitement dans `iviss-backend/src/utils/plate_format.rs` (`fuzzy_correct`, tables de
  confusion O↔0, `classify`, `extract_first`) — **à conserver tel quel**.
- Lire le chemin tessdata depuis `TESSDATA_PREFIX` (déjà positionné dans
  `iviss-backend/Dockerfile:91`) avec le chemin actuel en repli. Il est aujourd'hui codé en
  dur (`:264`), ce qui casse tout environnement non-Debian-5.x et empêche l'exécution native
  hors Docker.
- **Préchauffer** le moteur au démarrage, pour que la première requête ne paie pas l'initialisation.

**Sémantique de confiance** (`ocr_service.rs:234-256`) — conserver la confiance mesurée par
Tesseract au lieu de l'écraser, garder `format_valid` comme signal séparé (la forme de
`ScanResultData` reste inchangée, donc **pas de rupture du contrat OpenAPI**), puis
**re-calibrer** le seuil de stabilité du mode live.

### 4.4 Frontend — capture

**`frontend/src/hooks/feature/useCamera.ts`, `frontend/src/components/mobile/scan/ScanViewfinder.tsx`**

- Demander la résolution :
  `{ facingMode: {ideal:'environment'}, width: {ideal: 1920}, height: {ideal: 1080} }`.
- Ajouter `forceScreenshotSourceSize={true}` sur `<Webcam>`, en calquant
  `MobileCarteGrise.tsx:310-317` qui le fait déjà correctement.
- **Échelle de capture** (nouveau helper, p. ex. `frontend/src/utils/captureFrame.ts`), en
  récupérant la piste via `webcamRef.current.stream.getVideoTracks()[0]` :
  1. `ImageCapture.takePhoto()` — vraie capture *still* à la résolution photo du capteur (Chrome/Android) ;
  2. `ImageCapture.grabFrame()` — frame vidéo, mais à la résolution **native** de la piste ;
  3. `getScreenshot()` avec `forceScreenshotSourceSize` — repli (Safari/iOS, sans support `ImageCapture`).

  Détecter les capacités via `track.getCapabilities()` ; **ne jamais supposer le support**.
- **Métrage sur la ROI** : appliquer `pointsOfInterest` centré sur le cadre de visée via
  `applyConstraints`, lorsque la capacité existe. C'est le correctif direct du problème en
  plein soleil — on mesure l'exposition sur la plaque, pas sur le ciel. Capability-gated,
  sans échec si non supporté.

**`frontend/src/utils/imageProcessor.ts`**

- **Corriger `getSafeCrop` (`:198-233`).** Il utilise `window.innerWidth`/`window.innerHeight`,
  alors que la vidéo vit dans une boîte de `calc(100dvh-4rem)` sous un `pt-16` — la hauteur
  supposée est fausse de 64 px. Utiliser `video.getBoundingClientRect()` et
  `videoWidth`/`videoHeight` réels.
- **Faire dériver le cadre dessiné et le cadre découpé des mêmes constantes.** L'overlay fait
  `min(vw − 64px, 384px)` (`ScanViewfinder.tsx:55-56`) tandis que le crop fait
  `0.92 × window.innerWidth` (`imageProcessor.ts:206`) : l'utilisateur cadre dans une boîte
  qui n'est pas celle qu'on découpe (+10 % sur téléphone, **+145 % sur tablette**). Ils
  doivent être identiques **par construction**.
- **Cesser d'upscaler.** `preprocessForPhoto` (`:243-301`) force 1400×400 depuis une source de
  ~135 px et applique un noyau de netteté qui amplifie bruit et artefacts. Découper à la
  résolution native, ne redimensionner que **vers le bas** au-delà de ~1600 px de large, et
  **supprimer la convolution de netteté** (le backend fait son propre rehaussement).
- Encoder la charge utile OCR en JPEG **0.95**.
- Supprimer le code mort : `cropToViewfinder`, `preprocessForHighRes`, `preprocessForOCR`, `scaleImage`.

**`frontend/src/hooks/feature/usePhotoCapture.ts`**
- Retirer la double requête (`:157-161`) — le backend ne fait plus de double passe.
- Ajouter un **timeout client** sur `photoPlate()`.

### 4.5 Frontend — coaching temps réel (non bloquant)

- Extraire les métriques de `assessImageQuality` (`imageProcessor.ts:312-419`) dans un
  `ImageProcessor.measureRoiQuality()` renvoyant
  `{ meanLuma, localContrast, sharpness, roiFillRatio }`.
- **Corriger la dépendance à l'échelle** : calculer la variance de Laplacien sur une ROI
  normalisée à une **taille en pixels fixe**, au lieu du rééchantillonnage à 400×114 avec
  seuil absolu de 80.
- Remplacer la luminance moyenne globale par une mesure de **contraste local**. Unifier la
  formule de luminance (aujourd'hui `(R+G+B)/3` d'un côté, ITU-601 de l'autre).
- Faire tourner la mesure en continu à **~4-5 Hz** sur le flux et afficher des indications
  (« rapprochez-vous », « trop sombre », « stabilisez »). Utiliser `useStabilityDetection`
  (`frontend/src/hooks/feature/useStabilityDetection.ts`) pour anti-rebondir les messages.
- **Le bouton de capture reste actif en permanence** ; le coaching est purement indicatif.
- Supprimer le rejet post-capture (`usePhotoCapture.ts:123-128`) et son comportement
  *fail-open* (`imageProcessor.ts:411-416`).
- Ajouter les clés i18n manquantes `qualityTooDark` / `qualityTooBright` / `qualityTooBlurry`
  dans `frontend/src/i18n/locales/en.json` **et** `fr.json`.

### 4.6 Infrastructure (à faire **avant** d'augmenter la résolution)

- `frontend/nginx.conf:19` — ajouter `client_max_body_size 10m;` dans `location /api/`.
  Le défaut nginx est **1 Mo** ; sans cela, la montée en résolution produira des 413 opaques.
- `iviss-backend/src/routes.rs` — ajouter `DefaultBodyLimit::max(8 * 1024 * 1024)` sur les
  deux routes de scan.
- `frontend/vite.config.ts:86-97` — exclure `/api/v1/scan/plate` et `/api/v1/photo/plate` du
  cache Workbox `NetworkFirst`.

---

## 5. Hors périmètre — phase 2

À réévaluer sur les chiffres mesurés après la phase 1.

- **Contrôle de la torche** (bouton `disabled` en dur, `ScanTopControls.tsx:36-44`) et
  compensation d'exposition.
- **Capture multi-frame** : 4-5 frames en ~300 ms, sélection de la plus nette ou moyennage
  temporel (le bruit est aléatoire, le signal ne l'est pas → ~2× de SNR sur 4 frames). C'est
  la parade la plus efficace au bruit en basse lumière, à coût matériel nul.
- **Modèle Tesseract fine-tuné** sur la police des plaques camerounaises (`tesstrain`) —
  gain de précision typiquement le plus fort sur une police fixe ; nécessite quelques
  centaines de crops annotés.
- **Migration vers un détecteur + reconnaisseur ONNX** via le crate `ort` (architecture ALPR
  de production : ~20-50 ms/image CPU, bien plus robuste à l'éclairage et à la perspective).

---

## 6. Vérification

### Mesure de référence — **avant toute modification**

Constituer un jeu de ~30 photos réelles (plein soleil, ombre, crépuscule, intérieur de
parking) prises sur les appareils de test. Enregistrer le taux d'extraction correcte et la
latence p50/p95. **Sans ce point de départ, aucune des affirmations ci-dessous n'est
vérifiable.**

### Backend

- `cargo test` — les tests existants doivent rester verts : `src/tests/ocr_service_tests.rs`,
  `src/tests/photo_ocr_service_tests.rs`, `src/tests/plate_format_tests.rs`.
- Ajouter un test comparant la sortie de `morphology_open` réécrite à l'implémentation naïve
  actuelle sur une image de référence (équivalence pixel à pixel).
- Ajouter un test vérifiant que `deskew` retrouve un angle connu sur une image pivotée
  synthétiquement.
- Benchmark avant/après sur le jeu de référence via `curl` sur `/api/v1/photo/plate`, en
  lisant les nouveaux histogrammes par étage. **Critère : p95 serveur < 1,5 s.**
- Vérifier qu'aucun fichier `ocr_*.png` ne subsiste dans `/tmp` après une série de requêtes.
- Test de charge : ~20 requêtes concurrentes ; vérifier que le semaphore borne la concurrence
  et que la latence ne s'effondre pas.

### Frontend

- `npm test` — mettre à jour `src/utils/__tests__/imageProcessor.test.ts:252-307` (les tests
  actuels utilisent des remplissages uniformes qui ne valident pas les nouvelles métriques de
  contraste local), ainsi que `src/test/usePhotoCapture.test.ts` et `src/test/useScanPlate.test.ts`.
- Sur appareil réel : journaliser les dimensions de la ROI envoyée et **confirmer le passage
  de ~135 × 38 à ~600 × 170 pixels réels**. C'est la validation directe de la cause racine n°1.
- Vérifier l'échelle de repli sur au moins un appareil iOS (sans support `ImageCapture`) et
  un Android (`takePhoto()` disponible).
- Vérifier que le cadre de visée dessiné correspond exactement à la zone découpée (sauvegarder
  le crop envoyé et le comparer visuellement au cadrage).
- Confirmer qu'aucune capture n'est plus jamais rejetée et que le bouton reste actif en permanence.

### Bout en bout

- Rejouer le jeu de 30 photos et comparer taux de réussite et latence à la référence.
- Vérifier qu'un upload de ~2-3 Mo traverse nginx et axum sans 413.
