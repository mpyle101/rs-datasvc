use serde::Deserialize;


#[derive(Deserialize)]
pub struct CreateTag {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct AddTag {
    pub tag: String,
}