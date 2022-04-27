use std::convert::From;
use serde::Serialize;

use crate::schemas::{datahub, paging};

#[derive(Serialize)]
pub struct Tags {
    data: Vec<TagEnvelope>,
    paging: Option<paging::Paging>,
}

#[derive(Serialize)]
pub struct TagEnvelope {
    tag: Option<Tag>
}

#[derive(Serialize)]
pub struct Tag {
    id: String,
    name: Option<String>,
    description: Option<String>,
}

impl Tag {
    pub fn new(id: String, name: Option<String>, description: Option<String>) -> Tag
    {
        Tag { id, name, description }
    }
}

impl From<&datahub::Entity> for TagEnvelope {
    fn from(e: &datahub::Entity) -> Self
    {
        match e {
            datahub::Entity::Tag(tag) => TagEnvelope { 
                tag: Some(Tag::from(tag))
            },
            _ => panic!()
        }
        
    }
}

impl From<&datahub::TagEntity> for TagEnvelope {
    fn from(e: &datahub::TagEntity) -> Self
    {
        TagEnvelope { 
            tag: e.tag.as_ref().map(Tag::from)
        }
    }
}

impl From<&datahub::Tag> for TagEnvelope {
    fn from(tag: &datahub::Tag) -> Self
    {
        TagEnvelope { 
            tag: Some(Tag::from(tag))
        }
    }
}

impl From<&datahub::Tag> for Tag {
    fn from(tag: &datahub::Tag) -> Self
    {
        Tag {
            id: tag.urn.to_owned(),
            name: tag.properties.as_ref()
                .map(|props| props.name.to_owned()),
            description: tag.properties.as_ref()
                .map(|props| props.description.to_owned()),
        }
    }
}

impl From<&datahub::QueryResponse> for Tags {
    fn from(resp: &datahub::QueryResponse) -> Self
    {
        let (data, paging) = resp.process::<TagEnvelope>();
        Tags { data, paging }
    }
}