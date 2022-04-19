use std::collections::HashMap;

use axum::http::Response;
use hyper::Body;
use serde::{Serialize, Deserialize};
use serde_json::{Result, json};

#[derive(Serialize)]
pub(crate) struct Datasets {
    data: Vec<Dataset>,
    paging: Option<Paging>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Dataset {
    id: String,
    name: String,
    sub_type: Option<String>,
    short_name: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct Paging {
    total: i32,
    limit: i32,
    offset: i32,
}

pub(crate) async fn by_query(resp: Response<Body>) -> Result<Datasets>
{
    let bytes = hyper::body::to_bytes(resp.into_body())
        .await
        .unwrap();
    let body: QueryResponse = serde_json::from_slice(&bytes).unwrap();

    let data   = body.data.results.process();
    let paging = body.data.results.paging();

    Ok(Datasets { data, paging })
}

pub(crate) fn build_query(params: HashMap<&str, &str>) -> String
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
        "query": format!("{{ 
            results: autoComplete(input: {{ 
                type: DATASET,
                query: \"{query}\",
                limit: {limit},
            }}) {{
                __typename
                entities {{ 
                    urn
                    ... on Dataset {{ 
                        name
                        properties {{ 
                            name
                        }}
                        sub_types: subTypes {{
                            type_names: typeNames
                        }}
                    }}
                }}
            }}
        }}")
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
        "query": format!("{{ 
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
                        urn
                        ... on Dataset {{ 
                            name
                            properties {{ 
                                name
                            }}
                            sub_types: subTypes {{
                                type_names: typeNames
                            }}
                        }}
                    }}
                }}
            }}
        }}")
    })
}

fn build_datasets_query(start: &str, limit: &str) -> serde_json::Value
{
    json!({
        "query": format!("{{ 
            results: search(input: {{ 
                type: DATASET,
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
                        ... on Dataset {{ 
                            name
                            properties {{ 
                                name
                            }}
                            sub_types: subTypes {{
                                type_names: typeNames
                            }}
                        }}
                    }}
                }}
            }}
        }}")
    })
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
    sub_types: Option<DatasetSubType>,
    properties: Option<DatasetProperties>,
}

#[derive(Deserialize)]
struct DatasetSubType {
    type_names: Vec<String>
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
                    .map(|e| Dataset {
                            id: e.urn.to_owned(),
                            name: e.name.to_owned(),
                            sub_type: e.sub_types.as_ref()
                                .map(|st| st.type_names[0].to_owned()),
                            short_name: e.properties.as_ref()
                                .map(|props| props.name.to_owned()),
                        }
                    )
                .collect()
            },
            Self::SearchResults { entities, .. } => {
                entities.iter()
                    .map(|e| Dataset {
                            id: e.entity.urn.to_owned(),
                            name: e.entity.name.to_owned(),
                            sub_type: e.entity.sub_types.as_ref()
                                .map(|st| st.type_names[0].to_owned()),
                            short_name: e.entity.properties.as_ref()
                                .map(|props| props.name.to_owned()),
                        }
                    )
                .collect()
            },
        }
    }
}

