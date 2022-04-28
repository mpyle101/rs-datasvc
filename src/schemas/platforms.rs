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

impl From<&datahub::PlatformEntity> for PlatformEnvelope {
    fn from(e: &datahub::PlatformEntity) -> Self
    {
        PlatformEnvelope { platform: Platform::from(e) }
    }
}

impl From<&datahub::PlatformEntity> for Platform {
    fn from(e: &datahub::PlatformEntity) -> Self
    {
        Platform {
            id: e.platform.urn.to_owned(),
            name: e.platform.name.to_owned(),
            title: e.platform.properties.name.to_owned(),
            class: e.platform.properties.class.to_owned(),
        }
    }
}

impl From<&datahub::ListRecommendationsResponse> for Platforms {
    fn from(resp: &datahub::ListRecommendationsResponse) -> Self
    {
        let (data, paging) = resp.process();
        Platforms { data, paging }
    }
}