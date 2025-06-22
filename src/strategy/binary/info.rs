use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct BinaryInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub license: String,
}

impl Default for BinaryInfo {
    fn default() -> Self {
        Self {
            name: "Binary Options Strategy".to_string(),
            description: "A strategy for trading binary options.".to_string(),
            version: "1.0.0".to_string(),
            author: "None".to_string(),
            license: "MIT".to_string(),
        }
    }
}