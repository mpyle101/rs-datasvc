use std::convert::From;
use serde::Serialize;

use crate::schemas::{datahub, paging};

#[derive(Serialize)]
pub struct Platforms {
    data: Vec<PlatformEnvelope>,
    paging: Option<paging::Paging>,
}

#[derive(Serialize)]
pub struct PlatformEnvelope {
    pub platform: Platform
}

#[derive(Serialize)]
pub struct Platform {
    pub id: String,
    pub name: String,
    pub title: String,

    #[serde(rename(serialize = "type"))]
    pub class: String,
}

impl<'a> From<&datahub::PlatformEntity<'a>> for PlatformEnvelope {
    fn from(e: &datahub::PlatformEntity) -> Self
    {
        PlatformEnvelope { platform: Platform::from(e) }
    }
}

impl<'a> From<&datahub::PlatformEntity<'a>> for Platform {
    fn from(e: &datahub::PlatformEntity) -> Self
    {
        Platform {
            id: e.entity.urn.to_owned(),
            name: e.entity.name.to_owned(),
            title: e.entity.properties.name.to_owned(),
            class: e.entity.properties.class.to_owned(),
        }
    }
}

impl<'a> From<&datahub::ListRecommendationsResponse<'a>> for Platforms {
    fn from(resp: &datahub::ListRecommendationsResponse) -> Self
    {
        let (data, paging) = resp.process();
        Platforms { data, paging }
    }
}