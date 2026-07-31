# OCR / scan de plaques — dossier de reprise

Ce dossier condense tout le diagnostic et les correctifs conçus pour le pipeline
de capture / scan de plaques. Il est porté par la branche
`perf/ocr-pipeline-resolution-and-speed`, **chantier actif de cette
amélioration**. Aucun de ces correctifs n'est encore appliqué au code : le
pipeline OCR y est à l'état `dev`, pré-audit. Objectif : dérouler deux lots
(frontend, backend) sans re-diagnostiquer ce qui l'a déjà été.

## Comment utiliser ce dossier

1. Lire [`00_audit_original_dev.md`](00_audit_original_dev.md) — le diagnostic
   racine (§1-§2), toujours valable contre `dev`.
2. Lire [`06_validation_documentaire.md`](06_validation_documentaire.md) — le
   contrôle du diagnostic contre le code réel et la documentation officielle.
   **Il corrige des noms et des valeurs des deux tickets** (qui décrivaient une
   version antérieure du code) et ajoute trois défauts qu'ils ne couvraient pas.
3. Dérouler le lot frontend depuis [`02_ticket_frontend.md`](02_ticket_frontend.md).
4. Dérouler le lot backend depuis [`03_ticket_backend.md`](03_ticket_backend.md).
5. Avant de coder quoi que ce soit sur le prétraitement image ou le
   post-traitement de plaque, lire [`04_mesures_verifiees.md`](04_mesures_verifiees.md)
   — évite de refaire des mesures déjà faites (et une fausse piste déjà écartée).
6. En implémentant les correctifs, reprendre les tests de
   [`05_tests_de_non_regression.md`](05_tests_de_non_regression.md) — en
   particulier les 6 chaînes brutes tirées des logs de terrain, qui doivent
   toutes échouer à produire une plaque.

## Contenu

| Fichier | Rôle |
|---|---|
| `00_audit_original_dev.md` | Audit d'origine (root cause). §1, §2, §2.5, §2.7 valables. §4 obsolète. |
| `02_ticket_frontend.md` | Défauts + correctifs conçus, côté React/TS. Prêt à coller dans un ticket. |
| `03_ticket_backend.md` | Défauts + correctifs conçus, côté Rust/Tesseract. Prêt à coller dans un ticket. |
| `04_mesures_verifiees.md` | Chiffres obtenus par mesure directe (aspect de plaque, hauteur de capitale, biais du deskew, etc.) — à ne pas redériver. |
| `05_tests_de_non_regression.md` | Fixtures et assertions à réintroduire avec les correctifs. |
| `06_validation_documentaire.md` | Contrôle du dossier contre le code réel et la documentation officielle. Corrections de noms/valeurs, 3 défauts supplémentaires, limites vérifiées des crates. |
| `samples/` | Photo de référence, images avant/après, scripts d'analyse rejouables. |

## Contexte de la photo de référence

`samples/reference_plate_CE568LR.png` est une plaque camerounaise réelle,
plaque `CE568LR` (CEMAC, catégorie `civil_cemac`), avec un cadre de
concessionnaire portant "TAUNUS AUTO — Mercedes-Benz und smart in Wiesbaden"
et le logo CEMAC + "CMR" imprimés sur la plaque elle-même. C'est l'image sur
laquelle toutes les mesures de ce dossier ont été faites — la reprendre pour
toute nouvelle vérification garde les résultats comparables.

## Ce qui est déjà tranché (ne pas rouvrir sans nouvelle preuve)

- **Résolution de capture : 1920×1080, acté.** La documentation Tesseract ne
  prescrit aucune résolution de capture — elle parle de DPI et de hauteur de
  capitale. 1920×1080 est retenu parce qu'il donne la marge de cadrage et de
  crop nécessaire, et parce que sur iOS (pas de support `ImageCapture`) c'est
  le **seul** levier de résolution disponible. Ne pas le baisser.
- **Ce n'est plus un manque de résolution.** Avec les correctifs de capture, la
  hauteur de capitale mesurée est ~130px, là où le test de référence de
  Tesseract situe l'optimum autour de 30px. La famine (21-25px) est réglée.
  **En revanche, « ~130px = marge ×4 » n'est pas établi** : l'optimum n'est pas
  un plancher, et le même test mesure une dégradation en haute résolution. Le
  levier à mesurer est la **largeur du crop OCR** (800px aujourd'hui), pas la
  résolution de capture. Voir `04_mesures_verifiees.md` et
  `06_validation_documentaire.md` §4.
- **`ADAPTIVE_C` n'est pas en cause.** Testé C=5 vs C=15 vs C=20 sur la photo
  de référence : glyphes visuellement identiques. Ne pas y retoucher sans
  nouvelle mesure.
- **Binarisation : passage à Sauvola, acté.** Le seuillage actuel est une
  moyenne locale − C (Bradley/Wellner) : il ignore la variance locale. Sauvola
  (`t = m·(1 − k·(1 − s/128))`, k ≈ 0.35) l'utilise, ce qui est précisément le
  bon comportement sous éclairage inégal — le cas de terrain. À implémenter
  **en Rust**, par une seconde image intégrale des carrés : même coût O(1) par
  pixel, aucune dépendance nouvelle, aucun `unsafe`. Ni `thresholding_method`
  (Tesseract 5) ni `pixSauvolaBinarizeTiled` (Leptonica) ne sont atteignables
  depuis `leptess 0.14` — voir `06_validation_documentaire.md` §5.
- **`tessedit_char_whitelist` reste un indice, pas une contrainte.** Non
  honoré par le moteur LSTM en 4.x, support partiel en 5.x — confirmé par
  webrecherche (issues tesseract-ocr/tesseract #751, #998, PR #2294).
- **L'OEM ne peut pas être posé.** `leptess 0.14` n'expose aucun paramètre OEM
  sur `LepTess::new`, et `tessedit_ocr_engine_mode` est lu à l'Init — le poser
  comme variable après coup n'a aucun effet. **Vérifié plus fort encore** : le
  champ `LepTess.tess_api` est privé, donc le `TessApi.raw` de
  `tesseract-plumbing` est inatteignable. Non bloquant : le défaut est déjà
  LSTM sur `eng`. À revisiter seulement en cas de montée de version.
- **`tessedit_do_invert=0` est valable mais daté.** Le paramètre est déprécié
  et disparaît en Tesseract 6.0 au profit de `invert_threshold`. L'image Docker
  est bookworm + `libtesseract5` = Tesseract 5.3.0, où il fonctionne encore.
  `invert_threshold` n'est pas atteignable via `leptess` : la montée en
  Tesseract 6 imposera `tesseract-plumbing` en dépendance directe.
