use std::marker::PhantomData;

use crate::schemas::{
    GraphQL,
    Variables,
    Filter,
    SearchInput,
    AutoCompleteInput,
    ListRecommendationsInput
};
use crate::api::v1::{queries, params::QueryParams};


pub struct GetOneFactory<'a> {
    query: String,
    marker: PhantomData<&'a str>
}

impl<'a> GetOneFactory<'a> {
    pub fn new(class: &str, values: &str) -> GetOneFactory<'a>
    {
        GetOneFactory { query: queries::by_id(class, values), marker: PhantomData }
    }

    pub fn body(&'a self, id: &'a str) -> GraphQL<'a>
    {
        GraphQL::new(&self.query, Variables::Urn(id))
    }
}


pub struct GetAllFactory<'a> {
    class: &'a str,
    query: String,
}

impl<'a> GetAllFactory<'a> {
    pub fn new(class: &'a str, values: &str) -> GetAllFactory<'a>
    {
        GetAllFactory {
            class, query: queries::by_query(values)
        }
    }

    pub fn body(&self, params: &QueryParams) -> GraphQL
    {
        GraphQL::new(
            &self.query,
            Variables::SearchInput(
                SearchInput::new(self.class, "*".into(), params.start, params.limit, None)
            )
        )
    }
}

pub struct NameFactory<'a> {
    class: &'a str,
    query: String,
}

impl<'a> NameFactory<'a> {
    pub fn new(class: &'a str, values: &str) -> NameFactory<'a>
    {
        NameFactory { class, query: queries::by_name(values) }
    }

    pub fn body(&self, query: &'a str, params: &QueryParams) -> GraphQL
    {
        GraphQL::new(
            &self.query,
            Variables::AutoCompleteInput(
                AutoCompleteInput::new(self.class, query, params.limit)
            )
        )
    }
}

pub struct TagsFactory<'a> {
    class: &'a str,
    query: String,
}

impl<'a> TagsFactory<'a> {
    pub fn new(class: &'a str, values: &str) -> TagsFactory<'a>
    {
        TagsFactory {
            class,
            query: queries::by_query(values),
        }
    }

    pub fn body(&'a self, query: &str, params: &QueryParams) -> GraphQL<'a>
    {
        let tags = format!("tags:{query}");

        GraphQL::new(
            &self.query,
            Variables::SearchInput(
                SearchInput::new(self.class, tags, params.start, params.limit, None)
            )
        )
    }
}

pub struct QueryFactory<'a> {
    class: &'a str,
    query: String,
}

impl<'a> QueryFactory<'a> {
    pub fn new(class: &'a str, values: &str) -> QueryFactory<'a>
    {
        QueryFactory {
            class, query: queries::by_query(values)
        }
    }

    pub fn body(&self, query: &'a str, params: &QueryParams) -> GraphQL
    {
        let q = format!("*{query}*");

        GraphQL::new(
            &self.query,
            Variables::SearchInput(
                SearchInput::new(self.class, q, params.start, params.limit, None)
            )
        )
    }
}

pub struct FilterFactory<'a> {
    query: String,
    class: &'a str,
    filter: &'a str,
}

impl<'a> FilterFactory<'a> {
    pub fn new(class: &'a str, values: &str, filter: &'a str) -> FilterFactory<'a>
    {
        FilterFactory {
            class, filter, query: queries::by_query(values)
        }
    }

    pub fn body(&self, value: &'a str, params: &QueryParams) -> GraphQL
    {
        GraphQL::new(
            &self.query,
            Variables::SearchInput(
                SearchInput::new(
                    self.class, "*".into(), params.start, params.limit,
                    Some(Filter::new(self.filter, value))
                )
            )
        )
    }
}

pub struct PlatformsFactory<'a> {
    user: String,
    query: String,
    marker: PhantomData<&'a str>
}

impl<'a> PlatformsFactory<'a> {
    pub fn new(values: &str) -> PlatformsFactory<'a>
    {
        let user = "urn:li:corpuser:datahub".into();
        PlatformsFactory { user, query: queries::platforms(values), marker: PhantomData }
    }

    pub fn body(&'a self, params: &QueryParams) -> GraphQL<'a>
    {
        GraphQL::new(
            &self.query,
            Variables::ListRecommendationsInput(
                ListRecommendationsInput::new(&self.user, params.limit)
            )
        )
    }
}
