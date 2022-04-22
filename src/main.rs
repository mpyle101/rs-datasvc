mod datahub;
mod datasets;
mod tags;


use std::collections::HashMap;
use std::net::SocketAddr;

use axum::{
    Json,
    extract::{Extension, Path},
    handler::Handler,
    http::{header, Method, Request, StatusCode, Uri},
    response::{Html, IntoResponse},
    routing::{get, post, delete},
};
use hyper::{
    Body,
    client::HttpConnector,
};
use serde::Deserialize;

use datahub as dh;
use tags::{CreateRequest, RemoveRequest};

type Client = hyper::client::Client<HttpConnector, Body>;


#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let app = axum::Router::new()
        .route("/", get(root))
        .route("/tags",
            get(tags_by_query)
                .post(create_tag)
        )
        .route("/tags/:id", 
            get(tags_by_id)
                .delete(delete_tag)
        )
        .route("/datasets", get(datasets_by_query))
        .route("/datasets/:id", get(dataset_by_id))
        .route("/datasets/:id/tags", post(dataset_add_tag))
        .route("/datasets/:id/tags/:tid", delete(dataset_remove_tag))
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

async fn tags_by_id(
    Path(id): Path<String>,
    Extension(client): Extension<Client>,
) -> Json<tags::TagEnvelope>
{
    let body = tags::id_query_body(&id);
    let req  = graphql_request(body);
    let resp = client.request(req)
        .await
        .unwrap();
    let results = tags::by_id(resp)
        .await
        .unwrap();

    results.into()
}

async fn tags_by_query(
    Extension(client): Extension<Client>,
    req: Request<Body>
) -> Json<tags::Tags>
{
    let params: HashMap<_, _> = req.uri().query()
        .map_or_else(HashMap::new, parse_query);
    let body = tags::params_query_body(params);
    let req  = graphql_request(body);
    let resp = client.request(req)
        .await
        .unwrap();
    let results = tags::by_query(resp)
        .await
        .unwrap();

    results.into()
}

async fn create_tag(
    Extension(client): Extension<Client>,
    Json(payload): Json<CreateTag>
) -> impl IntoResponse
{
    let desc = payload.description.unwrap_or_else(|| "".to_string());
    let body = CreateRequest::with(payload.name.clone(), desc.clone());
    let req  = entities_request("ingest", body);
    let resp = client.request(req)
        .await
        .unwrap();
    
    let status = if resp.status() == StatusCode::OK {
        StatusCode::CREATED
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    let name = payload.name;
    (status, Json(tags::Tag {
        id: format!("urn:li:tag:{name}"),
        name: Some(name),
        description: Some(desc),
    }))
}

async fn delete_tag(
    Path(id): Path<String>,
    Extension(client): Extension<Client>,
) -> StatusCode
{
    let body = RemoveRequest::with(id, true);
    let req  = entities_request("ingest", body);
    let resp = client.request(req)
        .await
        .unwrap();

    if resp.status() == StatusCode::OK {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

async fn datasets_by_query(
    Extension(client): Extension<Client>,
    req: Request<Body>
) -> Json<datasets::Datasets>
{
    let params: HashMap<_, _> = req.uri().query()
        .map_or_else(HashMap::new, parse_query);
    let body = datasets::params_query_body(params);
    let req  = graphql_request(body);
    let resp = client.request(req)
        .await
        .unwrap();
    let results = datasets::by_query(resp)
        .await
        .unwrap();

    results.into()
}

async fn dataset_by_id(
    Path(id): Path<String>,
    Extension(client): Extension<Client>,
) -> (StatusCode, Json<datasets::DatasetEnvelope>)
{
    let body = datasets::id_query_body(&id);
    let req  = graphql_request(body);
    let resp = client.request(req)
        .await
        .unwrap();
    let envelope = datasets::by_id(resp)
        .await
        .unwrap();
    
    let status_code = if envelope.dataset.is_none() { 
        StatusCode::NOT_FOUND
    } else {
        StatusCode::OK
    };

    (status_code, envelope.into())
}

async fn dataset_add_tag(
    Path(id): Path<String>,
    Json(payload): Json<AddTag>,
    Extension(client): Extension<Client>
) -> StatusCode
{
    let body = tags::add_body(&id, &payload.tag);
    println!("BODY: {body}");
    let req  = graphql_request(body);
    let resp = client.request(req)
        .await
        .unwrap();
    println!("RESP: {:?}", resp);
    let status = resp.status();
    let bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    println!("BYTES: {:?}", bytes);
    let result: DatasetTagResponse = serde_json::from_slice(&bytes).unwrap();

    if let Some(errors) = result.errors {
        errors.iter().for_each(|msg| println!("{:?}", msg.message))
    }
    if status == StatusCode::OK {
        match result.data {
            Some(res) => if res.success { 
                    StatusCode::NO_CONTENT
                } else {
                    StatusCode::UNPROCESSABLE_ENTITY
                },
            None => StatusCode::UNPROCESSABLE_ENTITY
        }
    } else {
        status
    }
}

async fn dataset_remove_tag(
    Path((id, tid)): Path<(String, String)>,
    Extension(client): Extension<Client>
) -> StatusCode
{
    let body = tags::remove_body(&id, &tid);
    let req  = graphql_request(body);
    let resp = client.request(req)
        .await
        .unwrap();
    let status = resp.status();
    let _bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    
    if status == StatusCode::OK {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
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

fn graphql_request(data: String) -> Request<Body>
{
    Request::builder()
        .method(Method::POST)
        .uri("http://localhost:8080/api/graphql")
        .header("X-DataHub-Actor", "urn:li:corpuser:datahub")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(data))
        .unwrap()
}

fn entities_request(action: &str, data: impl std::fmt::Display) -> Request<Body>
{
    Request::builder()
        .method(Method::POST)
        .uri(format!("http://localhost:8080/entities?action={action}"))
        .header("X-DataHub-Actor", "urn:li:corpuser:datahub")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(data.to_string()))
        .unwrap()
}

#[derive(Deserialize)]
struct CreateTag {
    name: String,
    description: Option<String>,
}

#[derive(Deserialize)]
struct AddTag {
    tag: String,
}
#[derive(Deserialize)]
struct DatasetTagResponse {
    data: Option<DatasetTagResult>,
    errors: Option<Vec<dh::ErrorMessage>>,
}
#[derive(Deserialize)]
struct DatasetTagResult {
    success: bool
}
