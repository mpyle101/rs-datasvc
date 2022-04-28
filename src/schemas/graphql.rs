use serde::Serialize;

#[derive(Serialize)]
pub struct GraphQL {
    query: String,
    variables: Variables,
}

#[derive(Serialize)]
pub enum Variables {
    #[serde(rename = "urn")]
    Urn(String),

    #[serde(rename = "input")]
    SearchInput(SearchInput),

    #[serde(rename = "input")]
    AutoCompleteInput(AutoCompleteInput),

    #[serde(rename = "input")]
    TagAssociationInput(TagAssociationInput),

    #[serde(rename = "input")]
    ListRecommendationsInput(ListRecommendationsInput),
}

#[derive(Serialize)]
pub struct AutoCompleteInput {
    limit: i32,
    query: String,

    #[serde(rename = "type")]
    class: String,
}

#[derive(Serialize)]
pub struct SearchInput {
    start: i32,
    count: i32,
    query: String,

    #[serde(rename = "type")]
    class: String,

    #[serde(skip_serializing_if = "Option::is_none")]
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
    limit: i32,

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
    pub fn new(query: String, vars: Variables) -> GraphQL
    {
        GraphQL { query, variables: vars }
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
    pub fn new(class: String, query: String, limit: i32) -> AutoCompleteInput
    {
        AutoCompleteInput { class, query, limit }
    }
}

impl SearchInput {
    pub fn new(
        class: String,
        query: String,
        start: i32,
        count: i32,
        filters: Option<Filter>
    ) -> SearchInput
    {
        SearchInput { class, query, start, count, filters }
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
    pub fn new(user: String, limit: i32) -> ListRecommendationsInput
    {
        ListRecommendationsInput {
            user,
            limit,
            context: RequestContext { 
                scenario: "HOME".into()
            }
        }
    }
}