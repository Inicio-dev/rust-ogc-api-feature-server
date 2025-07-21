use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct Conformance {
    pub conforms_to: Vec<String>,
}
