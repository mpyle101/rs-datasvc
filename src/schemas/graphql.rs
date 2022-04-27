use serde::Serialize;

#[derive(Serialize)]
pub struct GraphQL {
    query: String,
    variables: Variables,
}

#[derive(Serialize)]
pub enum Variables {
    Urn(String),
    SearchInput(SearchInput),
    AutoCompleteInput(AutoCompleteInput),
    TagAssociationInput(TagAssociationInput),
    ListRecommendationsInput(ListRecommendationsInput),
}

#[derive(Serialize)]
pub struct AutoCompleteInput {
    #[serde(rename = "type")]
    class: String,
    query: String,
    limit: i32,
}

#[derive(Serialize)]
pub struct SearchInput {
    #[serde(rename = "type")]
    class: String,
    query: String,
    start: i32,
    limit: i32,
    filters: Option<Filter>,
}

#[derive(Serialize)]
pub struct Filter {
    field: String,
    value: String,
}

#[derive(Serialize)]
pub struct TagAssociationInput {
    #[serde(rename(serialize = "tagUrn"))]
    tag: String,

    #[serde(rename(serialize = "resourceUrn"))]
    resource: String
}

#[derive(Serialize)]
pub struct ListRecommendationsInput {
    #[serde(rename(serialize = "userUrn"))]
    user: String,

    #[serde(rename(serialize = "requestContext"))]
    context: RequestContext,
}

#[derive(Serialize)]
struct RequestContext {
    scenario: String,
}

impl GraphQL {
    pub fn new(query: &str, vars: Variables) -> GraphQL
    {
        GraphQL {
            query: query.to_string(),
            variables: vars
        }
    }
}

impl std::fmt::Display for GraphQL {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string(self) {
            Ok(s)   => write!(f, "{s}"),
            Err(..) => write!(f, "")
        }
    }
}

impl AutoCompleteInput {
    pub fn new(class: &str, query: String, limit: i32) -> AutoCompleteInput
    {
        AutoCompleteInput { 
            class: class.to_string(),
            query,
            limit
        }
    }
}

impl SearchInput {
    pub fn new(
        class: &str,
        query: String,
        start: i32,
        limit: i32,
        filters: Option<Filter>
    ) -> SearchInput
    {
        SearchInput { 
            class: class.to_string(), 
            query, 
            start,
            limit,
            filters
        }
    }
}

impl Filter {
    pub fn new(field: String, value: String) -> Filter
    {
        Filter { field, value }
    }
}

impl TagAssociationInput {
    pub fn new(resource: String, tag: String) -> TagAssociationInput
    {
        TagAssociationInput { tag, resource }
    }
}

impl ListRecommendationsInput {
    pub fn new(user: String) -> ListRecommendationsInput
    {
        ListRecommendationsInput {
            user,
            context: RequestContext { 
                scenario: "HOME".into()
            }
        }
    }
}