use crate::models::common::link::Link;
use geojson::{Bbox, Feature};
use serde::Serialize;

const OGC_API_FEATURE_COLLECTION_TYPE: &str = "FeatureCollection";

#[derive(Serialize, Debug)]
pub struct OgcApiFeatureCollection {
    #[serde(rename = "type")]
    type_: &'static str,
    bbox: Option<Bbox>,
    features: Vec<Feature>,
    #[serde(rename = "numberMatched")]
    number_matched: u64,
    #[serde(rename = "numberReturned")]
    number_returned: u64,
    links: Vec<Link>,
}

impl OgcApiFeatureCollection {
    pub fn new(
        features: Vec<Feature>,
        number_matched: u64,
        number_returned: u64,
        links: Vec<Link>,
        bbox: Option<Bbox>,
    ) -> OgcApiFeatureCollection {
        OgcApiFeatureCollection {
            type_: OGC_API_FEATURE_COLLECTION_TYPE,
            features,
            number_matched,
            number_returned,
            links,
            bbox,
        }
    }
}
