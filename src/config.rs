use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub title: String,
    pub description: String,
    pub url_base: String,
    #[serde(rename = "collections")]
    pub collections: HashMap<String, CollectionConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CollectionConfig {
    pub table: String,
    pub id_column: String,
    pub geometry_column: String,
    pub properties: Vec<String>,
}
