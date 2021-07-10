use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::Error;
use dotenv::dotenv;
use std::env;

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
        println!(
            "{}, {}, {}, {}",
            user.id, user.username, user.email, user.psswd_file
        );
    }

    Ok(results)
}

pub fn find_user(username_: &str) -> Result<Option<User>, Error> {
    use crate::schema::users::dsl::*;

    let connection = establish_connection();
    let results: Vec<User> = users
        .filter(deleted.eq(false))
        .filter(username.eq(username_))
        .limit(1)
        .load::<User>(&connection)?;
    let user: Option<User> = results.first().cloned();
    Ok(user)
}

pub fn add_user<'a>(username: &'a str, email: &'a str, psswd_file: &'a str) -> Result<User, Error> {
    use crate::schema::users;

    let new_user = NewUser {
        username,
        email,
        psswd_file,
    };

    let connection = establish_connection();
    //FIXME Don't allow duplicate username or email via DB constraints
    diesel::insert_into(users::table)
        .values(&new_user)
        .get_result(&connection)
}

fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}