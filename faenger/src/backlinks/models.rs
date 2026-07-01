use diesel::{Identifiable, Queryable, Selectable};
use serde::Serialize;

#[derive(Queryable, Selectable, Identifiable, Debug, Serialize)]
#[diesel(primary_key(url))]
#[diesel(table_name = crate::schema::backlinks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Backlink {
    pub url: String,
    pub lobsters_links: Option<String>,
    pub hn_links: Option<String>,
}
