use crate::{
    models::{
        DocFeatureCollectionSchema, DocFeatureSchema, GetItemsParams, Link, LinkRel,
        OgcApiFeatureCollection,
    },
    state::AppState,
    storage::FeaturesWithCount,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};

fn build_ogc_api_feature_collection(
    features_with_count: FeaturesWithCount,
    headers: &HeaderMap,
    collection_id: &str,
    params: &GetItemsParams,
) -> OgcApiFeatureCollection {
    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);

    let host = headers
        .get("host")
        .map(|h| h.to_str().unwrap_or(""))
        .unwrap_or("");
    let scheme = headers
        .get("x-forwarded-proto")
        .map(|h| h.to_str().unwrap_or("http"))
        .unwrap_or("http");
    let base_url = format!("{}://{}/", scheme, host);

    let mut links = vec![Link {
        href: format!("{}collections/{}/items", base_url, collection_id),
        rel: LinkRel::Self_,
        type_: Some("application/geo+json".to_string()),
        title: Some("this document".to_string()),
    }];

    if features_with_count.number_matched > (offset + limit) {
        let next_offset = offset + limit;
        links.push(Link {
            href: format!(
                "{}collections/{}/items?limit={}&offset={}{}",
                base_url,
                collection_id,
                limit,
                next_offset,
                params
                    .bbox
                    .as_ref()
                    .map(|bbox| format!("&bbox={},{},{},{}", bbox[0], bbox[1], bbox[2], bbox[3]))
                    .unwrap_or_default(),
            ),
            rel: LinkRel::Next,
            type_: Some("application/geo+json".to_string()),
            title: Some("next page".to_string()),
        });
    }

    OgcApiFeatureCollection::new(
        features_with_count.features,
        features_with_count.number_matched,
        features_with_count.number_returned,
        links,
        params.bbox.clone(),
    )
}

#[utoipa::path(
    get,
    path = "/collections/{collection_id}/items",
    responses(
        (status = 200, description = "Collection items", body = DocFeatureCollectionSchema)
    )
)]
pub async fn get_collection_items(
    State(state): State<AppState>,
    Path(collection_id): Path<String>,
    Query(params): Query<GetItemsParams>,
    headers: HeaderMap,
) -> Result<Json<OgcApiFeatureCollection>, (StatusCode, String)> {
    let page = state.store.get_features(&collection_id, &params).await?;

    Ok(Json(build_ogc_api_feature_collection(
        page,
        &headers,
        &collection_id,
        &params,
    )))
}

#[utoipa::path(
    get,
    path = "/collections/{collection_id}/items/{id}",
    responses(
        (status = 200, description = "Collection item", body = DocFeatureSchema)
    )
)]
pub async fn get_collection_item(
    State(state): State<AppState>,
    Path((collection_id, id)): Path<(String, String)>,
) -> Result<Json<geojson::Feature>, (StatusCode, String)> {
    let feature = state.store.get_feature(&collection_id, &id).await?;

    Ok(Json(feature))
}
