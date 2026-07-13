use crate::lexicon::PhonemeEntry;
use serde_json;
use std::collections::HashMap;

pub fn load_us_gold() -> HashMap<String, PhonemeEntry> {
    let data = include_str!("../data/us_gold.json");
    serde_json::from_str(data).expect("Failed to parse us_gold.json")
}

pub fn load_us_silver() -> HashMap<String, PhonemeEntry> {
    let data = include_str!("../data/us_silver.json");
    serde_json::from_str(data).expect("Failed to parse us_silver.json")
}

pub fn load_gb_gold() -> HashMap<String, PhonemeEntry> {
    let data = include_str!("../data/gb_gold.json");
    serde_json::from_str(data).expect("Failed to parse gb_gold.json")
}

pub fn load_gb_silver() -> HashMap<String, PhonemeEntry> {
    let data = include_str!("../data/gb_silver.json");
    serde_json::from_str(data).expect("Failed to parse gb_silver.json")
}
