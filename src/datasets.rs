use std::collections::HashMap;

use axum::http::Response;
use hyper::Body;
use serde::{Serialize, Deserialize};
use serde_json::{Result, json};

use crate::datahub;

const DATASET_QUERY: &str = "
    urn
    __typename
    ... on Dataset {
        name
        properties {
            name
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

#[derive(Serialize)]
pub struct Datasets {
    data: Vec<Dataset>,
    paging: Option<Paging>,
}

#[derive(Serialize)]
pub struct Dataset {
    id: String,
    path: String,
    name: Option<String>,

    #[serde(rename(serialize = "type"))]
    class: Option<String>,

    tags: Vec<Tag>,
    fields: Option<Vec<Field>>,
}

#[derive(Serialize)]
pub struct Tag {
    id: String,
    name: String,
}

#[derive(Serialize)]
pub struct Field {
    path: String,

    #[serde(rename(serialize = "type"))]
    class: String,

    #[serde(rename(serialize = "nativeType"))]
    native: String,
}

#[derive(Serialize)]
pub struct Paging {
    total: i32,
    limit: i32,
    offset: i32,
}

impl Field {
    fn from_dsfield(dsf: &DatasetField) -> Field
    {
        Field {
            path: dsf.path.to_owned(),
            class: dsf.class.to_owned(),
            native: dsf.native.to_owned(),
        }
    }
}

impl Tag {
    fn from_entity(entity: &datahub::TagEntity) -> Tag
    {
        Tag {
            id: entity.tag.urn.to_owned(),
            name: entity.tag.properties.as_ref()
                .map_or_else(String::new, |props| props.name.to_owned()),
        }
    }
}

impl Dataset {
    fn from_entity(entity: &DatasetEntity) -> Dataset
    {
        Dataset {
            id: entity.urn.to_owned(),
            path: entity.name.to_owned(),
            name: entity.properties.as_ref()
                .map(|props| props.name.to_owned()),
            class: entity.sub_types.as_ref()
                .map(|st| st.names[0].to_owned()),
            tags: entity.tags.as_ref()
                .map_or_else(Vec::new, |tags| tags.tags.iter()
                    .map(Tag::from_entity)
                    .collect()
                ),
            fields: entity.schema.as_ref()
                .map(|schema| schema.fields.iter()
                    .map(Field::from_dsfield)
                    .collect()
                ),
        }
    }
}

pub async fn by_id(resp: Response<Body>) -> Result<Dataset>
{
    let bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    let body: DatasetResponse = serde_json::from_slice(&bytes).unwrap();
    let entity = body.data.dataset;

    Ok(Dataset::from_entity(&entity))
}

pub async fn by_query(resp: Response<Body>) -> Result<Datasets>
{
    let bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    let body: QueryResponse = serde_json::from_slice(&bytes).unwrap();

    let data   = body.data.results.process();
    let paging = body.data.results.paging();

    Ok(Datasets { data, paging })
}

pub fn id_query_body(id: &str) -> String
{
    let value = json!({
        "query": format!(r#"{{ 
            dataset(urn: "{id}") {{
                {DATASET_QUERY}
            }}
        }}"#)
    });

    format!("{value}")
}

pub fn params_query_body(params: HashMap<&str, &str>) -> String
{
    let start = params.get("offset").unwrap_or(&"0");
    let limit = params.get("limit").unwrap_or(&"10");
    let value = if let Some(query) = params.get("query") {
        build_name_query(query, limit)
    } else if let Some(tags) = params.get("tags") {
        build_tags_query(tags, start, limit)
    } else {
        build_datasets_query(start, limit)
    };

    format!("{value}")
}

fn build_name_query(query: &str, limit: &str) -> serde_json::Value
{
    json!({
        "query": format!(r#"{{ 
            results: autoComplete(input: {{ 
                type: DATASET,
                query: "{query}",
                limit: {limit},
            }}) {{
                __typename
                entities {{
                    {DATASET_QUERY}
                }}
            }}
        }}"#)
    })
}

fn build_tags_query(
    tags: &str,
    start: &str,
    limit: &str
) -> serde_json::Value
{
    let query = tags.replace(',', " OR ");
    json!({
        "query": format!(r#"{{ 
            results: search(input: {{ 
                type: DATASET,
                query: "tags:{query}",
                start: {start},
                count: {limit},
            }}) {{
                __typename
                start,
                count,
                total,
                entities: searchResults {{
                    entity {{
                        {DATASET_QUERY}
                    }}
                }}
            }}
        }}"#)
    })
}

fn build_datasets_query(start: &str, limit: &str) -> serde_json::Value
{
    json!({
        "query": format!(r#"{{ 
            results: search(input: {{ 
                type: DATASET,
                query: "",
                start: {start},
                count: {limit},
            }}) {{
                __typename
                start
                count
                total
                entities: searchResults {{
                    entity {{
                        {DATASET_QUERY}
                    }}
                }}
            }}
        }}"#)
    })
}

#[derive(Deserialize)]
struct DatasetResponse {
    data: DatasetResponseData,
}

#[derive(Deserialize)]
struct DatasetResponseData {
    dataset: DatasetEntity,
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
        entities: Vec<DatasetEntity>
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
    entity: DatasetEntity,
}

#[derive(Deserialize)]
struct DatasetEntity {
    urn: String,
    name: String,
    tags: Option<datahub::Tags>,
    schema: Option<DatasetSchema>,
    sub_types: Option<DatasetSubType>,
    properties: Option<DatasetProperties>,
}

#[derive(Deserialize)]
struct DatasetSchema {
    fields: Vec<DatasetField>,
}

#[derive(Deserialize)]
struct DatasetField {
    path: String,
    class: String,
    native: String,
}

#[derive(Deserialize)]
struct DatasetSubType {
    names: Vec<String>
}

#[derive(Deserialize)]
struct DatasetProperties {
    name: String,
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

    fn process(&self) -> Vec<Dataset>
    {
        match self {
            Self::AutoCompleteResults { entities } => {
                entities.iter()
                    .map(Dataset::from_entity)
                    .collect()
            },
            Self::SearchResults { entities, .. } => {
                entities.iter()
                    .map(|e| Dataset::from_entity(&e.entity))
                    .collect()
            },
        }
    }
}

