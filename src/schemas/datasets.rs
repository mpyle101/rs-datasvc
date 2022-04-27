use serde::Serialize;
use std::convert::From;

use crate::schemas::{datahub, paging, tags};


#[derive(Serialize)]
pub struct Datasets {
    data: Vec<DatasetEnvelope>,
    paging: Option<paging::Paging>,
}

#[derive(Serialize)]
pub struct DatasetEnvelope {
    pub dataset: Option<Dataset>
}

#[derive(Serialize)]
pub struct Dataset {
    id: String,
    path: String,
    name: Option<String>,
    origin: Option<String>,
    platform: Option<String>,

    #[serde(rename(serialize = "platformType"))]
    platform_type: Option<String>,

    #[serde(rename(serialize = "platformName"))]
    platform_name: Option<String>,

    #[serde(rename(serialize = "type"))]
    class: Option<String>,

    tags: Vec<tags::TagEnvelope>,
    fields: Option<Vec<Field>>,
}

#[derive(Serialize)]
struct Field {
    path: String,

    #[serde(rename(serialize = "type"))]
    class: String,

    #[serde(rename(serialize = "nativeType"))]
    native: String,
}

impl From<&datahub::DatasetField> for Field {
    fn from(field: &datahub::DatasetField) -> Self
    {
        Field {
            path: field.path.to_owned(),
            class: field.class.to_owned(),
            native: field.native.to_owned(),
        }
    }
}

impl From<&datahub::Entity> for DatasetEnvelope {
    fn from(e: &datahub::Entity) -> Self
    {
        match e {
            datahub::Entity::Dataset(dataset) => DatasetEnvelope { 
                dataset: Some(Dataset::from(dataset))
            },
            _ => panic!()
        }
        
    }
}

impl From<&datahub::DatasetEntity> for DatasetEnvelope {
    fn from(e: &datahub::DatasetEntity) -> Self
    {
        DatasetEnvelope { 
            dataset: e.dataset.as_ref().map(Dataset::from)
        }
    }
}

impl From<&datahub::Dataset> for DatasetEnvelope {
    fn from(ds: &datahub::Dataset) -> Self
    {
        DatasetEnvelope { 
            dataset: Some(Dataset::from(ds))
        }
    }
}

impl From<&datahub::Dataset> for Dataset {
    fn from(e: &datahub::Dataset) -> Self
    {
        Dataset {
            id: e.urn.to_owned(),
            path: e.name.to_owned(),
            name: e.properties.as_ref()
                .map(|p| p.name.to_owned()),
            class: e.sub_types.as_ref()
                .map(|st| st.names[0].to_owned()),
            origin: e.properties.as_ref()
                .map(|p| p.origin.to_owned()),
            platform: e.platform.as_ref()
                .map(|p| p.name.to_owned()),
            platform_name: e.platform.as_ref()
                .map(|p| p.properties.name.to_owned()),
            platform_type: e.platform.as_ref()
                .map(|p| p.properties.class.to_owned()),
            tags: e.tags.as_ref()
                .map_or_else(Vec::new, |tags| tags.tags.iter()
                    .map(tags::TagEnvelope::from)
                    .collect()
                ),
            fields: e.schema.as_ref()
                .map(|schema| schema.fields.iter()
                    .map(Field::from)
                    .collect()
                ),
        }
    }
}

impl From<&datahub::QueryResponse> for Datasets {
    fn from(resp: &datahub::QueryResponse) -> Self
    {
        let (data, paging) = resp.process::<DatasetEnvelope>();
        Datasets { data, paging }
    }
}