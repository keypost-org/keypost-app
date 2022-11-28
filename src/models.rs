/// Database models (i.e. tables) only!
use super::schema::lockers;
use super::schema::users;
use diesel::pg::data_types::PgTimestamp;

// Fields must be in the same order as in schema.rs https://diesel.rs/guides/getting-started
#[derive(Clone, Queryable)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub psswd_file: String,
    pub deleted: bool,
    pub inserted_at: PgTimestamp,
    pub updated_at: PgTimestamp,
}

#[derive(Clone, Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub psswd_file: &'a str,
}

#[derive(Clone, Queryable)]
pub struct Locker {
    pub id: i32,
    pub email: String,
    pub locker_id: String,
    pub psswd_file: String,
    pub ciphertext: String,
    pub inserted_at: PgTimestamp,
    pub updated_at: PgTimestamp,
}

#[derive(Clone, Insertable)]
#[table_name = "lockers"]
pub struct NewLocker<'a> {
    pub email: &'a str,
    pub locker_id: &'a str,
    pub psswd_file: &'a str,
    pub ciphertext: &'a str,
}
