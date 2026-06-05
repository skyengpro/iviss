use std::collections::HashMap;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct FeatureFlags {
    pub enable_new_ocr_engine: bool,
    pub enable_advanced_analytics: bool,
    pub maintenance_mode: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            enable_new_ocr_engine: false,
            enable_advanced_analytics: false,
            maintenance_mode: false,
        }
    }
}

impl FeatureFlags {
    pub fn from_env() -> Self {
        Self {
            enable_new_ocr_engine: std::env::var("FF_NEW_OCR_ENGINE")
                .map(|v| v == "true")
                .unwrap_or(false),
            enable_advanced_analytics: std::env::var("FF_ADVANCED_ANALYTICS")
                .map(|v| v == "true")
                .unwrap_or(false),
            maintenance_mode: std::env::var("FF_MAINTENANCE_MODE")
                .map(|v| v == "true")
                .unwrap_or(false),
        }
    }

    pub fn to_hashmap(&self) -> HashMap<String, bool> {
        let mut map = HashMap::new();
        map.insert("new_ocr_engine".to_string(), self.enable_new_ocr_engine);
        map.insert("advanced_analytics".to_string(), self.enable_advanced_analytics);
        map.insert("maintenance_mode".to_string(), self.maintenance_mode);
        map
    }
}
