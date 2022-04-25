mod create;
mod remove;

use std::convert::From;
use std::collections::HashMap;

use axum::http::Response;
use hyper::Body;
use serde::{Serialize, Deserialize};
use serde_json::Result;

use crate::datahub as dh;

pub use create::CreateRequest;
pub use remove::RemoveRequest;

const DATASET_QUERY: &str = dh::DATASET_QUERY;
const TAG_QUERY: &str = "
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

#[derive(Serialize)]
pub struct Tags {
    data: Vec<TagEnvelope>,
    paging: Option<Paging>,
}

#[derive(Serialize)]
pub struct TagEnvelope {
    pub tag: Option<Tag>
}

#[derive(Serialize)]
pub struct Tag {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct Paging {
    total: i32,
    limit: i32,
    offset: i32,
}


impl From<&dh::TagEntity> for TagEnvelope {
    fn from(e: &dh::TagEntity) -> Self
    {
        TagEnvelope { 
            tag: e.tag.as_ref().map(Tag::from)
        }
    }
}
impl From<&dh::Tag> for TagEnvelope {
    fn from(tag: &dh::Tag) -> Self
    {
        TagEnvelope { 
            tag: Some(Tag::from(tag))
        }
    }
}

impl From<&dh::Tag> for Tag {
    fn from(tag: &dh::Tag) -> Self
    {
        Tag {
            id: tag.urn.to_owned(),
            name: tag.properties.as_ref()
                .map(|props| props.name.to_owned()),
            description: tag.properties.as_ref()
                .map(|props| props.description.to_owned()),
        }
    }
}

pub async fn by_id(resp: Response<Body>) -> Result<TagEnvelope>
{
    let bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    let body: TagResponse = serde_json::from_slice(&bytes).unwrap();

    Ok(TagEnvelope::from(&body.data))
}

pub async fn by_query(resp: Response<Body>) -> Result<Tags>
{
    let bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    let body: QueryResponse = serde_json::from_slice(&bytes).unwrap();

    let data   = body.data.results.process();
    let paging = body.data.results.paging();

    Ok(Tags { data, paging })
}

pub fn add_body(rsrc_id: &str, tag_id: &str) -> String
{
    format!(r#"{{
        "query": "mutation addTag {{
            success: addTag(input: {{ 
                tagUrn: \"{tag_id}\",
                resourceUrn: \"{rsrc_id}\"
            }})
        }}",
        "variables": {{}}
    }}"#)
    .replace('\n', "")
}

pub fn remove_body(rsrc_id: &str, tag_id: &str) -> String
{
    format!(r#"{{
        "query": "mutation removeTag {{
            success: removeTag(input: {{ 
                tagUrn: \"{tag_id}\",
                resourceUrn: \"{rsrc_id}\"
            }})
        }}",
        "variables": {{}}
    }}"#)
    .replace('\n', "")
}

pub fn id_query_body(id: &str) -> String
{
    format!(r#"{{
        "query": "{{ tag(urn: \"{id}\") {{ {TAG_QUERY} }} }}"
    }}"#)
    .replace('\n', "")
}

pub fn datasets_query_body(id: &str, params: HashMap<&str, &str>) -> String
{
    let start = params.get("offset").unwrap_or(&"0");
    let limit = params.get("limit").unwrap_or(&"10");

    format!(r#"{{
        "query": "{{ 
            results: search(input: {{ 
                type: DATASET, query: \"*\", start: {start}, count: {limit},
                filters: {{ field: \"tags\", value: \"{id}\" }}
            }}) {{
                __typename start count total
                entities: searchResults {{ entity {{ {DATASET_QUERY} }} }}
            }}
        }}"
    }}"#)
    .replace('\n', "")
}

pub fn params_query_body(params: HashMap<&str, &str>) -> String
{
    let start = params.get("offset").unwrap_or(&"0");
    let limit = params.get("limit").unwrap_or(&"10");

    if let Some(name) = params.get("name") {
        build_name_query(name, limit)
    } else if let Some(query) = params.get("query") {
        build_tags_query(query, start, limit)
    } else {
        build_all_query(start, limit)
    }
}

fn build_name_query(query: &str, limit: &str) -> String
{
    format!(r#"{{
        "query": "{{ 
            results: autoComplete(input: {{ 
                type: TAG, query: \"*{query}*\", limit: {limit},
            }}) {{
                __typename
                entities {{ {TAG_QUERY} }} 
            }}
        }}"
    }}"#)
    .replace('\n', "")
}

fn build_tags_query(query: &str, start: &str, limit: &str) -> String
{
    format!(r#"{{
        "query": "{{ 
            results: search(input: {{ 
                type: TAG, query: \"*{query}\", start: {start}, count: {limit},
            }}) {{
                __typename start count total
                entities: searchResults {{ entity {{ {TAG_QUERY} }} }}
            }}
        }}"
    }}"#)
    .replace('\n', "")
}

fn build_all_query(start: &str, limit: &str) -> String
{
    format!(r#"{{
        "query": "{{ 
            results: search(input: {{ 
                type: TAG, query: \"*\", start: {start}, count: {limit},
            }}) {{
                __typename start count total
                entities: searchResults {{ entity {{ {TAG_QUERY} }} }}
            }}
        }}"
    }}"#)
    .replace('\n', "")
}

#[derive(Deserialize)]
struct TagResponse {
    data: dh::TagEntity,
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
        entities: Vec<dh::Tag>
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
    entity: dh::Tag,
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

    fn process(&self) -> Vec<TagEnvelope>
    {
        match self {
            Self::AutoCompleteResults { entities } => {
                entities.iter()
                    .map(TagEnvelope::from)
                    .collect()
            },
            Self::SearchResults { entities, .. } => {
                entities.iter()
                    .map(|e| &e.entity)
                    .map(TagEnvelope::from)
                    .collect()
            },
        }
    }
}
