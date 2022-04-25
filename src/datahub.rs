use serde::Deserialize;

pub const DATASET_QUERY: &str = "
    urn
    __typename
    ... on Dataset {
        name
        platform {
            name
            properties {
                name: displayName
                class: type
            }
        }
        properties {
            name
            origin
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