use serde::Serialize;

#[derive(Serialize)]
pub struct Paging {
    total: i32,
    limit: i32,
    offset: i32,
}

impl Paging {
    pub fn new(offset: i32, limit: i32, total: i32) -> Paging
    {
        Paging { total, limit, offset }
    }
}