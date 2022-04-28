use std::{fmt, convert::From};
use serde::{Deserialize, Serialize};

use crate::schemas::Paging;
use crate::schemas::{PlatformEnvelope};

#[derive(Deserialize)]
pub struct Tags {
    pub tags: Vec<TagEntity>,
}

#[derive(Deserialize)]
pub struct TagEntity {
    pub tag: Option<Tag>,
}

#[derive(Deserialize)]
pub struct Tag {
    pub urn: String,
    pub properties: Option<TagProperties>,
}

#[derive(Deserialize)]
pub struct TagProperties {
    pub name: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct DatasetEntity {
    pub dataset: Option<Dataset>,
}

#[derive(Deserialize)]
pub struct Dataset {
    pub urn: String,
    pub name: String,
    pub tags: Option<Tags>,
    pub schema: Option<DatasetSchema>,
    pub platform: Option<DatasetPlatform>,
    pub sub_types: Option<DatasetSubType>,
    pub properties: Option<DatasetProperties>,
}

#[derive(Deserialize)]
pub struct DatasetSchema {
    pub fields: Vec<DatasetField>,
}

#[derive(Deserialize)]
pub struct DatasetPlatform {
    pub name: String,
    pub properties: DatasetPlatformProperties,
}

#[derive(Deserialize)]
pub struct DatasetPlatformProperties {
    pub name: String,
    pub class: String,
}

#[derive(Deserialize)]
pub struct DatasetField {
    pub path: String,
    pub class: String,
    pub native: String,
}

#[derive(Deserialize)]
pub struct DatasetSubType {
    pub names: Vec<String>
}

#[derive(Deserialize)]
pub struct DatasetProperties {
    pub name: String,
    pub origin: String,
}

#[derive(Deserialize)]
pub struct ErrorMessage {
    pub message: String
}


#[derive(Deserialize)]
pub struct QueryResponse {
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
        entities: Vec<Entity>
    },
    SearchResults {
        start: i32,
        count: i32,
        total: i32,
        entities: Vec<EntityEnvelope>
    },
}

#[derive(Deserialize)]
struct EntityEnvelope {
    entity: Entity,
}

#[derive(Deserialize)]
#[serde(tag = "__typename")]
pub enum Entity {
    Tag(Tag),
    Dataset(Dataset),
}

impl QueryResponse {
    pub fn process<'a, T>(&'a self) -> (Vec<T>, Option<Paging>)
        where T: From<&'a Entity>
    {
        (
            self.data.results.process::<'a, T>(),
            self.data.results.paging()
        )
    }
}

impl QueryResults {
    fn paging(&self) -> Option<Paging> {
        match self {
            Self::AutoCompleteResults { .. } => None,
            Self::SearchResults { start, count, total, .. } => {
                Some(Paging::new(*start, *count, *total))
            },
        }
    }

    fn process<'a, T>(&'a self) -> Vec<T>
        where T: From<&'a Entity>
    {
        match self {
            Self::AutoCompleteResults { entities } => {
                entities.iter()
                    .map(T::from)
                    .collect()
            },
            Self::SearchResults { entities, .. } => {
                entities.iter()
                    .map(|e| &e.entity)
                    .map(T::from)
                    .collect()
            },
        }
    }
}


#[derive(Deserialize)]
pub struct DatasetAddTagResponse {
    pub data: Option<DatasetAddTagResult>,
    pub errors: Option<Vec<ErrorMessage>>,
}

#[derive(Deserialize)]
pub struct DatasetAddTagResult {
    pub success: bool
}


#[derive(Serialize)]
pub struct CreateTag {
    entity: Value,
}

impl CreateTag {
    pub fn new(name: String, description: String) -> CreateTag
    {
        CreateTag {
            entity: Value { 
                value: Snapshot { 
                    snapshot: SnapshotValues {
                        urn: format!("urn:li:tag:{name}"),
                        aspects: vec![
                            Aspect::Properties {
                                name,
                                description,
                            }
                        ]
                    }
                }
            }
        }       
    }
}

impl fmt::Display for CreateTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(s)   => write!(f, "{s}"),
            Err(..) => write!(f, "")
        }
    }
}

#[derive(Serialize)]
pub struct DeleteTag {
    entity: Value,
}

impl DeleteTag {
    pub fn new(urn: String) -> DeleteTag
    {
        DeleteTag {
            entity: Value { 
                value: Snapshot { 
                    snapshot: SnapshotValues {
                        urn,
                        aspects: vec![
                            Aspect::Status {
                                removed: true
                            }
                        ]
                    }
                }
            }
        }       
    }
}

impl fmt::Display for DeleteTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(s)   => write!(f, "{s}"),
            Err(..) => write!(f, "")
        }
    }
}

#[derive(Serialize)]
struct Value {
    value: Snapshot,
}

#[derive(Serialize)]
struct Snapshot {
    #[serde(rename(serialize = "com.linkedin.metadata.snapshot.TagSnapshot"))]
    snapshot: SnapshotValues,
}

#[derive(Serialize)]
struct SnapshotValues {
    urn: String,
    aspects: Vec<Aspect>,
}

#[derive(Serialize)]
struct Status {
    removed: bool,
}

#[derive(Serialize)]
struct Properties {
    name: String,
    description: String,
}

#[derive(Serialize)]
enum Aspect {
    #[serde(rename(serialize = "com.linkedin.common.Status"))]
    Status { removed: bool },
    
    #[serde(rename(serialize = "com.linkedin.tag.TagProperties"))]
    Properties {
        name: String,
        description: String
    }
}


#[derive(Deserialize)]
pub struct ListRecommendationsResponse {
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
pub struct PlatformEntity {
    #[serde(rename(deserialize = "dataPlatform"))]
    pub platform: DataPlatform,
}

#[derive(Deserialize)]
pub struct DataPlatform {
    pub urn: String,
    pub name: String,
    pub properties: PlatformProperties
}

#[derive(Deserialize)]
pub struct PlatformProperties {
    pub name: String,
    pub class: String,
}

impl ListRecommendationsResponse {
    pub fn process(&self) -> (Vec<PlatformEnvelope>, Option<Paging>)
    {
        (
            self.data.results.modules.iter()
                .flat_map(|m| m.process())
                .collect(),
            None
        )
    }
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
