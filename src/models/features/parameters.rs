use geojson::Bbox;
use serde::{Deserialize, Deserializer, de};
use utoipa::IntoParams;
use utoipa::ToSchema;

/*
 * This is a custom deserializer for the bbox parameter.
 *
 * It allows the bbox to be either a string or a vector of numbers.
 * The bbox is a string in the query parameters, which is the most common way to pass the bbox.
 * The custom deserializer parses comma-separated strings into a vector of numbers and it is used in the `GetItemsParams` struct.
 */
fn deserialize_bbox_option<'de, D>(deserializer: D) -> Result<Option<Bbox>, D::Error>
where
    D: Deserializer<'de>,
{
    /// This allows the bbox to be either a string or a vector of numbers.
    /// This can be useful because the bbox is a string in the query parameters, but a vector of numbers in the request body.
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        String(String),
        Vec(Vec<f64>),
    }

    match Option::<StringOrVec>::deserialize(deserializer)? {
        Some(StringOrVec::String(s)) => {
            let parts: Result<Vec<f64>, _> = s.split(',').map(|p| p.trim().parse()).collect();
            match parts {
                Ok(parts) => {
                    if parts.len() == 4 || parts.len() == 6 {
                        Ok(Some(parts))
                    } else {
                        Err(de::Error::custom("bbox must have 4 or 6 components"))
                    }
                }
                Err(e) => Err(de::Error::custom(format!("invalid bbox format: {}", e))),
            }
        }
        Some(StringOrVec::Vec(v)) => Ok(Some(v)),
        None => Ok(None),
    }
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct GetItemsParams {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_bbox_option")]
    #[param(value_type = Option<String>, example = "5.3,43.2,13.8,51.6")]
    pub bbox: Option<Bbox>,
}
