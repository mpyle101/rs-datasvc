use axum::{
    Json, Router,
    extract::{Extension, Path},
    http::Request,
    routing::get,
};
use hyper::{client::HttpConnector, Body};
use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::{datahub::{post, GRAPHQL_ENDPOINT}, schemas::graphql::ListRecommendationsInput};
use crate::schemas::{
    self,
    GraphQL,
    Variables,
    SearchInput,
    Filter,
    Datasets,
    Platforms,
    PlatformEnvelope,
    QueryResponse,
    ListRecommendationsResponse
};

use crate::api::v1::{
    queries,
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

static QUERY_BY_ID: Lazy<String> = Lazy::new(|| queries::by_id("dataPlatform", QUERY_VALUES));
static ALL_PLATFORMS: Lazy<String> = Lazy::new(|| queries::platforms(QUERY_VALUES));
static DATASETS_BY_PLATFORM: Lazy<String> = Lazy::new(|| queries::by_query(DATASET_VALUES));

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
    let body = GraphQL::new(QUERY_BY_ID.to_owned(), Variables::Urn(id));
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
    let variables = Variables::ListRecommendationsInput(
        ListRecommendationsInput::new("urn:li:corpuser:datahub".into(), params.limit)
    );
    let body = GraphQL::new(ALL_PLATFORMS.to_owned(), variables);
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
    let variables = Variables::SearchInput(
        SearchInput::new(
            "DATASET".into(),
            "*".into(),
            params.start,
            params.limit,
            Some(Filter::new("platform".into(), id))
        )
    );

    let body = GraphQL::new(DATASETS_BY_PLATFORM.to_owned(), variables);
    let resp = post(&client, GRAPHQL_ENDPOINT, body)
        .await
        .unwrap();
    let bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    let body: QueryResponse = serde_json::from_slice(&bytes).unwrap();

    Datasets::from(&body).into()
}
