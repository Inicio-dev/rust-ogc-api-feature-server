mod config;
mod handlers;
mod models;
mod routes;
mod state;
mod storage;

use crate::{state::AppState, storage::drivers::Postgis};

use clap::Parser;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    let config_str =
        std::fs::read_to_string(&args.config).expect("Failed to read configuration file");
    let config: config::AppConfig =
        toml::from_str(&config_str).expect("Failed to parse configuration");
    let config = Arc::new(config);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(
            &std::env::var("DATABASE_URL").expect("`DATABASE_URL` must be set in the environment"),
        )
        .await
        .expect("Failed to connect to the database");

    let store = Arc::new(Postgis::new(pool, Arc::clone(&config)));

    let app_state = AppState { store, config };

    let app = routes::create_router(app_state);

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}
