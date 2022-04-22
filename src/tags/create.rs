use std::fmt;

use serde::Serialize;

#[derive(Serialize)]
pub struct CreateRequest {
    entity: Value,
}

impl CreateRequest {
    pub fn with(name: String, description: String) -> CreateRequest
    {
        CreateRequest {
            entity: Value { 
                value: Snapshot { 
                    snapshot: SnapshotValues {
                        urn: format!("urn:li:tag:{name}"),
                        aspects: vec![
                            Aspect {
                                aspect: Properties {
                                    name,
                                    description,
                                }
                            }
                        ]
                    }
                }
            }
        }       
    }
}

impl fmt::Display for CreateRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(s) => write!(f, "{s}"),
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
struct Aspect {
    #[serde(rename(serialize = "com.linkedin.tag.TagProperties"))]
    aspect: Properties,
}

#[derive(Serialize)]
struct Properties {
    name: String,
    description: String,
}