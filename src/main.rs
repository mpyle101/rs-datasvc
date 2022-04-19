mod datasets;
mod tags;

// Standard library
use std::collections::HashMap;
use std::net::SocketAddr;

// Installed crates
use axum::{
    Json,
    extract::Extension,
    handler::Handler,
    http::{header, Method, Request, StatusCode, Uri},
    response::{Html, IntoResponse},
    routing::get
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
        .route("/tags", get(tags_by_query))
        .route("/datasets", get(datasets_by_query))
        .layer(Extension(Client::new()))
        .fallback(not_found.into_service());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("datasvc listening on {addr}");

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
    println!("\nshutting down...");
}

// Route Handlers
async fn root() -> Html<&'static str>
{
    "<h1>Alteryx Data Service is alive!</h1>".into()
}

async fn tags_by_query(
    Extension(client): Extension<Client>,
    req: Request<Body>
) -> Json<tags::Tags>
{
    let params: HashMap<_, _> = req.uri().query()
        .map_or_else(|| HashMap::new(), |s| parse_query(s));
    let data = tags::build_query(params);
    let req  = build_request(data);
    let resp = client.request(req)
        .await
        .unwrap();
    let results = tags::by_query(resp)
        .await
        .unwrap();

    results.into()
}

async fn datasets_by_query(
    Extension(client): Extension<Client>,
    req: Request<Body>
) -> Json<datasets::Datasets>
{
    let params: HashMap<_, _> = req.uri().query()
        .map_or_else(|| HashMap::new(), |s| parse_query(s));
    let query = datasets::build_query(params);
    let req  = build_request(query);
    let resp = client.request(req)
        .await
        .unwrap();
    let results = datasets::by_query(resp)
        .await
        .unwrap();

    results.into()
}

async fn not_found(uri: Uri) -> impl IntoResponse
{
    (StatusCode::NOT_FOUND, format!("Not found: {}", uri))
}

fn parse_query(query: &str) -> HashMap<&str, &str>
{
    query.split('&')
        .map(|s| s.split('='))
        .map(|mut v| {
            let key = v.next().unwrap();
            match v.next() {
                Some(val) => (key, val),
                None => ("query", key)
            }
        })
        .collect()
}

fn build_request(data: String) -> Request<Body>
{
    Request::builder()
        .method(Method::POST)
        .uri("http://localhost:8080/api/graphql")
        .header("X-DataHub-Actor", "urn:li:corpuser:datahub")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(data))
        .unwrap()
}