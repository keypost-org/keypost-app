use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::Error;
use dotenv::dotenv;
use std::env;

use crate::cache;
use crate::models::{NewUser, User};

pub fn get_active_users() -> Result<Vec<User>, Error> {
    use crate::schema::users::dsl::*;

    let connection = establish_connection();
    let results = users
        .filter(deleted.eq(false))
        .limit(99)
        .load::<User>(&connection)?;

    println!("Displaying {} users", results.len());
    for user in &results {
        println!("{}, {}, {}", user.id, user.email, user.psswd_file);
    }

    Ok(results)
}

pub fn find_user(email_: &str) -> Result<Option<User>, Error> {
    use crate::schema::users::dsl::*;

    let connection = establish_connection();
    let results: Vec<User> = users
        .filter(deleted.eq(false))
        .filter(email.eq(email_))
        .limit(1)
        .load::<User>(&connection)?;
    let user: Option<User> = results.first().cloned();
    Ok(user)
}

pub fn add_user<'a>(email: &'a str, psswd_file: &'a str) -> Result<User, Error> {
    use crate::schema::users;

    let new_user = NewUser { email, psswd_file };

    let connection = establish_connection();
    diesel::insert_into(users::table)
        .values(&new_user)
        .get_result(&connection)
}

pub fn store_locker_contents(
    email: &str,
    locker_id: &str,
    psswd_file: &[u8],
    ciphertext: &[u8],
) -> Result<(), Error> {
    // TODO
    cache::insert(1, psswd_file.to_vec());
    cache::insert(2, ciphertext.to_vec());
    Ok(())
}

pub fn fetch_locker_contents(email: &str, locker_id: &str) -> Result<(Vec<u8>, Vec<u8>), Error> {
    // TODO
    let psswd_file = cache::get(&1);
    let ciphertext = cache::get(&2);
    Ok((psswd_file.unwrap(), ciphertext.unwrap()))
}

fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
