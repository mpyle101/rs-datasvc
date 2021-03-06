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

impl<'a> From<&'a datahub::Entity<'a>> for TagEnvelope {
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

impl<'a> From<&datahub::TagEntity<'a>> for TagEnvelope {
    fn from(e: &datahub::TagEntity) -> Self
    {
        TagEnvelope { 
            tag: e.entity.as_ref().map(Tag::from)
        }
    }
}

impl<'a> From<&datahub::Tag<'a>> for TagEnvelope {
    fn from(tag: &datahub::Tag) -> Self
    {
        TagEnvelope { 
            tag: Some(Tag::from(tag))
        }
    }
}

impl<'a> From<&datahub::Tag<'a>> for Tag {
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

impl<'a> From<&datahub::QueryResponse<'a>> for Tags {
    fn from(resp: &datahub::QueryResponse) -> Self
    {
        let (data, paging) = resp.process::<TagEnvelope>();
        Tags { data, paging }
    }
}