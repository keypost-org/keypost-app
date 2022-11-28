use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::Error;
use dotenv::dotenv;
use std::env;

use crate::models::{Locker, NewLocker, NewUser, User};
use crate::schema::lockers;
use crate::schema::users;

pub fn get_active_users() -> Result<Vec<User>, Error> {
    use crate::schema::users::dsl::*;
    let connection = establish_connection();
    let results = users
        .filter(deleted.eq(false))
        .limit(99)
        .load::<User>(&connection)?;

    println!("Displaying {} users", results.len());
    for user in &results {
        println!("{}", user.email);
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
) -> Result<Locker, Error> {
    add_locker(
        email,
        locker_id,
        base64::encode(psswd_file).as_str(),
        base64::encode(ciphertext).as_str(),
    )
}

fn add_locker<'a>(
    email: &'a str,
    locker_id: &'a str,
    psswd_file: &'a str,
    ciphertext: &'a str,
) -> Result<Locker, Error> {
    let new_locker: NewLocker = NewLocker {
        email,
        locker_id,
        psswd_file,
        ciphertext,
    };
    let connection = establish_connection();
    diesel::insert_into(lockers::table)
        .values(&new_locker)
        .get_result(&connection)
}

pub fn fetch_locker_contents(email_: &str, locker_id_: &str) -> Result<(Vec<u8>, Vec<u8>), Error> {
    use crate::schema::lockers::dsl::*;
    let connection = establish_connection();
    let results: Vec<Locker> = lockers
        .filter(locker_id.eq(locker_id_))
        .filter(email.eq(email_))
        .limit(1)
        .load::<Locker>(&connection)?;
    let locker: Locker = results.first().cloned().unwrap();
    Ok((
        base64::decode(locker.psswd_file).unwrap(),
        base64::decode(locker.ciphertext).unwrap(),
    ))
}

fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
