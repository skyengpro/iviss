# Tests de non-régression à réintroduire

À ajouter **après** avoir implémenté les correctifs des deux tickets — pas
avant, pour éviter de les retoucher à chaque modification (retour
d'expérience de cette session).

## Backend — `plate_format.rs` / `ocr_service.rs`

### Chaînes brutes de terrain qui ne doivent PLUS produire de plaque

Tirées verbatim des logs d'un scan réel de la plaque `CE568LR`, où le crop
avait capté le cadre de concessionnaire ("TAUNUS AUTO — Mercedes-Benz und
smart in Wiesbaden"). Chacune revenait `format_valid: true` sur la branche
abandonnée avant correctif (voir `03_ticket_backend.md` §5) :

```rust
#[test]
fn test_extract_plate_fuzzy_rejects_dealer_surround() {
    for raw in [
        "7\n\nIO\n\nA\n\n2\n\nTAUNUS\n\nM BU SMT W",
        "1\n\nFO\n\nPY\n\nTAUNUSAUTO\n\nSBW",
        "SS\n\nGC\n\nFF\n\nTAUNUSAUTO MB1 S W",
        "FO\n\nTAUNUSAUTO\n\nMP",
        "IO\n\nTAUNUSAUTOM",
        "IU\n\nLAY\n\nTANS\n\nA\n\nINUSAUTO\n\nM\n\nB",
    ] {
        assert_eq!(extract_plate_fuzzy(raw), None,
            "dealer surround must not yield a plate: {raw:?}");
    }
}
```

### Le correctif ne doit pas casser la récupération légitime

```rust
#[test]
fn test_extract_plate_fuzzy_still_recovers_plates_with_stray_glyphs() {
    for (raw, expected) in [
        ("CE568LR", "CE568LR"),
        ("KECE568LR", "CE568LR"),   // K = fragment du logo CEMAC
        ("CE 568 LR", "CE568LR"),
    ] {
        assert_eq!(extract_plate_fuzzy(raw).as_deref(), Some(expected), "{raw:?}");
    }
}
```

### Chaînes courtes — plus de repli `len >= 4`

```rust
assert_eq!(extract_plate_fuzzy("CE12"), None);   // pas "CE12"
assert_eq!(extract_plate_fuzzy("1234"), None);
assert_eq!(extract_plate_fuzzy("CE1"), None);
assert_eq!(extract_plate_fuzzy(""), None);
```

### Ensemble — plaque vide ne doit jamais gagner sur confiance seule

```rust
#[test]
fn test_pick_best_ensemble_skips_candidates_without_a_plate() {
    let textual_but_empty = ScanResultData {
        plate: String::new(), raw_text: "200".into(),
        confidence: 0.63, format_valid: false, plate_type: None,
    };
    let real = ScanResultData {
        plate: "CE568LR".into(), raw_text: "CE568LR".into(),
        confidence: 0.0, format_valid: true, plate_type: None,
    };
    let result = pick_best_ensemble(vec![Some(textual_but_empty), Some(real)]);
    assert_eq!(result.plate, "CE568LR");
}
```

### Accord entre passes — confiance = max, pas moyenne

```rust
#[test]
fn test_agreeing_passes_report_the_stronger_confidence() {
    // PSM 7 à 0.10, PSM 13 à 0.42, même plaque -> résultat à 0.42
}
```

### Deskew — fixture en polarité correcte

**Piège rencontré** : le fixture original (`striped_text_image`) était
"barres claires sur fond noir", et `rotate_about_center` remplissait aussi en
noir — le remplissage était donc invisible au test, qui passait alors même
que le remplissage sur une vraie plaque (dark-on-light) produisait des coins
noirs. **Réécrire le fixture en dark-on-light** (barres sombres sur fond
clair) et vérifier explicitement les coins après rotation :

```rust
#[test]
fn test_deskew_fills_corners_with_background() {
    // rotation de 5°, puis assert que chaque coin de la sortie == 255
}

#[test]
fn test_deskew_output_is_bilevel() {
    // aucun pixel de la sortie ne doit être différent de 0 ou 255
    // (l'interpolation bilinéaire produit des gris qu'il faut re-binariser)
}
```

### Polarité — mesurée sur la région centrale

```rust
#[test]
fn test_is_light_on_dark_ignores_dark_edges() {
    // surround sombre hors de l'inset de 20%, centre clair
    // -> ne doit PAS être rapporté comme light-on-dark
    // (sur toute l'image ce cas mesure ~51% dark et basculerait à tort)
}

#[test]
fn test_is_light_on_dark_detects_inverted_plate() {
    // glyphes clairs sur fond sombre au centre -> doit être détecté
}
```

### Constantes et tests fantômes — à ne pas réintroduire

Sur la branche abandonnée, `ocr_service_tests.rs` redéclarait localement
`const ADAPTIVE_RADIUS: u32 = 40` (alors que la production dérive désormais
ce rayon de la hauteur), un `TMP_COUNTER` et un `TESSERACT` thread-local
propres au fichier de test (alors que la production avait supprimé le
chemin `/tmp`). Les tests correspondants passaient sans exercer la moindre
ligne de production. **Vérifier, à chaque test ajouté, qu'il importe et
exerce le symbole réellement défini dans `ocr_service.rs`**, pas une
redéclaration locale.

## Frontend — `useStabilityDetection.ts` / `useScanPlate.ts`

### La régression qui bloquait le scan en boucle infinie

```ts
it('tolerates a misread interleaved between agreeing readings', () => {
  // CE568LR (16), CE568LB (0), CE568LR (42), CE568LR (4)
  // -> stableResult confirmé sur CE568LR, agreement >= 3
  // Avec l'ancienne logique (compteur consécutif), le misread au milieu
  // remettait le compteur à zéro et ce cas ne se résolvait jamais.
});
```

### Confiance rapportée = max, pas moyenne

```ts
it('reports the strongest agreeing reading, not the average', () => {
  // lectures à 0, 63, 0 -> stableResult.confidence === 63
});
```

### Fixtures de confiance sur le régime réel

**Piège rencontré** : toutes les fixtures de test (`usePhotoCapture.test.ts`,
`useScanPlate.test.ts`) utilisaient des confidences de 0.76 à 0.92 (soit
76-92 sur l'échelle 0-100 exposée au frontend). Les logs de terrain donnent
0 à 63, médiane proche de 0-16. **Écrire au moins un test avec une confiance
dans le régime réel** (< 20), pour ne pas valider un chemin qui n'existe pas
en production.

### Géométrie — overlay et crop identiques par construction

```ts
// utils/__tests__/viewfinder.test.ts
// computeViewfinderCrop doit être LA fonction utilisée à la fois par
// ScanViewfinder.tsx (overlay dessiné) et par cropToViewfinder
// (imageProcessor.ts, crop envoyé) — pas deux calculs indépendants.
```

## Photo de référence pour les tests manuels

`samples/reference_plate_CE568LR.png` — rejouer `samples/binarize_replica.py`
et `samples/frame_probe.py` dessus après chaque changement au pipeline de
prétraitement, pour comparer visuellement à `samples/AFTER_fixed_4.5_C5.png`
(résultat attendu) plutôt qu'à `samples/BEFORE_shipped_3.5_C5.png` (état
`dev`, avec le cadre de concessionnaire visible dans le crop).
