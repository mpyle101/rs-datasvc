use std::fmt;

use serde::Serialize;

#[derive(Serialize)]
pub struct RemoveRequest {
    entity: Value,
}

impl RemoveRequest {
    pub fn with(urn: String, removed: bool) -> RemoveRequest
    {
        RemoveRequest {
            entity: Value { 
                value: Snapshot { 
                    snapshot: SnapshotValues {
                        urn,
                        aspects: vec![
                            Aspect {
                                aspect: Status {
                                    removed
                                }
                            }
                        ]
                    }
                }
            }
        }       
    }
}

impl fmt::Display for RemoveRequest {
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
struct Aspect {
    #[serde(rename(serialize = "com.linkedin.common.Status"))]
    aspect: Status,
}

#[derive(Serialize)]
struct Status {
    removed: bool,
}