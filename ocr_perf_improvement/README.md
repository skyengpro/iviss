# OCR / scan de plaques — dossier de reprise

Ce dossier condense tout le diagnostic et les correctifs conçus sur la branche
`perf/ocr-pipeline-resolution-and-speed`, qui a été abandonnée **sans jamais
committer** — `dev` est donc revenu à l'état pré-audit. Rien de ce qui suit
n'existe dans le code actuel. Objectif : ouvrir deux tickets (frontend,
backend) sans re-diagnostiquer ce qui l'a déjà été.

## Comment utiliser ce dossier

1. Lire [`00_audit_original_dev.md`](00_audit_original_dev.md) — le diagnostic
   racine (§1-§2), toujours valable contre `dev`.
2. Ouvrir un ticket frontend depuis [`02_ticket_frontend.md`](02_ticket_frontend.md).
3. Ouvrir un ticket backend depuis [`03_ticket_backend.md`](03_ticket_backend.md).
4. Avant de coder quoi que ce soit sur le prétraitement image ou le
   post-traitement de plaque, lire [`04_mesures_verifiees.md`](04_mesures_verifiees.md)
   — évite de refaire des mesures déjà faites (et une fausse piste déjà écartée).
5. En implémentant les correctifs, reprendre les tests de
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
| `samples/` | Photo de référence, images avant/après, scripts d'analyse rejouables. |

## Contexte de la photo de référence

`samples/reference_plate_CE568LR.png` est une plaque camerounaise réelle,
plaque `CE568LR` (CEMAC, catégorie `civil_cemac`), avec un cadre de
concessionnaire portant "TAUNUS AUTO — Mercedes-Benz und smart in Wiesbaden"
et le logo CEMAC + "CMR" imprimés sur la plaque elle-même. C'est l'image sur
laquelle toutes les mesures de ce dossier ont été faites — la reprendre pour
toute nouvelle vérification garde les résultats comparables.

## Ce qui est déjà tranché (ne pas rouvrir sans nouvelle preuve)

- **Ce n'est pas un manque de résolution.** Avec les correctifs de capture
  (E1-E2 du plan original), la hauteur de capitale mesurée est ~130px contre
  un plancher documenté de 30-33px. Voir `04_mesures_verifiees.md`.
- **`ADAPTIVE_C` n'est pas en cause.** Testé C=5 vs C=15 vs C=20 sur la photo
  de référence : glyphes visuellement identiques. Ne pas y retoucher sans
  nouvelle mesure.
- **`tessedit_char_whitelist` reste un indice, pas une contrainte.** Non
  honoré par le moteur LSTM en 4.x, support partiel en 5.x — confirmé par
  webrecherche (issues tesseract-ocr/tesseract #751, PR #2294).
- **L'OEM ne peut pas être posé.** `leptess 0.14` n'expose aucun paramètre OEM
  sur `LepTess::new`, et `tessedit_ocr_engine_mode` est lu à l'Init — le poser
  comme variable après coup n'a aucun effet. Non bloquant : le défaut est déjà
  LSTM sur `eng`. À revisiter seulement en cas de montée de version de `leptess`.
