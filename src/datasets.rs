use std::convert::From;
use std::collections::HashMap;

use axum::http::Response;
use hyper::Body;
use serde::{Serialize, Deserialize};
use serde_json::Result;

use crate::datahub as dh;
use crate::tags;

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
    data: Vec<DatasetEnvelope>,
    paging: Option<Paging>,
}

#[derive(Serialize)]
pub struct DatasetEnvelope {
    pub dataset: Option<Dataset>
}

#[derive(Serialize)]
pub struct Dataset {
    id: String,
    path: String,
    name: Option<String>,

    #[serde(rename(serialize = "type"))]
    class: Option<String>,

    tags: Vec<tags::TagEnvelope>,
    fields: Option<Vec<Field>>,
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

impl From<&dh::DatasetField> for Field {
    fn from(dsf: &dh::DatasetField) -> Self
    {
        Field {
            path: dsf.path.to_owned(),
            class: dsf.class.to_owned(),
            native: dsf.native.to_owned(),
        }
    }
}

impl From<&dh::DatasetEntity> for DatasetEnvelope {
    fn from(e: &dh::DatasetEntity) -> Self
    {
        DatasetEnvelope { 
            dataset: e.dataset.as_ref().map(|ds| Dataset::from(ds))
        }
    }
}
impl From<&dh::Dataset> for DatasetEnvelope {
    fn from(ds: &dh::Dataset) -> Self
    {
        DatasetEnvelope { 
            dataset: Some(Dataset::from(ds))
        }
    }
}

impl From<&dh::Dataset> for Dataset {
    fn from(e: &dh::Dataset) -> Self
    {
        Dataset {
            id: e.urn.to_owned(),
            path: e.name.to_owned(),
            name: e.properties.as_ref()
                .map(|props| props.name.to_owned()),
            class: e.sub_types.as_ref()
                .map(|st| st.names[0].to_owned()),
            tags: e.tags.as_ref()
                .map_or_else(Vec::new, |tags| tags.tags.iter()
                    .map(tags::TagEnvelope::from)
                    .collect()
                ),
            fields: e.schema.as_ref()
                .map(|schema| schema.fields.iter()
                    .map(Field::from)
                    .collect()
                ),
        }
    }
}

pub async fn by_id(resp: Response<Body>) -> Result<DatasetEnvelope>
{
    let bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    let body: DatasetResponse = serde_json::from_slice(&bytes)?;

    Ok(DatasetEnvelope::from(&body.data))
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
    format!(r#"{{
        "query": "{{ 
            dataset(urn: \"{id}\") {{
                {DATASET_QUERY}
            }}
        }}"
    }}"#)
    .replace("\n", "")
}

pub fn params_query_body(params: HashMap<&str, &str>) -> String
{
    let start = params.get("offset").unwrap_or(&"0");
    let limit = params.get("limit").unwrap_or(&"10");

    if let Some(query) = params.get("query") {
        build_name_query(query, limit)
    } else if let Some(tags) = params.get("tags") {
        build_tags_query(tags, start, limit)
    } else {
        build_datasets_query(start, limit)
    }
}

fn build_name_query(query: &str, limit: &str) -> String
{
    format!(r#"{{
        "query": "{{ 
            results: autoComplete(input: {{ 
                type: DATASET,
                query: \"*{query}*\",
                limit: {limit},
            }}) {{
                __typename
                entities {{
                    {DATASET_QUERY}
                }}
            }}
        }}"
    }}"#)
    .replace("\n", "")
}

fn build_tags_query(
    tags: &str,
    start: &str,
    limit: &str
) -> String
{
    let query = tags.replace(',', " OR ");
    format!(r#"{{
        "query": "{{ 
            results: search(input: {{ 
                type: DATASET,
                query: \"tags:{query}\",
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
        }})
    }}"#)
    .replace("\n", "")
}

fn build_datasets_query(start: &str, limit: &str) -> String
{
    format!(r#"{{
        "query": "{{ 
            results: search(input: {{ 
                type: DATASET,
                query: \"*\",
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
        }}"
    }}"#)
    .replace("\n", "")
}

#[derive(Deserialize)]
struct DatasetResponse {
    data: dh::DatasetEntity,
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
        entities: Vec<dh::Dataset>
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
    entity: dh::Dataset,
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

    fn process(&self) -> Vec<DatasetEnvelope>
    {
        match self {
            Self::AutoCompleteResults { entities } => {
                entities.iter()
                    .map(DatasetEnvelope::from)
                    .collect()
            },
            Self::SearchResults { entities, .. } => {
                entities.iter()
                    .map(|e| &e.entity)
                    .map(DatasetEnvelope::from)
                    .collect()
            },
        }
    }
}

