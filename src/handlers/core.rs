use crate::{
    models::{Collection, Collections, Conformance, LandingPage, Link, LinkRel},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

fn build_collection(url_base: &str, id: &str) -> Collection {
    let collection_url = format!("{}/collections/{}", url_base, id);
    Collection {
        id: id.to_string(),
        title: id.to_string(),
        description: format!("Collection of {}", id),
        links: vec![
            Link {
                href: collection_url.clone(),
                rel: LinkRel::Self_,
                type_: Some("application/json".to_string()),
                title: Some("this document".to_string()),
            },
            Link {
                href: format!("{}/items", collection_url),
                rel: LinkRel::Items,
                type_: Some("application/geo+json".to_string()),
                title: Some("Items".to_string()),
            },
        ],
    }
}

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Landing page", body = LandingPage)
    )
)]
pub async fn get_landing_page(State(state): State<AppState>) -> Json<LandingPage> {
    let url_base = &state.config.url_base;
    Json(LandingPage {
        title: state.config.title.clone(),
        description: state.config.description.clone(),
        links: vec![
            Link {
                href: format!("{}/", url_base),
                rel: LinkRel::Self_,
                type_: Some("application/json".to_string()),
                title: Some("this document".to_string()),
            },
            Link {
                href: format!("{}/api/openapi.json", url_base),
                rel: LinkRel::ServiceDesc,
                type_: Some("application/vnd.oai.openapi+json;version=3.0".to_string()),
                title: Some("the API definition".to_string()),
            },
            Link {
                href: format!("{}/api.html", url_base),
                rel: LinkRel::ServiceDoc,
                type_: Some("text/html".to_string()),
                title: Some("the API documentation".to_string()),
            },
            Link {
                href: format!("{}/conformance", url_base),
                rel: LinkRel::Conformance,
                type_: Some("application/json".to_string()),
                title: Some("OGC API conformance classes implemented by this server".to_string()),
            },
            Link {
                href: format!("{}/collections", url_base),
                rel: LinkRel::Data,
                type_: Some("application/json".to_string()),
                title: Some("Information about the feature collections".to_string()),
            },
        ],
    })
}

#[utoipa::path(
    get,
    path = "/conformance",
    responses(
        (status = 200, description = "Conformance page", body = Conformance)
    )
)]
pub async fn get_conformance() -> Json<Conformance> {
    Json(Conformance {
        conforms_to: vec![
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core".to_string(),
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30".to_string(),
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson".to_string(),
        ],
    })
}

#[utoipa::path(
    get,
    path = "/collections",
    responses(
        (status = 200, description = "List of collections", body = Collections)
    )
)]
pub async fn get_collections(State(state): State<AppState>) -> Json<Collections> {
    let url_base = &state.config.url_base;
    let collections = state
        .config
        .collections
        .keys()
        .map(|id| build_collection(url_base, id))
        .collect();

    Json(Collections { collections })
}

#[utoipa::path(
    get,
    path = "/collections/{collection_id}",
    params(
        ("collection_id" = String, Path, description = "ID of the collection")
    ),
    responses(
        (status = 200, description = "Collection details", body = Collection),
        (status = 404, description = "Collection not found")
    )
)]
pub async fn get_collection(
    State(state): State<AppState>,
    Path(collection_id): Path<String>,
) -> Result<Json<Collection>, (StatusCode, String)> {
    let url_base = &state.config.url_base;
    if state.config.collections.contains_key(&collection_id) {
        let collection = build_collection(url_base, &collection_id);
        Ok(Json(collection))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            format!("Collection {} not found", collection_id),
        ))
    }
}
