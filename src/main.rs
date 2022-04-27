mod api;
mod datahub;
mod schemas;

use std::net::SocketAddr;

use axum::{
    extract::Extension,
    handler::Handler,
    http::{StatusCode, Uri},
    response::{Html, IntoResponse},
    routing::get,
};
use hyper::{
    Body,
    client::HttpConnector,
};

type Client = hyper::client::Client<HttpConnector, Body>;


#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let app = axum::Router::new()
        .route("/", get(root))
        .nest("/api", api::routes())
        .layer(Extension(Client::new()))
        .fallback(not_found.into_service());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {addr}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown())
        .await?;

    Ok(())
}

async fn shutdown()
{
    tokio::signal::ctrl_c()
        .await
        .expect("tokio signal ctrl-c");
    println!("\nShutting down...");
}

async fn root() -> Html<&'static str>
{
    "<h1>Alteryx Data Service is alive!</h1>".into()
}

async fn not_found(uri: Uri) -> impl IntoResponse
{
    (StatusCode::NOT_FOUND, format!("Not found: {}", uri))
}
