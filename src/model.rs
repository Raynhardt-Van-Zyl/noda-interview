use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct RawRecord {
    pub id: String,
    pub timestamp: String,
    pub value: f64,
    pub tag: String,
}
