use crate::{
    handlers::{self, core, features},
    state::AppState,
};
use axum::{Router, routing::get};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api/openapi.json", handlers::ApiDoc::openapi()))
        .route("/", get(core::get_landing_page))
        .route("/conformance", get(core::get_conformance))
        .route("/collections", get(core::get_collections))
        .route("/collections/{collection_id}", get(core::get_collection))
        .route(
            "/collections/{collection_id}/items",
            get(features::get_collection_items),
        )
        .route(
            "/collections/{collection_id}/items/{id}",
            get(features::get_collection_item),
        )
        .with_state(app_state)
}
