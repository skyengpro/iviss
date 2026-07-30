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
Plancher documenté par Tesseract (*ImproveQuality*) : 30-33px. Marge ×4.
**La famine de résolution décrite dans l'audit d'origine (§2.1) est réglée**
dès lors que la capture demande `1920×1080` + `forceScreenshotSourceSize` +
`grabFrame`/`takePhoto` (voir `02_ticket_frontend.md`). Ne pas ré-augmenter
la résolution demandée sans nouvelle preuve d'un manque.

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

## Qualité JPEG du chemin live

`LIVE_CROP_OPTIONS.quality` était à 0.7 sur `dev` (contre 0.95 pour le chemin
photo). Sur un crop 800px de large l'écart de poids réseau est de l'ordre de
15 Ko — la doc Tesseract déconseille explicitement les artefacts JPEG en
entrée (*ImproveQuality*), et à q0.7 le "ringing" JPEG s'imprime sur les
traits des glyphes. **Aligner à 0.95** ; ce n'est pas la largeur du crop qui
coûtait cher, c'est la largeur (`maxWidth: 800`), qui reste inchangée.

## Conformité Tesseract — tableau de correspondance

| Point | État `dev` | Recommandation documentaire | Statut |
|---|---|---|---|
| Hauteur de capitale ≥ 30-33px | ❌ ~21-25px avant capture correcte | — | ✅ Réglé par le fix de résolution frontend |
| Bordure autour du texte | ✅ 30px | ~10px minimum | conforme |
| Binarisation adaptative | ✅ | conforme (Otsu interne sous-optimal en éclairage inégal) | conforme |
| `load_system_dawg=0`, `load_freq_dawg=0` | ❌ absents sur `dev` | indispensable pour codes alphanumériques | à réappliquer |
| `tessedit_do_invert=0` | ❌ absent sur `dev` | évite un doublement silencieux du temps de calcul | à réappliquer |
| `tessedit_char_whitelist` | présent mais fiable à tort | ne pas en dépendre — non honoré par LSTM en 4.x | correction en post-traitement uniquement |
| OEM explicite | ❌ jamais posé | — | **impossible avec `leptess 0.14`**, voir README |
| Rayon de seuillage adaptatif fixe (40px) | ❌ absolu sur `dev` | doit être proportionnel à la hauteur | à réappliquer (`height/8`, borné [15,100]) |
| Remplir les rotations en fond, pas en noir | ❌ noir sur `dev` | supprime les "bordures sombres" | nouveau constat de cette session, voir mesure ci-dessus |
