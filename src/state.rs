use crate::{config::AppConfig, storage::Storage};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<dyn Storage>,
    pub config: Arc<AppConfig>,
}
