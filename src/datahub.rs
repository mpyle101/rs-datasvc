use serde::Deserialize;

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
    pub sub_types: Option<DatasetSubType>,
    pub properties: Option<DatasetProperties>,
}

#[derive(Deserialize)]
pub struct DatasetSchema {
    pub fields: Vec<DatasetField>,
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
}


#[derive(Deserialize)]
pub struct ErrorMessage {
    pub message: String
}