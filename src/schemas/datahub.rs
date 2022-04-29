use std::{fmt, convert::From};
use serde::{Deserialize, Serialize};

use crate::schemas::Paging;
use crate::schemas::{PlatformEnvelope};

#[derive(Deserialize)]
pub struct Tags<'a> {
    #[serde(borrow)]
    pub tags: Vec<TagEntity<'a>>,
}

#[derive(Deserialize)]
pub struct TagEntity<'a> {
    #[serde(borrow)]
    pub entity: Option<Tag<'a>>,
}

#[derive(Deserialize)]
pub struct Tag<'a> {
    pub urn: &'a str,
    pub properties: Option<TagProperties<'a>>,
}

#[derive(Deserialize)]
pub struct TagProperties<'a> {
    pub name: &'a str,
    pub description: &'a str,
}

#[derive(Deserialize)]
pub struct DatasetEntity<'a> {
    #[serde(borrow)]
    pub entity: Option<Dataset<'a>>,
}

#[derive(Deserialize)]
pub struct Dataset<'a> {
    pub urn: &'a str,
    pub name: &'a str,
    pub tags: Option<Tags<'a>>,
    pub schema: Option<DatasetSchema<'a>>,
    pub platform: Option<DatasetPlatform<'a>>,
    pub sub_types: Option<DatasetSubType<'a>>,
    pub properties: Option<DatasetProperties<'a>>,
}

#[derive(Deserialize)]
pub struct DatasetSchema<'a> {
    #[serde(borrow)]
    pub fields: Vec<DatasetField<'a>>,
}

#[derive(Deserialize)]
pub struct DatasetPlatform<'a> {
    pub name: &'a str,
    pub properties: DatasetPlatformProperties<'a>,
}

#[derive(Deserialize)]
pub struct DatasetPlatformProperties<'a> {
    pub name: &'a str,
    pub class: &'a str,
}

#[derive(Deserialize)]
pub struct DatasetField<'a> {
    pub path: &'a str,
    pub class: &'a str,
    pub native: &'a str,
}

#[derive(Deserialize)]
pub struct DatasetSubType<'a> {
    #[serde(borrow)]
    pub names: Vec<&'a str>
}

#[derive(Deserialize)]
pub struct DatasetProperties<'a> {
    pub name: &'a str,
    pub origin: &'a str,
}

#[derive(Deserialize)]
pub struct ErrorMessage<'a> {
    pub message: &'a str
}


#[derive(Deserialize)]
pub struct QueryResponse<'a> {
    #[serde(borrow)]
    data: QueryResponseData<'a>
}

#[derive(Deserialize)]
struct QueryResponseData<'a> {
    #[serde(borrow)]
    results: QueryResults<'a>
}

#[derive(Deserialize)]
#[serde(tag = "__typename")]
enum QueryResults<'a> {
    AutoCompleteResults { 
        entities: Vec<Entity<'a>>
    },
    SearchResults {
        start: i32,
        count: i32,
        total: i32,

        #[serde(borrow)]
        entities: Vec<EntityEnvelope<'a>>
    },
}

#[derive(Deserialize)]
struct EntityEnvelope<'a> {
    #[serde(borrow)]
    entity: Entity<'a>,
}

#[derive(Deserialize)]
#[serde(tag = "__typename")]
pub enum Entity<'a> {
    Tag(Tag<'a>),

    #[serde(borrow)]
    Dataset(Dataset<'a>),
}

impl<'a> QueryResponse<'a> {
    pub fn process<T>(&'a self) -> (Vec<T>, Option<Paging>)
        where T: From<&'a Entity<'a>>
    {
        (
            self.data.results.process::<T>(),
            self.data.results.paging()
        )
    }
}

impl<'a> QueryResults<'a> {
    fn paging(&self) -> Option<Paging> {
        match self {
            Self::AutoCompleteResults { .. } => None,
            Self::SearchResults { start, count, total, .. } => {
                Some(Paging::new(*start, *count, *total))
            },
        }
    }

    fn process<T>(&'a self) -> Vec<T>
        where T: From<&'a Entity<'a>>
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
pub struct DatasetAddTagResponse<'a> {
    pub data: Option<DatasetAddTagResult>,

    #[serde(borrow)]
    pub errors: Option<Vec<ErrorMessage<'a>>>,
}

#[derive(Deserialize)]
pub struct DatasetAddTagResult {
    pub success: bool
}


#[derive(Serialize)]
pub struct CreateTag<'a> {
    entity: Value<'a>,
}

impl<'a> CreateTag<'a> {
    pub fn new(urn: &'a str, name: &'a str, description: &'a str) -> CreateTag<'a>
    {
        CreateTag {
            entity: Value { 
                value: Snapshot { 
                    snapshot: SnapshotValues {
                        urn,
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

impl<'a> fmt::Display for CreateTag<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(s)   => write!(f, "{s}"),
            Err(..) => write!(f, "")
        }
    }
}

#[derive(Serialize)]
pub struct DeleteTag<'a> {
    entity: Value<'a>,
}

impl<'a> DeleteTag<'a> {
    pub fn new(urn: &str) -> DeleteTag
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

impl<'a> fmt::Display for DeleteTag<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(s)   => write!(f, "{s}"),
            Err(..) => write!(f, "")
        }
    }
}

#[derive(Serialize)]
struct Value<'a> {
    value: Snapshot<'a>,
}

#[derive(Serialize)]
struct Snapshot<'a> {
    #[serde(rename(serialize = "com.linkedin.metadata.snapshot.TagSnapshot"))]
    snapshot: SnapshotValues<'a>,
}

#[derive(Serialize)]
struct SnapshotValues<'a> {
    urn: &'a str,
    aspects: Vec<Aspect<'a>>,
}

#[derive(Serialize)]
enum Aspect<'a> {
    #[serde(rename(serialize = "com.linkedin.common.Status"))]
    Status { removed: bool },
    
    #[serde(rename(serialize = "com.linkedin.tag.TagProperties"))]
    Properties {
        name: &'a str,
        description: &'a str
    }
}


#[derive(Deserialize)]
pub struct ListRecommendationsResponse<'a> {
    #[serde(borrow)]
    data: RecommendationsResponseData<'a>
}

#[derive(Deserialize)]
struct RecommendationsResponseData<'a> {
    #[serde(borrow)]
    results: RecommendationsModules<'a>
}

#[derive(Deserialize)]
struct RecommendationsModules<'a> {
    #[serde(borrow)]
    modules: Vec<RecommendationsModule<'a>>,
}

#[derive(Deserialize)]
#[serde(tag = "id")]
enum RecommendationsModule<'a> {
    Platforms { 
        #[serde(borrow)]
        content: Vec<PlatformEntity<'a>>
    },
    RecentlyViewedEntities,
    HighUsageEntities,
    TopTags,
}

#[derive(Deserialize)]
pub struct PlatformEntity<'a> {
    #[serde(borrow)]
    pub entity: DataPlatform<'a>,
}

#[derive(Deserialize)]
pub struct DataPlatform<'a> {
    pub urn: &'a str,
    pub name: &'a str,
    pub properties: PlatformProperties<'a>
}

#[derive(Deserialize)]
pub struct PlatformProperties<'a> {
    pub name: &'a str,
    pub class: &'a str,
}

impl<'a> ListRecommendationsResponse<'a> {
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

impl<'a> RecommendationsModule<'a> {
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
