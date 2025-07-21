/*
    The purpose of these models is to provide a `ToSchema` implementation for the swagger documentation.
    The schema is not used for the actual data because the current implemntation uses the geojson crate structs.

    This is the schema for the OGC API - Features - Part 1: Core specification.
    https://docs.ogc.org/is/17-069r3/17-069r3.html#_feature_collection_schema
*/
use crate::models::Link;
use serde::Serialize;
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DocFeatureCollectionSchema {
    #[schema(example = "FeatureCollection")]
    pub r#type: String,
    pub features: Vec<DocFeatureSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<Vec<f64>>)]
    pub bbox: Option<Value>,
    pub links: Vec<Link>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_matched: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_returned: Option<u64>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DocFeatureSchema {
    #[schema(example = "Feature")]
    pub r#type: String,
    #[schema(value_type = Object)]
    pub geometry: Value,
    #[schema(value_type = Object, nullable = true)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}
