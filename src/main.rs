use axum::{
    routing::{get, post},
    Router,
    response::Redirect,
    http::StatusCode,
    extract::{Extension, Path},
    
};


use serde::{Deserialize, Serialize};

use sqlx::{FromRow, PgPool};
use url::Url;
use sqlx::postgres::PgPoolOptions;

use std::net::SocketAddr;
use anyhow::Context;

#[derive(Deserialize,Serialize, FromRow)]
struct StoredURL {
    pub id: String,
    pub url: String,
}


async fn redirect(Path(id): Path<String>, Extension(pool): Extension<PgPool>) -> Result<Redirect, (StatusCode, String)> {
    let stored_url: StoredURL = sqlx::query_as("SELECT * FROM url WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "Id Not Found".to_string()),
            _=> (StatusCode::INTERNAL_SERVER_ERROR, "Somenthing went wrong".to_string())

        })?;

        Ok(Redirect::to(&stored_url.url))
}

async fn shorten(url: String, Extension(pool): Extension<PgPool>) -> Result<String, StatusCode> {
    let id = &nanoid::nanoid!(6);

    let parserd_url = Url::parse(&url).map_err(|_err| {
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    sqlx::query("INSERT INTO url(id, url) VALUES ($1, $2)")
        .bind(id)
        .bind(parserd_url.as_str())
        .execute(&pool)
        .await
        .map_err(|_| {
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        Ok(format!("http://localhost:3000/{id}"))
}



#[tokio::main]
async fn main() -> anyhow::Result<(), anyhow::Error>{

    tracing_subscriber::fmt::init();
    
    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect("DATABASE_URL")
        .await
        .context("could not connect to database_url")?;

    sqlx::migrate!().run(&db).await?;

    let app = Router::new()
        .route("/hello", get(root))
        .route("/:id", get(redirect))
        .route("/", post(shorten))
        .layer(Extension(db));
        

        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        println!("listening on {}", addr);
        tracing::debug!("listening on {}", addr);
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();  
            
    Ok(())        
    

}

async fn root() -> &'static str {
    "Hello, World!"
}
