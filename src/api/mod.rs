pub mod v1;

pub fn routes() -> axum::Router
{
    let v1_routes = axum::Router::new()
        .nest("/tags", v1::tags::routes())
        .nest("/datasets", v1::datasets::routes())
        .nest("/platforms", v1::platforms::routes());

    axum::Router::new().nest("/v1", v1_routes)
}
