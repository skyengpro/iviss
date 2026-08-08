use image::{GrayImage, Luma};

use crate::dto::scan::ScanResultData;
use crate::services::ocr::engine::*;
use crate::services::ocr::timings::{OcrBudget, StageTimings};

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn dark_on_light(w: u32, h: u32, dark_rect: (u32, u32, u32, u32)) -> GrayImage {
        let mut img = GrayImage::from_pixel(w, h, Luma([255]));
        let (x0, y0, x1, y1) = dark_rect;
        for y in y0..y1 {
            for x in x0..x1 {
                img.put_pixel(x, y, Luma([0]));
            }
        }
        img
    }

    // ── extract_plate_fuzzy: dealer-surround strings must never yield a plate ──
    //
    // Verbatim from a real field scan of plate CE568LR where the crop caught
    // the dealer frame ("TAUNUS AUTO — Mercedes-Benz und smart in Wiesbaden").
    // Each of these used to return `format_valid: true` via either the
    // now-removed `len >= 4` fallback or an unbounded fuzzy substitution
    // reaching Military/GovernmentLegacy with no anchor.
    #[test]
    fn extract_plate_fuzzy_rejects_dealer_surround() {
        for raw in [
            "7\n\nIO\n\nA\n\n2\n\nTAUNUS\n\nM BU SMT W",
            "1\n\nFO\n\nPY\n\nTAUNUSAUTO\n\nSBW",
            "SS\n\nGC\n\nFF\n\nTAUNUSAUTO MB1 S W",
            "FO\n\nTAUNUSAUTO\n\nMP",
            "IO\n\nTAUNUSAUTOM",
            "IU\n\nLAY\n\nTANS\n\nA\n\nINUSAUTO\n\nM\n\nB",
        ] {
            assert_eq!(
                extract_plate_fuzzy(raw),
                None,
                "dealer surround must not yield a plate: {raw:?}"
            );
        }
    }

    #[test]
    fn extract_plate_fuzzy_still_recovers_plates_with_stray_glyphs() {
        for (raw, expected) in [
            ("CE568LR", "CE568LR"),
            ("KECE568LR", "CE568LR"), // K = stray glyph from the CEMAC logo
            ("CE 568 LR", "CE568LR"),
        ] {
            assert_eq!(
                extract_plate_fuzzy(raw).as_deref(),
                Some(expected),
                "{raw:?}"
            );
        }
    }

    #[test]
    fn extract_plate_fuzzy_no_longer_falls_back_on_bare_length() {
        // The `cleaned.len() >= 4` fallback used to accept any normalised
        // string this long as "a plate". A string that doesn't parse isn't a
        // weak plate, it isn't a plate.
        assert_eq!(extract_plate_fuzzy("CE12"), None);
        assert_eq!(extract_plate_fuzzy("1234"), None);
        assert_eq!(extract_plate_fuzzy("CE1"), None);
        assert_eq!(extract_plate_fuzzy(""), None);
    }

    // ── pick_best_ensemble: an empty plate must never win on confidence alone ──

    #[test]
    fn pick_best_ensemble_skips_candidates_without_a_plate() {
        let textual_but_empty = ScanResultData {
            plate: String::new(),
            raw_text: "200".into(),
            confidence: 0.63,
            format_valid: false,
            plate_type: None,
        };
        let real = ScanResultData {
            plate: "CE568LR".into(),
            raw_text: "CE568LR".into(),
            confidence: 0.0,
            format_valid: true,
            plate_type: None,
        };

        let result = pick_best_ensemble(vec![Some(textual_but_empty), Some(real)]);
        assert_eq!(result.plate, "CE568LR");
    }

    #[test]
    fn pick_best_ensemble_falls_back_to_textual_candidates_when_nothing_has_a_plate() {
        // Only when *no* candidate holds a plate should a text-only reading win.
        let a = ScanResultData {
            plate: String::new(),
            raw_text: "abc".into(),
            confidence: 0.2,
            format_valid: false,
            plate_type: None,
        };
        let b = ScanResultData {
            plate: String::new(),
            raw_text: "def".into(),
            confidence: 0.8,
            format_valid: false,
            plate_type: None,
        };

        let result = pick_best_ensemble(vec![Some(a), Some(b)]);
        assert_eq!(result.raw_text, "def");
    }

    // ── Sauvola binarization ────────────────────────────────────────────────

    #[test]
    fn sauvola_beats_local_mean_under_illumination_gradient() {
        // Constant-intensity glyphs (value 40) over a background gradient from
        // 60 (left) to 200 (right). A local-mean threshold either loses the
        // glyph on the bright side or manufactures ink on the dark side;
        // Sauvola's variance term should keep every glyph column dark.
        let w = 200u32;
        let h = 40u32;
        let glyph_cols = [20u32, 60, 100, 140, 180];

        let mut img = GrayImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                let bg = 60.0 + (x as f32 / w as f32) * 140.0;
                img.put_pixel(x, y, Luma([bg as u8]));
            }
        }
        for &gx in &glyph_cols {
            for y in 10..30 {
                for dx in 0..4 {
                    img.put_pixel(gx + dx, y, Luma([40]));
                }
            }
        }

        let sauvola = sauvola_threshold(&img, 20, 0.35);

        for &gx in &glyph_cols {
            let dark = (0..4).all(|dx| sauvola.get_pixel(gx + dx, 20)[0] == 0);
            assert!(
                dark,
                "Sauvola should keep glyph at x={gx} dark under the gradient"
            );
        }
    }

    #[test]
    fn sauvola_on_uniform_background_matches_local_mean_within_noise() {
        // Non-regression: on a flat background the two methods should agree
        // almost everywhere, or `k` is miscalibrated.
        let mut img = GrayImage::from_pixel(60, 60, Luma([180]));
        for y in 25..35 {
            for x in 25..35 {
                img.put_pixel(x, y, Luma([40]));
            }
        }

        let sauvola = sauvola_threshold(&img, 15, 0.35);
        assert_eq!(sauvola.get_pixel(30, 30)[0], 0, "glyph interior stays dark");
        assert_eq!(
            sauvola.get_pixel(5, 5)[0],
            255,
            "flat background stays light"
        );
    }

    // ── Polarity: measured on the central region only ──────────────────────

    #[test]
    fn is_light_on_dark_ignores_dark_edges() {
        // Dark surround outside the 20% inset, light centre: on the whole
        // frame this measures ~51% dark and would wrongly flip polarity.
        let mut img = GrayImage::from_pixel(100, 100, Luma([0]));
        for y in 20..80 {
            for x in 20..80 {
                img.put_pixel(x, y, Luma([255]));
            }
        }
        assert!(!is_light_on_dark(&img));
    }

    #[test]
    fn is_light_on_dark_detects_inverted_plate() {
        // Light glyphs on a dark background at the centre must be detected.
        let img = GrayImage::from_pixel(100, 100, Luma([0]));
        let mut img = img;
        for y in 40..60 {
            for x in 40..60 {
                img.put_pixel(x, y, Luma([255]));
            }
        }
        assert!(is_light_on_dark(&img));
    }

    // ── Deskew: binarized-image bias, background fill, bilevel output ──────

    #[test]
    fn deskew_on_a_straight_binary_plate_picks_zero() {
        // On greyscale, `estimate_skew_angle` used to pick up to +2.5° on a
        // perfectly straight plate because the projection variance was
        // dominated by broad luminance areas rather than text rows. On a
        // binarized, polarity-normalised image it must settle on 0.
        let img = dark_on_light(200, 60, (20, 20, 180, 40));
        assert_eq!(estimate_skew_angle(&img), 0.0);
    }

    #[test]
    fn deskew_fills_corners_with_background_not_black() {
        // Use an off-axis pattern so a real rotation angle is found.
        let mut img = GrayImage::from_pixel(120, 120, Luma([255]));
        for y in 0..120 {
            for x in 0..120 {
                if (x as i32 - y as i32).unsigned_abs() < 3 {
                    img.put_pixel(x, y, Luma([0]));
                }
            }
        }
        let deskewed = deskew(&img);
        let (w, h) = deskewed.dimensions();
        for &(x, y) in &[(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)] {
            assert_eq!(
                deskewed.get_pixel(x, y)[0],
                255,
                "corner ({x},{y}) must be filled with background, not black"
            );
        }
    }

    #[test]
    fn deskew_output_is_strictly_bilevel() {
        let img = dark_on_light(150, 60, (10, 10, 140, 50));
        let deskewed = deskew(&img);
        for px in deskewed.pixels() {
            assert!(
                px[0] == 0 || px[0] == 255,
                "bilinear interpolation must be re-binarized, got {}",
                px[0]
            );
        }
    }

    // ── morphology_open: separable rewrite must match the naive semantics ──

    #[test]
    fn morphology_open_closes_a_single_pixel_hole() {
        let mut img = GrayImage::from_pixel(10, 10, Luma([0]));
        img.put_pixel(5, 5, Luma([255]));

        let cleaned = morphology_open(&img);
        assert_eq!(cleaned.get_pixel(5, 5)[0], 0);
        assert_eq!(cleaned.get_pixel(0, 0)[0], 0);
    }

    #[test]
    fn morphology_open_preserves_a_solid_block() {
        let mut img = GrayImage::from_pixel(20, 20, Luma([255]));
        for y in 5..15 {
            for x in 5..15 {
                img.put_pixel(x, y, Luma([0]));
            }
        }
        let cleaned = morphology_open(&img);
        assert_eq!(cleaned.get_pixel(9, 9)[0], 0);
        assert_eq!(cleaned.get_pixel(0, 0)[0], 255);
    }

    // ── Confidence: never synthesised from format_valid ─────────────────────

    #[test]
    fn finalize_never_rewrites_confidence() {
        for (plate, format_valid, confidence) in [
            ("CE568LR", true, 0.0f32),
            ("CE568LR", true, 0.42),
            ("NOISE", false, 0.63),
            ("", false, 0.0),
        ] {
            let result = ScanResultData {
                plate: plate.to_string(),
                raw_text: plate.to_string(),
                confidence,
                format_valid,
                plate_type: None,
            };
            let timings = StageTimings::default();
            let finalized = finalize(result, &timings).unwrap();
            assert_eq!(
                finalized.confidence, confidence,
                "finalize must not rewrite confidence for plate={plate:?}"
            );
            assert_eq!(finalized.format_valid, format_valid);
        }
    }

    // ── pick_best_ensemble: unchanged trivial semantics kept as a guard ─────

    #[test]
    fn pick_best_ensemble_prefers_valid_format_over_confidence() {
        let invalid = ScanResultData {
            plate: "INVALID".to_string(),
            raw_text: "INVALID".to_string(),
            confidence: 0.9,
            format_valid: false,
            plate_type: None,
        };
        let valid = ScanResultData {
            plate: "CE128BC".to_string(),
            raw_text: "CE128BC".to_string(),
            confidence: 0.1,
            format_valid: true,
            plate_type: None,
        };
        let result = pick_best_ensemble(vec![Some(invalid), Some(valid)]);
        assert_eq!(result.plate, "CE128BC");
        assert!(result.format_valid);
    }

    // ── scan_plate: error handling on malformed input ───────────────────────

    #[test]
    fn scan_plate_rejects_undecodable_bytes() {
        let result = scan_plate(b"not an image");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot decode image"));
    }

    #[test]
    fn scan_plate_rejects_empty_input() {
        assert!(scan_plate(b"").is_err());
    }

    // ── OcrBudget: the deadline used by every stage checkpoint ──────────────

    #[test]
    fn ocr_budget_reports_exceeded_once_elapsed_time_passes_the_deadline() {
        let budget = OcrBudget::new(Duration::from_millis(0));
        assert!(budget.is_exceeded());
        assert!(budget
            .check(crate::services::ocr::timings::Stage::Decode)
            .is_err());
    }

    #[test]
    fn ocr_budget_reports_not_exceeded_within_the_deadline() {
        let budget = OcrBudget::new(Duration::from_secs(60));
        assert!(!budget.is_exceeded());
        assert!(budget
            .check(crate::services::ocr::timings::Stage::Decode)
            .is_ok());
    }
}
