use crate::models::GetItemsParams;
use async_trait::async_trait;
use axum::http::StatusCode;

pub struct FeaturesWithCount {
    pub features: Vec<geojson::Feature>,
    pub number_matched: u64,
    pub number_returned: u64,
}

impl FeaturesWithCount {
    pub fn new(features: Vec<geojson::Feature>, number_matched: u64, number_returned: u64) -> Self {
        Self {
            features,
            number_matched,
            number_returned,
        }
    }
}

#[async_trait]
pub trait Storage: Send + Sync {
    async fn get_features(
        &self,
        collection_id: &str,
        params: &GetItemsParams,
    ) -> Result<FeaturesWithCount, (StatusCode, String)>;

    async fn get_feature(&self, collection_id: &str, id: &str) -> Result<geojson::Feature, (StatusCode, String)>;
}
