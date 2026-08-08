use crate::dto::search_vehicle::VehicleInfo;
use crate::utils::plate_format::{self, PlateCategory};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub const S3_CACHE_PREFIX: &str = "vehicle-cache/";
pub const OTHER_CACHE_PARTITION: &str = "others";
pub const PLATE_PREFIX_CODES: &[&str] = &[
    "AD", "CE", "EN", "ES", "LT", "NO", "NW", "OU", "SU", "SW", "SO", "CMD", "CPC", "CD", "CC",
    "PA", "RT", "IS", "SN", "IT",
];

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CachedEntry {
    pub plate_number: String,
    pub vehicle: VehicleInfo,
    pub cached_at: String,
}

#[derive(Clone, Debug)]
pub struct CachedVehicleData {
    pub plate_number: String,
    pub vehicle: VehicleInfo,
    pub cached_at: time::OffsetDateTime,
}

pub fn cache_partition_for_plate(plate: &str) -> &str {
    let Some(found) = plate_format::classify(plate) else {
        return OTHER_CACHE_PARTITION;
    };

    match found.category {
        PlateCategory::CivilCemac
        | PlateCategory::CivilLegacy
        | PlateCategory::Trailer
        | PlateCategory::BikeCemac
        | PlateCategory::TestVehicle => plate
            .get(..2)
            .filter(|region| PLATE_PREFIX_CODES.contains(region))
            .unwrap_or(OTHER_CACHE_PARTITION),
        PlateCategory::State
        | PlateCategory::Diplomatic
        | PlateCategory::Temporary
        | PlateCategory::Transit
        | PlateCategory::Postal
        | PlateCategory::SpecialInvestment
        | PlateCategory::NationalSecurity
        | PlateCategory::Military
        | PlateCategory::PostalTelecom
        | PlateCategory::GovernmentLegacy => OTHER_CACHE_PARTITION,
    }
}

pub fn object_key(plate: &str) -> Result<String> {
    if !plate.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(anyhow!(
            "vehicle cache plate key contains invalid characters"
        ));
    }

    let partition = cache_partition_for_plate(plate);
    Ok(format!("{}{partition}/{plate}.json", S3_CACHE_PREFIX))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_partition_routes_regional_plates_by_region_code() {
        assert_eq!(cache_partition_for_plate("LT893DK"), "LT");
        assert_eq!(cache_partition_for_plate("CE128BC"), "CE");
        assert_eq!(cache_partition_for_plate("NW777AB"), "NW");
        assert_eq!(cache_partition_for_plate("LTSR9652A"), "LT");
    }

    #[test]
    fn cache_partition_routes_special_formats_to_others() {
        // State plates (CA / AN prefix)
        assert_eq!(cache_partition_for_plate("CA1234A"), "others");
        // NationalSecurity
        assert_eq!(cache_partition_for_plate("SN1234"), "others");
        // Diplomatic
        assert_eq!(cache_partition_for_plate("CMD02RC521"), "others");
    }
}
