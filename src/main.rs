use std::env;

use axum::{Router, routing::{get_service, post}};
use sqlx::{SqlitePool, types::chrono::DateTime};
use tower_http::services::ServeDir;

async fn add_location(pool: &SqlitePool, location: String) -> Result<i64, sqlx::Error> {
    let mut conn = pool.acquire().await?;

    let time = DateTime::from_timestamp_secs(1000);
    let id = sqlx::query!(
        r#"
INSERT INTO locations ( latitude, longitude, timestamp )
VALUES ( ?1, ?2, ?3 )
        "#,
        location,
        location,
        time,
    )
    .execute(&mut *conn)
    .await?
    .last_insert_rowid();

    Ok(id)
}

#[tokio::main]
async fn main() {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL").unwrap()).await.unwrap();

    let app = Router::new()
        .route("/", post(|| add_location(&pool, "test".to_string())))
        .fallback_service(get_service(ServeDir::new("./web")));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
