# Ticket backend — pipeline OCR

Aucun de ces correctifs n'existe sur `dev`. Chaque section = un défaut
diagnostiqué + le correctif conçu et validé (tests unitaires + mesure sur
photo réelle) sur la branche abandonnée.

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

Dans `init_tesseract` (posé une fois, au démarrage) :
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

```
gray
  → contrast_stretch_percentile
  → adaptive_threshold(radius = height/8, C)
  → is_light_on_dark ?  invert            (mesuré sur la région CENTRALE, pas toute l'image)
  → deskew(binaire)                        (remplissage 255, re-binarisation post-rotation)
  → morphology_open                        (sur image déjà polarité-normalisée)
  → add_border(30, 255)
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
- **`color_adaptive_crop` sauté quand le frontend a déjà envoyé un crop
  plaque-shaped** (aspect ≥ 3.0) : redondant une fois le viseur frontend
  corrigé (§ticket frontend #3), et son profil de couleur orange attrape
  aussi peau, bois, terre — sur un crop déjà serré il ne peut que dégrader.
- **Double passe supprimée** : ne relancer la variante contraste/netteté
  rehaussée que si la passe native n'a produit **aucun texte du tout**,
  jamais simplement parce que `format_valid` est faux.

## 7. Sémantique de confiance (déjà correcte sur cette branche, à vérifier en réimplémentant)

`finalize` ne doit **jamais** réécrire `confidence` depuis `format_valid`
(l'ancien 0.90/0.50 dupliquait un signal et en jetait un autre). `confidence`
reste la mesure brute de Tesseract (`mean_text_conf`), `format_valid` reste
indépendant. C'est ce qui permet au ticket frontend §5 de recalibrer le seuil
de stabilité sur des données réelles plutôt que sur 0/50/90 arbitraires.

## Tests à ajouter

Voir `05_tests_de_non_regression.md`. En particulier : les **6 chaînes
brutes tirées des logs de terrain** (cadre de concessionnaire OCRisé) qui
doivent toutes retourner `None` depuis `extract_plate_fuzzy`, et les
fixtures de deskew en polarité correcte (dev-on-light, pas l'inverse comme
c'était le cas dans les tests originaux — l'ancien fixture masquait le
remplissage noir parce qu'il était lui-même en fond noir).
