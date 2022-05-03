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

use crate::api::v1::{
    graphql::{
        FilterFactory,
        GetAllFactory,
        GetOneFactory,
        QueryFactory,
        NameFactory,
    },
    datasets::QUERY_VALUES as DATASET_VALUES,
    params::{QueryParams, QueryType}
};
use crate::datahub::{post, GRAPHQL_ENDPOINT, INGEST_ENDPOINT};
use crate::schemas::{
    self,
    requests,
    CreateTag,
    DeleteTag,
    Datasets,
    QueryResponse,
    Tags,
    TagEnvelope,
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

static GET_ALL: Lazy<GetAllFactory>     = Lazy::new(|| GetAllFactory::new("TAG", QUERY_VALUES));
static GET_BY_ID: Lazy<GetOneFactory>   = Lazy::new(|| GetOneFactory::new("tag", QUERY_VALUES));
static GET_BY_NAME: Lazy<NameFactory>   = Lazy::new(|| NameFactory::new("TAG", QUERY_VALUES));
static GET_BY_QUERY: Lazy<QueryFactory> = Lazy::new(|| QueryFactory::new("TAG", QUERY_VALUES));
static DATASETS_BY_TAG: Lazy<FilterFactory>
    = Lazy::new(|| FilterFactory::new("DATASETS", DATASET_VALUES, "tags"));

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
    let body = GET_BY_ID.body(&id);
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
    let body = match params.query {
        QueryType::All          => GET_ALL.body(&params),
        QueryType::Name(name)   => GET_BY_NAME.body(name, &params),
        QueryType::Query(query) => GET_BY_QUERY.body(query, &params),
        QueryType::Tags(..) => panic!("?tags not supported")
    };
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
    let body = DATASETS_BY_TAG.body(&id, &params);
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
