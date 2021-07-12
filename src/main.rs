#![feature(decl_macro)]

#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;

use api::*;
use rocket_contrib::json;
use rocket_contrib::json::{Json, JsonValue};
use rocket_contrib::serve::StaticFiles;

mod api;
mod cache;
mod crypto;
mod persistence;

pub mod models;
pub mod schema;

/// curl -X POST -H "Content-Type: application/json" -d '{ "e": "jon@example.com", "i": "Zm9vYmFyCg==" }' http://localhost:8000/register/start
#[post("/register/start", format = "json", data = "<payload>")]
fn register_start(payload: Json<RegisterStart>) -> JsonValue {
    //TODO Should we do a PKCE-type protocol instead of just a nonce? Maybe OPAQUE internals does this already?
    println!("{:?}", &payload);
    let opaque = crypto::Opaque::new();
    let server_registration_start = opaque.server_side_registration_start(&payload.i, &payload.e);
    let nonce = rand::random::<u32>();
    let response_bytes = server_registration_start.message.serialize();
    let response = base64::encode(response_bytes);
    json!({ "id": &nonce, "o": &response })
}

/// curl -X POST -H "Content-Type: application/json" -d '{ "id": 1234, "e": "jon@example.com", "i": "Zm9vYmFyCg==" }' http://localhost:8000/register/finish
#[post("/register/finish", format = "json", data = "<payload>")]
fn register_finish(payload: Json<RegisterFinish>) -> JsonValue {
    println!("{:?}", &payload);
    let opaque = crypto::Opaque::new();
    let password_file = opaque.server_side_registration_finish(&payload.i);
    match persistence::add_user(&payload.e, base64::encode(password_file).as_str()) {
        Ok(_user) => {
            json!({ "id": &payload.id, "o": "ok" })
        }
        Err(err) => {
            println!("Could not create new user! {}, {:?}", &payload.id, err);
            json!({ "id": &payload.id, "o": "error" })
        }
    }
}

#[post("/login/start", format = "json", data = "<payload>")]
fn login_start(payload: Json<LoginStart>) -> JsonValue {
    //TODO Should we do a PKCE-type protocol instead of just a nonce? Maybe OPAQUE internals does this already?
    println!("{:?}", &payload);
    let opaque = crypto::Opaque::new();
    let user = persistence::find_user(&payload.e).expect("User not found!");
    let password_file_bytes = user.map_or(
        {
            println!("No user in the database!");
            vec![]
        },
        |user| base64::decode(user.psswd_file).expect("Could not base64 decode!"),
    );
    let server_login_start_result =
        opaque.login_start(&payload.e, &password_file_bytes, &payload.i);
    let nonce = rand::random::<u32>();
    let server_login_bytes = server_login_start_result.state.serialize();
    cache::insert(nonce, server_login_bytes);
    let response_bytes = server_login_start_result.message.serialize();
    let response = base64::encode(response_bytes);
    json!({ "id": &nonce, "o": &response })
}

#[post("/login/finish", format = "json", data = "<payload>")]
fn login_finish(payload: Json<LoginFinish>) -> JsonValue {
    println!("{:?}", &payload);
    let server_login_bytes = cache::get(&payload.id).unwrap();
    let opaque = crypto::Opaque::new();
    let response_bytes = opaque.login_finish(&server_login_bytes, &payload.i);
    let response = base64::encode(response_bytes);
    json!({ "id": &payload.id, "o": &response })
}

/// https://github.com/SergioBenitez/Rocket/tree/v0.4.10/examples
fn rocket() -> rocket::Rocket {
    persistence::get_active_users().expect("Could not get users!");

    rocket::ignite()
        .mount(
            "/",
            routes![register_start, register_finish, login_start, login_finish],
        )
        .mount("/", StaticFiles::from("static").rank(-1))
}

fn main() {
    rocket().launch();
}
