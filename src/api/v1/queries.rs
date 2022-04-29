
pub fn by_id(entity: &str, values: &str) -> String
{
    format!(r#"
        query by_id($urn: String!) {{
            entity: {entity}(urn: $urn) {{ {values} }}
        }}
    "#).replace('\n', "").replace("  ", " ")
}

pub fn by_name(values: &str) -> String
{
    format!(r#"
        query by_name($input: AutoCompleteInput!) {{
            results: autoComplete(input: $input) {{
                __typename
                entities {{ {values} }}
            }}
        }}
    "#).replace('\n', "").replace("  ", " ")
}

pub fn by_query(values: &str) -> String
{
    format!(r#"
        query by_query($input: SearchInput!) {{
            results: search(input: $input) {{
                __typename start count total
                entities: searchResults {{ entity {{ {values} }} }}
            }}
        }}
    "#).replace('\n', "").replace("  ", " ")
}

pub fn add_tag() -> String
{
    "mutation add_tag($input: TagAssociationInput!) {
        success: addTag(input: $input)
    }".replace('\n', "").replace("  ", " ")
}

pub fn remove_tag() -> String
{
    "mutation remove_tag($input: TagAssociationInput!) {
        success: removeTag(input: $input)
    }".replace('\n', "")
}

pub fn platforms(values: &str) -> String
{
    format!(r#"
        query platforms($input: ListRecommendationsInput!) {{ 
            results: listRecommendations(input: $input) {{
                modules {{
                    id: moduleId
                    content {{ entity {{ {values} }} }}
                }}
            }}
        }}
    "#).replace('\n', "").replace("  ", " ")
}