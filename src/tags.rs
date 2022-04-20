use std::collections::HashMap;

use axum::http::Response;
use hyper::Body;
use serde::{Serialize, Deserialize};
use serde_json::{Result, json};

use crate::datahub;

#[derive(Serialize)]
pub(crate) struct Tags {
    data: Vec<Tag>,
    paging: Option<Paging>,
}

#[derive(Serialize)]
pub(crate) struct Tag {
    id: String,
    name: String,
}

#[derive(Serialize)]
pub(crate) struct Paging {
    total: i32,
    limit: i32,
    offset: i32,
}

impl Tag {
    fn from_entity(tag: &datahub::Tag) -> Tag
    {
        Tag {
            id: tag.urn.to_owned(),
            name: tag.properties.name.to_owned(),
        }
    }
}

pub(crate) async fn by_id(resp: Response<Body>) -> Result<Tag>
{
    let bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    let body: TagResponse = serde_json::from_slice(&bytes).unwrap();

    Ok(Tag::from_entity(&body.data.tag))
}

pub(crate) fn build_id_query(id: &str) -> String
{
    let value = json!({
        "query": format!("{{ 
            tag(urn: \"{id}\") {{
                urn
                properties {{ 
                    name
                }}
            }}
        }}")
    });

    format!("{value}")
}

pub(crate) async fn by_query(resp: Response<Body>) -> Result<Tags>
{
    let bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    let body: QueryResponse = serde_json::from_slice(&bytes).unwrap();

    let data   = body.data.results.process();
    let paging = body.data.results.paging();

    Ok(Tags { data, paging })
}

pub(crate) fn build_query(params: HashMap<&str, &str>) -> String
{
    let start = params.get("offset").unwrap_or(&"0");
    let limit = params.get("limit").unwrap_or(&"10");
    let value = match params.get("query") {
        Some(query) => build_name_query(query, limit),
        None        => build_tags_query(start, limit),
    };

    format!("{value}")
}

fn build_name_query(query: &str, limit: &str) -> serde_json::Value
{
    json!({
        "query": format!("{{ 
            results: autoComplete(input: {{ 
                type: TAG,
                query: \"{query}\",
                limit: {limit},
            }}) {{
                __typename
                entities {{ 
                    urn
                    ... on Tag {{
                        urn
                        properties {{ 
                            name
                        }}
                    }}
                }} 
            }}
        }}")
    })
}

fn build_tags_query(start: &str, limit: &str) -> serde_json::Value
{
    json!({
        "query": format!("{{ 
            results: search(input: {{ 
                type: TAG,
                query: \"\",
                start: {start},
                count: {limit},
            }}) {{
                __typename
                start
                count
                total
                entities: searchResults {{
                    entity {{
                        urn
                        ... on Tag {{
                            urn
                            properties {{ 
                                name
                            }}
                        }}
                    }}
                }}
            }}
        }}")
    })
}

#[derive(Deserialize)]
struct TagResponse {
    data: datahub::TagEntity,
}

#[derive(Deserialize)]
struct QueryResponse {
    data: QueryResponseData
}

#[derive(Deserialize)]
struct QueryResponseData {
    results: QueryResults
}

#[derive(Deserialize)]
#[serde(tag = "__typename")]
enum QueryResults {
    AutoCompleteResults { 
        entities: Vec<datahub::Tag>
    },
    SearchResults {
        start: i32,
        count: i32,
        total: i32,
        entities: Vec<SearchEntity>
    },
}

#[derive(Deserialize)]
struct SearchEntity {
    entity: datahub::Tag,
}

impl QueryResults {
    fn paging(&self) -> Option<Paging> {
        match self {
            Self::AutoCompleteResults { .. } => None,
            Self::SearchResults { start, count, total, .. } => {
                Some(Paging { offset: *start, limit: *count, total: *total })
            },
        }
    }

    fn process(&self) -> Vec<Tag>
    {
        match self {
            Self::AutoCompleteResults { entities } => {
                entities.iter()
                    .map(Tag::from_entity)
                    .collect()
            },
            Self::SearchResults { entities, .. } => {
                entities.iter()
                    .map(|e| Tag::from_entity(&e.entity))
                    .collect()
            },
        }
    }
}

