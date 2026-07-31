# Ticket backend — pipeline OCR

Aucun de ces correctifs n'est encore appliqué au code. Chaque section = un
défaut diagnostiqué + le correctif conçu, validé par tests unitaires et par
mesure sur photo réelle lors d'une itération antérieure.

> **Lire d'abord [`06_validation_documentaire.md`](06_validation_documentaire.md) §3 et §5.**
> Ce ticket a été rédigé contre une version antérieure du code. Corrections
> reportées en ligne ci-dessous, marquées « ⚠️ ». Deux ajouts de fond :
> **la binarisation passe à Sauvola** (§3) et **la confiance est fabriquée à
> deux endroits que le §7 ne visait pas** (§6).

## 1. Performance et observabilité (fondations — à faire en premier)

- **`Cargo.toml` `[profile.release]` : `opt-level = "z"` → `3`.** Un pipeline
  de traitement d'image CPU-bound compilé pour la taille du binaire ;
  `opt-level="z"` désactive vectorisation et inlining agressif, or
  `get_pixel`/`put_pixel` ne sont bon marché qu'inlinés. Gain attendu ×2-×5.
  Garder `lto = true`, `codegen-units = 1`, `strip = true`.
- **Le conteneur de dev tourne en profil `debug` et rend tous les timings
  ininterprétables.** `docker-compose.yml` cible `target: development` →
  `cargo watch -x run`, qui n'applique jamais `[profile.release]`. Ajouter :
  ```toml
  [profile.dev]
  opt-level = 1
  [profile.dev.package."*"]
  opt-level = 3
  ```
  (le crate compile vite, les dépendances — `image`, `imageproc`, `leptess` —
  tournent vite). Sans ça, un décodage JPEG 800×229 mesure ~130ms au lieu de
  quelques ms, et toute mesure prise pendant le développement est trompeuse.
- **Instrumentation par étage.** Nouveau module `ocr_timings.rs` :
  `StageTimings` (decode/crop/deskew/contrast/threshold/morphology/
  border/tess_init/ocr_passes/total + dimensions d'entrée), publié en logs
  structurés + métriques Prometheus (`iviss_ocr_stage_duration_seconds`,
  `iviss_ocr_duration_seconds`, `iviss_ocr_passes`, `iviss_ocr_input_pixels`).
  `OcrBudget` : échéance vérifiée entre chaque étage (`budget.check()`), pour
  qu'une requête abandonnée cesse réellement de consommer du CPU au
  prochain point de contrôle plutôt que de tourner jusqu'au bout —
  `JoinHandle::abort()` est un no-op sur une tâche `spawn_blocking` déjà
  démarrée.
- **Bug d'instrumentation à éviter en réimplémentant** : mesurer `total` aux
  points d'entrée (`scan_plate`, `photo_plate`) avec un `Instant::now()`
  local, pas en accumulant dans `scan_plate_image` après le décodage/crop —
  sinon `preprocessing() > total` (le premier inclut décodage+crop, le second
  les rate).
- **Semaphore de concurrence OCR.** `tokio::sync::Semaphore` dimensionné au
  nombre de cœurs (`std::thread::available_parallelism()`), acquis avant
  `spawn_blocking`. Sans lui, un burst de requêtes transforme chaque
  timeout en dégradation du suivant (pool bloquant Tokio par défaut : 512
  threads, tous en contention sur les mêmes cœurs).
- **Fichiers `/tmp` supprimés.** Encoder en BMP en mémoire
  (`image::codecs::bmp::BmpEncoder`, `set_image_from_mem`) au lieu du
  round-trip PNG-vers-disque. Coût proche d'un memcpy, sans perte, et
  supprime la fuite de fichiers sur les chemins de retour anticipé.
- **`morphology_open` réécrit en séparable** (érosion 1×3 puis 3×1, dilation
  idem, sur slices brutes `as_raw()`/`as_mut()`) : ~6 accès/pixel non bornés
  au lieu de ~36 accès bornés `get_pixel`/`put_pixel`.
- **`ADAPTIVE_RADIUS` relatif à la hauteur** : `(height/8).clamp(15, 100)` au
  lieu d'un rayon fixe de 40px, qui ne veut pas dire la même chose selon la
  résolution d'entrée.
- **Réencodage JPEG q75 supprimé entre crop et OCR** : extraire
  `scan_plate_image(&DynamicImage)` pour que le pipeline photo passe l'image
  décodée directement, sans repasser par un encodage JPEG (perte de détail
  gratuite juste avant reconnaissance).

## 2. Configuration Tesseract

> **⚠️ Correction.** La fonction s'appelle **`take_tesseract`**
> (`ocr_service.rs:260`), pas `init_tesseract`. C'est elle qui contient le
> `LepTess::new` paresseux et le seul `set_variable` actuel (la whitelist).

Dans `take_tesseract` (posé une fois, au démarrage) :
- `load_system_dawg=0`, `load_freq_dawg=0` — sans ça le modèle de langue LSTM
  tord un code alphanumérique type `CE128BC` vers du vocabulaire anglais.
- `tessedit_do_invert=0` — la polarité est gérée nous-mêmes (§4) ; sans ça
  Tesseract retente l'inversion en interne, doublant silencieusement le
  temps de calcul.
- `tessedit_char_whitelist` conservé mais **jamais comme garantie** : non
  honoré par le moteur LSTM en Tesseract 4.x, support rétabli seulement
  partiellement en 5.x (voir `README.md` de ce dossier pour les sources). La
  correction vit dans `plate_format.rs`, pas dans cette variable.
- **OEM impossible à poser explicitement** avec `leptess 0.14` (pas de
  paramètre sur `LepTess::new`, `tessedit_ocr_engine_mode` lu à l'Init). Ne
  pas perdre de temps dessus sans upgrade de version.
- `TESSDATA_PREFIX` lu depuis l'environnement, chemin Debian en repli — pas
  codé en dur, pour rester exécutable hors de l'image Docker.
- Préchauffer un `LepTess` par worker au démarrage (barrière `std::sync::
  Barrier` pour forcer Tokio à distribuer sur des threads distincts), pour
  que la première requête ne paie pas l'initialisation.

> **⚠️ Précisions vérifiées (2026-07-31) — ne pas re-chercher.**
>
> - `LoadSystemDawg`, `LoadFreqDawg`, `TesseditDoInvert` et
>   `TesseditCharWhitelist` existent bien dans l'énum `leptess::Variable`
>   (`leptess-0.14.0/src/variable.rs`). Les trois réglages sont applicables.
> - **La conclusion sur l'OEM est confirmée et plus forte qu'énoncée** : outre
>   l'absence de paramètre sur `LepTess::new`, le champ `LepTess.tess_api` est
>   **privé** (`lib.rs:97`), donc le `TessApi.raw` public de
>   `tesseract-plumbing` est inatteignable depuis un `LepTess`.
> - **Conséquence non relevée : `invert_threshold` et `thresholding_method` sont
>   eux aussi inatteignables.** `set_variable` n'accepte que l'énum `Variable`,
>   qui ne les contient pas, et il n'existe aucun setter par chaîne libre.
> - **`tessedit_do_invert` est valable mais daté** : déprécié, retiré en
>   Tesseract **6.0**, remplacé par `invert_threshold` (défaut 0.7 ; 0.0 pour
>   désactiver). L'image est `debian:bookworm-slim` + `libtesseract5`, soit
>   **Tesseract 5.3.0**, où il fonctionne encore. À tracer comme dette : la
>   montée en Tesseract 6 imposera `tesseract-plumbing 0.8` en dépendance
>   directe (`init_4` accepte un `TessOcrEngineMode`, et
>   `TessBaseApi::set_variable` prend deux `&CStr`, ce qui débloquerait du même
>   coup l'OEM explicite, `invert_threshold` et `thresholding_method`).

## 3. Pipeline de prétraitement — ordre corrigé

**Défaut mesuré (`04_mesures_verifiees.md`)** : sur `dev`, l'ordre est
`deskew(gray) → contrast → threshold → morphology → polarité → border`.
`estimate_skew_angle` tourne alors sur l'image en niveaux de gris, où le
discriminant de la recherche d'angle s'effondre à 2% (dominé par les grandes
plages de luminance — carrosserie, plaque — plutôt que par les lignes de
texte) : sur une plaque parfaitement droite, l'angle choisi peut être +2.5°,
différent à chaque frame. Le remplissage des coins en noir (`Luma([0])`)
laisse par ailleurs des bordures sombres après seuillage — lues comme
caractères parasites par Tesseract.

**Ordre corrigé, à réimplémenter ainsi :**

```text
gray
  → contrast_stretch_percentile
  → sauvola_threshold(radius = height/8, k = 0.35)
  → is_light_on_dark ?  invert            (mesuré sur la région CENTRALE, pas toute l'image)
  → deskew(binaire)                        (remplissage 255, re-binarisation post-rotation)
  → morphology_open                        (sur image déjà polarité-normalisée)
  → add_border(30, 255)                    (UNE SEULE FOIS, après la polarité — voir plus bas)
```

Points clés :
- **`estimate_skew_angle` doit recevoir une image déjà binarisée et
  polarité-normalisée** (ink=0, bg=255 par construction). Testé : le
  discriminant remonte de 2% à 88% au-dessus du plancher, et l'angle mesuré
  sur une plaque droite redevient 0.0°.
- **`deskew` remplit les coins en 255 (fond), jamais en 0.** Après rotation
  bilinéaire, re-binariser (seuil à 128) pour éviter les gris
  d'interpolation en sortie — la morphologie et Tesseract veulent une image
  à deux niveaux stricts.
- **`morphology_open` tourne après normalisation de polarité**, pas avant :
  une ouverture retire les structures *claires*. Avant normalisation, elle
  nettoyait le bruit sur une plaque sombre-sur-clair mais attaquait les
  glyphes d'une plaque claire-sur-sombre (peu observé aux résolutions
  actuelles, mais l'ordre est structurellement faux et coûte gratuitement à
  corriger).
- **`is_light_on_dark` mesuré sur la région centrale** (± 20% d'inset), pas
  sur toute l'image : la marge globale mesurée n'était que de 5-10 points
  (40-45% dark contre un seuil de bascule à 50%), et avec
  `tessedit_do_invert=0` un mauvais appel est irrécupérable.

### 3 bis. Binarisation — passer de la moyenne locale à Sauvola

**Défaut.** `adaptive_threshold` (`ocr_service.rs:493`) est un seuillage à
**moyenne locale − C** (Bradley/Wellner) : il ne regarde que la moyenne du
voisinage et **ignore complètement la variance locale**. Sur une zone
uniforme faiblement contrastée (plaque à l'ombre, reflet, sur-exposition
partielle), la moyenne suffit à basculer des pixels de fond en encre.

**Correctif — implémenter Sauvola en Rust.** Formule de référence Leptonica
(`src/binarize.c`) :

```text
t = m · (1 − k · (1 − s / 128))
```

où `m` et `s` sont moyenne et écart-type locaux, et `k ≈ 0.35`
(plage documentée 0.2–0.5). L'idée : plus le contraste local est fort, plus le
seuil se rapproche de la moyenne ; plus il est faible, plus le seuil descend
sous la moyenne — donc moins on fabrique d'encre dans le bruit.

Points d'implémentation :

- **Pas de nouvelle dépendance, pas d'`unsafe`.** `adaptive_threshold` construit
  déjà une image intégrale ; il suffit d'en construire une **seconde sur les
  carrés** (`i64` puis `f64` pour la variance) pour obtenir `s` en O(1) par
  pixel, exactement comme `m` aujourd'hui. Le surcoût est un second passage
  d'accumulation, pas un changement de complexité.
- **Ni `thresholding_method` (Tesseract 5) ni `pixSauvolaBinarizeTiled`
  (Leptonica) ne sont utilisables ici** : le premier n'est pas dans l'énum
  `leptess::Variable`, le second exigerait `leptonica-sys` en dépendance directe
  et de la FFI `unsafe`. Voir `06_validation_documentaire.md` §5.
- **Garder la même signature de fenêtre** que le correctif de rayon relatif :
  `radius = (height/8).clamp(15, 100)`. Leptonica note un `whsize` minimum de 2
  et typiquement ≥ 7 — le plancher de 15 est confortablement au-dessus.
- **Attention à `ADAPTIVE_C`** : la constante disparaît avec la moyenne locale,
  remplacée par `k`. Ne pas la conserver « au cas où » — deux paramètres pour un
  seul effet est exactement ce qui a fait perdre du temps sur le balayage C.
- **Honnêteté sur le gain attendu.** Le balayage `ADAPTIVE_C` de
  `04_mesures_verifiees.md` a montré des glyphes visuellement identiques sur la
  photo de référence : sur *ce* corpus, la binarisation n'est pas le facteur
  limitant, et le gain de Sauvola est **théorique**. Le motif du changement est
  qu'il est mieux fondé sous éclairage inégal — le cas de terrain que la photo
  de référence ne représente pas. **Rejouer `samples/binarize_replica.py` avant
  et après**, et conserver un test comparant les deux binarisations sur une
  fixture à illumination volontairement inégale (gradient), où Sauvola doit
  strictement faire mieux. Si la mesure de terrain ne montre rien, le
  changement reste défendable mais ne doit pas être présenté comme un gain.

### 3 ter. Bordure noire sur le chemin inversé (défaut non listé jusqu'ici)

`ocr_service.rs:137` et `:194` font `add_border(&invert_image(&binary), 0, 255)`.
Or `binary` porte **déjà** une bordure blanche de 30 px, posée en `:96`.
L'inverser la rend **noire**, et le `add_border(..., 0, ...)` est un no-op —
il n'en repose aucune. **Toutes les passes inversées voient donc un cadre noir
de 30 px**, ce que la documentation Tesseract (*ImproveQuality*) décrit
précisément comme « erroneously picked up as extra characters ».

L'ordre corrigé du §3 supprime le défaut par construction (la polarité est
normalisée **avant** l'unique `add_border`, il n'y a plus de seconde image
inversée à border). Le point est consigné ici pour deux raisons : il explique
une part du bruit observé en terrain, et il justifie un **test dédié** —
vérifier que l'image finalement envoyée à Tesseract a ses quatre coins à 255,
sur les deux polarités d'entrée.

## 4. Ensemble de passes PSM — exiger un accord, pas un premier succès

**Défaut.** L'échelle de passes retournait dès qu'une passe produisait un
résultat `format_valid`, y compris à confiance mesurée 0.00. Puisque le
post-traitement (§5) peut corriger du bruit en quelque chose de bien formé,
"bien formé" seul ne prouvait rien. `PSM 11` (*sparse text*, conçu pour
trouver du texte n'importe où dans une scène) tournait systématiquement et
récupérait fidèlement le cadre de concessionnaire, qui nourrissait ensuite
le correcteur flou.

**Correctif :**
- PSM 7 et PSM 13 tournent toujours tous les deux.
- Si les deux s'accordent sur la **même** plaque bien formée → accepter,
  rapporter la **meilleure des deux confiances** (pas une moyenne ni un bonus
  d'accord).
- Sinon, si l'une des deux a lu du texte → `pick_best_ensemble` entre elles
  deux seulement.
- **PSM 11 ne tourne que si aucune des deux n'a lu le moindre texte** — le
  cas pour lequel il est réellement conçu, pas un troisième vote systématique.
- `pick_best_ensemble` doit **sauter les candidats à plaque vide** avant de
  comparer par confiance — sinon une passe qui a lu du texte sans qu'aucune
  plaque n'en sorte (ex. `"200"` à confiance 0.63) peut gagner sur une passe
  qui tient une vraie plaque à confiance plus basse, et renvoyer une plaque
  vide au client.

## 5. Anti-hallucination — `plate_format.rs`

**Défaut, source directe des lectures fantômes observées en test terrain**
(`SSGCFFTAUNUSAUTOMB1SW`, `7104214`, `AU1058W` — dérivées de bruit de cadre
de concessionnaire, jamais présentes sur la vraie plaque) :

1. **Repli `cleaned.len() >= 4`** dans `extract_plate_fuzzy` : renvoyait
   n'importe quelle chaîne normalisée de 4+ caractères comme "plaque" quand
   rien d'autre n'avait parsé. **À supprimer entièrement** — une lecture qui
   ne parse pas n'est pas une plaque faible, ce n'est pas une plaque.
2. **`fuzzy_correct` glissait un masque de correction sur toute la chaîne
   reconnue**, sans borne de longueur. Sur une ligne entière de texte
   publicitaire, une fenêtre quelque part corrige toujours vers quelque
   chose de bien formé. **Borner le bruit toléré à 3 caractères** sur ce
   chemin de *substitution* uniquement (`trimmed_chars(...) >
   MAX_FUZZY_NOISE_CHARS`, avec `MAX_FUZZY_NOISE_CHARS = 3`) — ne pas borner
   `find_candidate`/`extract_first`, qui lisent le texte **tel qu'OCR l'a
   rapporté**, sans substitution : deux tests existants (`extracts_first_
   plate_from_noisy_ocr_text`, `test_extract_plate_strict_with_noise`)
   dépendent de ce chemin non borné pour retrouver une plaque valide au
   milieu de texte bruité — comportement légitime, à ne pas casser.
3. **`^\d{7}$` (Military) et `^[A-Z]{2}\d{4}[A-Z]$` (GovernmentLegacy) sont
   atteignables par la seule table de confusion**, sans aucun préfixe
   régional ni marqueur littéral pour ancrer la correction :
   `TAUNUS...` → `7104214` (Military), `AUTOSBW` → `AU1058W`
   (GovernmentLegacy). **Exclure ces deux catégories du chemin de
   correction par substitution** (`is_fuzzy_reachable`), tout en les
   gardant pleinement reconnues par `classify`/`extract_first` (lecture
   directe, sans substitution — un vrai `1234567` doit continuer à
   fonctionner).

## 6. Post-traitement photo — `photo_ocr_service.rs`

- **Vote caractère par caractère supprimé de `pick_best`.** Entre deux
  lectures invalides de même longueur, l'ancien code construisait une
  troisième chaîne en prenant chaque position depuis la lecture la plus
  confiante, puis la promouvait `format_valid = true` si le composite
  matchait un pattern — **une plaque qu'aucune passe n'a réellement lue**.
  Sélection restreinte à renvoyer une lecture exactement telle que reconnue.
- **`enhance_photo_result` : `format_valid` dérivé de la classification**,
  pas asserté inconditionnellement à `true` après `extract_plate_strict` —
  cette fonction ne peut pas garantir que la chaîne extraite classifie
  toujours (invariant d'un autre module).
- **`enhance_photo_result` ne doit plus fabriquer de confiance non plus.**
  `photo_ocr_service.rs:175-177` fait `if r.confidence < 0.90 { r.confidence =
  0.90 }` — le même défaut que `finalize`, à un autre endroit (voir §7).
  À supprimer.
- **`color_adaptive_crop` sauté quand le frontend a déjà envoyé un crop
  plaque-shaped** (aspect ≥ 3.0) : redondant une fois le viseur frontend
  corrigé (§ticket frontend #3), et son profil de couleur orange attrape
  aussi peau, bois, terre — sur un crop déjà serré il ne peut que dégrader.
- **Double passe supprimée** : ne relancer la variante contraste/netteté
  rehaussée que si la passe native n'a produit **aucun texte du tout**,
  jamais simplement parce que `format_valid` est faux.

## 7. Sémantique de confiance

`finalize` ne doit **jamais** réécrire `confidence` depuis `format_valid`
(l'ancien 0.90/0.50 dupliquait un signal et en jetait un autre). `confidence`
reste la mesure brute de Tesseract (`mean_text_conf`), `format_valid` reste
indépendant. C'est ce qui permet au ticket frontend §5 de recalibrer le seuil
de stabilité sur des données réelles plutôt que sur 0/50/90 arbitraires.

> **⚠️ Correction — la confiance est fabriquée à TROIS endroits, pas un.**
> Ce paragraphe ne visait que `ocr_service::finalize` (`:240-244`). Vérification
> du 2026-07-31 :
>
> | Emplacement | Ce qu'il fait |
> |---|---|
> | `ocr_service.rs:240-244` | `finalize` : 0.90 si `format_valid`, sinon 0.50 si plaque non vide |
> | `photo_ocr_service.rs:175-177` | `enhance_photo_result` : plancher à 0.90 (voir §6) |
> | `photo_ocr_service.rs:222` | `pick_best`, branche de vote caractère par caractère : 0.90 sur le composite |
>
> `pick_best` est réglé incidemment par la suppression du bloc de vote (§6),
> mais **`enhance_photo_result` ne l'est par aucune autre section**. Si on
> l'oublie, **toute recalibration du seuil frontend reste inopérante sur le
> chemin photo**, puisque `enhance_photo_result` est traversé à chaque appel de
> `photo_plate` (`:35`). Les trois doivent tomber ensemble.
>
> Le contrat OpenAPI n'est pas rompu : la forme de `ScanResultData` est
> inchangée, seule la sémantique de la valeur l'est. Prévenir le frontend —
> c'est exactement le préalable du ticket frontend §5.

## Tests à ajouter

Voir `05_tests_de_non_regression.md`. En particulier : les **6 chaînes
brutes tirées des logs de terrain** (cadre de concessionnaire OCRisé) qui
doivent toutes retourner `None` depuis `extract_plate_fuzzy`, et les
fixtures de deskew en polarité correcte (dev-on-light, pas l'inverse comme
c'était le cas dans les tests originaux — l'ancien fixture masquait le
remplissage noir parce qu'il était lui-même en fond noir).
