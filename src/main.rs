use std::env;

use axum::{
    Router,
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

async fn add_location(
    State(pool): State<SqlitePool>,
    location: String,
) -> Result<String, (StatusCode, String)> {
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let time = Local::now().to_utc();

    let id = sqlx::query!(
        r#"
INSERT INTO locations ( latitude, longitude, generated, received )
VALUES ( ?1, ?2, ?3, ?4 )
        "#,
        location,
        location,
        time,
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
struct LocationEntry {
    latitude: f64,
    longitude: f64,
    generated: DateTime<Utc>,
    received: DateTime<Utc>,
}

async fn get_location(State(pool): State<SqlitePool>) -> Result<String, (StatusCode, String)> {
    let location = sqlx::query_as!(
        LocationEntry,
        r#"
    SELECT 
        latitude, 
        longitude, 
        generated as "generated: _", 
        received as "received: _"
    FROM locations 
    ORDER BY id DESC
    "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(format!("{:?}", location))
}

#[tokio::main]
async fn main() {
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
