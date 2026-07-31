# Mesures vérifiées

Chiffres obtenus par mesure directe sur `samples/reference_plate_CE568LR.png`
(1080×810), avec les scripts de `samples/`. Rejouables : `python3
samples/frame_probe.py samples/reference_plate_CE568LR.png`, etc. (Pillow
seul requis, pas de numpy.)

## Géométrie de la plaque

```
Plaque orange détectée par profil HSV : x=47 y=411, 1011×220 px
Aspect réel : 4.60   (norme CEMAC 520×110mm = 4.7)
```

### Débord vertical du viseur selon `VIEWFINDER_ASPECT`

> **⚠️ Nom réel : `ImageProcessor.VF_ASPECT`** (`imageProcessor.ts:190`).
> `VIEWFINDER_ASPECT` n'existe pas sur `dev`. Valeur 3.5 correcte. Noter aussi
> qu'il existe **trois** calculs de cadre indépendants aujourd'hui, dont un
> troisième aspect codé en dur à 2.0 dans `preprocessForPhotoCapture`
> (`imageProcessor.ts:154`) — voir `06_validation_documentaire.md` §3.2.

Un agent cadre la plaque pour qu'elle remplisse la largeur du viseur. La
hauteur de la boîte dépend alors de l'aspect choisi :

| Aspect viseur | Hauteur boîte | Hauteur plaque | Débord vertical |
|---|---|---|---|
| 3.5 (valeur d'origine sur `dev`) | 289 px | 220 px | **+69px (+31,3%)** |
| 4.0 | 253 px | 220 px | +33px (+14,9%) |
| **4.5 (recommandé)** | 225 px | 220 px | **+5px (+2,1%)** |
| 4.7 (norme exacte) | 215 px | 220 px | -5px (-2,2%) |

**Conclusion : passer `VIEWFINDER_ASPECT` de 3.5 à 4.5.** Les 69px de débord à
3.5 tombent exactement sur la bande où se trouve le cadre de concessionnaire
("TAUNUS AUTO..."), qui alimente ensuite `PSM 11` et le correcteur flou.
`CMR` et le logo CEMAC restent inclus quel que soit l'aspect — ils sont
imprimés sur la plaque elle-même, pas sur le cadre. Voir
`03_ticket_backend.md` pour le traitement de ce bruit résiduel.

## Hauteur de capitale (résolution)

Mesurée sur le chemin live (crop 800×229, cadrage serré) : **~130px**.

**La famine de résolution décrite dans l'audit d'origine (§2.1) est réglée**
dès lors que la capture demande `1920×1080` + `forceScreenshotSourceSize` +
`grabFrame`/`takePhoto` (voir `02_ticket_frontend.md`). **La résolution de
capture 1920×1080 est actée et ne doit pas être baissée** — sur iOS, faute de
support `ImageCapture`, c'est le seul levier de résolution disponible.

> **⚠️ Correction (2026-07-31) : le « plancher documenté de 30-33px » n'existe
> pas comme tel.** Vérification faite :
>
> - *ImproveQuality* ne donne **aucun** chiffre de hauteur de capitale. Il ne
>   pose qu'un minimum de **300 dpi** et renvoie au test « Optimal image
>   resolution » de willus.com.
> - Ce test conclut : « **there is a sweet spot for Tesseract of about 30 pixels
>   for the height of a capital letter** ». C'est un **optimum**, pas un
>   plancher — et son auteur note explicitement une dégradation en haute
>   résolution (« Tess v4.0.0 definitely has a consistent issue with high-res
>   fonts »).
>
> Conséquence : **« ~130px = marge ×4 » n'est pas établi.** 130px est ×4
> au-delà de l'optimum, du côté où le test de référence mesure une perte de
> précision. Cela ne remet pas en cause la cause racine (21-25px était bien trop
> bas), mais la conclusion « ne plus y toucher » n'est pas démontrée.
>
> **Mesure à faire (non bloquante, ne conditionne aucun autre correctif) :**
> balayer la **largeur du crop OCR** — `maxWidth = 800` aujourd'hui — sur la
> photo de référence, même protocole que le balayage `ADAPTIVE_C` ci-dessous.
> C'est la largeur du crop, et non la résolution de capture, qui est le levier.
> Tant que la mesure n'est pas faite, **garder 800px** : c'est l'état sur lequel
> les ~130px ont été mesurés.

## Biais du deskew (niveaux de gris vs binaire)

`estimate_skew_angle` cherche l'angle qui maximise la variance de projection
par ligne. Sur une plaque **parfaitement droite** :

| Variante | Angle retenu | Pic/plancher | Avantage sur 0° |
|---|---|---|---|
| Recherche sur **niveaux de gris** (état `dev`) | **+2.5°** ❌ | 1.10× | 2,1% |
| Recherche sur **image binarisée** (recommandé) | **+0.0°** ✅ | 1.88× | — |

Sur niveaux de gris, la variance de projection est dominée par les grandes
plages de luminance (carrosserie, plaque, pare-chocs), pas par les lignes de
texte : le discriminant s'effondre à 2%, et l'angle choisi devient du bruit
— différent à chaque frame. Sur binaire (ink=0/bg=255), le discriminant
remonte à 88% au-dessus du niveau plancher.

**Conclusion : binariser (seuillage adaptatif + normalisation de polarité)
AVANT de chercher l'angle de rotation, pas après.** Voir
`03_ticket_backend.md` pour l'ordre complet du pipeline.

Reproductible avec `samples/skew_probe.py` (sur fixture synthétique) et via
`samples/binarize_replica.py` (sur la photo réelle).

## Remplissage du deskew et polarité

Le remplissage noir (`Luma([0])`) que `rotate_about_center` insère aux coins
lors d'une rotation :
- **Ne biaise pas la recherche d'angle** : à 7° de rotation, le remplissage
  ne couvre que 10,9% du cadre — loin d'affecter l'argmax.
- **Ne bascule pas la polarité** globale (`is_light_on_dark` sur toute
  l'image) : il faudrait 39% de couverture pour franchir le seuil de 50%,
  un remplissage à 7° n'en fait que 10,9%.
- **Mais reste visible dans l'image finale** : un fond binarisé + remplissage
  noir laisse des coins sombres qui, une fois seuillés, restent à 100% noir
  — exactement les "bordures sombres détectées comme caractères parasites"
  que la doc Tesseract dit de retirer.

**Conclusion : remplir en blanc (255), pas en noir, et seulement après avoir
normalisé la polarité** (pour que 255 = fond soit vrai par construction). Voir
`03_ticket_backend.md`.

## Marge de la détection de polarité

`is_light_on_dark` mesurée sur toute l'image plutôt que sur la région
centrale : fraction sombre mesurée entre 40% (cadrage serré) et 45%
(cadrage large incluant carrosserie), contre un seuil de bascule à 50%.
**Marge de 5 à 10 points seulement**, et `tessedit_do_invert=0` rend un
mauvais choix irrécupérable. Recommandation : mesurer sur la région centrale
(± 20% d'inset de chaque côté), qui isole la plaque de la carrosserie
environnante.

## `ADAPTIVE_C` — hypothèse écartée

Balayage C=5 / C=10 / C=15 / C=20 sur la photo de référence (chemins live ET
photo) : la fraction de pixels sombres varie de 45% à 40%, mais **les
glyphes restent visuellement identiques** à l'inspection des PNG produits.
**Ne pas retoucher cette constante** pour expliquer un échec OCR — chercher
ailleurs (géométrie du viseur, deskew, ensemble de passes).

> **Note (2026-07-31).** `ADAPTIVE_C` **disparaît** avec le passage à Sauvola
> (`03_ticket_backend.md` §3 bis) : le paramètre devient `k` (≈ 0.35), qui joue
> sur la variance locale et non sur un décalage constant de la moyenne. Cette
> mesure reste néanmoins la preuve que **la binarisation n'est pas le facteur
> limitant sur ce corpus** — le gain attendu de Sauvola est théorique, il porte
> sur l'éclairage inégal que la photo de référence ne représente pas. Ne pas
> présenter le changement comme un gain mesuré tant qu'une fixture à gradient
> et un jeu de terrain ne l'ont pas montré.

## Qualité JPEG du chemin live

`LIVE_CROP_OPTIONS.quality` était à 0.7 sur `dev` (contre 0.95 pour le chemin
photo). Sur un crop 800px de large l'écart de poids réseau est de l'ordre de
15 Ko — la doc Tesseract déconseille explicitement les artefacts JPEG en
entrée (*ImproveQuality*), et à q0.7 le "ringing" JPEG s'imprime sur les
traits des glyphes. **Aligner à 0.95** ; ce n'est pas la largeur du crop qui
coûtait cher, c'est la largeur (`maxWidth: 800`), qui reste inchangée.

> **⚠️ Correction (2026-07-31).** `LIVE_CROP_OPTIONS` n'existe pas sur `dev` et
> les deux valeurs citées sont fausses — dans le sens défavorable :
>
> | | Annoncé ici | Réel sur `dev` |
> |---|---|---|
> | Chemin live | 0.7 | **0.65** (`imageProcessor.ts:665`) |
> | Chemin photo | 0.95 | **0.92** (`imageProcessor.ts:292`) |
>
> La conclusion est inchangée et même renforcée : aligner **les deux** à 0.95,
> en introduisant l'objet `LIVE_CROP_OPTIONS: { maxWidth: 800, quality: 0.95 }`.

## Conformité Tesseract — tableau de correspondance

Tableau revérifié contre la documentation officielle le 2026-07-31 (les
formulations « recommandation documentaire » sont désormais citées, pas
paraphrasées).

| Point | État `dev` | Recommandation documentaire | Statut |
|---|---|---|---|
| Hauteur de capitale | ❌ ~21-25px avant capture correcte, ~130px après | *ImproveQuality* : **aucun chiffre**, seulement ≥300 dpi. Le test référencé (willus.com) donne un **optimum** « of about 30 pixels », pas un plancher | ✅ famine réglée ; ⚠️ la marge ×4 reste **à mesurer**, voir plus haut |
| Bordure autour du texte | ✅ 30px | « a small border (e.g. 10 px) » ; les grandes bordures « cause problems » | acceptable, **ne pas augmenter** |
| Binarisation adaptative | ⚠️ moyenne locale − C (Bradley/Wellner) | Tesseract 5 embarque Sauvola via `thresholding_method` ; Leptonica : `t = m·(1 − k·(1 − s/128))`, k ≈ 0.35 | **à remplacer par Sauvola en Rust** — `thresholding_method` inatteignable via `leptess`, voir `06` §5 |
| `load_system_dawg=0`, `load_freq_dawg=0` | ❌ absents sur `dev` | indispensable pour codes alphanumériques | à réappliquer — présents dans l'énum `leptess::Variable` |
| `tessedit_do_invert=0` | ❌ absent sur `dev` | confirmé : par défaut Tesseract tente l'OCR **deux fois** (normal + inversé) | à réappliquer, mais **déprécié, retiré en Tesseract 6.0** au profit de `invert_threshold` (inatteignable via `leptess`). Image actuelle : Tesseract 5.3.0, OK |
| `tessedit_char_whitelist` | présent mais fiable à tort | non honoré par LSTM (issues #751, #998 ; PR #2294) | correction en post-traitement uniquement |
| OEM explicite | ❌ jamais posé | — | **impossible avec `leptess 0.14`** : pas de paramètre sur `new`, **et** `tess_api` privé. Voir README |
| Rayon de seuillage fixe (40px) | ❌ absolu sur `dev` | Leptonica : `whsize` min 2, typiquement ≥ 7 | à réappliquer (`height/8`, borné [15,100]) |
| Remplir les rotations en fond, pas en noir | ❌ noir sur `dev` | les bordures sombres sont « erroneously picked up as extra characters » | à réappliquer, voir mesure ci-dessus |
| Bordure du chemin **inversé** | ❌ **noire de 30px** (`ocr_service.rs:137`, `:194`) | idem ci-dessus | **défaut découvert le 2026-07-31**, voir `03_ticket_backend.md` §3 ter |
| Polarité de l'image soumise | ❌ deux polarités tentées à l'aveugle | « for 4.x version use dark text on light background » | à normaliser nous-mêmes avant Tesseract |
