use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum LinkRel {
    #[serde(rename = "self")]
    Self_,
    Next,
    Alternate,
    Collection,
    Items,
    ServiceDesc,
    ServiceDoc,
    Conformance,
    Data,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct Link {
    pub href: String,
    pub rel: LinkRel,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}
