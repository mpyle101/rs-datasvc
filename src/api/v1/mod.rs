mod queries;

pub mod tags;
pub mod datasets;
pub mod platforms;

use std::convert::From;
use std::collections::HashMap;

use axum::http::Request;
use hyper::Body;

pub struct QueryParams<'a> {
    name: Option<&'a str>,
    tags: Option<&'a str>,
    query: Option<&'a str>,
    limit: i32,
    start: i32, 
}

impl<'a> From<&'a Request<Body>> for QueryParams<'a> {
    fn from(req: &'a Request<Body>) -> QueryParams<'a>
    {
        let params: HashMap<_, _> = req.uri().query()
            .map_or_else(HashMap::new, parse_query);

        let limit = params.get("limit").map_or(10, |s| s.parse::<i32>().unwrap());
        let start = params.get("offset").map_or(10, |s| s.parse::<i32>().unwrap());

        QueryParams {
            limit,
            start,
            name: params.get("name").map(|s| *s),
            tags: params.get("tags").map(|s| *s),
            query: params.get("query").map(|s| *s),
        }
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
