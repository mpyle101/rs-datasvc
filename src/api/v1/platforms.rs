use axum::{
    Json, Router,
    extract::{Extension, Path},
    http::Request,
    routing::get,
};
use hyper::{client::HttpConnector, Body};
use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::{datahub::{post, GRAPHQL_ENDPOINT}};
use crate::schemas::{
    self,
    Datasets,
    Platforms,
    PlatformEnvelope,
    QueryResponse,
    ListRecommendationsResponse
};

use crate::api::v1::{
    graphql::{GetOneFactory, FilterFactory, PlatformsFactory},
    datasets::QUERY_VALUES as DATASET_VALUES,
    params::QueryParams
};

const QUERY_VALUES: &str = "
    urn
    __typename
    ... on DataPlatform {
        name
        properties {
            name: displayName
            class: type
        }
    }
";

static GET_ALL: Lazy<PlatformsFactory>           = Lazy::new(|| PlatformsFactory::new(QUERY_VALUES));
static GET_BY_ID: Lazy<GetOneFactory>            = Lazy::new(|| GetOneFactory::new("dataPlatform", QUERY_VALUES));
static DATASETS_BY_PLATFORM: Lazy<FilterFactory> = Lazy::new(|| FilterFactory::new("DATASETS", DATASET_VALUES, "platform"));

type Client = hyper::client::Client<HttpConnector, Body>;


#[derive(Deserialize)]
struct PlatformResponse<'a> {
    #[serde(borrow)]
    data: schemas::datahub::PlatformEntity<'a>,
}

pub fn routes() -> Router
{
    Router::new()
        .route("/", get(by_query))
        .route("/:id", get(by_id))
        .route("/:id/datasets", get(datasets_by_platform))
}

async fn by_id(
    Path(id): Path<String>,
    Extension(client): Extension<Client>,
) -> Json<PlatformEnvelope>
{
    let body = GET_BY_ID.body(&id);
    let resp = post(&client, GRAPHQL_ENDPOINT, body)
        .await
        .unwrap();
    let bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    let body: PlatformResponse = serde_json::from_slice(&bytes).unwrap();

    PlatformEnvelope::from(&body.data).into()
}

async fn by_query(
    Extension(client): Extension<Client>,
    req: Request<Body>
) -> Json<schemas::Platforms>
{
    let params = QueryParams::from(&req);
    let body = GET_ALL.body(&params);
    let resp = post(&client, GRAPHQL_ENDPOINT, body)
        .await
        .unwrap();
    let bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    let body: ListRecommendationsResponse = serde_json::from_slice(&bytes).unwrap();

    Platforms::from(&body).into()
}

async fn datasets_by_platform(
    Path(id): Path<String>,
    Extension(client): Extension<Client>,
    req: Request<Body>
) -> Json<schemas::Datasets>
{
    let params = QueryParams::from(&req);
    let body = DATASETS_BY_PLATFORM.body(&id, &params);
    let resp = post(&client, GRAPHQL_ENDPOINT, body)
        .await
        .unwrap();
    let bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    let body: QueryResponse = serde_json::from_slice(&bytes).unwrap();

    Datasets::from(&body).into()
}
