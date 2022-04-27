use axum::http::{header, Method, Request};
use hyper::{
    Body,
    client::{HttpConnector, ResponseFuture},
};

type HyperClient = hyper::client::Client<HttpConnector, Body>;

pub const INGEST_ENDPOINT: &str = "http://localhost:8080/entities?action=ingest";
pub const GRAPHQL_ENDPOINT: &str = "http://localhost:8080/api/graphql";

pub fn post(client: &HyperClient, url: &str, data: impl std::fmt::Display) -> ResponseFuture
{
    let req = Request::builder()
        .method(Method::POST)
        .uri(url)
        .header("X-DataHub-Actor", "urn:li:corpuser:datahub")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(data.to_string()))
        .unwrap();

    client.request(req)
}
