pub mod core;
pub mod features;

pub use crate::models::{
    Collection, Collections, Conformance, DocFeatureCollectionSchema, DocFeatureSchema, GetItemsParams,
    LandingPage, Link,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        core::get_landing_page,
        core::get_conformance,
        core::get_collections,
        core::get_collection,
        features::get_collection_items
    ),
    components(schemas(
        LandingPage,
        Conformance,
        Collections,
        Collection,
        Link,
        GetItemsParams,
        DocFeatureCollectionSchema,
        DocFeatureSchema
    ))
)]
pub struct ApiDoc;
