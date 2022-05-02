use serde::Serialize;

#[derive(Serialize)]
pub struct GraphQL<'a> {
    query: &'a str,
    variables: Variables<'a>,
}

#[derive(Serialize)]
pub enum Variables<'a> {
    #[serde(rename = "urn")]
    Urn(&'a str),

    #[serde(rename = "input")]
    SearchInput(SearchInput<'a>),

    #[serde(rename = "input")]
    AutoCompleteInput(AutoCompleteInput<'a>),

    #[serde(rename = "input")]
    TagAssociationInput(TagAssociationInput<'a>),

    #[serde(rename = "input")]
    ListRecommendationsInput(ListRecommendationsInput<'a>),
}

#[derive(Serialize)]
pub struct AutoCompleteInput<'a> {
    limit: i32,
    query: &'a str,

    #[serde(rename = "type")]
    class: &'a str,
}

#[derive(Serialize)]
pub struct SearchInput<'a> {
    start: i32,
    count: i32,
    query: String,

    #[serde(rename = "type")]
    class: &'a str,

    #[serde(skip_serializing_if = "Option::is_none")]
    filters: Option<Filter<'a>>,
}

#[derive(Serialize)]
pub struct Filter<'a> {
    field: &'a str,
    value: &'a str,
}

#[derive(Serialize)]
pub struct TagAssociationInput<'a> {
    #[serde(rename(serialize = "tagUrn"))]
    tag: &'a str,

    #[serde(rename(serialize = "resourceUrn"))]
    resource: &'a str
}

#[derive(Serialize)]
pub struct ListRecommendationsInput<'a> {
    limit: i32,

    #[serde(rename(serialize = "userUrn"))]
    user: &'a str,

    #[serde(rename(serialize = "requestContext"))]
    context: RequestContext<'a>,
}

#[derive(Serialize)]
struct RequestContext<'a> {
    scenario: &'a str,
}

impl<'a> GraphQL<'a> {
    pub fn new(query: &'a str, vars: Variables<'a>) -> GraphQL<'a>
    {
        GraphQL { query, variables: vars }
    }
}

impl<'a> std::fmt::Display for GraphQL<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string(self) {
            Ok(s)   => write!(f, "{s}"),
            Err(..) => write!(f, "")
        }
    }
}

impl<'a> AutoCompleteInput<'a> {
    pub fn new(class: &'a str, query: &'a str, limit: i32) -> AutoCompleteInput<'a>
    {
        AutoCompleteInput { class, query, limit }
    }
}

impl<'a> SearchInput<'a> {
    pub fn new(
        class: &'a str,
        query: String,
        start: i32,
        count: i32,
        filters: Option<Filter<'a>>
    ) -> SearchInput<'a>
    {
        SearchInput { class, query, start, count, filters }
    }
}

impl<'a> Filter<'a> {
    pub fn new(field: &'a str, value: &'a str) -> Filter<'a>
    {
        Filter { field, value }
    }
}

impl<'a> TagAssociationInput<'a> {
    pub fn new(resource: &'a str, tag: &'a str) -> TagAssociationInput<'a>
    {
        TagAssociationInput { tag, resource }
    }
}

impl<'a> ListRecommendationsInput<'a> {
    pub fn new(user: &'a str, limit: i32) -> ListRecommendationsInput<'a>
    {
        ListRecommendationsInput {
            user,
            limit,
            context: RequestContext { 
                scenario: "HOME"
            }
        }
    }
}