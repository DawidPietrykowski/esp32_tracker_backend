use dotenv::dotenv;
use std::env;

use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::StatusCode,
    routing::{get, get_service, post},
};
use sqlx::{
    SqlitePool,
    types::chrono::{DateTime, Local, Utc},
};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

use chrono::serde::ts_seconds;
use serde::{Deserialize, Serialize};
use wincode::{SchemaRead, SchemaWrite};

#[derive(SchemaWrite, SchemaRead, Serialize, Debug)]
pub struct LocationFrame {
    latitude: f64,
    longitude: f64,
    signal: f64,
    battery: f64,
    timestamp: u64,
}

async fn add_location(
    State(pool): State<SqlitePool>,
    body: Bytes,
) -> Result<String, (StatusCode, String)> {
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let frame: LocationFrame = wincode::deserialize(&body).unwrap();

    let time = Local::now().to_utc();
    let generated_time = DateTime::from_timestamp(frame.timestamp as i64, 0).ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to decode frame".to_string(),
        )
    })?;

    let id = sqlx::query!(
        r#"
INSERT INTO locations ( latitude, longitude, signal, battery, generated, received )
VALUES ( ?1, ?2, ?3, ?4, ?5, ?6 )
        "#,
        frame.latitude,
        frame.longitude,
        frame.signal,
        frame.battery,
        generated_time,
        time,
    )
    .execute(&mut *conn)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .last_insert_rowid();

    Ok(id.to_string())
}

#[derive(Debug)]
#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
struct LocationEntry {
    latitude: f64,
    longitude: f64,
    signal: f64,
    battery: f64,
    #[serde(with = "ts_seconds")]
    generated: DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    received: DateTime<Utc>,
}

async fn get_location(State(pool): State<SqlitePool>) -> Result<String, (StatusCode, String)> {
    let location = sqlx::query_as!(
        LocationEntry,
        r#"
    SELECT 
        latitude,
        longitude,
        signal,
        battery,
        generated as "generated: _", 
        received as "received: _"
    FROM locations 
    ORDER BY id DESC
    "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    serde_json::to_string(&location).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[tokio::main]
async fn main() {
    // Load .env file
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let pool = SqlitePool::connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let app = Router::new()
        .route("/pos", post(add_location))
        .route("/pos", get(get_location))
        .fallback_service(get_service(ServeDir::new("./web")))
        .layer(TraceLayer::new_for_http())
        .with_state(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Starting server");
    println!("Listening on 0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}
