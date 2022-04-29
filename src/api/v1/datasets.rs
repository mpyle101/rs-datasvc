use axum::{
    Json, Router,
    extract::{Extension, Path},
    http::{Request, StatusCode},
    routing::{get, post, delete}
};
use hyper::{client::HttpConnector, Body};
use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::datahub::{self, GRAPHQL_ENDPOINT};
use crate::schemas::{
    self,
    requests,
    GraphQL,
    Variables,
    AutoCompleteInput,
    SearchInput,
    TagAssociationInput,
    Datasets,
    DatasetEnvelope,
    DatasetAddTagResponse,
    QueryResponse,
};

use crate::api::v1::{queries, params::QueryParams};

pub const QUERY_VALUES: &str = "
    urn
    __typename
    ... on Dataset {
        name
        platform {
            name
            properties {
                name: displayName
                class: type
            }
        }
        properties {
            name
            origin
        }
        schema: schemaMetadata {
            fields {
                path: fieldPath
                class: type
                native: nativeDataType
            }
        }
        sub_types: subTypes {
            names: typeNames
        }
        tags {
            tags {
                tag {
                    urn
                    properties {
                        name
                        description
                    }
                }
            }
        }
    }
";

static ADD_TAG: Lazy<String>         = Lazy::new(queries::add_tag);
static REMOVE_TAG: Lazy<String>      = Lazy::new(queries::remove_tag);
static QUERY_BY_ID: Lazy<String>     = Lazy::new(|| queries::by_id("dataset", QUERY_VALUES));
static QUERY_BY_NAME: Lazy<String>   = Lazy::new(|| queries::by_name(QUERY_VALUES));
static QUERY_BY_QUERY: Lazy<String>  = Lazy::new(|| queries::by_query(QUERY_VALUES));

type Client = hyper::client::Client<HttpConnector, Body>;

#[derive(Deserialize)]
struct DatasetResponse<'a> {
    #[serde(borrow)]
    data: schemas::datahub::DatasetEntity<'a>,
}

pub fn routes() -> Router
{
    Router::new()
        .route("/", get(by_query))
        .route("/:id", get(by_id))
        .route("/:id/tags", post(add_tag))
        .route("/:id/tags/:tag_id", delete(remove_tag))
}

async fn by_id(
    Path(id): Path<String>,
    Extension(client): Extension<Client>,
) -> Json<DatasetEnvelope>
{
    let body = GraphQL::new(&*QUERY_BY_ID, Variables::Urn(&id));
    let resp = datahub::post(&client, GRAPHQL_ENDPOINT, body)
        .await
        .unwrap();
    let bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    let body: DatasetResponse = serde_json::from_slice(&bytes).unwrap();

    DatasetEnvelope::from(&body.data).into()
}

async fn by_query(
    Extension(client): Extension<Client>,
    req: Request<Body>
) -> Json<schemas::Datasets>
{
    let tmp: String;
    let params = QueryParams::from(&req);
    let (query, variables) = if let Some(query) = params.name {
        (
            &*QUERY_BY_NAME,
            Variables::AutoCompleteInput(
                AutoCompleteInput::new("DATASET", query, params.limit)
            )
        )
    } else if let Some(query) = params.query {
        tmp = format!("*{query}*");
        (
            &*QUERY_BY_QUERY,
            Variables::SearchInput(
                SearchInput::new("DATASET", &tmp, params.start, params.limit, None)
            )
        )
    } else if let Some(query) = params.tags {
        tmp = format!("tags:{query}");
        (
            &*QUERY_BY_QUERY,
            Variables::SearchInput(
                SearchInput::new("DATASET", &tmp, params.start, params.limit, None)
            )
        )
    } else {
        (
            &*QUERY_BY_QUERY,
            Variables::SearchInput(
                SearchInput::new("DATASET", "*", params.start, params.limit, None)
            )
        )
    };

    let body = GraphQL::new(query, variables);
    let resp = datahub::post(&client, GRAPHQL_ENDPOINT, body)
        .await
        .unwrap();
    let bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    let body: QueryResponse = serde_json::from_slice(&bytes).unwrap();

    Datasets::from(&body).into()
}

async fn add_tag(
    Path(id): Path<String>,
    Json(payload): Json<requests::AddTag>,
    Extension(client): Extension<Client>
) -> StatusCode
{
    let variables = Variables::TagAssociationInput(
        TagAssociationInput::new(&id, &payload.tag)
    );
    let body = GraphQL::new(&*ADD_TAG, variables);
    let resp = datahub::post(&client, GRAPHQL_ENDPOINT, body)
        .await
        .unwrap();
    let status = resp.status();
    let bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    let result: DatasetAddTagResponse = serde_json::from_slice(&bytes).unwrap();

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

async fn remove_tag(
    Path((id, tag_id)): Path<(String, String)>,
    Extension(client): Extension<Client>
) -> StatusCode
{
    let variables = Variables::TagAssociationInput(
        TagAssociationInput::new(&id, &tag_id)
    );
    let body = GraphQL::new(&*REMOVE_TAG, variables);
    let resp = datahub::post(&client, GRAPHQL_ENDPOINT, body)
        .await
        .unwrap();

    if resp.status() == StatusCode::OK {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
