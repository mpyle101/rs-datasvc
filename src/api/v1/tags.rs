use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::{Extension, Path},
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::get,
};
use hyper::{client::HttpConnector, Body};
use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::datahub::{post, GRAPHQL_ENDPOINT, INGEST_ENDPOINT};
use crate::schemas::{
    self,
    requests,
    GraphQL,
    Variables,
    AutoCompleteInput,
    SearchInput,
    Filter,
    Datasets,
    Tags,
    TagEnvelope,
    CreateTag,
    DeleteTag,
    QueryResponse
};

use crate::api::v1::{queries, datasets::QUERY_VALUES as DATASET_VALUES};

const QUERY_VALUES: &str = "
    urn
    __typename
    ... on Tag {
        urn
        properties {
            name
            description
        }
    }
";

static QUERY_BY_ID: Lazy<String>     = Lazy::new(|| queries::by_id("tag", QUERY_VALUES));
static QUERY_BY_NAME: Lazy<String>   = Lazy::new(|| queries::by_name(QUERY_VALUES));
static QUERY_BY_QUERY: Lazy<String>  = Lazy::new(|| queries::by_query(QUERY_VALUES));
static DATASETS_BY_TAG: Lazy<String> = Lazy::new(|| queries::by_query(DATASET_VALUES));

type Client = hyper::client::Client<HttpConnector, Body>;

#[derive(Deserialize)]
struct TagResponse {
    data: schemas::datahub::TagEntity,
}

pub fn routes() -> Router
{
    Router::new()
        .route("/",
            get(by_query)
                .post(create_tag)
        )
        .route("/:id", 
            get(by_id)
                .delete(delete_tag)
        )
        .route("/:id/datasets", get(datasets_by_tag))
}

async fn by_id(
    Path(id): Path<String>,
    Extension(client): Extension<Client>,
) -> Json<TagEnvelope>
{
    let body = GraphQL::new(&*QUERY_BY_ID, Variables::Urn(id));
    let resp = post(&client, GRAPHQL_ENDPOINT, body)
        .await
        .unwrap();
    let bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    let body: TagResponse = serde_json::from_slice(&bytes).unwrap();

    TagEnvelope::from(&body.data).into()
}

async fn by_query(
    Extension(client): Extension<Client>,
    req: Request<Body>
) -> Json<schemas::Tags>
{
    let params: HashMap<_, _> = req.uri().query()
        .map_or_else(HashMap::new, parse_query);
    let start = params.get("offset").unwrap_or(&"0");
    let limit = params.get("limit").unwrap_or(&"10");

    let (query, variables) = if let Some(query) = params.get("name") {
        (
            &*QUERY_BY_QUERY,
            Variables::AutoCompleteInput(
                AutoCompleteInput::new(
                    "TAG",
                    query.to_string(),
                    limit.parse::<i32>().unwrap()
                )
            )
        )
    } else if let Some(query) = params.get("query") {
        (
            &*QUERY_BY_NAME,
            Variables::SearchInput(
                SearchInput::new(
                    "TAG",
                    format!("*{query}*"),
                    start.parse::<i32>().unwrap(),
                    limit.parse::<i32>().unwrap(),
                    None
                )
            )
        )
    } else {
        (
            &*QUERY_BY_QUERY,
            Variables::SearchInput(
                SearchInput::new(
                    "TAG",
                    "*".into(),
                    start.parse::<i32>().unwrap(),
                    limit.parse::<i32>().unwrap(),
                    None
                )
            )
        )
    };

    let body = GraphQL::new(query, variables);
    let resp = post(&client, GRAPHQL_ENDPOINT, body)
        .await
        .unwrap();
    let bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    let body: QueryResponse = serde_json::from_slice(&bytes).unwrap();

    Tags::from(&body).into()
}

async fn datasets_by_tag(
    Path(id): Path<String>,
    Extension(client): Extension<Client>,
    req: Request<Body>
) -> Json<schemas::Datasets>
{
    let params: HashMap<_, _> = req.uri().query()
        .map_or_else(HashMap::new, parse_query);
    let start = params.get("offset").unwrap_or(&"0");
    let limit = params.get("limit").unwrap_or(&"10");

    let variables = Variables::SearchInput(
        SearchInput::new(
            "DATASET",
            "*".into(),
            start.parse::<i32>().unwrap(),
            limit.parse::<i32>().unwrap(),
            Some(Filter::new("tags".into(), id))
        )
    );

    let body = GraphQL::new(&*DATASETS_BY_TAG, variables);
    let resp = post(&client, GRAPHQL_ENDPOINT, body)
        .await
        .unwrap();
    let bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    let body: QueryResponse = serde_json::from_slice(&bytes).unwrap();

    Datasets::from(&body).into()
}

async fn create_tag(
    Extension(client): Extension<Client>,
    Json(payload): Json<requests::CreateTag>
) -> impl IntoResponse
{
    let desc = payload.description.unwrap_or_else(|| "".to_string());
    let body = CreateTag::with(payload.name.clone(), desc.clone());
    let resp = post(&client, INGEST_ENDPOINT, body)
        .await
        .unwrap();
    
    let status = if resp.status() == StatusCode::OK {
        StatusCode::CREATED
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    let name = payload.name;
    let urn = format!("urn:li:tag:{name}");

    (status, Json(schemas::Tag::new(urn, Some(name), Some(desc))))
}

async fn delete_tag(
    Path(id): Path<String>,
    Extension(client): Extension<Client>,
) -> StatusCode
{
    let body = DeleteTag::with(id);
    let resp = post(&client, INGEST_ENDPOINT, body)
        .await
        .unwrap();

    if resp.status() == StatusCode::OK {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
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
