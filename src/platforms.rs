use std::convert::From;
use std::collections::HashMap;

use axum::http::Response;
use hyper::Body;
use serde::{Serialize, Deserialize};
use serde_json::Result;

use crate::datahub as dh;

const DATASET_QUERY: &str = dh::DATASET_QUERY;
const PLATFORM_QUERY: &str = "
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

#[derive(Serialize)]
pub struct Platforms {
    data: Vec<PlatformEnvelope>,
    paging: Option<Paging>,
}

#[derive(Serialize)]
pub struct PlatformEnvelope {
    pub platform: Platform
}

#[derive(Serialize)]
pub struct Platform {
    pub id: String,
    pub name: String,
    pub title: String,

    #[serde(rename(serialize = "type"))]
    pub class: String,
}

#[derive(Serialize)]
pub struct Paging {
    total: i32,
    limit: i32,
    offset: i32,
}

impl From<&PlatformEntity> for PlatformEnvelope {
    fn from(e: &PlatformEntity) -> Self
    {
        PlatformEnvelope { platform: Platform::from(e) }
    }
}

impl From<&PlatformEntity> for Platform {
    fn from(e: &PlatformEntity) -> Self
    {
        Platform {
            id: e.entity.urn.to_owned(),
            name: e.entity.name.to_owned(),
            title: e.entity.properties.name.to_owned(),
            class: e.entity.properties.class.to_owned(),
        }
    }
}

pub async fn by_id(resp: Response<Body>) -> Result<PlatformEnvelope>
{
    let bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    let body: PlatformResponse = serde_json::from_slice(&bytes).unwrap();

    Ok(PlatformEnvelope::from(&body.data))
}

pub async fn by_query(resp: Response<Body>) -> Result<Platforms>
{
    let bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    let body: ListRecommendationsResponse = serde_json::from_slice(&bytes).unwrap();

    let data = body.data.results.modules.iter()
        .flat_map(|m| m.process())
        .collect();

    Ok(Platforms { data, paging: None })
}

pub fn id_query_body(id: &str) -> String
{
    format!(r#"{{
        "query": "{{ entity: dataPlatform(urn: \"{id}\") {{ {PLATFORM_QUERY} }} }}"
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
                filters: {{ field: \"platform\", value: \"{id}\" }}
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
    let user  = params.get("urn").unwrap_or(&"urn:li:corpuser:datahub");
    let limit = params.get("limit").unwrap_or(&"10");

    format!(r#"{{
        "query": "{{ 
            results: listRecommendations(input: {{ 
                userUrn: \"{user}\", limit: {limit},
                requestContext: {{ scenario: HOME }}
            }}) {{
                modules {{
                    id: moduleId
                    content {{ entity {{ {PLATFORM_QUERY} }} }}
                }}
            }}
        }}"
    }}"#)
    .replace('\n', "")
}

#[derive(Deserialize)]
struct PlatformResponse {
    data: PlatformEntity,
}

#[derive(Deserialize)]
struct ListRecommendationsResponse {
    data: RecommendationsResponseData
}

#[derive(Deserialize)]
struct RecommendationsResponseData {
    results: RecommendationsModules
}

#[derive(Deserialize)]
struct RecommendationsModules {
    modules: Vec<RecommendationsModule>,
}

#[derive(Deserialize)]
#[serde(tag = "id")]
enum RecommendationsModule {
    Platforms { 
        content: Vec<PlatformEntity>
    },
    RecentlyViewedEntities,
    HighUsageEntities,
    TopTags,
}

#[derive(Deserialize)]
struct PlatformEntity {
    entity: PlatformData,
}

#[derive(Deserialize)]
struct PlatformData {
    urn: String,
    name: String,
    properties: PlatformProperties
}

#[derive(Deserialize)]
struct PlatformProperties {
    name: String,
    class: String,
}

impl RecommendationsModule {
    fn process(&self) -> Vec<PlatformEnvelope>
    {
        match self {
            Self::Platforms { content } => {
                content.iter()
                    .map(PlatformEnvelope::from)
                    .collect()
            },
            _ => vec![]
        }
    }
}
