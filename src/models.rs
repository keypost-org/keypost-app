/// Database models (i.e. tables) only!
use super::schema::users;
use diesel::pg::data_types::PgTimestamp;

// Fields must be in the same order as in schema.rs https://diesel.rs/guides/getting-started
#[derive(Clone, Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub psswd_file: String,
    pub deleted: bool,
    pub inserted_at: PgTimestamp,
    pub updated_at: PgTimestamp,
}

#[derive(Clone, Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub psswd_file: &'a str,
}
