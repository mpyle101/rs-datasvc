use std::convert::From;
use std::collections::HashMap;

use axum::http::Request;
use hyper::Body;

pub enum QueryType<'a> {
    All,
    Name(&'a str),
    Tags(&'a str),
    Query(&'a str),
}

pub struct QueryParams<'a> {
    pub query: QueryType<'a>,
    pub limit: i32,
    pub start: i32, 
}

impl<'a> From<&'a Request<Body>> for QueryParams<'a> {
    fn from(req: &'a Request<Body>) -> QueryParams<'a>
    {
        let params: HashMap<_, _> = req.uri().query()
            .map_or_else(HashMap::new, parse_query);

        let limit = params.get("limit").map_or(10, |s| s.parse::<i32>().unwrap());
        let start = params.get("offset").map_or(0, |s| s.parse::<i32>().unwrap());

        let query = if let Some(query) = params.get("query") {
            QueryType::Query(query)
        } else if let Some(name) = params.get("name") {
            QueryType::Name(name)
        } else if let Some(tags) = params.get("tags") {
            QueryType::Tags(tags)
        } else {
            QueryType::All
        };

        QueryParams { query, start, limit }
    }
}

fn parse_query(query: &str) -> HashMap<&str, &str>
{
    query.split('&')
        .map(|s| s.split('='))
        .map(|mut v| {
            let key = v.next().unwrap();
            match v.next() {
                Some(val) => (key, val),
                None => ("query", key)
            }
        })
        .collect()
}
