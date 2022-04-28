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

use crate::api::v1::{
    queries,
    datasets::QUERY_VALUES as DATASET_VALUES,
    params::QueryParams
};

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
    let body = GraphQL::new(QUERY_BY_ID.to_owned(), Variables::Urn(id));
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
    let params = QueryParams::from(&req);
    let (query, variables) = if let Some(query) = params.name {
        (
            QUERY_BY_QUERY.to_owned(),
            Variables::AutoCompleteInput(
                AutoCompleteInput::new(
                    "TAG".into(),
                    query.to_string(),
                    params.limit
                )
            )
        )
    } else if let Some(query) = params.query {
        (
            QUERY_BY_NAME.to_owned(),
            Variables::SearchInput(
                SearchInput::new(
                    "TAG".into(),
                    format!("*{query}*"),
                    params.start,
                    params.limit,
                    None
                )
            )
        )
    } else {
        (
            QUERY_BY_QUERY.to_owned(),
            Variables::SearchInput(
                SearchInput::new(
                    "TAG".into(),
                    "*".into(),
                    params.start,
                    params.limit,
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
    let params = QueryParams::from(&req);
    let variables = Variables::SearchInput(
        SearchInput::new(
            "DATASET".into(),
            "*".into(),
            params.start,
            params.limit,
            Some(Filter::new("tags".into(), id))
        )
    );

    let body = GraphQL::new(DATASETS_BY_TAG.to_owned(), variables);
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
    let body = CreateTag::new(payload.name.clone(), desc.clone());
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
    let body = DeleteTag::new(id);
    let resp = post(&client, INGEST_ENDPOINT, body)
        .await
        .unwrap();

    if resp.status() == StatusCode::OK {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
