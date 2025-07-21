use serde::Serialize;
use utoipa::ToSchema;

use crate::models::Link;

#[derive(Serialize, ToSchema)]
pub struct Collections {
    pub collections: Vec<Collection>,
}

#[derive(Serialize, ToSchema)]
pub struct Collection {
    pub id: String,
    pub title: String,
    pub description: String,
    pub links: Vec<Link>,
}
