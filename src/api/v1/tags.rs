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
struct TagResponse<'a> {
    #[serde(borrow)]
    data: schemas::datahub::TagEntity<'a>,
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
    let body = GraphQL::new(&*QUERY_BY_ID, Variables::Urn(&id));
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
    let tmp: String;
    let params = QueryParams::from(&req);
    let (query, variables) = if let Some(query) = params.name {
        (
            &*QUERY_BY_QUERY,
            Variables::AutoCompleteInput(
                AutoCompleteInput::new("TAG", query, params.limit)
            )
        )
    } else if let Some(query) = params.query {
        tmp = format!("*{query}*");
        (
            &*QUERY_BY_NAME,
            Variables::SearchInput(
                SearchInput::new("TAG", &tmp, params.start, params.limit, None)
            )
        )
    } else {
        (
            &*QUERY_BY_QUERY,
            Variables::SearchInput(
                SearchInput::new("TAG", "*", params.start, params.limit, None)
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
            "DATASET", "*", params.start, params.limit,
            Some(Filter::new("tags", &id))
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
    let name = payload.name;
    let urn  = format!("urn:li:tag:{name}");
    let desc = payload.description.as_ref().map_or_else(|| "", |s| s);
    let body = CreateTag::new(&urn, &name, desc);
    let resp = post(&client, INGEST_ENDPOINT, body)
        .await
        .unwrap();

    let status = if resp.status() == StatusCode::OK {
        StatusCode::CREATED
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    (status, Json(schemas::Tag::new(urn, Some(name), Some(desc.to_string()))))
}

async fn delete_tag(
    Path(id): Path<String>,
    Extension(client): Extension<Client>,
) -> StatusCode
{
    let body = DeleteTag::new(&id);
    let resp = post(&client, INGEST_ENDPOINT, body)
        .await
        .unwrap();

    if resp.status() == StatusCode::OK {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
