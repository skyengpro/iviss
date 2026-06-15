use crate::utils::plate_format::{self, PlateCategory};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_supported_pdf_formats() {
        let cases = [
            ("CE128BC", PlateCategory::CivilCemac, "civil_cemac"),
            ("LT4568A", PlateCategory::CivilLegacy, "civil_legacy"),
            ("LTSR9652A", PlateCategory::Trailer, "trailer"),
            ("AN9652E", PlateCategory::State, "state"),
            ("PA02RC521", PlateCategory::Diplomatic, "diplomatic"),
            ("IT21052RC", PlateCategory::Temporary, "temporary"),
            ("CE2456WG", PlateCategory::TestVehicle, "test_vehicle"),
            ("WT1202082", PlateCategory::Transit, "transit"),
            ("PT01200", PlateCategory::Postal, "postal"),
            (
                "IS245642RC",
                PlateCategory::SpecialInvestment,
                "special_investment",
            ),
        ];

        for (plate, category, category_name) in cases {
            let found = plate_format::classify(plate).expect("plate should classify");
            assert_eq!(found.plate, plate);
            assert_eq!(found.category, category);
            assert_eq!(found.category.as_str(), category_name);
        }
    }

    #[test]
    fn preserves_existing_iviss_formats() {
        let cases = [
            ("SN1234", PlateCategory::NationalSecurity),
            ("1234567", PlateCategory::Military),
            ("RT123456", PlateCategory::PostalTelecom),
            ("AB1234X", PlateCategory::GovernmentLegacy),
        ];

        for (plate, category) in cases {
            let found = plate_format::classify(plate).expect("legacy IVISS plate should classify");
            assert_eq!(found.category, category);
        }
    }

    #[test]
    fn accepts_sw_and_so_region_codes() {
        assert_eq!(
            plate_format::classify("SW128BC").map(|m| m.category),
            Some(PlateCategory::CivilCemac)
        );
        assert_eq!(
            plate_format::classify("SO128BC").map(|m| m.category),
            Some(PlateCategory::CivilCemac)
        );
    }

    #[test]
    fn extracts_first_plate_from_noisy_ocr_text() {
        let found = plate_format::extract_first("OCR result: pa 02 rc 521 / ok")
            .expect("plate should be extracted");

        assert_eq!(found.plate, "PA02RC521");
        assert_eq!(found.category, PlateCategory::Diplomatic);
    }

    #[test]
    fn fuzzy_corrects_common_ocr_confusions() {
        let civil = plate_format::fuzzy_correct("5W1O8BC").expect("civil plate should correct");
        assert_eq!(civil.plate, "SW108BC");
        assert_eq!(civil.category, PlateCategory::CivilCemac);

        let temporary =
            plate_format::fuzzy_correct("1T21O52RC").expect("temporary plate should correct");
        assert_eq!(temporary.plate, "IT21052RC");
        assert_eq!(temporary.category, PlateCategory::Temporary);
    }

    #[test]
    fn rejects_empty_and_partial_values() {
        assert!(plate_format::classify("").is_none());
        assert!(plate_format::classify("CE12").is_none());
        assert!(plate_format::extract_first("nothing useful").is_none());
    }

    #[test]
    fn formats_display_values_without_changing_compact_validation() {
        assert_eq!(plate_format::format_display("CE128BC"), "CE 128 BC");
        assert_eq!(plate_format::format_display("LTSR9652A"), "LT SR 9652 A");
        assert_eq!(plate_format::format_display("SN1234"), "SN 1234");
    }
}
