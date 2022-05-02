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
    TagAssociationInput,
    Datasets,
    DatasetEnvelope,
    DatasetAddTagResponse,
    QueryResponse,
};

use crate::api::v1::{
    queries,
    graphql::{GetAllFactory, GetOneFactory, NameFactory, TagsFactory, QueryFactory},
    params::{QueryParams, QueryType}
};

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

static ADD_TAG: Lazy<String>    = Lazy::new(queries::add_tag);
static REMOVE_TAG: Lazy<String> = Lazy::new(queries::remove_tag);
static GET_ALL: Lazy<GetAllFactory>
    = Lazy::new(|| GetAllFactory::new("DATASET", QUERY_VALUES));
static GET_BY_ID: Lazy<GetOneFactory>
    = Lazy::new(|| GetOneFactory::new("dataset", QUERY_VALUES));
static GET_BY_NAME: Lazy<NameFactory>
    = Lazy::new(|| NameFactory::new("DATASET", QUERY_VALUES));
static GET_BY_TAGS: Lazy<TagsFactory>
    = Lazy::new(|| TagsFactory::new("DATASET", QUERY_VALUES));
static GET_BY_QUERY: Lazy<QueryFactory>
    = Lazy::new(|| QueryFactory::new("DATASET", QUERY_VALUES));

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
    let body = GET_BY_ID.body(&id);
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
    let params = QueryParams::from(&req);
    let body = match params.query {
        QueryType::All          => GET_ALL.body(&params),
        QueryType::Name(name)   => GET_BY_NAME.body(name, &params),
        QueryType::Tags(tags)   => GET_BY_TAGS.body(tags, &params),
        QueryType::Query(query) => GET_BY_QUERY.body(query, &params),
    };
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
