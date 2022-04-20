use serde::Deserialize;

#[derive(Deserialize)]
pub struct Tags {
    pub tags: Vec<TagEntity>,
}

#[derive(Deserialize)]
pub struct TagEntity {
    pub tag: Tag,
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