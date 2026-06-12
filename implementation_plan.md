# Amélioration de la reconnaissance multi-format des plaques d'immatriculation camerounaises

## Contexte

L'application IVISS ne reconnaît actuellement qu'un seul format de plaque d'immatriculation : le format CEMAC civil standard `XX ###XX` (2 lettres + 3 chiffres + 2 lettres, ex: `CE 128 BC`). Or, le Cameroun utilise **10 catégories distinctes** de plaques selon les PDFs fournis. Cette limitation impacte directement la capacité des agents à contrôler les véhicules sur le terrain.

## Diagnostic de l'existant

### Problèmes identifiés

| Composant | Fichier | Problème |
|-----------|---------|----------|
| OCR Backend | [ocr_service.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/services/ocr_service.rs#L13) | `PLATE_REGEX` = `^[A-Z]{2}[0-9]{3}[A-Z]{2}$` — un seul format |
| Photo OCR | [photo_ocr_service.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/services/photo_ocr_service.rs#L12-L13) | `PHOTO_PLATE_REGEX` — même limitation |
| Search handler | [search_vehicle.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/handlers/search_vehicle.rs#L192-L198) | Partiellement mis à jour mais formats incomplets/incorrects |
| Frontend | [imageProcessor.ts](file:///home/lonsti-ws/Documents/iviss/frontend/src/utils/imageProcessor.ts#L453-L467) | `validateCameroonPlate` — un seul format |
| Scan DTO | [scan.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/dto/scan.rs#L13-L14) | Docstring mentionne uniquement "XX###XX", pas de champ `plate_type` |
| Fuzzy extraction | [ocr_service.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/services/ocr_service.rs#L562-L614) | `extract_plate_fuzzy` — correction OCR codée en dur pour 7 caractères seulement |

### Duplication du code

La logique de format de plaque est **dupliquée 3 fois** : `ocr_service.rs`, `photo_ocr_service.rs`, et `search_vehicle.rs`. Chacune avec sa propre regex et sa propre logique de validation.

---

## Catalogue complet des formats (PDFs)

| # | Catégorie | Structure (compact) | Exemple | Regex compact |
|---|-----------|---------------------|---------|---------------|
| 1 | **Civil CEMAC** (nouveau) | `[REGION][3D][2L]` | `CE128BC` | `(?:REGIONS)\d{3}[A-Z]{2}` |
| 2 | **Civil série** (historique) | `[REGION][4D][1-2L]` | `LT4568A` | `(?:REGIONS)\d{4}[A-Z]{1,2}` |
| 3 | **Véhicule attelé** (RE/SR/SE/TR) | `[REGION][TYPE][4D][1-2L]` | `LTSR9652A` | `(?:REGIONS)(?:RE\|SR\|SE\|TR)\d{4}[A-Z]{1,2}` |
| 4 | **État** (CA/AN) | `[CA\|AN][4D][1-2L]` | `AN9652E` | `(?:CA\|AN)\d{4}[A-Z]{1,2}` |
| 5 | **Diplomatique** | `[CMD\|CPC\|CD\|CC\|PA][2-3D]RC[1-4D]` | `PA02RC521` | `(?:CMD\|CPC\|CD\|CC\|PA)\d{2,3}RC\d{1,4}` |
| 6 | **Temporaire** (IT) | `IT[5D]RC` | `IT21052RC` | `IT\d{5}RC` |
| 7 | **Essai** (WG) | `[REGION][4D]WG` | `CE2456WG` | `(?:REGIONS)\d{4}WG` |
| 8 | **Transit** (WT) | `WT[6D][0-1D]` | `WT1202082` | `WT\d{6,7}` |
| 9 | **Postes** (PT) | `PT[3D][2D]` | `PT01200` | `PT\d{3}\d{2}` |
| 10 | **Spécial investissement** (IS) | `IS[5-6D]RC` | `IS245642RC` | `IS\d{5,6}RC` |

> `REGIONS` = `AD|CE|EN|ES|LT|NO|NW|OU|SU|SW|SO` (SO inclus pour rétro-compatibilité avec l'ancien code "Sud-Ouest")

> [!IMPORTANT]
> Le format civil historique (format 2) est très courant au Cameroun et n'est pas du tout reconnu par l'OCR actuel. C'est probablement la source principale de faux négatifs.

---

## Proposed Changes

### Nouveau module partagé : `plate_format`

#### [NEW] [plate_format.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/utils/plate_format.rs)

Création d'un module centralisé qui sera la **source unique de vérité** pour tous les formats de plaques. Ce module sera utilisé par l'OCR, le photo OCR et la validation manuelle.

Contenu du module :
- `PlateCategory` — enum sérialisable représentant les 10 catégories + `Unknown`
- `PlateMatch` — struct avec `plate: String`, `category: PlateCategory`
- `REGION_CODES` — constante avec les 11 codes régionaux
- Regexes compilées une seule fois (`Lazy<Regex>`) pour chaque catégorie
- `classify(compact: &str) -> Option<PlateMatch>` — identifie le format d'une plaque compactée
- `is_valid(compact: &str) -> bool` — validation rapide
- `extract_first(text: &str) -> Option<PlateMatch>` — recherche dans du texte brut (pour OCR)
- `normalise(raw: &str) -> String` — normalisation (uppercase, strip non-alphanum)
- `format_display(compact: &str) -> String` — formatage lisible (`CE128BC` → `CE 128 BC`)
- `fuzzy_correct(raw: &str) -> Option<PlateMatch>` — correction OCR multi-format avec corrections position-aware par catégorie

---

### Backend OCR

#### [MODIFY] [ocr_service.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/services/ocr_service.rs)

- Supprimer la `PLATE_REGEX` locale (ligne 13) — remplacée par le module `plate_format`
- Réécrire `extract_plate_fuzzy` pour utiliser `plate_format::extract_first` puis `plate_format::fuzzy_correct` pour correction OCR multi-format
- Mettre à jour `try_ocr_path` : `format_valid` sera déterminé par `plate_format::is_valid`
- Conserver `normalise_plate` comme wrapper vers `plate_format::normalise`
- **Aucun changement** au pipeline d'image (deskew, threshold, morphology, etc.)

#### [MODIFY] [photo_ocr_service.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/services/photo_ocr_service.rs)

- Supprimer `PHOTO_PLATE_REGEX` (ligne 12-13)
- Réécrire `extract_plate_strict` pour utiliser `plate_format::extract_first`
- Mettre à jour `pick_best` et `enhance_photo_result` pour utiliser le module partagé

---

### Handler de recherche

#### [MODIFY] [search_vehicle.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/handlers/search_vehicle.rs)

- Remplacer la `PLATE_REGEX` locale (lignes 192-199) par un appel à `plate_format::is_valid`
- Simplifier `validate_plate_format` — normalise + valide via le module partagé
- Mise à jour du message d'erreur avec les formats désormais supportés

---

### DTO

#### [MODIFY] [scan.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/dto/scan.rs)

- Ajouter un champ optionnel `plate_type: Option<String>` à `ScanResultData`
  - Sérialisé avec `#[serde(skip_serializing_if = "Option::is_none")]`
  - Valeurs: `"civil_cemac"`, `"civil_legacy"`, `"trailer"`, `"state"`, `"diplomatic"`, `"temporary"`, `"test_vehicle"`, `"transit"`, `"postal"`, `"special_investment"`, ou absent si non classifiable
- Mettre à jour la docstring et les impls `PartialEq`/`Default`

---

### Module registration

#### [MODIFY] [utils/mod.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/utils/mod.rs)

- Ajouter `pub mod plate_format;`

---

### Frontend

#### [MODIFY] [imageProcessor.ts](file:///home/lonsti-ws/Documents/iviss/frontend/src/utils/imageProcessor.ts)

- Réécrire `validateCameroonPlate` pour supporter tous les formats
- Retourner un objet `{ formatted: string, category: string }` au lieu d'un simple `string | null`

> [!WARNING]
> Ce changement modifie le type de retour de `validateCameroonPlate`. Il faudra vérifier si d'autres composants consomment cette méthode pour adapter les appels.

---

### Tests

#### [MODIFY] [ocr_service_tests.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/tests/ocr_service_tests.rs)

- Adapter les tests existants pour le multi-format
- Ajouter des tests pour chaque catégorie de plaque
- Tester la correction fuzzy pour chaque format

#### [MODIFY] [photo_ocr_service_tests.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/tests/photo_ocr_service_tests.rs)

- Mettre à jour pour le multi-format

#### [NEW] [plate_format_tests.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/tests/plate_format_tests.rs)

- Tests unitaires exhaustifs pour le nouveau module partagé
- Tests de classification, validation, extraction, correction fuzzy
- Tests edge cases (strings vides, caractères spéciaux, formats partiels)

#### [MODIFY] [imageProcessor.test.ts](file:///home/lonsti-ws/Documents/iviss/frontend/src/utils/__tests__/imageProcessor.test.ts)

- Adapter les tests `validateCameroonPlate` pour le multi-format

---

## Open Questions

> [!IMPORTANT]
> **Q1 — Type de retour frontend** : Le changement de `validateCameroonPlate` de `string | null` à `{ formatted: string, category: string } | null` est un breaking change dans le frontend. Est-ce acceptable ou préférez-vous garder la signature actuelle et ajouter une méthode séparée `classifyPlate()` ?

> [!IMPORTANT]
> **Q2 — Code régional SO vs SW** : Le PDF utilise "SO" pour Sud-Ouest, mais le code CEMAC standard utilise "SW". L'implémentation actuelle dans `search_vehicle.rs` utilise "SW". Dois-je accepter les deux ("SO" et "SW") pour la rétro-compatibilité ?

> [!IMPORTANT]
> **Q3 — Formats SN (Police/Sécurité) et militaire (7 chiffres)** : Ces formats existent dans le handler `search_vehicle.rs` actuel mais ne sont pas mentionnés dans les PDFs. Dois-je les conserver quand même ?

---

## Verification Plan

### Automated Tests
```bash
# Tests unitaires du nouveau module plate_format
cargo test --lib tests::plate_format_tests

# Tests OCR existants mis à jour
cargo test --lib tests::ocr_service_tests

# Tests photo OCR
cargo test --lib tests::photo_ocr_service_tests

# Tests du handler search_vehicle
cargo test --lib handlers::search_vehicle::tests

# Compilation complète sans warnings
cargo build --release 2>&1 | grep -E "warning|error"

# Tests frontend
cd frontend && npx vitest run src/utils/__tests__/imageProcessor.test.ts
```

### Manual Verification
- Vérifier que le backend compile sans warnings
- Confirmer que l'API `POST /api/v1/scan/plate` retourne le `plate_type` dans la réponse
- Vérifier que l'API `POST /api/v1/vehicles/search` accepte tous les formats documentés
