use serde::Serialize;
use utoipa::ToSchema;

use crate::models::Link;

#[derive(Serialize, ToSchema)]
pub struct LandingPage {
    pub title: String,
    pub description: String,
    pub links: Vec<Link>,
}
