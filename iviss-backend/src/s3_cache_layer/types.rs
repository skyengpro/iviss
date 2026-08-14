use crate::dto::search_vehicle::VehicleInfo;
use crate::utils::plate_format::{self, PlateCategory};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub const S3_CACHE_PREFIX: &str = "vehicle-cache/";
pub const RETRY_QUEUE_PREFIX: &str = "retry-queue/";
pub const UNREGISTERED_PREFIX: &str = "unregistered/";
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QueueMarker {
    pub plate_number: String,
    /// RFC3339 timestamp.
    pub queued_at: String,
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

fn ensure_valid_plate(plate: &str) -> Result<()> {
    if plate.is_empty() || !plate.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(anyhow!("plate key contains invalid characters"));
    }
    Ok(())
}

pub fn object_key(plate: &str) -> Result<String> {
    ensure_valid_plate(plate)?;
    let partition = cache_partition_for_plate(plate);
    Ok(format!("{}{partition}/{plate}.json", S3_CACHE_PREFIX))
}

pub fn retry_queue_key(org_id: uuid::Uuid, plate: &str) -> Result<String> {
    ensure_valid_plate(plate)?;
    Ok(format!("{}{org_id}/{plate}.json", RETRY_QUEUE_PREFIX))
}

pub fn unregistered_key(org_id: uuid::Uuid, plate: &str) -> Result<String> {
    ensure_valid_plate(plate)?;
    Ok(format!("{}{org_id}/{plate}.json", UNREGISTERED_PREFIX))
}

pub fn org_prefix(prefix: &str, org_id: uuid::Uuid) -> String {
    format!("{prefix}{org_id}/")
}

/// Parses `{prefix}{uuid}/{PLATE}.json`. Returns `None` on any mismatch:
/// missing UUID segment, non-UUID segment, extra path components, or wrong prefix.
pub fn org_plate_from_key(key: &str, prefix: &str) -> Option<(uuid::Uuid, String)> {
    let rest = key.strip_prefix(prefix)?;
    let (uuid_str, rest) = rest.split_once('/')?;
    let org_id = uuid_str.parse::<uuid::Uuid>().ok()?;
    if rest.contains('/') {
        return None;
    }
    let plate = rest.strip_suffix(".json")?;
    if plate.is_empty() {
        return None;
    }
    Some((org_id, plate.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn cache_partition_routes_regional_plates_by_region_code() {
        assert_eq!(cache_partition_for_plate("LT893DK"), "LT");
        assert_eq!(cache_partition_for_plate("CE128BC"), "CE");
        assert_eq!(cache_partition_for_plate("NW777AB"), "NW");
        assert_eq!(cache_partition_for_plate("LTSR9652A"), "LT");
    }

    #[test]
    fn cache_partition_routes_special_formats_to_others() {
        assert_eq!(cache_partition_for_plate("CA1234A"), "others");
        assert_eq!(cache_partition_for_plate("SN1234"), "others");
        assert_eq!(cache_partition_for_plate("CMD02RC521"), "others");
    }

    #[test]
    fn retry_queue_key_round_trips() {
        let org = Uuid::new_v4();
        let key = retry_queue_key(org, "LT893DK").unwrap();
        let (parsed_org, parsed_plate) = org_plate_from_key(&key, RETRY_QUEUE_PREFIX).unwrap();
        assert_eq!(parsed_org, org);
        assert_eq!(parsed_plate, "LT893DK");
    }

    #[test]
    fn unregistered_key_round_trips() {
        let org = Uuid::new_v4();
        let key = unregistered_key(org, "CE128BC").unwrap();
        let (parsed_org, parsed_plate) = org_plate_from_key(&key, UNREGISTERED_PREFIX).unwrap();
        assert_eq!(parsed_org, org);
        assert_eq!(parsed_plate, "CE128BC");
    }

    #[test]
    fn org_plate_from_key_rejects_non_uuid_segment() {
        assert!(
            org_plate_from_key("unregistered/not-a-uuid/LT893DK.json", UNREGISTERED_PREFIX)
                .is_none()
        );
    }

    #[test]
    fn org_plate_from_key_rejects_traversal_segment() {
        let org = Uuid::new_v4();
        let key = format!("unregistered/{org}/../../etc/passwd.json");
        assert!(org_plate_from_key(&key, UNREGISTERED_PREFIX).is_none());
    }

    #[test]
    fn org_plate_from_key_rejects_flat_key_without_uuid() {
        assert!(org_plate_from_key("unregistered/LT893DK.json", UNREGISTERED_PREFIX).is_none());
    }

    #[test]
    fn retry_queue_key_rejects_invalid_plate() {
        let org = Uuid::new_v4();
        assert!(retry_queue_key(org, "").is_err());
        assert!(retry_queue_key(org, "LT/893DK").is_err());
    }
}
