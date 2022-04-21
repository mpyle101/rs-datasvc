use std::collections::HashMap;

use axum::http::Response;
use hyper::Body;
use serde::{Serialize, Deserialize};
use serde_json::{Result, json};

use crate::datahub as dh;

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

impl TagEnvelope {
    pub fn from_entity(e: &dh::TagEntity) -> TagEnvelope
    {
        TagEnvelope { 
            tag: e.tag.as_ref().map(|tag| Tag::from_entity(&tag))
        }
    }

    fn from_tag(tag: &dh::Tag) -> TagEnvelope
    {
        TagEnvelope { 
            tag: Some(Tag::from_entity(tag))
        }
    }
}

impl Tag {
    fn from_entity(tag: &dh::Tag) -> Tag
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

    Ok(TagEnvelope::from_entity(&body.data))
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
    let value = json!({
        "query": format!(r#"mutation addTag {{
            success: addTag(input: {{
                tagUrn: "{tag_id}",
                resourceUrn: "{rsrc_id}"
            }})
        }}"#),
        "variables": {}
    });

    format!("{value}")
}

pub fn remove_body(rsrc_id: &str, tag_id: &str) -> String
{
    let value = json!({
        "query": format!(r#"mutation addTag {{
            success: removeTag(input: {{ 
                tagUrn: "{tag_id}",
                resourceUrn: "{rsrc_id}"
            }})
        }}"#),
        "variables": {}
    });

    format!("{value}")
}

pub fn create_body(name: &str, desc: &str) -> String
{
    let value = CreateTag {
        entity: TagValue { 
            value: TagSnapshot { 
                snapshot: TagData {
                    urn: format!("urn:li:tag:{name}"),
                    aspects: vec![
                        TagAspect {
                            properties: TagProperties {
                                name: name.to_string(),
                                description: desc.to_string(),
                            }
                        }
                    ]
                }
            }
        }
    };
    
    serde_json::to_string(&value).unwrap()
}

pub fn delete_body(id: &str) -> String
{
    format!(r#"{{"urn": "{id}"}}"#)
}

pub fn id_query_body(id: &str) -> String
{
    let value = json!({
        "query": format!(r#"{{ 
            tag(urn: "{id}") {{
                urn
                properties {{ 
                    name
                    description
                }}
            }}
        }}"#)
    });

    format!("{value}")
}

pub fn params_query_body(params: HashMap<&str, &str>) -> String
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
        "query": format!(r#"{{ 
            results: autoComplete(input: {{ 
                type: TAG,
                query: "*{query}*",
                limit: {limit},
            }}) {{
                __typename
                entities {{ 
                    urn
                    ... on Tag {{
                        urn
                        properties {{ 
                            name
                            description
                        }}
                    }}
                }} 
            }}
        }}"#)
    })
}

fn build_tags_query(start: &str, limit: &str) -> serde_json::Value
{
    json!({
        "query": format!(r#"{{ 
            results: search(input: {{ 
                type: TAG,
                query: "*",
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
                                description
                            }}
                        }}
                    }}
                }}
            }}
        }}"#)
    })
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
                    .map(TagEnvelope::from_tag)
                    .collect()
            },
            Self::SearchResults { entities, .. } => {
                entities.iter()
                    .map(|e| &e.entity)
                    .map(TagEnvelope::from_tag)
                    .collect()
            },
        }
    }
}

#[derive(Serialize)]
struct CreateTag {
    entity: TagValue,
}

#[derive(Serialize)]
struct TagValue {
    value: TagSnapshot,
}

#[derive(Serialize)]
struct TagSnapshot {
    #[serde(rename(serialize = "com.linkedin.metadata.snapshot.TagSnapshot"))]
    snapshot: TagData,
}

#[derive(Serialize)]
struct TagData {
    urn: String,
    aspects: Vec<TagAspect>,
}

#[derive(Serialize)]
struct TagAspect {
    #[serde(rename(serialize = "com.linkedin.tag.TagProperties"))]
    properties: TagProperties,
}

#[derive(Serialize)]
struct TagProperties {
    name: String,
    description: String,
}