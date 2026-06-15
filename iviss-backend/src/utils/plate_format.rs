use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;

pub const REGION_CODES: &[&str] = &[
    "AD", "CE", "EN", "ES", "LT", "NO", "NW", "OU", "SU", "SW", "SO",
];

const MIN_PLATE_LEN: usize = 6;
const MAX_PLATE_LEN: usize = 12;

static CIVIL_CEMAC_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:AD|CE|EN|ES|LT|NO|NW|OU|SU|SW|SO)\d{3}[A-Z]{2}$").unwrap());
static BIKE_CEMAC_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:AD|CE|EN|ES|LT|NO|NW|OU|SU|SW|SO)MT\d{3}[A-Z]{2}$").unwrap());
static CIVIL_LEGACY_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:AD|CE|EN|ES|LT|NO|NW|OU|SU|SW|SO)\d{4}[A-Z]{1,2}$").unwrap());
static TRAILER_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:AD|CE|EN|ES|LT|NO|NW|OU|SU|SW|SO)(?:RE|SR|SE|TR)\d{1,4}[A-Z]{1,2}$").unwrap()
});
static STATE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(?:CA|AN)\d{4}[A-Z]{1,2}$").unwrap());
static DIPLOMATIC_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:(?:CMD|CPC|CD|CC|PA)\d{2,3}RC\d{1,4}|CD\d{1,6})$").unwrap());
static TEMPORARY_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^IT\d{5}RC$").unwrap());
static TEST_VEHICLE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:AD|CE|EN|ES|LT|NO|NW|OU|SU|SW|SO)\d{4}WG$").unwrap());
static TRANSIT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^WT\d{6,7}$").unwrap());
static POSTAL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^PT\d{5}$").unwrap());
static SPECIAL_INVESTMENT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^IS\d{5,6}RC$").unwrap());
static NATIONAL_SECURITY_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^SN\d{4}$").unwrap());
static MILITARY_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d{7}$").unwrap());
static POSTAL_TELECOM_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^RT\d{6}$").unwrap());
static GOVERNMENT_LEGACY_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Z]{2}\d{4}[A-Z]$").unwrap());

static FUZZY_MASKS: Lazy<Vec<String>> = Lazy::new(|| {
    let mut masks = vec![
        "??###??".to_string(),
        "??####?".to_string(),
        "??####??".to_string(),
        "????####?".to_string(),
        "????####??".to_string(),
        "IT#####RC".to_string(),
        "??####WG".to_string(),
        "WT######".to_string(),
        "WT#######".to_string(),
        "PT#####".to_string(),
        "IS#####RC".to_string(),
        "IS######RC".to_string(),
        "SN####".to_string(),
        "#######".to_string(),
        "RT######".to_string(),
    ];

    for prefix in ["CMD", "CPC", "CD", "CC", "PA"] {
        for country_digits in 2..=3 {
            for serial_digits in 1..=4 {
                masks.push(format!(
                    "{}{}RC{}",
                    prefix,
                    "#".repeat(country_digits),
                    "#".repeat(serial_digits)
                ));
            }
        }
    }

    masks.sort_by_key(|b| std::cmp::Reverse(b.len()));
    masks
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlateCategory {
    CivilCemac,
    CivilLegacy,
    Trailer,
    BikeCemac,
    State,
    Diplomatic,
    Temporary,
    TestVehicle,
    Transit,
    Postal,
    SpecialInvestment,
    NationalSecurity,
    Military,
    PostalTelecom,
    GovernmentLegacy,
}

impl PlateCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CivilCemac => "civil_cemac",
            Self::CivilLegacy => "civil_legacy",
            Self::Trailer => "trailer",
            Self::BikeCemac => "bike_cemac",
            Self::State => "state",
            Self::Diplomatic => "diplomatic",
            Self::Temporary => "temporary",
            Self::TestVehicle => "test_vehicle",
            Self::Transit => "transit",
            Self::Postal => "postal",
            Self::SpecialInvestment => "special_investment",
            Self::NationalSecurity => "national_security",
            Self::Military => "military",
            Self::PostalTelecom => "postal_telecom",
            Self::GovernmentLegacy => "government_legacy",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlateMatch {
    pub plate: String,
    pub category: PlateCategory,
}

pub fn normalise(raw: &str) -> String {
    raw.to_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

pub fn classify(raw: &str) -> Option<PlateMatch> {
    let compact = normalise(raw);
    let category = classify_compact(&compact)?;
    Some(PlateMatch {
        plate: compact,
        category,
    })
}

pub fn is_valid(raw: &str) -> bool {
    classify(raw).is_some()
}

pub fn extract_first(raw: &str) -> Option<PlateMatch> {
    let compact = normalise(raw);
    if compact.is_empty() {
        return None;
    }

    if let Some(found) = classify(&compact) {
        return Some(found);
    }

    find_candidate(&compact)
}

pub fn fuzzy_correct(raw: &str) -> Option<PlateMatch> {
    if let Some(found) = extract_first(raw) {
        return Some(found);
    }

    let compact = normalise(raw);
    if compact.len() < MIN_PLATE_LEN {
        return None;
    }

    for start in 0..compact.len() {
        for mask in FUZZY_MASKS.iter() {
            let end = start + mask.len();
            if end > compact.len() {
                continue;
            }

            let candidate = &compact[start..end];
            if let Some(corrected) = correct_with_mask(candidate, mask) {
                if let Some(found) = classify(&corrected) {
                    return Some(found);
                }
            }
        }
    }

    None
}

pub fn format_display(raw: &str) -> String {
    let compact = normalise(raw);
    let Some(category) = classify_compact(&compact) else {
        return compact;
    };

    match category {
        PlateCategory::CivilCemac => {
            format!("{} {} {}", &compact[0..2], &compact[2..5], &compact[5..7])
        }
        PlateCategory::BikeCemac => {
            format!(
                "{} {} {} {}",
                &compact[0..2],
                &compact[2..4],
                &compact[4..7],
                &compact[7..9]
            )
        }
        PlateCategory::CivilLegacy => {
            format!("{} {} {}", &compact[0..2], &compact[2..6], &compact[6..])
        }
        PlateCategory::Trailer => {
            // Format: region(2) + type(2) + digits(1-4) + letters(1-2)
            let region = &compact[0..2];
            let trailer_type = &compact[2..4];
            // Find where letters start (after digits)
            let digit_end = compact[4..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .count()
                + 4;
            let digits = &compact[4..digit_end];
            let letters = &compact[digit_end..];
            format!("{} {} {} {}", region, trailer_type, digits, letters)
        }
        PlateCategory::State => format!("{} {} {}", &compact[0..2], &compact[2..6], &compact[6..]),
        PlateCategory::Diplomatic if compact.contains("RC") => {
            let rc = compact.find("RC").expect("contains checked above");
            format!(
                "{} {} RC {}",
                &compact[..prefix_len(&compact)],
                &compact[prefix_len(&compact)..rc],
                &compact[rc + 2..]
            )
        }
        PlateCategory::Diplomatic => format!("{} {}", &compact[0..2], &compact[2..]),
        PlateCategory::Temporary => format!("IT {} RC", &compact[2..7]),
        PlateCategory::TestVehicle => format!("{} {} WG", &compact[0..2], &compact[2..6]),
        PlateCategory::Transit => format!("WT {}", &compact[2..]),
        PlateCategory::Postal => format!("PT {}", &compact[2..]),
        PlateCategory::SpecialInvestment => format!("IS {} RC", &compact[2..compact.len() - 2]),
        PlateCategory::NationalSecurity => format!("SN {}", &compact[2..]),
        PlateCategory::Military => compact,
        PlateCategory::PostalTelecom => format!("RT {}", &compact[2..]),
        PlateCategory::GovernmentLegacy => {
            format!("{} {} {}", &compact[0..2], &compact[2..6], &compact[6..])
        }
    }
}

fn classify_compact(compact: &str) -> Option<PlateCategory> {
    let checks: &[(PlateCategory, &Lazy<Regex>)] = &[
        (PlateCategory::Trailer, &TRAILER_RE),
        (PlateCategory::TestVehicle, &TEST_VEHICLE_RE),
        (PlateCategory::CivilCemac, &CIVIL_CEMAC_RE),
        (PlateCategory::BikeCemac, &BIKE_CEMAC_RE),
        (PlateCategory::CivilLegacy, &CIVIL_LEGACY_RE),
        (PlateCategory::State, &STATE_RE),
        (PlateCategory::Diplomatic, &DIPLOMATIC_RE),
        (PlateCategory::Temporary, &TEMPORARY_RE),
        (PlateCategory::Transit, &TRANSIT_RE),
        (PlateCategory::Postal, &POSTAL_RE),
        (PlateCategory::SpecialInvestment, &SPECIAL_INVESTMENT_RE),
        (PlateCategory::NationalSecurity, &NATIONAL_SECURITY_RE),
        (PlateCategory::Military, &MILITARY_RE),
        (PlateCategory::PostalTelecom, &POSTAL_TELECOM_RE),
        (PlateCategory::GovernmentLegacy, &GOVERNMENT_LEGACY_RE),
    ];

    checks
        .iter()
        .find_map(|(category, regex)| regex.is_match(compact).then_some(*category))
}

fn find_candidate(compact: &str) -> Option<PlateMatch> {
    for len in (MIN_PLATE_LEN..=MAX_PLATE_LEN).rev() {
        if compact.len() < len {
            continue;
        }

        for start in 0..=(compact.len() - len) {
            let candidate = &compact[start..start + len];
            if let Some(found) = classify(candidate) {
                return Some(found);
            }
        }
    }

    None
}

fn correct_with_mask(candidate: &str, mask: &str) -> Option<String> {
    if candidate.len() != mask.len() {
        return None;
    }

    candidate
        .chars()
        .zip(mask.chars())
        .map(|(actual, expected)| match expected {
            '?' => Some(correct_letter(actual)),
            '#' => correct_digit(actual),
            literal => {
                let corrected = correct_letter(actual);
                (corrected == literal).then_some(literal)
            }
        })
        .collect()
}

fn correct_letter(c: char) -> char {
    match c {
        '0' => 'O',
        '1' => 'I',
        '2' => 'Z',
        '5' => 'S',
        '6' => 'G',
        '8' => 'B',
        _ => c,
    }
}

fn correct_digit(c: char) -> Option<char> {
    match c {
        '0'..='9' => Some(c),
        'O' | 'Q' => Some('0'),
        'I' | 'L' | 'T' => Some('1'),
        'Z' => Some('2'),
        'A' => Some('4'),
        'S' => Some('5'),
        'G' => Some('6'),
        'B' => Some('8'),
        _ => None,
    }
}

fn prefix_len(compact: &str) -> usize {
    ["CMD", "CPC", "CD", "CC", "PA"]
        .iter()
        .find_map(|prefix| compact.starts_with(prefix).then_some(prefix.len()))
        .unwrap_or(2)
}
